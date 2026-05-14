use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use php_runtime::{
    ArityExpectation, ArrayColumnKey, ArrayKey, ArrayKeyCase, ClassId, Comparison, ObjectProperty,
    PhpArray, PhpClassConstantMetadata, PhpClassTable, PhpMethodMetadata, PhpObject,
    PhpObjectPropertyInitializer, PhpPropertyMetadata, RuntimeError, RuntimeResult, Value,
    Visibility,
};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, ClassConstantDecl, ClassDecl, ClassMember,
    ClassPropertyDecl, ClassVisibility, CompoundAssignOp, Expr, ForAction, FunctionDecl,
    IncrementDecrementOp, IncrementDecrementPosition, Program, Span, StaticLocalDeclarator, Stmt,
    SwitchCase, UnaryOp, UnsetTarget,
};
use crate::error::{CompileResult, Diagnostic, Phase};
use crate::parser::parse_source;

pub const MAX_USER_FUNCTION_CALL_DEPTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub fn run_program(program: &Program) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program, None)?;
    interpreter.run(program)
}

pub fn run_program_with_source_file(
    program: &Program,
    source_file: impl Into<String>,
) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program, Some(source_file.into()))?;
    interpreter.run(program)
}

pub fn class_metadata(program: &Program) -> CompileResult<PhpClassTable> {
    Interpreter::from_program(program, None).map(|interpreter| interpreter.classes)
}

struct Interpreter {
    functions: HashMap<String, Rc<FunctionDecl>>,
    methods: HashMap<(ClassId, String), Rc<FunctionDecl>>,
    class_constants: HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: HashMap<(ClassId, String), Value>,
    classes: PhpClassTable,
    constants: ConstantTable,
    required_once: HashSet<PathBuf>,
    static_locals: HashMap<(String, String), Value>,
    active_static_locals: Vec<Vec<String>>,
    source_file: Option<String>,
    call_depth: usize,
    next_object_id: i64,
    function_context: Vec<String>,
    class_context: Vec<ClassId>,
    called_class_context: Vec<ClassId>,
    stdout: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedRequirePath {
    read_path: PathBuf,
    source_file: PathBuf,
}

#[derive(Debug, Clone)]
enum Callable {
    Builtin(String),
    User(Rc<FunctionDecl>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayFilterMode {
    Value,
    Both,
    Key,
}

fn parse_array_filter_string_mode(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = trimmed.parse::<i64>() {
        return Some(value);
    }
    trimmed.parse::<f64>().ok().and_then(integral_float_to_i64)
}

fn repo_root_relative_path(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return None;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent()?;
    let candidate = repo_root.join(path);
    candidate.exists().then_some(candidate)
}

fn integral_float_to_i64(value: f64) -> Option<i64> {
    if !value.is_finite() || value.fract() != 0.0 {
        return None;
    }
    if value < i64::MIN as f64 || value > i64::MAX as f64 {
        return None;
    }
    Some(value as i64)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompoundAssignmentPlace {
    Variable(String),
    ArrayIndex {
        name: String,
        key: ArrayKey,
    },
    ObjectProperty {
        object: String,
        property: String,
    },
    StaticProperty {
        declaring_class_id: ClassId,
        property: String,
    },
}

#[derive(Debug, Clone, Default)]
struct SymbolTable {
    // Static variables and future dynamic variable names share the same
    // materialized storage path; current syntax only calls the static methods.
    symbols: HashMap<String, Value>,
}

impl SymbolTable {
    fn new() -> Self {
        Self::default()
    }

    fn read_static(&self, name: &str, span: Span) -> CompileResult<Value> {
        self.read_named(name)
            .cloned()
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_variable(name)))
    }

    fn write_static(&mut self, name: &str, value: Value) {
        self.write_named(name.to_string(), value);
    }

    fn is_set_static(&self, name: &str) -> bool {
        matches!(self.read_named(name), Some(value) if !matches!(value, Value::Null))
    }

    fn unset_static(&mut self, name: &str) {
        self.symbols.remove(name);
    }

    fn array_slot_for_static_write(&mut self, name: &str) -> &mut Value {
        self.symbols
            .entry(name.to_string())
            .or_insert_with(|| Value::Array(PhpArray::new()))
    }

    fn object_slot_for_static_write(
        &mut self,
        name: &str,
        span: Span,
    ) -> CompileResult<&mut Value> {
        self.symbols
            .get_mut(name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_variable(name)))
    }

    fn read_named(&self, name: &str) -> Option<&Value> {
        self.symbols.get(name)
    }

    fn write_named(&mut self, name: String, value: Value) {
        self.symbols.insert(name, value);
    }
}

#[derive(Debug, Clone, Default)]
struct ConstantTable {
    values: HashMap<String, Value>,
}

impl ConstantTable {
    fn new() -> Self {
        Self::default()
    }

    fn define(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        if builtin_global_constant_value(name).is_some() || self.values.contains_key(name) {
            return Err(RuntimeError::duplicate_constant(name));
        }

        self.values.insert(name.to_string(), value);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| builtin_global_constant_value(name).map(Value::Int))
    }

    fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name) || builtin_global_constant_value(name).is_some()
    }
}

enum Flow {
    Normal,
    Break(Span),
    Continue(Span),
    Return(Value),
    Goto { label: String, span: Span },
}

impl Interpreter {
    fn from_program(program: &Program, source_file: Option<String>) -> CompileResult<Self> {
        let mut functions = HashMap::new();
        let mut methods = HashMap::new();
        let mut class_constants = HashMap::new();
        let mut static_properties = HashMap::new();
        let mut classes = PhpClassTable::with_core_classes();
        for stmt in &program.statements {
            match stmt {
                Stmt::Function(function) => {
                    let key = function.name.to_ascii_lowercase();
                    if functions.contains_key(&key) {
                        return Err(runtime_error(
                            function.span,
                            RuntimeError::duplicate_function(callable_name(&function.name)),
                        ));
                    }
                    functions.insert(key, Rc::new(function.clone()));
                }
                Stmt::Class(class) if !class.is_nested => {
                    register_class_name(&mut classes, class)?;
                }
                _ => {}
            }
        }

        for stmt in &program.statements {
            if let Stmt::Class(class) = stmt {
                if class.is_nested {
                    continue;
                }
                let class_id = register_class_members(&mut classes, class)?;
                register_class_member_runtime_tables(
                    &mut class_constants,
                    &mut static_properties,
                    &mut methods,
                    class_id,
                    class,
                );
            }
        }

        let mut interpreter = Self {
            functions,
            methods,
            class_constants,
            static_properties,
            classes,
            constants: ConstantTable::new(),
            required_once: HashSet::new(),
            static_locals: HashMap::new(),
            active_static_locals: Vec::new(),
            source_file,
            call_depth: 0,
            next_object_id: 1,
            function_context: Vec::new(),
            class_context: Vec::new(),
            called_class_context: Vec::new(),
            stdout: String::new(),
        };
        interpreter.initialize_static_property_defaults(program)?;
        Ok(interpreter)
    }

    fn initialize_static_property_defaults(&mut self, program: &Program) -> CompileResult<()> {
        for stmt in &program.statements {
            let Stmt::Class(class) = stmt else {
                continue;
            };
            if class.is_nested {
                continue;
            }
            let class_id = self
                .classes
                .lookup_class_id(&class.name)
                .expect("class registration should declare class id");

            for member in &class.members {
                let ClassMember::Property(property) = member else {
                    continue;
                };
                if !property.is_static {
                    continue;
                }
                let Some(default) = &property.default else {
                    continue;
                };

                let mut default_scope = SymbolTable::new();
                let value = self.evaluate(default, &mut default_scope)?;
                self.static_properties
                    .insert((class_id, property.name.clone()), value);
            }
        }

        Ok(())
    }

    fn register_included_declarations(&mut self, program: &Program) -> CompileResult<()> {
        for stmt in &program.statements {
            match stmt {
                Stmt::Function(function) => {
                    let key = function.name.to_ascii_lowercase();
                    if self.functions.contains_key(&key) {
                        return Err(runtime_error(
                            function.span,
                            RuntimeError::duplicate_function(callable_name(&function.name)),
                        ));
                    }
                    self.functions.insert(key, Rc::new(function.clone()));
                }
                Stmt::Class(class) if !class.is_nested => {
                    register_class_name(&mut self.classes, class)?;
                }
                _ => {}
            }
        }

        for stmt in &program.statements {
            let Stmt::Class(class) = stmt else {
                continue;
            };
            if class.is_nested {
                continue;
            }
            let class_id = register_class_members(&mut self.classes, class)?;
            register_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                class_id,
                class,
            );
        }

        self.initialize_static_property_defaults(program)
    }

    fn register_nested_class_declaration(&mut self, class: &ClassDecl) -> CompileResult<()> {
        let class_id = register_class_name(&mut self.classes, class)?;
        if let Err(error) = register_class_members(&mut self.classes, class) {
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }
        register_class_member_runtime_tables(
            &mut self.class_constants,
            &mut self.static_properties,
            &mut self.methods,
            class_id,
            class,
        );
        if let Err(error) = self.initialize_static_property_defaults_for_class(class_id, class) {
            remove_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                class_id,
            );
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }

        Ok(())
    }

    fn initialize_static_property_defaults_for_class(
        &mut self,
        class_id: ClassId,
        class: &ClassDecl,
    ) -> CompileResult<()> {
        for member in &class.members {
            let ClassMember::Property(property) = member else {
                continue;
            };
            if !property.is_static {
                continue;
            }
            let Some(default) = &property.default else {
                continue;
            };

            let mut default_scope = SymbolTable::new();
            let value = self.evaluate(default, &mut default_scope)?;
            self.static_properties
                .insert((class_id, property.name.clone()), value);
        }

        Ok(())
    }

    fn run(&mut self, program: &Program) -> CompileResult<Execution> {
        let mut scope = SymbolTable::new();
        match self.execute_statements(&program.statements, &mut scope)? {
            Flow::Normal | Flow::Return(_) => Ok(Execution {
                stdout: self.stdout.clone(),
                stderr: String::new(),
                exit_code: 0,
            }),
            Flow::Break(span) => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
            )),
            Flow::Continue(span) => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("continue cannot be used outside a loop"),
            )),
            Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
        }
    }

    fn execute_statements(
        &mut self,
        statements: &[Stmt],
        scope: &mut SymbolTable,
    ) -> CompileResult<Flow> {
        let mut labels = HashMap::new();
        for (index, stmt) in statements.iter().enumerate() {
            if let Stmt::Label { name, .. } = stmt {
                labels.insert(name.clone(), index);
            }
        }

        let mut index = 0;
        while index < statements.len() {
            match self.execute_statement(&statements[index], scope)? {
                Flow::Normal => {}
                Flow::Goto { label, span } => {
                    let Some(target) = labels.get(&label) else {
                        return Ok(Flow::Goto { label, span });
                    };
                    index = *target;
                    continue;
                }
                flow @ (Flow::Break(_) | Flow::Continue(_) | Flow::Return(_)) => return Ok(flow),
            }
            index += 1;
        }
        Ok(Flow::Normal)
    }

    fn execute_statement(&mut self, stmt: &Stmt, scope: &mut SymbolTable) -> CompileResult<Flow> {
        match stmt {
            Stmt::Namespace { .. } | Stmt::Use { .. } => Ok(Flow::Normal),
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.evaluate(expr, scope)?;
                    let output = value
                        .try_echo_string()
                        .map_err(|error| runtime_error(expr.span(), error))?;
                    self.stdout.push_str(&output);
                }
                Ok(Flow::Normal)
            }
            Stmt::Print { expr, .. } => {
                let value = self.evaluate(expr, scope)?;
                let output = value
                    .try_echo_string()
                    .map_err(|error| runtime_error(expr.span(), error))?;
                self.stdout.push_str(&output);
                Ok(Flow::Normal)
            }
            Stmt::Assign { target, expr, .. } => {
                self.execute_assignment(target, expr, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => {
                self.execute_compound_assignment(target, *op, expr, *span, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::IncrementDecrement { target, op, span } => {
                self.execute_increment_decrement(target, *op, *span, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::NullCoalesceAssign { target, expr, .. } => {
                self.evaluate_null_coalesce_assignment(target, expr, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::Expr { expr, .. } => {
                self.evaluate(expr, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::Goto { label, span } => Ok(Flow::Goto {
                label: label.clone(),
                span: *span,
            }),
            Stmt::Label { .. } => Ok(Flow::Normal),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                if self.evaluate(condition, scope)?.is_truthy() {
                    self.execute_statements(then_branch, scope)
                } else {
                    self.execute_statements(else_branch, scope)
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                while self.evaluate(condition, scope)?.is_truthy() {
                    match self.execute_statements(body, scope)? {
                        Flow::Normal | Flow::Continue(_) => {}
                        Flow::Break(_) => break,
                        flow @ (Flow::Return(_) | Flow::Goto { .. }) => return Ok(flow),
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                loop {
                    match self.execute_statements(body, scope)? {
                        Flow::Normal | Flow::Continue(_) => {}
                        Flow::Break(_) => break,
                        flow @ (Flow::Return(_) | Flow::Goto { .. }) => return Ok(flow),
                    }

                    if !self.evaluate(condition, scope)?.is_truthy() {
                        break;
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
                ..
            } => {
                if let Some(initializer) = initializer {
                    self.execute_for_action(initializer, scope)?;
                }

                loop {
                    if let Some(condition) = condition {
                        if !self.evaluate(condition, scope)?.is_truthy() {
                            break;
                        }
                    }

                    match self.execute_statements(body, scope)? {
                        Flow::Normal | Flow::Continue(_) => {}
                        Flow::Break(_) => break,
                        flow @ (Flow::Return(_) | Flow::Goto { .. }) => return Ok(flow),
                    }

                    if let Some(increment) = increment {
                        self.execute_for_action(increment, scope)?;
                    }
                }

                Ok(Flow::Normal)
            }
            Stmt::Switch { value, cases, .. } => self.execute_switch(value, cases, scope),
            Stmt::Foreach {
                iterable,
                key,
                value,
                body,
                span,
            } => {
                let iterable = self.evaluate(iterable, scope)?;
                let array = match iterable {
                    Value::Array(array) => array,
                    other => {
                        return Err(runtime_error(
                            *span,
                            RuntimeError::invalid_foreach(format!(
                                "can only iterate arrays in the current subset, got {}",
                                other.type_name()
                            )),
                        ));
                    }
                };

                for entry in array.entries() {
                    if let Some(key) = key {
                        scope.write_static(key, value_from_array_key(&entry.key));
                    }
                    scope.write_static(value, entry.value.clone());
                    match self.execute_statements(body, scope)? {
                        Flow::Normal | Flow::Continue(_) => {}
                        Flow::Break(_) => break,
                        flow @ (Flow::Return(_) | Flow::Goto { .. }) => return Ok(flow),
                    }
                }

                Ok(Flow::Normal)
            }
            Stmt::UnsetVariable { name, .. } => {
                scope.unset_static(name);
                Ok(Flow::Normal)
            }
            Stmt::UnsetArrayIndex { name, index, span } => {
                self.execute_unset_array_index(name, index, *span, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetStaticProperty {
                class_name,
                property,
                span,
            } => {
                self.execute_unset_named_static_property(class_name, property, *span)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetSelfStaticProperty { property, span } => {
                self.execute_unset_self_static_property(property, *span)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetParentStaticProperty { property, span } => {
                self.execute_unset_parent_static_property(property, *span)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetLateStaticProperty { property, span } => {
                self.execute_unset_late_static_property(property, *span)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetMany { targets, span } => {
                for target in targets {
                    self.execute_unset_target(target, *span, scope)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::ConstDeclaration { declarations, .. } => {
                for declaration in declarations {
                    self.execute_const_declaration(
                        &declaration.name,
                        &declaration.value,
                        declaration.span,
                        scope,
                    )?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Require { path, once, span } => {
                self.execute_file_include(path, *once, true, *span, scope)
            }
            Stmt::Include { path, once, span } => {
                self.execute_file_include(path, *once, false, *span, scope)
            }
            Stmt::Function(_) => Ok(Flow::Normal),
            Stmt::Class(class) => {
                if class.is_nested {
                    self.register_nested_class_declaration(class)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.evaluate(expr, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(value))
            }
            Stmt::Throw { expr: _, span } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "throw",
                    "exception objects and stack unwinding are not implemented",
                ),
            )),
            Stmt::Try { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "try",
                    "exception handling and stack unwinding are not implemented",
                ),
            )),
            Stmt::Break { span } => Ok(Flow::Break(*span)),
            Stmt::Continue { span } => Ok(Flow::Continue(*span)),
            Stmt::Global { span, .. } => {
                if self.function_context.is_empty() {
                    Ok(Flow::Normal)
                } else {
                    Err(runtime_error(
                        *span,
                        RuntimeError::unsupported_global(
                            "importing globals into function scope is not implemented",
                        ),
                    ))
                }
            }
            Stmt::StaticLocal { declarations, span } => {
                self.execute_static_local_declaration(declarations, *span, scope)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn execute_static_local_declaration(
        &mut self,
        declarations: &[StaticLocalDeclarator],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let Some(function_name) = self.function_context.last().cloned() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "static",
                    "top-level static local declarations are not implemented",
                ),
            ));
        };
        let function_key = function_name.to_ascii_lowercase();

        for declaration in declarations {
            if let Some(active) = self.active_static_locals.last_mut() {
                if !active.contains(&declaration.name) {
                    active.push(declaration.name.clone());
                }
            }

            let key = (function_key.clone(), declaration.name.clone());
            let value = if let Some(value) = self.static_locals.get(&key) {
                value.clone()
            } else {
                let value = match &declaration.default {
                    Some(default) => self.evaluate(default, scope)?,
                    None => Value::Null,
                };
                self.static_locals.insert(key, value.clone());
                value
            };
            scope.write_static(&declaration.name, value);
        }

        Ok(())
    }

    fn execute_for_action(
        &mut self,
        action: &ForAction,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        match action {
            ForAction::Assign { target, expr } => self.execute_assignment(target, expr, scope),
            ForAction::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => self.execute_compound_assignment(target, *op, expr, *span, scope),
            ForAction::IncrementDecrement { target, op, span } => {
                self.execute_increment_decrement(target, *op, *span, scope)
            }
            ForAction::Expr { expr } => {
                self.evaluate(expr, scope)?;
                Ok(())
            }
        }
    }

    fn execute_file_include(
        &mut self,
        path: &Expr,
        once: bool,
        required: bool,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Flow> {
        let construct = if required {
            if once {
                "require_once"
            } else {
                "require"
            }
        } else if once {
            "include_once"
        } else {
            "include"
        };
        let path_value = self.evaluate(path, scope)?;
        let Value::String(path_value) = path_value else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    construct,
                    "path must evaluate to a string in the current subset",
                ),
            ));
        };

        let path = self.resolve_required_path(&path_value, construct, span)?;
        let once_key = if once {
            Some(fs::canonicalize(&path.read_path).unwrap_or_else(|_| path.read_path.clone()))
        } else {
            None
        };
        if once_key
            .as_ref()
            .is_some_and(|key| self.required_once.contains(key))
        {
            return Ok(Flow::Normal);
        }

        let source = fs::read_to_string(&path.read_path).map_err(|error| {
            if !required {
                return runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        construct,
                        "missing-file warning recovery is not implemented",
                    ),
                );
            }
            Diagnostic::new(
                Phase::Io,
                span.line,
                span.column,
                format!(
                    "failed to read required file {}: {error}",
                    path.source_file.display()
                ),
            )
        })?;
        let program = parse_source(&source).map_err(|error| error.with_file(&path.source_file))?;
        if let Some(key) = once_key {
            self.required_once.insert(key);
        }

        let previous_source_file = self.source_file.clone();
        self.source_file = Some(path.source_file.display().to_string());
        let flow = (|| {
            self.register_included_declarations(&program)?;
            self.execute_statements(&program.statements, scope)
        })();
        self.source_file = previous_source_file;

        match flow? {
            Flow::Normal | Flow::Return(_) => Ok(Flow::Normal),
            Flow::Break(span) => Ok(Flow::Break(span)),
            Flow::Continue(span) => Ok(Flow::Continue(span)),
            Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
        }
    }

    fn resolve_required_path(
        &self,
        path: &str,
        construct: &'static str,
        span: Span,
    ) -> CompileResult<ResolvedRequirePath> {
        if path.contains("://") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    construct,
                    "stream and URL require paths are not implemented",
                ),
            ));
        }

        let path = PathBuf::from(path);
        if path.is_absolute() {
            return Ok(ResolvedRequirePath {
                read_path: path.clone(),
                source_file: path,
            });
        }

        let base = self
            .source_file
            .as_deref()
            .and_then(|source_file| {
                let parent = Path::new(source_file).parent()?;
                if parent.as_os_str().is_empty() {
                    None
                } else {
                    Some(parent.to_path_buf())
                }
            })
            .unwrap_or_else(|| PathBuf::from("."));
        let source_file = base.join(path);
        let read_path = if source_file.exists() {
            source_file.clone()
        } else {
            // Rust fixture tests run from the crate directory while committed
            // source-map snapshots use repo-relative fixture paths.
            repo_root_relative_path(&source_file).unwrap_or_else(|| source_file.clone())
        };
        Ok(ResolvedRequirePath {
            read_path,
            source_file,
        })
    }

    fn execute_switch(
        &mut self,
        value: &Expr,
        cases: &[SwitchCase],
        scope: &mut SymbolTable,
    ) -> CompileResult<Flow> {
        let switch_value = self.evaluate(value, scope)?;
        let mut default_index = None;
        let mut matched_index = None;

        for (index, case) in cases.iter().enumerate() {
            let Some(condition) = &case.condition else {
                if default_index.is_none() {
                    default_index = Some(index);
                }
                continue;
            };

            let case_value = self.evaluate(condition, scope)?;
            let matched = switch_value
                .php_cmp_checked(&case_value, Comparison::Eq)
                .map_err(|error| runtime_error(condition.span(), error))?;
            if matched {
                matched_index = Some(index);
                break;
            }
        }

        let Some(mut index) = matched_index.or(default_index) else {
            return Ok(Flow::Normal);
        };

        while index < cases.len() {
            match self.execute_statements(&cases[index].body, scope)? {
                Flow::Normal => {}
                Flow::Break(_) => return Ok(Flow::Normal),
                Flow::Continue(span) => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control(
                            "continue inside switch is not implemented; use break for switch cases in the current subset",
                        ),
                    ));
                }
                flow @ (Flow::Return(_) | Flow::Goto { .. }) => return Ok(flow),
            }
            index += 1;
        }

        Ok(Flow::Normal)
    }

    fn execute_unset_target(
        &mut self,
        target: &UnsetTarget,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        match target {
            UnsetTarget::Variable { name, .. } => {
                scope.unset_static(name);
                Ok(())
            }
            UnsetTarget::ArrayIndex { name, index, .. } => {
                self.execute_unset_array_index(name, index, span, scope)
            }
            UnsetTarget::StaticProperty {
                class_name,
                property,
                ..
            } => self.execute_unset_named_static_property(class_name, property, span),
            UnsetTarget::SelfStaticProperty { property, .. } => {
                self.execute_unset_self_static_property(property, span)
            }
            UnsetTarget::ParentStaticProperty { property, .. } => {
                self.execute_unset_parent_static_property(property, span)
            }
            UnsetTarget::LateStaticProperty { property, .. } => {
                self.execute_unset_late_static_property(property, span)
            }
        }
    }

    fn execute_unset_array_index(
        &mut self,
        name: &str,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let key = self.evaluate_array_key(index, scope)?;

        match scope.read_named(name).cloned() {
            Some(Value::Array(mut array)) => {
                array.remove(key);
                scope.write_static(name, Value::Array(array));
                Ok(())
            }
            Some(Value::Null) | None => Ok(()),
            Some(other) => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot unset offset on {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn execute_unset_named_static_property(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        self.reject_static_property_unset(class_name, property, span)
    }

    fn execute_unset_self_static_property(&self, property: &str, span: Span) -> CompileResult<()> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::${property}"),
                    "self static property access requires instance method context",
                ),
            ));
        };

        let class_name = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata")
            .name();
        self.reject_static_property_unset(class_name, property, span)
    }

    fn execute_unset_parent_static_property(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<()> {
        let (parent_class_id, parent_class_name) =
            self.resolve_parent_static_property_context(property, span)?;
        self.classes
            .get(parent_class_id)
            .expect("parent class id should resolve to class metadata");
        self.reject_static_property_unset(&parent_class_name, property, span)
    }

    fn execute_unset_late_static_property(&self, property: &str, span: Span) -> CompileResult<()> {
        let (called_class_id, called_class_name) =
            self.resolve_late_static_property_context(property, span)?;
        self.classes
            .get(called_class_id)
            .expect("called class id should resolve to class metadata");
        self.reject_static_property_unset(&called_class_name, property, span)
    }

    fn reject_static_property_unset(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<()> {
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                format!("{class_name}::${property}"),
                "static property unset is not supported; assign null to the static property in the current subset",
            ),
        ))
    }

    fn execute_const_declaration(
        &mut self,
        name: &str,
        value: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let value = self.evaluate(value, scope)?;
        if let Some(type_name) = unsupported_runtime_constant_value_type(&value) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "const declaration",
                    format!(
                        "value must be null, bool, int, float, string, or array values in the current subset, got {type_name}"
                    ),
                ),
            ));
        }
        self.constants
            .define(name, value)
            .map_err(|error| runtime_error(span, error))
    }

    fn evaluate(&mut self, expr: &Expr, scope: &mut SymbolTable) -> CompileResult<Value> {
        match expr {
            Expr::Null(_) => Ok(Value::Null),
            Expr::Bool(value, _) => Ok(Value::Bool(*value)),
            Expr::Int(value, _) => Ok(Value::Int(*value)),
            Expr::Float(value, _) => Ok(Value::Float(*value)),
            Expr::String(value, _) => Ok(Value::String(value.clone())),
            Expr::Variable(name, span) => {
                if name.eq_ignore_ascii_case("this") && scope.read_named("this").is_none() {
                    return Err(runtime_error(
                        *span,
                        RuntimeError::unsupported_call(
                            "$this",
                            "object context is only available during instance method execution",
                        ),
                    ));
                }
                scope.read_static(name, *span)
            }
            Expr::MagicLine { span } => Ok(Value::Int(span.line as i64)),
            Expr::MagicFile { .. } => {
                Ok(Value::String(self.source_file.clone().unwrap_or_default()))
            }
            Expr::MagicDir { .. } => Ok(Value::String(self.magic_dir_value())),
            Expr::MagicFunction { .. } => Ok(Value::String(
                self.function_context.last().cloned().unwrap_or_default(),
            )),
            Expr::GlobalConstant { name, span } => self.evaluate_global_constant(name, *span),
            Expr::ClassNameConstant { class_name, .. } => Ok(Value::String(class_name.clone())),
            Expr::SelfClassNameConstant { span } => self.evaluate_self_class_name_constant(*span),
            Expr::ParentClassNameConstant { span } => {
                self.evaluate_parent_class_name_constant(*span)
            }
            Expr::StaticClassNameConstant { span } => {
                self.evaluate_static_class_name_constant(*span)
            }
            Expr::ClassConstant {
                class_name,
                constant,
                span,
            } => self.evaluate_named_class_constant(class_name, constant, *span),
            Expr::SelfClassConstant { constant, span } => {
                self.evaluate_self_class_constant(constant, *span)
            }
            Expr::ParentClassConstant { constant, span } => {
                self.evaluate_parent_class_constant(constant, *span)
            }
            Expr::LateStaticClassConstant { constant, span } => {
                self.evaluate_late_static_class_constant(constant, *span)
            }
            Expr::Array { items, span } => self.evaluate_array(items, *span, scope),
            Expr::Index {
                target,
                index,
                span,
            } => self.evaluate_array_index(target, index, *span, scope),
            Expr::AppendIndex { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call("[]", "append offset reads are not implemented"),
            )),
            Expr::Property {
                target,
                property,
                span,
            } => self.evaluate_property_read(target, property, *span, scope),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => self.evaluate_named_static_property(class_name, property, *span),
            Expr::SelfStaticProperty { property, span } => {
                self.evaluate_self_static_property(property, *span)
            }
            Expr::ParentStaticProperty { property, span } => {
                self.evaluate_parent_static_property(property, *span)
            }
            Expr::LateStaticProperty { property, span } => {
                self.evaluate_late_static_property(property, *span)
            }
            Expr::MethodCall {
                target,
                method,
                args,
                span,
            } => self.call_instance_method(target, method, args, *span, scope),
            Expr::ParentMethodCall { method, args, span } => {
                self.call_parent_method(method, args, *span, scope)
            }
            Expr::StaticMethodCall {
                class_name,
                method,
                args,
                span,
            } => self.call_named_static_method(class_name, method, args, *span, scope),
            Expr::ObjectStaticMethodCall {
                target,
                method,
                args,
                span,
            } => self.call_object_static_method(target, method, args, *span, scope),
            Expr::SelfMethodCall { method, args, span } => {
                self.call_self_method(method, args, *span, scope)
            }
            Expr::LateStaticMethodCall { method, args, span } => {
                self.call_late_static_method(method, args, *span, scope)
            }
            Expr::Call { name, args, span } => self.call_function(name, args, *span, scope),
            Expr::DynamicCall { callee, args, span } => {
                self.call_dynamic_function(callee, args, *span, scope)
            }
            Expr::InstanceOf {
                expr, class_name, ..
            } => {
                let value = self.evaluate(expr, scope)?;
                Ok(Value::Bool(self.value_instanceof(&value, class_name)))
            }
            Expr::Closure { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "closure",
                    "anonymous function values and invocation are not implemented",
                ),
            )),
            Expr::New {
                class_name,
                args,
                span,
            } => self.instantiate_object(class_name, args, *span, scope),
            Expr::Unary { op, expr, span } => {
                let value = self.evaluate(expr, scope)?;
                self.apply_unary(*op, value, *span)
            }
            Expr::Cast { kind, expr, span } => {
                let value = self.evaluate(expr, scope)?;
                self.apply_cast(*kind, value, *span)
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                if self.evaluate(condition, scope)?.is_truthy() {
                    self.evaluate(if_true, scope)
                } else {
                    self.evaluate(if_false, scope)
                }
            }
            Expr::ShortTernary {
                condition,
                if_false,
                ..
            } => {
                let condition_value = self.evaluate(condition, scope)?;
                if condition_value.is_truthy() {
                    Ok(condition_value)
                } else {
                    self.evaluate(if_false, scope)
                }
            }
            Expr::Assign { target, expr, .. } => self.evaluate_assignment(target, expr, scope),
            Expr::CompoundAssign {
                target,
                op,
                expr,
                span,
            } => self.evaluate_compound_assignment(target, *op, expr, *span, scope),
            Expr::NullCoalesceAssign { target, expr, .. } => {
                self.evaluate_null_coalesce_assignment(target, expr, scope)
            }
            Expr::IncrementDecrement {
                target,
                op,
                position,
                span,
            } => self.evaluate_increment_decrement(target, *op, *position, *span, scope),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if matches!(op, BinaryOp::NullCoalesce) {
                    return self.evaluate_null_coalescing(left, right, *span, scope);
                }
                if matches!(op, BinaryOp::LogicalAnd) {
                    let left = self.evaluate(left, scope)?;
                    if !left.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                    return Ok(Value::Bool(self.evaluate(right, scope)?.is_truthy()));
                }
                if matches!(op, BinaryOp::LogicalOr) {
                    let left = self.evaluate(left, scope)?;
                    if left.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                    return Ok(Value::Bool(self.evaluate(right, scope)?.is_truthy()));
                }
                if matches!(op, BinaryOp::LogicalXor) {
                    let left = self.evaluate(left, scope)?.is_truthy();
                    let right = self.evaluate(right, scope)?.is_truthy();
                    return Ok(Value::Bool(left ^ right));
                }
                let left = self.evaluate(left, scope)?;
                let right = self.evaluate(right, scope)?;
                self.apply_binary(*op, left, right, *span)
            }
        }
    }

    fn instantiate_object(
        &mut self,
        class_name: &str,
        args: &[Expr],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let (class_id, declared_class_name) = {
            let class = self
                .classes
                .lookup_class(class_name)
                .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
            (class.id(), class.name().to_string())
        };

        let constructor = self.resolve_instance_method(class_id, "__construct");

        let class = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata");

        let object_id = self.next_object_id;
        self.next_object_id = self
            .next_object_id
            .checked_add(1)
            .expect("object id counter fits in i64");

        let inherited_properties = self.inherited_instance_properties(class_id);
        let object = PhpObject::from_class_with_inherited_properties_with_id(
            class,
            &inherited_properties,
            object_id,
        );
        let Some((
            constructor_class_id,
            constructor_class_name,
            constructor_name,
            constructor_visibility,
            constructor_is_static,
        )) = constructor
        else {
            if !args.is_empty() {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_object_instantiation(
                        declared_class_name,
                        "constructor arguments are not implemented",
                    ),
                ));
            }
            return Ok(Value::Object(object));
        };

        if constructor_is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(
                    declared_class_name,
                    "static constructors are not implemented",
                ),
            ));
        }

        if !self.can_call_constructor(constructor_class_id, constructor_visibility) {
            let reason = match constructor_visibility {
                Visibility::Private => format!(
                    "private constructor {}::__construct() requires same-class construction context",
                    constructor_class_name
                ),
                Visibility::Protected => format!(
                    "protected constructor {}::__construct() requires same-class or child-class construction context",
                    constructor_class_name
                ),
                Visibility::Public => unreachable!("public constructors are always callable"),
            };
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(declared_class_name, reason),
            ));
        }

        let function = self
            .methods
            .get(&(constructor_class_id, constructor_name.to_ascii_lowercase()))
            .cloned()
            .expect("declared constructor metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, scope)?);
        }

        self.call_user_function_with_this(
            function,
            object.clone(),
            values,
            Some(constructor_class_id),
            Some(object.class_id()),
        )?;
        Ok(Value::Object(object))
    }

    fn inherited_instance_properties(
        &self,
        class_id: ClassId,
    ) -> Vec<PhpObjectPropertyInitializer> {
        let mut ancestors = Vec::new();
        let mut current = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata")
            .parent_id();
        while let Some(ancestor_id) = current {
            ancestors.push(ancestor_id);
            current = self
                .classes
                .get(ancestor_id)
                .expect("ancestor class id should resolve to metadata")
                .parent_id();
        }
        ancestors.reverse();

        let mut properties = Vec::new();
        for ancestor_id in ancestors {
            let ancestor = self
                .classes
                .get(ancestor_id)
                .expect("ancestor class id should resolve to metadata");
            properties.extend(ancestor.properties().iter().filter_map(|property| {
                if property.is_static() {
                    return None;
                }

                Some(PhpObjectPropertyInitializer::new(
                    ancestor.id(),
                    ancestor.name().to_string(),
                    property.clone(),
                ))
            }));
        }
        properties
    }

    fn execute_assignment(
        &mut self,
        target: &AssignTarget,
        expr: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        self.evaluate_assignment(target, expr, scope)?;
        Ok(())
    }

    fn evaluate_assignment(
        &mut self,
        target: &AssignTarget,
        expr: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.evaluate(expr, scope)?;
                scope.write_static(name, value.clone());
                Ok(value)
            }
            AssignTarget::List { names, span } => {
                self.evaluate_list_assignment(names, expr, *span, scope)
            }
            AssignTarget::ArrayIndex { name, index, span } => {
                let key = match index {
                    Some(index) => Some(self.evaluate_array_key(index, scope)?),
                    None => None,
                };
                let value = self.evaluate(expr, scope)?;
                let slot = scope.array_slot_for_static_write(name);

                if matches!(slot, Value::Null) {
                    *slot = Value::Array(PhpArray::new());
                }

                match slot {
                    Value::Array(array) => match key {
                        Some(key) => {
                            array.insert(key, value.clone());
                        }
                        None => {
                            array
                                .append(value.clone())
                                .map_err(|error| runtime_error(*span, error))?;
                        }
                    },
                    other => {
                        return Err(runtime_error(
                            *span,
                            RuntimeError::invalid_array_access(format!(
                                "cannot write offset on {}",
                                other.type_name()
                            )),
                        ));
                    }
                }

                Ok(value)
            }
            AssignTarget::Property {
                object,
                property,
                span,
            } => {
                let value = self.evaluate(expr, scope)?;
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                let slot = scope.object_slot_for_static_write(object, *span)?;

                match slot {
                    Value::Object(object) => object
                        .write_property_from_context(
                            property,
                            value.clone(),
                            current_class_id,
                            &protected_class_ids,
                        )
                        .map(|()| value)
                        .map_err(|error| runtime_error(*span, error)),
                    other => Err(runtime_error(
                        *span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot write property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                }
            }
            AssignTarget::StaticProperty {
                class_name,
                property,
                span,
            } => {
                let value = self.evaluate(expr, scope)?;
                self.write_named_static_property(class_name, property, value, *span)
            }
            AssignTarget::SelfStaticProperty { property, span } => {
                let value = self.evaluate(expr, scope)?;
                self.write_self_static_property(property, value, *span)
            }
            AssignTarget::ParentStaticProperty { property, span } => {
                let value = self.evaluate(expr, scope)?;
                self.write_parent_static_property(property, value, *span)
            }
            AssignTarget::LateStaticProperty { property, span } => {
                let value = self.evaluate(expr, scope)?;
                self.write_late_static_property(property, value, *span)
            }
        }
    }

    fn evaluate_list_assignment(
        &mut self,
        names: &[String],
        expr: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let value = self.evaluate(expr, scope)?;
        let array = match &value {
            Value::Array(array) => array,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "list()",
                        format!("right-hand side must be array, got {}", other.type_name()),
                    ),
                ));
            }
        };

        let assignments: Vec<(String, Value)> = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let element = array
                    .get(ArrayKey::Int(index as i64))
                    .cloned()
                    .unwrap_or(Value::Null);
                (name.clone(), element)
            })
            .collect();

        for (name, element) in assignments {
            scope.write_static(&name, element);
        }

        Ok(value)
    }

    fn execute_compound_assignment(
        &mut self,
        target: &AssignTarget,
        op: CompoundAssignOp,
        expr: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        self.evaluate_compound_assignment(target, op, expr, span, scope)
            .map(|_| ())
    }

    fn evaluate_compound_assignment(
        &mut self,
        target: &AssignTarget,
        op: CompoundAssignOp,
        expr: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let (place, left) = self.read_compound_assignment_left(target, span, scope)?;
        let right = self.evaluate(expr, scope)?;
        let value = Self::apply_compound_assignment_op(left, op, &right, span)?;
        self.write_compound_assignment_place(place, value.clone(), span, scope)?;
        Ok(value)
    }

    fn read_compound_assignment_left(
        &mut self,
        target: &AssignTarget,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<(CompoundAssignmentPlace, Value)> {
        match target {
            AssignTarget::Variable { name, .. } => Ok((
                CompoundAssignmentPlace::Variable(name.clone()),
                scope.read_static(name, span)?,
            )),
            AssignTarget::List { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "list destructuring targets are not implemented",
                ),
            )),
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                ..
            } => {
                let key = self.evaluate_array_key(index, scope)?;
                match scope.read_named(name) {
                    Some(Value::Array(array)) => {
                        let value = array.get(key.clone()).cloned().ok_or_else(|| {
                            runtime_error(
                                span,
                                RuntimeError::undefined_array_key(key.diagnostic_key()),
                            )
                        })?;
                        Ok((
                            CompoundAssignmentPlace::ArrayIndex {
                                name: name.clone(),
                                key,
                            },
                            value,
                        ))
                    }
                    Some(other) => Err(runtime_error(
                        span,
                        RuntimeError::invalid_array_access(format!(
                            "cannot read offset from {}",
                            other.type_name()
                        )),
                    )),
                    None => Err(runtime_error(span, RuntimeError::undefined_variable(name))),
                }
            }
            AssignTarget::ArrayIndex { index: None, .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "append-offset targets are not implemented",
                ),
            )),
            AssignTarget::Property {
                object, property, ..
            } => match scope.read_named(object) {
                Some(Value::Object(value)) => {
                    let (current_class_id, protected_class_ids) =
                        self.current_property_access_context();
                    let left = value
                        .read_property_from_context(
                            property,
                            current_class_id,
                            &protected_class_ids,
                        )
                        .map_err(|error| runtime_error(span, error))?;
                    Ok((
                        CompoundAssignmentPlace::ObjectProperty {
                            object: object.clone(),
                            property: property.clone(),
                        },
                        left,
                    ))
                }
                Some(other) => Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} from {}",
                        other.type_name()
                    )),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_variable(object),
                )),
            },
            AssignTarget::StaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. } => {
                let (declaring_class_id, property, value) =
                    self.read_static_property_target(target, span)?;
                Ok((
                    CompoundAssignmentPlace::StaticProperty {
                        declaring_class_id,
                        property,
                    },
                    value,
                ))
            }
        }
    }

    fn write_compound_assignment_place(
        &mut self,
        place: CompoundAssignmentPlace,
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        match place {
            CompoundAssignmentPlace::Variable(name) => {
                scope.write_static(&name, value);
                Ok(())
            }
            CompoundAssignmentPlace::ArrayIndex { name, key } => {
                match scope.read_named(&name).cloned() {
                    Some(Value::Array(mut array)) => {
                        array.insert(key, value);
                        scope.write_static(&name, Value::Array(array));
                        Ok(())
                    }
                    Some(other) => Err(runtime_error(
                        span,
                        RuntimeError::invalid_array_access(format!(
                            "cannot write offset on {}",
                            other.type_name()
                        )),
                    )),
                    None => Err(runtime_error(span, RuntimeError::undefined_variable(name))),
                }
            }
            CompoundAssignmentPlace::ObjectProperty { object, property } => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                let slot = scope.object_slot_for_static_write(&object, span)?;

                match slot {
                    Value::Object(object) => object
                        .write_property_from_context(
                            &property,
                            value,
                            current_class_id,
                            &protected_class_ids,
                        )
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot write property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                }
            }
            CompoundAssignmentPlace::StaticProperty {
                declaring_class_id,
                property,
            } => {
                self.static_properties
                    .insert((declaring_class_id, property), value);
                Ok(())
            }
        }
    }

    fn apply_compound_assignment_op(
        left: Value,
        op: CompoundAssignOp,
        right: &Value,
        span: Span,
    ) -> CompileResult<Value> {
        let value = match op {
            CompoundAssignOp::Add => left.php_add(right),
            CompoundAssignOp::Sub => left.php_sub(right),
            CompoundAssignOp::Mul => left.php_mul(right),
            CompoundAssignOp::Div => left.php_div(right),
            CompoundAssignOp::Mod => left.php_mod(right),
            CompoundAssignOp::Concat => left.php_concat(right),
            CompoundAssignOp::BitwiseAnd => left.php_bitwise_and(right),
            CompoundAssignOp::BitwiseOr => left.php_bitwise_or(right),
            CompoundAssignOp::BitwiseXor => left.php_bitwise_xor(right),
            CompoundAssignOp::ShiftLeft => left.php_shift_left(right),
            CompoundAssignOp::ShiftRight => left.php_shift_right(right),
        };

        value.map_err(|error| runtime_error(span, error))
    }

    fn execute_increment_decrement(
        &mut self,
        target: &AssignTarget,
        op: IncrementDecrementOp,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (place, value) = self.read_increment_decrement_left(target, span, scope)?;
        let updated = Self::increment_decrement_value(value, op, span)?;
        self.write_compound_assignment_place(place, updated, span, scope)?;
        Ok(())
    }

    fn evaluate_increment_decrement(
        &mut self,
        target: &AssignTarget,
        op: IncrementDecrementOp,
        position: IncrementDecrementPosition,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let (place, previous) = self.read_increment_decrement_left(target, span, scope)?;
        let updated = Self::increment_decrement_value(previous.clone(), op, span)?;
        self.write_compound_assignment_place(place, updated.clone(), span, scope)?;

        Ok(match position {
            IncrementDecrementPosition::Pre => updated,
            IncrementDecrementPosition::Post => previous,
        })
    }

    fn read_increment_decrement_left(
        &mut self,
        target: &AssignTarget,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<(CompoundAssignmentPlace, Value)> {
        match target {
            AssignTarget::Variable { name, .. } => Ok((
                CompoundAssignmentPlace::Variable(name.clone()),
                scope.read_static(name, span)?,
            )),
            AssignTarget::List { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "list destructuring targets are not implemented",
                ),
            )),
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                ..
            } => {
                let key = self.evaluate_array_key(index, scope)?;
                match scope.read_named(name) {
                    Some(Value::Array(array)) => {
                        let value = array.get(key.clone()).cloned().ok_or_else(|| {
                            runtime_error(
                                span,
                                RuntimeError::undefined_array_key(key.diagnostic_key()),
                            )
                        })?;
                        Ok((
                            CompoundAssignmentPlace::ArrayIndex {
                                name: name.clone(),
                                key,
                            },
                            value,
                        ))
                    }
                    Some(other) => Err(runtime_error(
                        span,
                        RuntimeError::invalid_array_access(format!(
                            "cannot read offset from {}",
                            other.type_name()
                        )),
                    )),
                    None => Err(runtime_error(span, RuntimeError::undefined_variable(name))),
                }
            }
            AssignTarget::ArrayIndex { index: None, .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "append-offset targets are not implemented",
                ),
            )),
            AssignTarget::Property {
                object, property, ..
            } => match scope.read_named(object) {
                Some(Value::Object(value)) => {
                    let (current_class_id, protected_class_ids) =
                        self.current_property_access_context();
                    let left = value
                        .read_property_from_context(
                            property,
                            current_class_id,
                            &protected_class_ids,
                        )
                        .map_err(|error| runtime_error(span, error))?;
                    Ok((
                        CompoundAssignmentPlace::ObjectProperty {
                            object: object.clone(),
                            property: property.clone(),
                        },
                        left,
                    ))
                }
                Some(other) => Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} from {}",
                        other.type_name()
                    )),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_variable(object),
                )),
            },
            AssignTarget::StaticProperty { .. }
            | AssignTarget::SelfStaticProperty { .. }
            | AssignTarget::ParentStaticProperty { .. }
            | AssignTarget::LateStaticProperty { .. } => {
                let (declaring_class_id, property, value) =
                    self.read_static_property_target(target, span)?;
                Ok((
                    CompoundAssignmentPlace::StaticProperty {
                        declaring_class_id,
                        property,
                    },
                    value,
                ))
            }
        }
    }

    fn increment_decrement_value(
        value: Value,
        op: IncrementDecrementOp,
        span: Span,
    ) -> CompileResult<Value> {
        Ok(match (value, op) {
            (Value::Int(value), IncrementDecrementOp::Increment) => {
                Value::Int(value.wrapping_add(1))
            }
            (Value::Int(value), IncrementDecrementOp::Decrement) => {
                Value::Int(value.wrapping_sub(1))
            }
            (Value::Float(value), IncrementDecrementOp::Increment) => Value::Float(value + 1.0),
            (Value::Float(value), IncrementDecrementOp::Decrement) => Value::Float(value - 1.0),
            (other, _) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "increment/decrement",
                        format!(
                            "only int and float variables, array offsets, object properties, or static properties are implemented, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        })
    }

    fn evaluate_null_coalesce_assignment(
        &mut self,
        target: &AssignTarget,
        expr: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        match target {
            AssignTarget::Variable { name, .. } => {
                if let Some(value) = scope.read_named(name) {
                    if !matches!(value, Value::Null) {
                        return Ok(value.clone());
                    }
                }
                let value = self.evaluate(expr, scope)?;
                scope.write_static(name, value.clone());
                Ok(value)
            }
            AssignTarget::List { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "??=",
                    "list destructuring targets are not implemented",
                ),
            )),
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                span,
            } => {
                let key = self.evaluate_array_key(index, scope)?;
                let should_assign = match scope.read_named(name) {
                    Some(Value::Array(array)) => match array.get(key.clone()) {
                        Some(value) if !matches!(value, Value::Null) => {
                            return Ok(value.clone());
                        }
                        _ => true,
                    },
                    Some(Value::Null) | None => true,
                    Some(other) => {
                        return Err(runtime_error(
                            *span,
                            RuntimeError::invalid_array_access(format!(
                                "cannot write offset on {}",
                                other.type_name()
                            )),
                        ));
                    }
                };

                if should_assign {
                    let value = self.evaluate(expr, scope)?;
                    let slot = scope.array_slot_for_static_write(name);

                    if matches!(slot, Value::Null) {
                        *slot = Value::Array(PhpArray::new());
                    }

                    match slot {
                        Value::Array(array) => {
                            array.insert(key, value.clone());
                            Ok(value)
                        }
                        other => Err(runtime_error(
                            *span,
                            RuntimeError::invalid_array_access(format!(
                                "cannot write offset on {}",
                                other.type_name()
                            )),
                        )),
                    }
                } else {
                    unreachable!("non-null array entries return before assignment")
                }
            }
            AssignTarget::ArrayIndex {
                index: None, span, ..
            } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call("??=", "append-offset targets are not implemented"),
            )),
            AssignTarget::Property {
                object,
                property,
                span,
            } => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                let should_assign = match scope.read_named(object) {
                    Some(Value::Object(object)) => {
                        match object
                            .read_property_for_isset_from_context(
                                property,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(*span, error))?
                        {
                            Some(value) if !matches!(value, Value::Null) => {
                                return Ok(value.clone());
                            }
                            Some(_) | None => true,
                        }
                    }
                    Some(_) | None => true,
                };

                if should_assign {
                    let value = self.evaluate(expr, scope)?;
                    let slot = scope.object_slot_for_static_write(object, *span)?;

                    match slot {
                        Value::Object(object) => object
                            .write_property_from_context(
                                property,
                                value.clone(),
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map(|()| value)
                            .map_err(|error| runtime_error(*span, error)),
                        other => Err(runtime_error(
                            *span,
                            RuntimeError::invalid_property_access(format!(
                                "cannot write property ${property} on {}",
                                other.type_name()
                            )),
                        )),
                    }
                } else {
                    unreachable!("non-null object properties return before assignment")
                }
            }
            AssignTarget::StaticProperty { span, .. }
            | AssignTarget::SelfStaticProperty { span, .. }
            | AssignTarget::ParentStaticProperty { span, .. }
            | AssignTarget::LateStaticProperty { span, .. } => {
                let (declaring_class_id, property, current) =
                    self.read_static_property_target(target, *span)?;
                if !matches!(current, Value::Null) {
                    return Ok(current);
                }

                let value = self.evaluate(expr, scope)?;
                self.static_properties
                    .insert((declaring_class_id, property), value.clone());
                Ok(value)
            }
        }
    }

    fn evaluate_global_constant(&self, name: &str, span: Span) -> CompileResult<Value> {
        self.constants
            .get(name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_constant(name)))
    }

    fn evaluate_array(
        &mut self,
        items: &[ArrayItem],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let mut array = PhpArray::new();

        for item in items {
            let key = match &item.key {
                Some(expr) => Some(self.evaluate_array_key(expr, scope)?),
                None => None,
            };
            let value = self.evaluate(&item.value, scope)?;

            match key {
                Some(key) => {
                    array.insert(key, value);
                }
                None => {
                    array
                        .append(value)
                        .map_err(|error| runtime_error(span, error))?;
                }
            }
        }

        Ok(Value::Array(array))
    }

    fn magic_dir_value(&self) -> String {
        let Some(source_file) = &self.source_file else {
            return String::new();
        };
        let Some(parent) = Path::new(source_file).parent() else {
            return ".".to_string();
        };
        if parent.as_os_str().is_empty() {
            ".".to_string()
        } else {
            parent.to_string_lossy().into_owned()
        }
    }

    fn evaluate_array_index(
        &mut self,
        target: &Expr,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let target_value = self.evaluate(target, scope)?;
        let key = self.evaluate_array_key(index, scope)?;

        match target_value {
            Value::Array(array) => array.get(key.clone()).cloned().ok_or_else(|| {
                runtime_error(
                    span,
                    RuntimeError::undefined_array_key(key.diagnostic_key()),
                )
            }),
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot read offset from {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn evaluate_null_coalescing(
        &mut self,
        left: &Expr,
        right: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let value = match left {
            Expr::Variable(name, _) => scope
                .read_named(name)
                .cloned()
                .filter(|value| !matches!(value, Value::Null)),
            Expr::Index { target, index, .. } => {
                self.evaluate_direct_array_offset_for_null_coalescing(target, index, scope)?
            }
            Expr::Property {
                target,
                property,
                span,
            } => self.evaluate_direct_object_property_for_null_coalescing(
                target, property, *span, scope,
            )?,
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => self
                .evaluate_named_static_property_for_null_coalescing(class_name, property, *span)?,
            Expr::SelfStaticProperty { property, span } => {
                self.evaluate_self_static_property_for_null_coalescing(property, *span)?
            }
            Expr::ParentStaticProperty { property, span } => {
                self.evaluate_parent_static_property_for_null_coalescing(property, *span)?
            }
            Expr::LateStaticProperty { property, span } => {
                self.evaluate_late_static_property_for_null_coalescing(property, *span)?
            }
            _ => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "??",
                        "left operand must be a direct variable, direct array offset, direct object property, or supported static property in the current subset",
                    ),
                ));
            }
        };

        match value {
            Some(value) => Ok(value),
            None => self.evaluate(right, scope),
        }
    }

    fn evaluate_direct_array_offset_for_null_coalescing(
        &mut self,
        target: &Expr,
        index: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<Option<Value>> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "??",
                        "left operand must be a direct variable, direct array offset, direct object property, or supported static property in the current subset",
                ),
            ));
        };

        let key = self.evaluate_array_key(index, scope)?;
        match scope.read_named(name).cloned() {
            Some(Value::Array(array)) => Ok(array
                .get(key)
                .cloned()
                .filter(|value| !matches!(value, Value::Null))),
            Some(_) | None => Ok(None),
        }
    }

    fn evaluate_direct_object_property_for_null_coalescing(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Option<Value>> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "??",
                        "left operand must be a direct variable, direct array offset, direct object property, or supported static property in the current subset",
                ),
            ));
        };

        match scope.read_named(name) {
            Some(Value::Object(object)) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                object
                    .read_property_for_isset_from_context(
                        property,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map(|value| value.filter(|value| !matches!(value, Value::Null)))
                    .map_err(|error| runtime_error(span, error))
            }
            Some(_) | None => Ok(None),
        }
    }

    fn evaluate_named_static_property_for_null_coalescing(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        self.read_resolved_static_property_for_isset(class_id, class_name, property, span)
    }

    fn evaluate_self_static_property_for_null_coalescing(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::${property}"),
                    "self static property access requires instance method context",
                ),
            ));
        };

        let class_name = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata")
            .name()
            .to_string();
        self.read_resolved_static_property_for_isset(current_class_id, &class_name, property, span)
    }

    fn evaluate_parent_static_property_for_null_coalescing(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let (parent_class_id, parent_class_name) =
            self.resolve_parent_static_property_context(property, span)?;
        self.read_resolved_static_property_for_isset(
            parent_class_id,
            &parent_class_name,
            property,
            span,
        )
    }

    fn evaluate_late_static_property_for_null_coalescing(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let (called_class_id, called_class_name) =
            self.resolve_late_static_property_context(property, span)?;
        self.read_resolved_static_property_for_isset(
            called_class_id,
            &called_class_name,
            property,
            span,
        )
    }

    fn evaluate_property_read(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let target_value = self.evaluate(target, scope)?;

        match target_value {
            Value::Object(object) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                object
                    .read_property_from_context(property, current_class_id, &protected_class_ids)
                    .map_err(|error| runtime_error(span, error))
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_property_access(format!(
                    "cannot read property ${property} from {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn call_instance_method(
        &mut self,
        target: &Expr,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let target_value = self.evaluate(target, caller_scope)?;
        let object = match target_value {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        format!("{method_name}()"),
                        format!("receiver must be object, got {}", other.type_name()),
                    ),
                ));
            }
        };

        let (class_id, class_name, resolved_method_name, visibility, is_static) = {
            let receiver_class = self
                .classes
                .get(object.class_id())
                .expect("object class id should resolve to class metadata");
            let Some(method) = self.resolve_instance_method(object.class_id(), method_name) else {
                return Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{}::{method_name}()",
                        receiver_class.name()
                    )),
                ));
            };
            method
        };

        if is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "static method dispatch through object receivers is not implemented",
                ),
            ));
        }

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        let function = self
            .methods
            .get(&(class_id, resolved_method_name.to_ascii_lowercase()))
            .cloned()
            .expect("declared method metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        let called_class_id = object.class_id();
        self.call_user_function_with_this(
            function,
            object,
            values,
            Some(class_id),
            Some(called_class_id),
        )
    }

    fn call_parent_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::{method_name}()"),
                    "parent method calls require instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        let Some(parent_class_id) = current_class.parent_id() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::{method_name}()"),
                    "parent method calls require a parent class",
                ),
            ));
        };

        let parent_class = self
            .classes
            .get(parent_class_id)
            .expect("parent class id should resolve to class metadata");
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(parent_class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!(
                    "{}::{method_name}()",
                    parent_class.name()
                )),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        let function = self
            .methods
            .get(&(class_id, resolved_method_name.to_ascii_lowercase()))
            .cloned()
            .expect("declared parent method metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        let called_class_id = self
            .called_class_context
            .last()
            .copied()
            .unwrap_or(current_class_id);

        if is_static {
            self.call_user_function_with_checked_values(
                function,
                values,
                None,
                Some(class_id),
                Some(called_class_id),
            )
        } else {
            let this_object = match caller_scope.read_named("this") {
                Some(Value::Object(object)) => object.clone(),
                _ => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "non-static method dispatch through parent:: requires current $this object context",
                        ),
                    ));
                }
            };
            self.call_user_function_with_this(
                function,
                this_object,
                values,
                Some(class_id),
                Some(called_class_id),
            )
        }
    }

    fn call_named_static_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        let receiver_class = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata");
        let Some((
            declaring_class_id,
            declaring_class_name,
            _resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!(
                    "{}::{method_name}()",
                    receiver_class.name()
                )),
            ));
        };

        if is_static {
            self.ensure_instance_method_visible(
                declaring_class_id,
                &declaring_class_name,
                method_name,
                visibility,
                span,
            )?;

            let function = self
                .methods
                .get(&(
                    declaring_class_id,
                    _resolved_method_name.to_ascii_lowercase(),
                ))
                .cloned()
                .expect("declared static method metadata should have a stored function body");
            let function = function.as_ref();
            ensure_user_function_arity(function, args.len(), span)?;
            ensure_supported_function_signature(function, span)?;
            self.ensure_user_function_call_depth(function, span)?;

            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(self.evaluate(arg, caller_scope)?);
            }

            return self.call_user_function_with_checked_values(
                function,
                values,
                None,
                Some(declaring_class_id),
                Some(class_id),
            );
        }

        self.ensure_instance_method_visible(
            declaring_class_id,
            &declaring_class_name,
            method_name,
            visibility,
            span,
        )?;

        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                format!("{declaring_class_name}::{method_name}()"),
                "non-static method dispatch through named static receivers is not implemented",
            ),
        ))
    }

    fn call_object_static_method(
        &mut self,
        target: &Expr,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let target_value = self.evaluate(target, caller_scope)?;
        let receiver_class_id = match target_value {
            Value::Object(object) => object.class_id(),
            Value::String(class_name) => self
                .classes
                .lookup_class_id(&class_name)
                .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(&class_name)))?,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        format!("{method_name}()"),
                        format!(
                            "dynamic static method receiver must be object or class string, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };

        let receiver_class = self
            .classes
            .get(receiver_class_id)
            .expect("receiver class id should resolve to class metadata");
        let Some((
            declaring_class_id,
            declaring_class_name,
            resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(receiver_class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!(
                    "{}::{method_name}()",
                    receiver_class.name()
                )),
            ));
        };

        self.ensure_instance_method_visible(
            declaring_class_id,
            &declaring_class_name,
            method_name,
            visibility,
            span,
        )?;

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::{method_name}()"),
                    "non-static method dispatch through dynamic static receivers is not implemented",
                ),
            ));
        }

        let function = self
            .methods
            .get(&(
                declaring_class_id,
                resolved_method_name.to_ascii_lowercase(),
            ))
            .cloned()
            .expect("declared object static method metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        self.call_user_function_with_checked_values(
            function,
            values,
            None,
            Some(declaring_class_id),
            Some(receiver_class_id),
        )
    }

    fn evaluate_self_class_name_constant(&self, span: Span) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "self::class",
                    "self::class requires instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        Ok(Value::String(current_class.name().to_string()))
    }

    fn evaluate_parent_class_name_constant(&self, span: Span) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "parent::class",
                    "parent::class requires instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        let Some(parent_class_id) = current_class.parent_id() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "parent::class",
                    "parent::class requires a parent class",
                ),
            ));
        };

        let parent_class = self
            .classes
            .get(parent_class_id)
            .expect("parent class id should resolve to class metadata");
        Ok(Value::String(parent_class.name().to_string()))
    }

    fn evaluate_static_class_name_constant(&self, span: Span) -> CompileResult<Value> {
        let Some(called_class_id) = self.called_class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "static::class",
                    "static::class requires method or static class context",
                ),
            ));
        };

        let called_class = self
            .classes
            .get(called_class_id)
            .expect("called class context should resolve to class metadata");
        Ok(Value::String(called_class.name().to_string()))
    }

    fn evaluate_named_static_property(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        self.read_resolved_static_property(class_id, class_name, property, span)
    }

    fn evaluate_self_static_property(&self, property: &str, span: Span) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::${property}"),
                    "self static property access requires instance method context",
                ),
            ));
        };

        let class_name = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata")
            .name()
            .to_string();
        self.read_resolved_static_property(current_class_id, &class_name, property, span)
    }

    fn evaluate_parent_static_property(&self, property: &str, span: Span) -> CompileResult<Value> {
        let (parent_class_id, parent_class_name) =
            self.resolve_parent_static_property_context(property, span)?;
        self.read_resolved_static_property(parent_class_id, &parent_class_name, property, span)
    }

    fn evaluate_late_static_property(&self, property: &str, span: Span) -> CompileResult<Value> {
        let (called_class_id, called_class_name) =
            self.resolve_late_static_property_context(property, span)?;
        self.read_resolved_static_property(called_class_id, &called_class_name, property, span)
    }

    fn write_named_static_property(
        &mut self,
        class_name: &str,
        property: &str,
        value: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        self.write_resolved_static_property(class_id, class_name, property, value, span)
    }

    fn write_self_static_property(
        &mut self,
        property: &str,
        value: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::${property}"),
                    "self static property access requires instance method context",
                ),
            ));
        };

        let class_name = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata")
            .name()
            .to_string();
        self.write_resolved_static_property(current_class_id, &class_name, property, value, span)
    }

    fn write_parent_static_property(
        &mut self,
        property: &str,
        value: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let (parent_class_id, parent_class_name) =
            self.resolve_parent_static_property_context(property, span)?;
        self.write_resolved_static_property(
            parent_class_id,
            &parent_class_name,
            property,
            value,
            span,
        )
    }

    fn write_late_static_property(
        &mut self,
        property: &str,
        value: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let (called_class_id, called_class_name) =
            self.resolve_late_static_property_context(property, span)?;
        self.write_resolved_static_property(
            called_class_id,
            &called_class_name,
            property,
            value,
            span,
        )
    }

    fn read_static_property_target(
        &self,
        target: &AssignTarget,
        span: Span,
    ) -> CompileResult<(ClassId, String, Value)> {
        let (class_id, class_name, property) = match target {
            AssignTarget::StaticProperty {
                class_name,
                property,
                ..
            } => {
                let class_id = self.classes.lookup_class_id(class_name).ok_or_else(|| {
                    runtime_error(span, RuntimeError::undefined_class(class_name))
                })?;
                (class_id, class_name.clone(), property.clone())
            }
            AssignTarget::SelfStaticProperty { property, .. } => {
                let Some(current_class_id) = self.class_context.last().copied() else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("self::${property}"),
                            "self static property access requires instance method context",
                        ),
                    ));
                };
                let class_name = self
                    .classes
                    .get(current_class_id)
                    .expect("active class context should resolve to class metadata")
                    .name()
                    .to_string();
                (current_class_id, class_name, property.clone())
            }
            AssignTarget::ParentStaticProperty { property, .. } => {
                let (parent_class_id, parent_class_name) =
                    self.resolve_parent_static_property_context(property, span)?;
                (parent_class_id, parent_class_name, property.clone())
            }
            AssignTarget::LateStaticProperty { property, .. } => {
                let (called_class_id, called_class_name) =
                    self.resolve_late_static_property_context(property, span)?;
                (called_class_id, called_class_name, property.clone())
            }
            _ => unreachable!("static property target helper called for non-static target"),
        };

        let (declaring_class_id, _declaring_class_name) =
            self.resolve_static_property_storage(class_id, &class_name, &property, span)?;
        let value = self
            .static_properties
            .get(&(declaring_class_id, property.clone()))
            .cloned()
            .unwrap_or(Value::Null);

        Ok((declaring_class_id, property, value))
    }

    fn resolve_parent_static_property_context(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<(ClassId, String)> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::${property}"),
                    "parent static property access requires instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        let Some(parent_class_id) = current_class.parent_id() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::${property}"),
                    "parent static property access requires a parent class",
                ),
            ));
        };
        let parent_class_name = self
            .classes
            .get(parent_class_id)
            .expect("parent class id should resolve to class metadata")
            .name()
            .to_string();

        Ok((parent_class_id, parent_class_name))
    }

    fn resolve_late_static_property_context(
        &self,
        property: &str,
        span: Span,
    ) -> CompileResult<(ClassId, String)> {
        let Some(called_class_id) = self.called_class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("static::${property}"),
                    "static property access requires method or static class context",
                ),
            ));
        };

        let called_class_name = self
            .classes
            .get(called_class_id)
            .expect("called class context should resolve to class metadata")
            .name()
            .to_string();

        Ok((called_class_id, called_class_name))
    }

    fn read_resolved_static_property(
        &self,
        class_id: ClassId,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let (declaring_class_id, _declaring_class_name) =
            self.resolve_static_property_storage(class_id, class_name, property, span)?;

        Ok(self
            .static_properties
            .get(&(declaring_class_id, property.to_string()))
            .cloned()
            .unwrap_or(Value::Null))
    }

    fn read_resolved_static_property_for_isset(
        &self,
        class_id: ClassId,
        _class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let Some((declaring_class_id, declaring_class_name, visibility)) =
            self.resolve_static_property(class_id, property)
        else {
            return Ok(None);
        };

        self.ensure_static_property_visible(
            declaring_class_id,
            &declaring_class_name,
            property,
            visibility,
            span,
        )?;

        Ok(self
            .static_properties
            .get(&(declaring_class_id, property.to_string()))
            .cloned()
            .filter(|value| !matches!(value, Value::Null)))
    }

    fn write_resolved_static_property(
        &mut self,
        class_id: ClassId,
        class_name: &str,
        property: &str,
        value: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let (declaring_class_id, _declaring_class_name) =
            self.resolve_static_property_storage(class_id, class_name, property, span)?;

        self.static_properties
            .insert((declaring_class_id, property.to_string()), value.clone());
        Ok(value)
    }

    fn resolve_static_property_storage(
        &self,
        class_id: ClassId,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<(ClassId, String)> {
        let (declaring_class_id, declaring_class_name, visibility) = self
            .resolve_static_property(class_id, property)
            .ok_or_else(|| {
                runtime_error(span, RuntimeError::undefined_property(class_name, property))
            })?;

        self.ensure_static_property_visible(
            declaring_class_id,
            &declaring_class_name,
            property,
            visibility,
            span,
        )?;

        Ok((declaring_class_id, declaring_class_name))
    }

    fn resolve_static_property(
        &self,
        class_id: ClassId,
        property: &str,
    ) -> Option<(ClassId, String, Visibility)> {
        let mut current = Some(class_id);
        while let Some(current_id) = current {
            let class = self
                .classes
                .get(current_id)
                .expect("class id should resolve to class metadata");
            if let Some(metadata) = class.property(property) {
                if metadata.is_static() {
                    return Some((current_id, class.name().to_string(), metadata.visibility()));
                }
                return None;
            }
            current = class.parent_id();
        }

        None
    }

    fn ensure_static_property_visible(
        &self,
        declaring_class_id: ClassId,
        declaring_class_name: &str,
        property: &str,
        visibility: Visibility,
        span: Span,
    ) -> CompileResult<()> {
        match visibility {
            Visibility::Public => Ok(()),
            Visibility::Private
                if self.class_context.last().copied() == Some(declaring_class_id) =>
            {
                Ok(())
            }
            Visibility::Protected
                if self
                    .class_context
                    .last()
                    .copied()
                    .is_some_and(|current_id| {
                        current_id == declaring_class_id
                            || self.classes.is_subclass_of(current_id, declaring_class_id)
                    }) =>
            {
                Ok(())
            }
            Visibility::Private => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::${property}"),
                    "private static property is not visible from the current class context",
                ),
            )),
            Visibility::Protected => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::${property}"),
                    "protected static property is not visible from the current class context",
                ),
            )),
        }
    }

    fn evaluate_named_class_constant(
        &mut self,
        class_name: &str,
        constant: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        self.evaluate_resolved_class_constant(class_id, class_name, constant, span)
    }

    fn evaluate_self_class_constant(&mut self, constant: &str, span: Span) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::{constant}"),
                    "self class constant access requires instance method context",
                ),
            ));
        };

        let class_name = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata")
            .name()
            .to_string();
        self.evaluate_resolved_class_constant(current_class_id, &class_name, constant, span)
    }

    fn evaluate_parent_class_constant(
        &mut self,
        constant: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::{constant}"),
                    "parent class constant access requires instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        let Some(parent_class_id) = current_class.parent_id() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("parent::{constant}"),
                    "parent class constant access requires a parent class",
                ),
            ));
        };

        let parent_class_name = self
            .classes
            .get(parent_class_id)
            .expect("parent class id should resolve to class metadata")
            .name()
            .to_string();
        self.evaluate_resolved_class_constant(parent_class_id, &parent_class_name, constant, span)
    }

    fn evaluate_late_static_class_constant(
        &mut self,
        constant: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let Some(called_class_id) = self.called_class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("static::{constant}"),
                    "static class constant access requires method or static class context",
                ),
            ));
        };

        let called_class_name = self
            .classes
            .get(called_class_id)
            .expect("called class context should resolve to class metadata")
            .name()
            .to_string();
        self.evaluate_resolved_class_constant(called_class_id, &called_class_name, constant, span)
    }

    fn evaluate_resolved_class_constant(
        &mut self,
        class_id: ClassId,
        class_name: &str,
        constant: &str,
        span: Span,
    ) -> CompileResult<Value> {
        let Some((declaring_class_id, declaring_class_name, visibility, value)) =
            self.resolve_class_constant(class_id, constant)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_constant(format!("{class_name}::{constant}")),
            ));
        };

        self.ensure_class_constant_visible(
            declaring_class_id,
            &declaring_class_name,
            constant,
            visibility,
            span,
        )?;

        let mut constant_scope = SymbolTable::new();
        let value = self.evaluate(&value, &mut constant_scope)?;
        if let Some(type_name) = unsupported_runtime_constant_value_type(&value) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::{constant}"),
                    format!(
                        "class constant value must be null, bool, int, float, string, or array values in the current subset, got {type_name}"
                    ),
                ),
            ));
        }

        Ok(value)
    }

    fn resolve_class_constant(
        &self,
        class_id: ClassId,
        constant: &str,
    ) -> Option<(ClassId, String, Visibility, Expr)> {
        let mut current = Some(class_id);
        while let Some(current_id) = current {
            let class = self
                .classes
                .get(current_id)
                .expect("class id should resolve to class metadata");
            if let Some(metadata) = class.constant(constant) {
                let value = self
                    .class_constants
                    .get(&(current_id, metadata.name().to_string()))
                    .expect("class constant metadata should have stored value")
                    .value
                    .clone();
                return Some((
                    current_id,
                    class.name().to_string(),
                    metadata.visibility(),
                    value,
                ));
            }
            current = class.parent_id();
        }

        None
    }

    fn ensure_class_constant_visible(
        &self,
        declaring_class_id: ClassId,
        declaring_class_name: &str,
        constant: &str,
        visibility: Visibility,
        span: Span,
    ) -> CompileResult<()> {
        match visibility {
            Visibility::Public => Ok(()),
            Visibility::Private
                if self.class_context.last().copied() == Some(declaring_class_id) =>
            {
                Ok(())
            }
            Visibility::Protected
                if self
                    .class_context
                    .last()
                    .copied()
                    .is_some_and(|current_id| {
                        current_id == declaring_class_id
                            || self.classes.is_subclass_of(current_id, declaring_class_id)
                    }) =>
            {
                Ok(())
            }
            Visibility::Private => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::{constant}"),
                    "private class constant is not visible from the current class context",
                ),
            )),
            Visibility::Protected => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{declaring_class_name}::{constant}"),
                    "protected class constant is not visible from the current class context",
                ),
            )),
        }
    }

    fn call_self_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("self::{method_name}()"),
                    "self method calls require instance method context",
                ),
            ));
        };

        let current_class = self
            .classes
            .get(current_class_id)
            .expect("active class context should resolve to class metadata");
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(current_class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!(
                    "{}::{method_name}()",
                    current_class.name()
                )),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        let function = self
            .methods
            .get(&(class_id, resolved_method_name.to_ascii_lowercase()))
            .cloned()
            .expect("declared self method metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        let called_class_id = self
            .called_class_context
            .last()
            .copied()
            .unwrap_or(current_class_id);

        if is_static {
            self.call_user_function_with_checked_values(
                function,
                values,
                None,
                Some(class_id),
                Some(called_class_id),
            )
        } else {
            let this_object = match caller_scope.read_named("this") {
                Some(Value::Object(object)) => object.clone(),
                _ => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "non-static method dispatch through self:: requires current $this object context",
                        ),
                    ));
                }
            };
            self.call_user_function_with_this(
                function,
                this_object,
                values,
                Some(class_id),
                Some(called_class_id),
            )
        }
    }

    fn call_late_static_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let Some(called_class_id) = self.called_class_context.last().copied() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("static::{method_name}()"),
                    "static method calls require method or static class context",
                ),
            ));
        };

        let called_class = self
            .classes
            .get(called_class_id)
            .expect("called class context should resolve to class metadata");
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(called_class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!(
                    "{}::{method_name}()",
                    called_class.name()
                )),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "non-static method dispatch through static:: is not implemented",
                ),
            ));
        }

        let function = self
            .methods
            .get(&(class_id, resolved_method_name.to_ascii_lowercase()))
            .cloned()
            .expect("declared late static method metadata should have a stored function body");
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        self.call_user_function_with_checked_values(
            function,
            values,
            None,
            Some(class_id),
            Some(called_class_id),
        )
    }

    fn resolve_instance_method(
        &self,
        class_id: ClassId,
        method_name: &str,
    ) -> Option<(ClassId, String, String, Visibility, bool)> {
        let mut current = Some(class_id);
        while let Some(current_id) = current {
            let class = self
                .classes
                .get(current_id)
                .expect("class id should resolve to class metadata");
            if let Some(method) = class.method(method_name) {
                return Some((
                    class.id(),
                    class.name().to_string(),
                    method.name().to_string(),
                    method.visibility(),
                    method.is_static(),
                ));
            }
            current = class.parent_id();
        }
        None
    }

    fn class_has_property_in_hierarchy(&self, class_id: ClassId, property_name: &str) -> bool {
        let mut current = Some(class_id);
        while let Some(current_id) = current {
            let class = self
                .classes
                .get(current_id)
                .expect("class id should resolve to class metadata");
            if let Some(property) = class.property(property_name) {
                if current_id == class_id || property.visibility() != Visibility::Private {
                    return true;
                }
            }
            current = class.parent_id();
        }
        false
    }

    fn append_public_class_vars(&self, class_id: ClassId, properties: &mut PhpArray) {
        let class = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata");
        for property in class.properties() {
            if property.visibility() == Visibility::Public {
                properties.insert(ArrayKey::from(property.name()), Value::Null);
            }
        }
        if let Some(parent_id) = class.parent_id() {
            self.append_public_class_vars(parent_id, properties);
        }
    }

    fn current_property_access_context(&self) -> (Option<ClassId>, Vec<ClassId>) {
        let Some(current_class_id) = self.class_context.last().copied() else {
            return (None, Vec::new());
        };

        let mut protected_class_ids = vec![current_class_id];
        let mut current = self
            .classes
            .get(current_class_id)
            .expect("current class id should resolve to metadata")
            .parent_id();
        while let Some(class_id) = current {
            protected_class_ids.push(class_id);
            current = self
                .classes
                .get(class_id)
                .expect("ancestor class id should resolve to metadata")
                .parent_id();
        }

        (Some(current_class_id), protected_class_ids)
    }

    fn can_call_constructor(&self, class_id: ClassId, visibility: Visibility) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::Private => self.class_context.last().copied() == Some(class_id),
            Visibility::Protected => {
                self.class_context
                    .last()
                    .copied()
                    .is_some_and(|current_class_id| {
                        current_class_id == class_id
                            || self.classes.is_subclass_of(current_class_id, class_id)
                    })
            }
        }
    }

    fn ensure_instance_method_visible(
        &self,
        class_id: ClassId,
        class_name: &str,
        method_name: &str,
        visibility: Visibility,
        span: Span,
    ) -> CompileResult<()> {
        match visibility {
            Visibility::Public => Ok(()),
            Visibility::Private if self.class_context.last().copied() == Some(class_id) => Ok(()),
            Visibility::Protected
                if self
                    .class_context
                    .last()
                    .copied()
                    .is_some_and(|current_class_id| {
                        current_class_id == class_id
                            || self.classes.is_subclass_of(current_class_id, class_id)
                    }) =>
            {
                Ok(())
            }
            Visibility::Private => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "private method dispatch requires same-class method context",
                ),
            )),
            Visibility::Protected => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "protected method dispatch requires same-class or child method context",
                ),
            )),
        }
    }

    fn evaluate_array_key(
        &mut self,
        expr: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<ArrayKey> {
        let key = self.evaluate(expr, scope)?;
        ArrayKey::from_value(&key).map_err(|error| runtime_error(expr.span(), error))
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let key = name.to_ascii_lowercase();
        if key == "isset" {
            return self.call_isset(args, span, caller_scope);
        }
        if key == "empty" {
            return self.call_empty(args, span, caller_scope);
        }

        self.call_named_function(name, args, span, caller_scope)
    }

    fn call_dynamic_function(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let callee_value = self.evaluate(callee, caller_scope)?;
        let name = match callee_value {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "dynamic function call",
                        format!(
                            "callable expression must evaluate to string, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };

        self.call_named_function(&name, args, span, caller_scope)
    }

    fn call_named_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        match self.lookup_function(name).ok_or_else(|| {
            runtime_error(span, RuntimeError::undefined_function(callable_name(name)))
        })? {
            Callable::Builtin(key) => {
                if key == "spl_autoload_register" {
                    return self.call_spl_autoload_register(args, span, caller_scope);
                }
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.evaluate(arg, caller_scope)?);
                }
                self.call_builtin(&key, values, span)
            }
            Callable::User(function) => self.call_user_function(function, args, span, caller_scope),
        }
    }

    fn call_callable_with_values(
        &mut self,
        callable: Callable,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        match callable {
            Callable::Builtin(key) => self.call_builtin(&key, args, span),
            Callable::User(function) => self.call_user_function_with_values(function, args, span),
        }
    }

    fn call_spl_autoload_register(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if !(1..=3).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "spl_autoload_register()",
                    ArityExpectation::Between { min: 1, max: 3 },
                    args.len(),
                ),
            ));
        }

        match &args[0] {
            Expr::Closure { .. } => {}
            callback => match self.evaluate(callback, caller_scope)? {
                Value::String(_) => {}
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "spl_autoload_register()",
                            format!(
                                "callback argument must be closure or string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }
            },
        }

        for (index, arg) in args.iter().enumerate().skip(1) {
            match self.evaluate(arg, caller_scope)? {
                Value::Bool(_) => {}
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "spl_autoload_register()",
                            format!(
                                "argument #{} must be bool in the current subset, got {}",
                                index + 1,
                                other.type_name()
                            ),
                        ),
                    ));
                }
            }
        }

        Ok(Value::Bool(true))
    }

    fn lookup_function(&self, name: &str) -> Option<Callable> {
        let key = name.to_ascii_lowercase();
        if is_builtin(&key) {
            return Some(Callable::Builtin(key));
        }

        self.functions.get(&key).cloned().map(Callable::User)
    }

    fn call_user_function(
        &mut self,
        function: Rc<FunctionDecl>,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        self.call_user_function_with_checked_values(function, values, None, None, None)
    }

    fn call_user_function_with_values(
        &mut self,
        function: Rc<FunctionDecl>,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;
        self.call_user_function_with_checked_values(function, args, None, None, None)
    }

    fn call_user_function_with_checked_values(
        &mut self,
        function: &FunctionDecl,
        args: Vec<Value>,
        this_object: Option<PhpObject>,
        class_context: Option<ClassId>,
        called_class_context: Option<ClassId>,
    ) -> CompileResult<Value> {
        self.function_context.push(function.name.clone());
        if let Some(class_context) = class_context {
            self.class_context.push(class_context);
        }
        if let Some(called_class_context) = called_class_context {
            self.called_class_context.push(called_class_context);
        }
        let mut local_scope = SymbolTable::new();
        if let Some(this_object) = this_object {
            local_scope.write_static("this", Value::Object(this_object));
        }
        for (index, param) in function.params.iter().enumerate() {
            let value = if let Some(arg) = args.get(index) {
                arg.clone()
            } else {
                let default = param
                    .default
                    .as_ref()
                    .expect("arity check ensures missing params have defaults");
                let mut default_scope = SymbolTable::new();
                match self.evaluate(default, &mut default_scope) {
                    Ok(value) => value,
                    Err(error) => {
                        self.function_context.pop();
                        if class_context.is_some() {
                            self.class_context.pop();
                        }
                        if called_class_context.is_some() {
                            self.called_class_context.pop();
                        }
                        return Err(error);
                    }
                }
            };
            local_scope.write_static(&param.name, value);
        }

        self.call_depth += 1;
        self.active_static_locals.push(Vec::new());
        let flow = self.execute_statements(&function.body, &mut local_scope);
        let static_names = self.active_static_locals.pop().unwrap_or_default();
        let function_key = function.name.to_ascii_lowercase();
        for name in static_names {
            if let Some(value) = local_scope.read_named(&name) {
                self.static_locals
                    .insert((function_key.clone(), name), value.clone());
            }
        }
        self.call_depth -= 1;
        self.function_context.pop();
        if class_context.is_some() {
            self.class_context.pop();
        }
        if called_class_context.is_some() {
            self.called_class_context.pop();
        }

        match flow? {
            Flow::Normal => Ok(Value::Null),
            Flow::Break(span) => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
            )),
            Flow::Continue(span) => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("continue cannot be used outside a loop"),
            )),
            Flow::Return(value) => Ok(value),
            Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
        }
    }

    fn call_user_function_with_this(
        &mut self,
        function: &FunctionDecl,
        this_object: PhpObject,
        args: Vec<Value>,
        class_context: Option<ClassId>,
        called_class_context: Option<ClassId>,
    ) -> CompileResult<Value> {
        self.call_user_function_with_checked_values(
            function,
            args,
            Some(this_object),
            class_context,
            called_class_context,
        )
    }

    fn ensure_user_function_call_depth(
        &self,
        function: &FunctionDecl,
        span: Span,
    ) -> CompileResult<()> {
        if self.call_depth >= MAX_USER_FUNCTION_CALL_DEPTH {
            return Err(runtime_error(
                span,
                RuntimeError::call_depth_exceeded(
                    callable_name(&function.name),
                    MAX_USER_FUNCTION_CALL_DEPTH,
                ),
            ));
        }

        Ok(())
    }

    fn value_class_id(&self, object_or_class: &Value, allow_string: bool) -> Option<ClassId> {
        match object_or_class {
            Value::Object(object) => Some(object.class_id()),
            Value::String(candidate) if allow_string => self.classes.lookup_class_id(candidate),
            _ => None,
        }
    }

    fn value_is_a(&self, object_or_class: &Value, class_name: &str, allow_string: bool) -> bool {
        let Some(target_class) = self.classes.lookup_class(class_name) else {
            return false;
        };
        let Some(candidate_id) = self.value_class_id(object_or_class, allow_string) else {
            return false;
        };

        candidate_id == target_class.id()
            || self.classes.is_subclass_of(candidate_id, target_class.id())
    }

    fn value_instanceof(&self, value: &Value, class_name: &str) -> bool {
        let Value::Object(object) = value else {
            return false;
        };
        let Some(target_class) = self.classes.lookup_class(class_name) else {
            return false;
        };

        object.class_id() == target_class.id()
            || self
                .classes
                .is_subclass_of(object.class_id(), target_class.id())
    }

    fn value_is_subclass_of(
        &self,
        object_or_class: &Value,
        class_name: &str,
        allow_string: bool,
    ) -> bool {
        let Some(target_class) = self.classes.lookup_class(class_name) else {
            return false;
        };
        let Some(candidate_id) = self.value_class_id(object_or_class, allow_string) else {
            return false;
        };

        self.classes.is_subclass_of(candidate_id, target_class.id())
    }

    fn parent_class_name(&self, class_id: ClassId) -> Option<String> {
        let class = self.classes.get(class_id)?;
        let parent_id = class.parent_id()?;
        Some(self.classes.get(parent_id)?.name().to_string())
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        match name {
            "define" => {
                if !(2..=3).contains(&args.len()) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "define()",
                            ArityExpectation::Between { min: 2, max: 3 },
                            args.len(),
                        ),
                    ));
                }

                if args.len() == 3 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "define()",
                            "case-insensitive constant definitions are not implemented; pass exactly two arguments in the current subset",
                        ),
                    ));
                }

                let name = match &args[0] {
                    Value::String(name) if is_supported_runtime_constant_name(name) => name.clone(),
                    Value::String(name) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "define()",
                                format!(
                                    "constant name must be a non-empty unqualified identifier in the current subset, got {name}"
                                ),
                            ),
                        ));
                    }
                    other => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "define()",
                                format!(
                                    "name argument must be string in the current subset, got {}",
                                    other.type_name()
                                ),
                            ),
                        ));
                    }
                };

                if let Some(type_name) = unsupported_runtime_constant_value_type(&args[1]) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "define()",
                            format!(
                                "value must be null, bool, int, float, string, or array values in the current subset, got {type_name}"
                            ),
                        ),
                    ));
                }

                self.constants
                    .define(&name, args[1].clone())
                    .map_err(|error| runtime_error(span, error))?;
                Ok(Value::Bool(true))
            }
            "strlen" => {
                expect_arity(name, &args, 1, span)?;
                if matches!(&args[0], Value::Array(_)) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call("strlen()", "arrays are not supported"),
                    ));
                }
                let value = args[0]
                    .try_echo_string()
                    .map_err(|error| runtime_error(span, error))?;
                Ok(Value::Int(value.as_bytes().len() as i64))
            }
            "dirname" => {
                if !(1..=2).contains(&args.len()) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "dirname()",
                            ArityExpectation::Between { min: 1, max: 2 },
                            args.len(),
                        ),
                    ));
                }

                let path = match &args[0] {
                    Value::String(path) => path,
                    other => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "dirname()",
                                format!(
                                    "path argument must be string in the current subset, got {}",
                                    other.type_name()
                                ),
                            ),
                        ));
                    }
                };

                let levels = match args.get(1) {
                    Some(Value::Int(levels)) if *levels >= 1 => *levels,
                    Some(Value::Int(_)) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "dirname()",
                                "levels argument must be greater than or equal to 1 in the current subset",
                            ),
                        ));
                    }
                    Some(other) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "dirname()",
                                format!(
                                    "levels argument must be int in the current subset, got {}",
                                    other.type_name()
                                ),
                            ),
                        ));
                    }
                    None => 1,
                };

                Ok(Value::String(dirname_path(path, levels)))
            }
            "count" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(value) => Ok(Value::Int(value.len() as i64)),
                    _ => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call("count()", "only arrays are supported"),
                    )),
                }
            }
            "constant" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(name) if is_supported_runtime_constant_name(name) => self
                        .constants
                        .get(name)
                        .ok_or_else(|| {
                            runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "constant()",
                                    format!(
                                        "constant {name} is not defined in the current runtime-defined or built-in constant subset"
                                    ),
                                ),
                            )
                        }),
                    Value::String(name) => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "constant()",
                            format!(
                                "constant name must be a non-empty unqualified identifier in the current subset, got {name}"
                            ),
                        ),
                    )),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "constant()",
                            format!(
                                "name argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "defined" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(name) if is_supported_runtime_constant_name(name) => {
                        Ok(Value::Bool(self.constants.contains(name)))
                    }
                    Value::String(name) => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "defined()",
                            format!(
                                "constant name must be a non-empty unqualified identifier in the current subset, got {name}"
                            ),
                        ),
                    )),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "defined()",
                            format!(
                                "name argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "array_key_exists" => {
                expect_arity(name, &args, 2, span)?;
                let key = ArrayKey::from_array_key_exists_value(&args[0])
                    .map_err(|error| runtime_error(span, error))?;
                match &args[1] {
                    Value::Array(array) => Ok(Value::Bool(array.contains_key(key))),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_key_exists()",
                            format!("second argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_values" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => Ok(Value::Array(array.values_reindexed())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_values()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_key_first" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => Ok(array.first_key_value()),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_key_first()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_key_last" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => Ok(array.last_key_value()),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_key_last()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_is_list" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => Ok(Value::Bool(array.is_list())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_is_list()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_keys" => match args.as_slice() {
                [Value::Array(array)] => Ok(Value::Array(array.keys_reindexed())),
                [Value::Array(array), search_value] => array
                    .keys_matching_loose_scalar(search_value)
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(array), search_value, Value::Bool(true)] => array
                    .keys_matching_strict_scalar(search_value)
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(array), search_value, Value::Bool(false)] => array
                    .keys_matching_loose_scalar(search_value)
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(_), _, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_keys()",
                        format!(
                            "strict mode argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [other] | [other, _] | [other, _, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_keys()",
                        format!("argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_keys()",
                        ArityExpectation::Between { min: 1, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "array_change_key_case" => match args.as_slice() {
                [Value::Array(array)] => {
                    Ok(Value::Array(array.keys_with_ascii_case(ArrayKeyCase::Lower)))
                }
                [Value::Array(array), Value::Int(case)] => {
                    Ok(Value::Array(array.keys_with_ascii_case(
                        ArrayKeyCase::from_flag(*case),
                    )))
                }
                [Value::Array(_), other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_change_key_case()",
                        format!(
                            "case flag must be int in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [other] | [other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_change_key_case()",
                        format!("argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_change_key_case()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "array_column" => match args.as_slice() {
                [Value::Array(array), column_key] => {
                    let column_key = ArrayColumnKey::from_value(column_key)
                        .map_err(|error| runtime_error(span, error))?;
                    array
                        .column_values(column_key, None)
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error))
                }
                [Value::Array(array), column_key, index_key] => {
                    let column_key = ArrayColumnKey::from_value(column_key)
                        .map_err(|error| runtime_error(span, error))?;
                    let index_key = ArrayColumnKey::index_from_value(index_key)
                        .map_err(|error| runtime_error(span, error))?;
                    array
                        .column_values(column_key, index_key)
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error))
                }
                [other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_column()",
                        format!("first argument must be array, got {}", other.type_name()),
                    ),
                )),
                [other, _, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_column()",
                        format!("first argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_column()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "array_reverse" => match args.as_slice() {
                [Value::Array(array)] => Ok(Value::Array(array.reversed_reindexed())),
                [Value::Array(array), Value::Bool(false)] => {
                    Ok(Value::Array(array.reversed_reindexed()))
                }
                [Value::Array(array), Value::Bool(true)] => {
                    Ok(Value::Array(array.reversed_preserving_keys()))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_reverse()",
                        format!("argument must be array, got {}", other.type_name()),
                    ),
                )),
                [Value::Array(_), other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_reverse()",
                        format!(
                            "preserve_keys argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_reverse()",
                        format!("argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_reverse()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "array_slice" => match args.as_slice() {
                [Value::Array(array), Value::Int(offset)] => {
                    Ok(Value::Array(array.sliced_from_offset(*offset)))
                }
                [Value::Array(array), Value::Int(offset), Value::Int(length)] => {
                    Ok(Value::Array(array.sliced(*offset, Some(*length))))
                }
                [Value::Array(array), Value::Int(offset), Value::Null] => {
                    Ok(Value::Array(array.sliced(*offset, None)))
                }
                [Value::Array(array), Value::Int(offset), Value::Int(length), Value::Bool(true)] => {
                    Ok(Value::Array(
                        array.sliced_preserving_keys(*offset, Some(*length)),
                    ))
                }
                [Value::Array(array), Value::Int(offset), Value::Null, Value::Bool(true)] => {
                    Ok(Value::Array(array.sliced_preserving_keys(*offset, None)))
                }
                [Value::Array(array), Value::Int(offset), Value::Int(length), Value::Bool(false)] => {
                    Ok(Value::Array(array.sliced(*offset, Some(*length))))
                }
                [Value::Array(array), Value::Int(offset), Value::Null, Value::Bool(false)] => {
                    Ok(Value::Array(array.sliced(*offset, None)))
                }
                [Value::Array(_), Value::Int(_), Value::Int(_) | Value::Null, other] => {
                    Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_slice()",
                            format!(
                                "preserve_keys argument must be bool in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ))
                }
                [Value::Array(_), Value::Int(_), other]
                | [Value::Array(_), Value::Int(_), other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_slice()",
                        format!(
                            "length argument must be int or null in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [Value::Array(_), other, ..] if !matches!(other, Value::Int(_)) => {
                    Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_slice()",
                            format!(
                                "offset argument must be int in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ))
                }
                [other, _, ..] if !matches!(other, Value::Array(_)) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_slice()",
                        format!("first argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_slice()",
                        ArityExpectation::Between { min: 2, max: 4 },
                        args.len(),
                    ),
                )),
            },
            "array_chunk" => match args.as_slice() {
                [Value::Array(array), Value::Int(length)] if *length > 0 => {
                    let length = usize::try_from(*length).map_err(|_| {
                        runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "array_chunk()",
                                format!(
                                    "length argument is too large in the current subset, got {length}"
                                ),
                            ),
                        )
                    })?;
                    Ok(Value::Array(array.chunked_reindexed(length)))
                }
                [Value::Array(array), Value::Int(length), Value::Bool(preserve_keys)]
                    if *length > 0 =>
                {
                    let length = usize::try_from(*length).map_err(|_| {
                        runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "array_chunk()",
                                format!(
                                    "length argument is too large in the current subset, got {length}"
                                ),
                            ),
                        )
                    })?;
                    if *preserve_keys {
                        Ok(Value::Array(array.chunked_preserving_keys(length)))
                    } else {
                        Ok(Value::Array(array.chunked_reindexed(length)))
                    }
                }
                [Value::Array(_), Value::Int(length)] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_chunk()",
                        format!(
                            "length argument must be greater than 0 in the current subset, got {length}"
                        ),
                    ),
                )),
                [Value::Array(_), Value::Int(length), _] if *length <= 0 => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_chunk()",
                        format!(
                            "length argument must be greater than 0 in the current subset, got {length}"
                        ),
                    ),
                )),
                [Value::Array(_), Value::Int(_), other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_chunk()",
                        format!(
                            "preserve_keys argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [Value::Array(_), other] | [Value::Array(_), other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_chunk()",
                        format!(
                            "length argument must be int in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [other, _] | [other, _, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_chunk()",
                        format!("first argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_chunk()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "array_pad" => {
                expect_arity(name, &args, 3, span)?;
                match args.as_slice() {
                    [Value::Array(array), Value::Int(length), value] => array
                        .padded(*length, value.clone())
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error)),
                    [Value::Array(_), other, _] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_pad()",
                            format!(
                                "length argument must be int in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                    [other, _, _] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_pad()",
                            format!("first argument must be array, got {}", other.type_name()),
                        ),
                    )),
                    _ => unreachable!("array_pad arity is checked above"),
                }
            },
            "array_merge" => {
                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_merge()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                Ok(Value::Array(PhpArray::merged_from(arrays)))
            }
            "array_replace" => {
                if args.is_empty() {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "array_replace()",
                            ArityExpectation::AtLeast(1),
                            args.len(),
                        ),
                    ));
                }

                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_replace()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                let first = arrays[0];
                Ok(Value::Array(
                    first.replaced_with_all(arrays.iter().skip(1).copied()),
                ))
            }
            "array_flip" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => array
                        .flipped()
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_flip()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_fill_keys" => {
                expect_arity(name, &args, 2, span)?;
                match &args[0] {
                    Value::Array(keys) => keys
                        .filled_keys(args[1].clone())
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_fill_keys()",
                            format!("first argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_combine" => {
                expect_arity(name, &args, 2, span)?;
                match args.as_slice() {
                    [Value::Array(keys), Value::Array(values)] => keys
                        .combined_with(values)
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error)),
                    [Value::Array(_), other] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_combine()",
                            format!("second argument must be array, got {}", other.type_name()),
                        ),
                    )),
                    [other, _] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_combine()",
                            format!("first argument must be array, got {}", other.type_name()),
                        ),
                    )),
                    _ => unreachable!("array_combine arity is checked above"),
                }
            }
            "array_intersect_key" => {
                if args.len() < 2 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "array_intersect_key()",
                            ArityExpectation::AtLeast(2),
                            args.len(),
                        ),
                    ));
                }

                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_intersect_key()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                let (left, others) = arrays
                    .split_first()
                    .expect("array_intersect_key requires at least two arrays");
                Ok(Value::Array(
                    left.intersect_keys_with_all(others.iter().copied()),
                ))
            }
            "array_diff_key" => {
                if args.len() < 2 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "array_diff_key()",
                            ArityExpectation::AtLeast(2),
                            args.len(),
                        ),
                    ));
                }

                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_diff_key()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                let (left, others) = arrays
                    .split_first()
                    .expect("array_diff_key requires at least two arrays");
                Ok(Value::Array(left.diff_keys_with_all(others.iter().copied())))
            }
            "array_diff" => {
                if args.len() < 2 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "array_diff()",
                            ArityExpectation::AtLeast(2),
                            args.len(),
                        ),
                    ));
                }

                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_diff()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                let (left, others) = arrays
                    .split_first()
                    .expect("array_diff requires at least two arrays");
                left.diff_values_with_all(others.iter().copied())
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error))
            }
            "array_intersect" => {
                if args.len() < 2 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "array_intersect()",
                            ArityExpectation::AtLeast(2),
                            args.len(),
                        ),
                    ));
                }

                let mut arrays = Vec::with_capacity(args.len());
                for (index, arg) in args.iter().enumerate() {
                    match arg {
                        Value::Array(array) => arrays.push(array),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "array_intersect()",
                                    format!(
                                        "{} must be array, got {}",
                                        positional_argument_label(index),
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    }
                }

                let (left, others) = arrays
                    .split_first()
                    .expect("array_intersect requires at least two arrays");
                left.intersect_values_with_all(others.iter().copied())
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error))
            }
            "array_unique" => match args.as_slice() {
                [Value::Array(array)] => array
                    .unique_values_by_string()
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(array), Value::Int(0)] => array
                    .unique_values_regular()
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(array), Value::Int(1)] => array
                    .unique_values_by_numeric()
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(array), Value::Int(2)] => array
                    .unique_values_by_string()
                    .map(Value::Array)
                    .map_err(|error| runtime_error(span, error)),
                [Value::Array(_), _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_unique()",
                        "sort flags other than SORT_REGULAR, SORT_NUMERIC, or SORT_STRING are not supported in the current subset",
                    ),
                )),
                [other] | [other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_unique()",
                        format!("argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_unique()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "array_count_values" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => array
                        .count_values()
                        .map(Value::Array)
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_count_values()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_sum" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => array
                        .sum_values()
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_sum()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_product" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => array
                        .product_values()
                        .map_err(|error| runtime_error(span, error)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_product()",
                            format!("argument must be array, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "array_reduce" => self.call_array_reduce(args, span),
            "array_filter" => self.call_array_filter(args, span),
            "array_map" => self.call_array_map(args, span),
            "in_array" => match args.as_slice() {
                [needle, Value::Array(array)] => array
                    .contains_value_loose_scalar(needle)
                    .map(Value::Bool)
                    .map_err(|error| runtime_error(span, error)),
                [needle, Value::Array(array), Value::Bool(true)] => array
                    .contains_value_strict_scalar(needle)
                    .map(Value::Bool)
                    .map_err(|error| runtime_error(span, error)),
                [needle, Value::Array(array), Value::Bool(false)] => array
                    .contains_value_loose_scalar(needle)
                    .map(Value::Bool)
                    .map_err(|error| runtime_error(span, error)),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "in_array()",
                        format!("second argument must be array, got {}", other.type_name()),
                    ),
                )),
                [_, Value::Array(_), other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "in_array()",
                        format!(
                            "strict mode argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "in_array()",
                        format!("second argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "in_array()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "array_search" => match args.as_slice() {
                [needle, Value::Array(array)] => array
                    .search_value_loose_scalar(needle)
                    .map(|key| match key {
                        Some(ArrayKey::Int(value)) => Value::Int(value),
                        Some(ArrayKey::String(value)) => Value::String(value),
                        None => Value::Bool(false),
                    })
                    .map_err(|error| runtime_error(span, error)),
                [needle, Value::Array(array), Value::Bool(true)] => array
                    .search_value_strict_scalar(needle)
                    .map(|key| match key {
                        Some(ArrayKey::Int(value)) => Value::Int(value),
                        Some(ArrayKey::String(value)) => Value::String(value),
                        None => Value::Bool(false),
                    })
                    .map_err(|error| runtime_error(span, error)),
                [needle, Value::Array(array), Value::Bool(false)] => array
                    .search_value_loose_scalar(needle)
                    .map(|key| match key {
                        Some(ArrayKey::Int(value)) => Value::Int(value),
                        Some(ArrayKey::String(value)) => Value::String(value),
                        None => Value::Bool(false),
                    })
                    .map_err(|error| runtime_error(span, error)),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_search()",
                        format!("second argument must be array, got {}", other.type_name()),
                    ),
                )),
                [_, Value::Array(_), other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_search()",
                        format!(
                            "strict mode argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, other, _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_search()",
                        format!("second argument must be array, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "array_search()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "gettype" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::String(args[0].gettype_name().to_string()))
            }
            "is_null" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Null)))
            }
            "is_bool" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Bool(_))))
            }
            "is_int" | "is_integer" | "is_long" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Int(_))))
            }
            "is_float" | "is_double" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Float(_))))
            }
            "is_string" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::String(_))))
            }
            "is_array" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Array(_))))
            }
            "is_scalar" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(args[0].is_scalar()))
            }
            "is_numeric" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(args[0].is_numeric()))
            }
            "is_countable" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(args[0].is_countable()))
            }
            "is_iterable" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(args[0].is_iterable()))
            }
            "is_callable" => {
                if !(1..=2).contains(&args.len()) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "is_callable()",
                            ArityExpectation::Between { min: 1, max: 2 },
                            args.len(),
                        ),
                    ));
                }
                if let Some(other) = args.get(1).filter(|value| !matches!(value, Value::Bool(_))) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_callable()",
                            format!(
                                "syntax_only argument must be bool in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }

                let syntax_only = matches!(args.get(1), Some(Value::Bool(true)));
                match &args[0] {
                    Value::String(name) if syntax_only => Ok(Value::Bool(true)),
                    Value::String(name) => Ok(Value::Bool(self.lookup_function(name).is_some())),
                    Value::Array(array) if syntax_only => {
                        Ok(Value::Bool(is_array_callable_syntax_shape(array)))
                    }
                    Value::Array(array) => {
                        Ok(Value::Bool(is_array_callable_resolved(&self.classes, array)))
                    }
                    _ => Ok(Value::Bool(false)),
                }
            }
            "function_exists" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(name) => Ok(Value::Bool(self.lookup_function(name).is_some())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "function_exists()",
                            format!(
                                "function name argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "extension_loaded" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(_) => Ok(Value::Bool(false)),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "extension_loaded()",
                            format!(
                                "extension name argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "get_class" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Object(object) => Ok(Value::String(object.class_name().to_string())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "get_class()",
                            format!("argument must be object, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "is_object" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(matches!(&args[0], Value::Object(_))))
            }
            "get_debug_type" => {
                expect_arity(name, &args, 1, span)?;
                let type_name = match &args[0] {
                    Value::Object(object) => object.class_name().to_string(),
                    other => other.type_name().to_string(),
                };
                Ok(Value::String(type_name))
            }
            "class_exists" => {
                match args.as_slice() {
                    [Value::String(class_name)] => {
                        Ok(Value::Bool(self.classes.lookup_class(class_name).is_some()))
                    }
                    [Value::String(class_name), autoload] => {
                        let _autoload =
                            metadata_exists_autoload_flag("class_exists()", autoload, span)?;
                        Ok(Value::Bool(self.classes.lookup_class(class_name).is_some()))
                    }
                    [other] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "class_exists()",
                            format!("class name argument must be string, got {}", other.type_name()),
                        ),
                    )),
                    [_, other] => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "class_exists()",
                            format!(
                                "autoload argument must be bool-like scalar in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                    _ => Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "class_exists()",
                            ArityExpectation::Between { min: 1, max: 2 },
                            args.len(),
                        ),
                    )),
                }
            }
            "interface_exists" => match args.as_slice() {
                [Value::String(_interface_name)] => Ok(Value::Bool(false)),
                [Value::String(_interface_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("interface_exists()", autoload, span)?;
                    Ok(Value::Bool(false))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "interface_exists()",
                        format!(
                            "interface name argument must be string, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "interface_exists()",
                        format!(
                            "autoload argument must be bool-like scalar in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "interface_exists()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "trait_exists" => match args.as_slice() {
                [Value::String(_trait_name)] => Ok(Value::Bool(false)),
                [Value::String(_trait_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("trait_exists()", autoload, span)?;
                    Ok(Value::Bool(false))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "trait_exists()",
                        format!("trait name argument must be string, got {}", other.type_name()),
                    ),
                )),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "trait_exists()",
                        format!(
                            "autoload argument must be bool-like scalar in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "trait_exists()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "enum_exists" => match args.as_slice() {
                [Value::String(_enum_name)] => Ok(Value::Bool(false)),
                [Value::String(_enum_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("enum_exists()", autoload, span)?;
                    Ok(Value::Bool(false))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "enum_exists()",
                        format!("enum name argument must be string, got {}", other.type_name()),
                    ),
                )),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "enum_exists()",
                        format!(
                            "autoload argument must be bool-like scalar in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "enum_exists()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            "get_declared_classes" => {
                expect_arity(name, &args, 0, span)?;
                let mut classes = PhpArray::new();
                for class in self.classes.classes() {
                    classes
                        .append(Value::String(class.name().to_string()))
                        .expect("declared class count fits in array keys");
                }
                Ok(Value::Array(classes))
            }
            "get_declared_interfaces" => {
                expect_arity(name, &args, 0, span)?;
                Ok(Value::Array(PhpArray::new()))
            }
            "get_declared_traits" => {
                expect_arity(name, &args, 0, span)?;
                Ok(Value::Array(PhpArray::new()))
            }
            "get_called_class" => {
                expect_arity(name, &args, 0, span)?;
                let Some(called_class_id) = self.called_class_context.last().copied() else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "get_called_class()",
                            "method or static class context is required",
                        ),
                    ));
                };
                let called_class = self
                    .classes
                    .get(called_class_id)
                    .expect("called class context should resolve to class metadata");
                Ok(Value::String(called_class.name().to_string()))
            }
            "spl_object_id" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Object(object) => Ok(Value::Int(object.id())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "spl_object_id()",
                            format!("argument must be object, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "spl_object_hash" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Object(object) => Ok(Value::String(object.hash())),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "spl_object_hash()",
                            format!("argument must be object, got {}", other.type_name()),
                        ),
                    )),
                }
            }
            "property_exists" => match args.as_slice() {
                [object_or_class, Value::String(property_name)] => {
                    let exists = match object_or_class {
                        Value::Object(object) => self
                            .class_has_property_in_hierarchy(object.class_id(), property_name),
                        Value::String(class_name) => self
                            .classes
                            .lookup_class_id(class_name)
                            .is_some_and(|class_id| {
                                self.class_has_property_in_hierarchy(class_id, property_name)
                            }),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "property_exists()",
                                    format!(
                                        "object_or_class argument must be object or string, got {}",
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    };
                    Ok(Value::Bool(exists))
                }
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "property_exists()",
                        format!(
                            "property argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "property_exists()",
                        ArityExpectation::Exactly(2),
                        args.len(),
                    ),
                )),
            },
            "method_exists" => match args.as_slice() {
                [object_or_class, Value::String(method_name)] => {
                    let exists = match object_or_class {
                        Value::Object(object) => self
                            .resolve_instance_method(object.class_id(), method_name)
                            .is_some(),
                        Value::String(class_name) => self
                            .classes
                            .lookup_class_id(class_name)
                            .is_some_and(|class_id| {
                                self.resolve_instance_method(class_id, method_name).is_some()
                            }),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "method_exists()",
                                    format!(
                                        "object_or_class argument must be object or string, got {}",
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    };
                    Ok(Value::Bool(exists))
                }
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "method_exists()",
                        format!(
                            "method argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "method_exists()",
                        ArityExpectation::Exactly(2),
                        args.len(),
                    ),
                )),
            },
            "get_class_methods" => match args.as_slice() {
                [object_or_class] => {
                    let class = match object_or_class {
                        Value::Object(object) => self.classes.get(object.class_id()),
                        Value::String(class_name) => self.classes.lookup_class(class_name),
                        other => {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "get_class_methods()",
                                    format!(
                                        "object_or_class argument must be object or declared class string, got {}",
                                        other.type_name()
                                    ),
                                ),
                            ));
                        }
                    };
                    let Some(class) = class else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "get_class_methods()",
                                "string argument must name a declared class in the current subset",
                            ),
                        ));
                    };

                    let mut methods = PhpArray::new();
                    let mut current = Some(class.id());
                    while let Some(class_id) = current {
                        let current_class = self
                            .classes
                            .get(class_id)
                            .expect("class id should resolve to metadata");
                        for method in current_class.methods() {
                            if method.visibility() == Visibility::Public {
                                methods
                                    .append(Value::String(method.name().to_string()))
                                    .expect("method count fits in array keys");
                            }
                        }
                        current = current_class.parent_id();
                    }
                    Ok(Value::Array(methods))
                }
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "get_class_methods()",
                        ArityExpectation::Exactly(1),
                        args.len(),
                    ),
                )),
            },
            "get_class_vars" => match args.as_slice() {
                [Value::String(class_name)] => {
                    let Some(class) = self.classes.lookup_class(class_name) else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "get_class_vars()",
                                "string argument must name a declared class in the current subset",
                            ),
                        ));
                    };

                    let mut properties = PhpArray::new();
                    self.append_public_class_vars(class.id(), &mut properties);
                    Ok(Value::Array(properties))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "get_class_vars()",
                        format!("class name argument must be string, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "get_class_vars()",
                        ArityExpectation::Exactly(1),
                        args.len(),
                    ),
                )),
            },
            "get_object_vars" => match args.as_slice() {
                [Value::Object(object)] => {
                    let mut properties = PhpArray::new();
                    for property in object.properties() {
                        if property.visibility() == Visibility::Public {
                            properties.insert(
                                ArrayKey::from(property.name()),
                                property.value().clone(),
                            );
                        }
                    }
                    Ok(Value::Array(properties))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "get_object_vars()",
                        format!("argument must be object, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "get_object_vars()",
                        ArityExpectation::Exactly(1),
                        args.len(),
                    ),
                )),
            },
            "get_mangled_object_vars" => match args.as_slice() {
                [Value::Object(object)] => {
                    let mut properties = PhpArray::new();
                    for property in object.properties() {
                        properties.insert(
                            ArrayKey::String(property.mangled_name()),
                            property.value().clone(),
                        );
                    }
                    Ok(Value::Array(properties))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "get_mangled_object_vars()",
                        format!("argument must be object, got {}", other.type_name()),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "get_mangled_object_vars()",
                        ArityExpectation::Exactly(1),
                        args.len(),
                    ),
                )),
            },
            "is_a" => match args.as_slice() {
                [object_or_class, Value::String(class_name)] => {
                    Ok(Value::Bool(self.value_is_a(object_or_class, class_name, false)))
                }
                [object_or_class, Value::String(class_name), Value::Bool(allow_string)] => Ok(
                    Value::Bool(self.value_is_a(object_or_class, class_name, *allow_string)),
                ),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_a()",
                        format!(
                            "class name argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, _, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_a()",
                        format!(
                            "allow_string argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "is_a()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "is_subclass_of" => match args.as_slice() {
                [object_or_class @ Value::Object(_), Value::String(class_name)] => Ok(Value::Bool(
                    self.value_is_subclass_of(object_or_class, class_name, false),
                )),
                [object_or_class @ Value::String(_), Value::String(class_name)] => Ok(Value::Bool(
                    self.value_is_subclass_of(object_or_class, class_name, true),
                )),
                [object_or_class @ Value::Object(_), Value::String(class_name), Value::Bool(allow_string)] => Ok(Value::Bool(
                    self.value_is_subclass_of(object_or_class, class_name, *allow_string),
                )),
                [object_or_class @ Value::String(_), Value::String(class_name), Value::Bool(allow_string)] => Ok(Value::Bool(
                    self.value_is_subclass_of(object_or_class, class_name, *allow_string),
                )),
                [other, Value::String(_), Value::Bool(_)] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_subclass_of()",
                        format!(
                            "object_or_class argument must be object or string, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, other, Value::Bool(_)] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_subclass_of()",
                        format!(
                            "class name argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [other, Value::String(_)] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_subclass_of()",
                        format!(
                            "object_or_class argument must be object or string, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_subclass_of()",
                        format!(
                            "class name argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                [_, _, other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "is_subclass_of()",
                        format!(
                            "allow_string argument must be bool in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "is_subclass_of()",
                        ArityExpectation::Between { min: 2, max: 3 },
                        args.len(),
                    ),
                )),
            },
            "get_parent_class" => match args.as_slice() {
                [Value::Object(object)] => Ok(self
                    .parent_class_name(object.class_id())
                    .map(Value::String)
                    .unwrap_or(Value::Bool(false))),
                [Value::String(class_name)] => {
                    let Some(class) = self.classes.lookup_class(class_name) else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "get_parent_class()",
                                "string argument must name a declared class in the current subset",
                            ),
                        ));
                    };
                    Ok(self
                        .parent_class_name(class.id())
                        .map(Value::String)
                        .unwrap_or(Value::Bool(false)))
                }
                [other] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "get_parent_class()",
                        format!(
                            "object_or_class argument must be object or string, got {}",
                            other.type_name()
                        ),
                    ),
                )),
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "get_parent_class()",
                        ArityExpectation::Exactly(1),
                        args.len(),
                    ),
                )),
            },
            "var_dump" => {
                for value in &args {
                    self.stdout.push_str(&format_var_dump(value));
                }
                Ok(Value::Null)
            }
            "print_r" => match args.as_slice() {
                [value] => {
                    self.stdout.push_str(&format_print_r(value));
                    Ok(Value::Bool(true))
                }
                [value, return_output] if return_output.is_truthy() => {
                    Ok(Value::String(format_print_r(value)))
                }
                [value, _] => {
                    self.stdout.push_str(&format_print_r(value));
                    Ok(Value::Bool(true))
                }
                _ => Err(runtime_error(
                    span,
                    RuntimeError::arity_mismatch(
                        "print_r()",
                        ArityExpectation::Between { min: 1, max: 2 },
                        args.len(),
                    ),
                )),
            },
            _ => unreachable!("is_builtin keeps this match exhaustive for callers"),
        }
    }

    fn call_array_reduce(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        match args.as_slice() {
            [Value::Array(array), callback] => {
                self.reduce_array_with_callback(array, callback, Value::Null, span)
            }
            [Value::Array(array), callback, initial] => {
                self.reduce_array_with_callback(array, callback, initial.clone(), span)
            }
            [other, _] => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_reduce()",
                    format!("first argument must be array, got {}", other.type_name()),
                ),
            )),
            [other, _, _] => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_reduce()",
                    format!("first argument must be array, got {}", other.type_name()),
                ),
            )),
            _ => Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "array_reduce()",
                    ArityExpectation::Between { min: 2, max: 3 },
                    args.len(),
                ),
            )),
        }
    }

    fn reduce_array_with_callback(
        &mut self,
        array: &PhpArray,
        callback: &Value,
        initial: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let callable = self.resolve_array_reduce_callback(callback, span)?;
        let mut accumulator = initial;

        for entry in array.entries() {
            accumulator = self.call_callable_with_values(
                callable.clone(),
                vec![accumulator, entry.value.clone()],
                span,
            )?;
        }

        Ok(accumulator)
    }

    fn resolve_array_reduce_callback(
        &self,
        callback: &Value,
        span: Span,
    ) -> CompileResult<Callable> {
        let callback_name = match callback {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_reduce()",
                        format!(
                            "callback must evaluate to string, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };
        self.lookup_function(callback_name).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_function(callable_name(callback_name)),
            )
        })
    }

    fn call_array_filter(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        match args.as_slice() {
            [Value::Array(array)] => Ok(Value::Array(array.filtered_without_callback())),
            [Value::Array(array), Value::Null] => {
                Ok(Value::Array(array.filtered_without_callback()))
            }
            [Value::Array(array), Value::Null, mode] => {
                Self::array_filter_mode(mode, span)?;
                Ok(Value::Array(array.filtered_without_callback()))
            }
            [other] => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_filter()",
                    format!("argument must be array, got {}", other.type_name()),
                ),
            )),
            [Value::Array(array), callback] => Ok(Value::Array(self.filter_array_with_callback(
                array,
                callback,
                ArrayFilterMode::Value,
                span,
            )?)),
            [Value::Array(array), callback, mode] => {
                let mode = Self::array_filter_mode(mode, span)?;
                Ok(Value::Array(
                    self.filter_array_with_callback(array, callback, mode, span)?,
                ))
            }
            [other, _] | [other, _, _] => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_filter()",
                    format!("argument must be array, got {}", other.type_name()),
                ),
            )),
            _ => Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "array_filter()",
                    ArityExpectation::Between { min: 1, max: 3 },
                    args.len(),
                ),
            )),
        }
    }

    fn array_filter_mode(mode: &Value, span: Span) -> CompileResult<ArrayFilterMode> {
        match mode {
            Value::Bool(false) => Ok(ArrayFilterMode::Value),
            Value::Bool(true) => Ok(ArrayFilterMode::Both),
            Value::Int(0) => Ok(ArrayFilterMode::Value),
            Value::Int(1) => Ok(ArrayFilterMode::Both),
            Value::Int(2) => Ok(ArrayFilterMode::Key),
            Value::Int(value) => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_filter()",
                    format!(
                        "mode flag must be integer 0, 1, or 2 in the current subset, got {value}"
                    ),
                ),
            )),
            Value::Float(value) => match integral_float_to_i64(*value) {
                Some(0) => Ok(ArrayFilterMode::Value),
                Some(1) => Ok(ArrayFilterMode::Both),
                Some(2) => Ok(ArrayFilterMode::Key),
                Some(value) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_filter()",
                        format!(
                            "mode flag float must coerce to integer 0, 1, or 2 in the current subset, got {value}"
                        ),
                    ),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_filter()",
                        "mode flag float must be finite and integral in the current subset"
                            .to_string(),
                    ),
                )),
            },
            Value::String(value) => match parse_array_filter_string_mode(value) {
                Some(0) => Ok(ArrayFilterMode::Value),
                Some(1) => Ok(ArrayFilterMode::Both),
                Some(2) => Ok(ArrayFilterMode::Key),
                Some(value) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_filter()",
                        format!(
                            "mode flag string must coerce to integer 0, 1, or 2 in the current subset, got {value}"
                        ),
                    ),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_filter()",
                        "mode flag string must be an integral numeric string in the current subset"
                            .to_string(),
                    ),
                )),
            },
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_filter()",
                    format!(
                        "mode flag must be integer 0, 1, 2, bool, finite integral float, or integral numeric string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            )),
        }
    }

    fn filter_array_with_callback(
        &mut self,
        array: &PhpArray,
        callback: &Value,
        mode: ArrayFilterMode,
        span: Span,
    ) -> CompileResult<PhpArray> {
        let callback_name = match callback {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_filter()",
                        format!(
                            "callback must evaluate to string, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };
        let callable = self.lookup_function(callback_name).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_function(callable_name(callback_name)),
            )
        })?;

        let mut filtered = PhpArray::new();
        for entry in array.entries() {
            let arguments = match mode {
                ArrayFilterMode::Value => vec![entry.value.clone()],
                ArrayFilterMode::Both => {
                    vec![entry.value.clone(), value_from_array_key(&entry.key)]
                }
                ArrayFilterMode::Key => vec![value_from_array_key(&entry.key)],
            };
            let result = self.call_callable_with_values(callable.clone(), arguments, span)?;
            if result.is_truthy() {
                filtered.insert(entry.key.clone(), entry.value.clone());
            }
        }

        Ok(filtered)
    }

    fn call_array_map(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        let args = args.as_slice();
        if args.len() < 2 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "array_map()",
                    ArityExpectation::AtLeast(2),
                    args.len(),
                ),
            ));
        }

        let mut arrays = Vec::new();
        for (index, arg) in args.iter().enumerate().skip(1) {
            match arg {
                Value::Array(array) => arrays.push(array),
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "array_map()",
                            format!(
                                "{} must be array, got {}",
                                positional_argument_label(index),
                                other.type_name()
                            ),
                        ),
                    ));
                }
            }
        }

        let callback = &args[0];
        if matches!(callback, Value::Null) {
            return match arrays.as_slice() {
                [array] => Ok(Value::Array((*array).clone())),
                arrays => Ok(Value::Array(self.zip_arrays_for_array_map(arrays, span)?)),
            };
        }

        match arrays.as_slice() {
            [array] => Ok(Value::Array(
                self.map_array_with_callback(callback, array, span)?,
            )),
            arrays => Ok(Value::Array(
                self.map_arrays_with_callback(callback, arrays, span)?,
            )),
        }
    }

    fn map_array_with_callback(
        &mut self,
        callback: &Value,
        array: &PhpArray,
        span: Span,
    ) -> CompileResult<PhpArray> {
        let callable = self.resolve_array_map_callback(callback, span)?;

        let mut mapped = PhpArray::new();
        for entry in array.entries() {
            let value =
                self.call_callable_with_values(callable.clone(), vec![entry.value.clone()], span)?;
            mapped.insert(entry.key.clone(), value);
        }

        Ok(mapped)
    }

    fn map_arrays_with_callback(
        &mut self,
        callback: &Value,
        arrays: &[&PhpArray],
        span: Span,
    ) -> CompileResult<PhpArray> {
        let callable = self.resolve_array_map_callback(callback, span)?;
        let max_len = arrays
            .iter()
            .map(|array| array.entries().len())
            .max()
            .unwrap_or(0);

        let mut mapped = PhpArray::new();
        for index in 0..max_len {
            let values = arrays
                .iter()
                .map(|array| {
                    array
                        .entries()
                        .get(index)
                        .map(|entry| entry.value.clone())
                        .unwrap_or(Value::Null)
                })
                .collect();
            let value = self.call_callable_with_values(callable.clone(), values, span)?;
            mapped
                .append(value)
                .map_err(|error| runtime_error(span, error))?;
        }

        Ok(mapped)
    }

    fn zip_arrays_for_array_map(
        &self,
        arrays: &[&PhpArray],
        span: Span,
    ) -> CompileResult<PhpArray> {
        let max_len = arrays
            .iter()
            .map(|array| array.entries().len())
            .max()
            .unwrap_or(0);
        let mut mapped = PhpArray::new();

        for index in 0..max_len {
            let mut tuple = PhpArray::new();
            for array in arrays {
                let value = array
                    .entries()
                    .get(index)
                    .map(|entry| entry.value.clone())
                    .unwrap_or(Value::Null);
                tuple
                    .append(value)
                    .map_err(|error| runtime_error(span, error))?;
            }
            mapped
                .append(Value::Array(tuple))
                .map_err(|error| runtime_error(span, error))?;
        }

        Ok(mapped)
    }

    fn resolve_array_map_callback(&self, callback: &Value, span: Span) -> CompileResult<Callable> {
        let callback_name = match callback {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_map()",
                        format!(
                            "callback must evaluate to string, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };
        self.lookup_function(callback_name).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_function(callable_name(callback_name)),
            )
        })
    }

    fn call_isset(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch("isset()", ArityExpectation::AtLeast(1), args.len()),
            ));
        }

        for arg in args {
            if !self.is_isset_operand(arg, caller_scope)? {
                return Ok(Value::Bool(false));
            }
        }

        Ok(Value::Bool(true))
    }

    fn is_isset_operand(
        &mut self,
        arg: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        match arg {
            Expr::Variable(name, _) => Ok(caller_scope.is_set_static(name)),
            Expr::Index {
                target,
                index,
                ..
            } => self.is_direct_array_offset_set(target, index, caller_scope),
            Expr::Property {
                target,
                property,
                span,
            } => self.is_direct_object_property_set(target, property, *span, caller_scope),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => self.is_named_static_property_set(class_name, property, *span),
            Expr::SelfStaticProperty { property, span } => {
                self.is_self_static_property_set(property, *span)
            }
            Expr::ParentStaticProperty { property, span } => {
                self.is_parent_static_property_set(property, *span)
            }
            Expr::LateStaticProperty { property, span } => {
                self.is_late_static_property_set(property, *span)
            }
            _ => Err(runtime_error(
                arg.span(),
                RuntimeError::unsupported_call(
                    "isset()",
                    "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            )),
        }
    }

    fn is_direct_array_offset_set(
        &mut self,
        target: &Expr,
        index: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "isset()",
                        "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name).cloned() {
            Some(Value::Array(array)) => {
                let key = self.evaluate_array_key(index, caller_scope)?;
                Ok(matches!(array.get(key), Some(value) if !matches!(value, Value::Null)))
            }
            Some(_) | None => Ok(false),
        }
    }

    fn is_direct_object_property_set(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "isset()",
                        "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name) {
            Some(Value::Object(object)) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                object
                    .is_property_set_from_context(property, current_class_id, &protected_class_ids)
                    .map_err(|error| runtime_error(span, error))
            }
            Some(_) | None => Ok(false),
        }
    }

    fn is_named_static_property_set(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<bool> {
        Ok(self
            .evaluate_named_static_property_for_null_coalescing(class_name, property, span)?
            .is_some())
    }

    fn is_self_static_property_set(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_self_static_property_for_null_coalescing(property, span)?
            .is_some())
    }

    fn is_parent_static_property_set(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_parent_static_property_for_null_coalescing(property, span)?
            .is_some())
    }

    fn is_late_static_property_set(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_late_static_property_for_null_coalescing(property, span)?
            .is_some())
    }

    fn call_empty(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() != 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch("empty()", ArityExpectation::Exactly(1), args.len()),
            ));
        }

        Ok(Value::Bool(self.is_empty_operand(&args[0], caller_scope)?))
    }

    fn is_empty_operand(
        &mut self,
        arg: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        match arg {
            Expr::Variable(name, _) => Ok(caller_scope
                .read_named(name)
                .map_or(true, |value| !value.is_truthy())),
            Expr::Index { target, index, .. } => {
                self.is_direct_array_offset_empty(target, index, caller_scope)
            }
            Expr::Property {
                target,
                property,
                span,
            } => self.is_direct_object_property_empty(target, property, *span, caller_scope),
            Expr::StaticProperty {
                class_name,
                property,
                span,
            } => self.is_named_static_property_empty(class_name, property, *span),
            Expr::SelfStaticProperty { property, span } => {
                self.is_self_static_property_empty(property, *span)
            }
            Expr::ParentStaticProperty { property, span } => {
                self.is_parent_static_property_empty(property, *span)
            }
            Expr::LateStaticProperty { property, span } => {
                self.is_late_static_property_empty(property, *span)
            }
            _ => Err(runtime_error(
                arg.span(),
                RuntimeError::unsupported_call(
                    "empty()",
                    "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            )),
        }
    }

    fn is_direct_array_offset_empty(
        &mut self,
        target: &Expr,
        index: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "empty()",
                        "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name).cloned() {
            Some(Value::Array(array)) => {
                let key = self.evaluate_array_key(index, caller_scope)?;
                Ok(array.get(key).map_or(true, |value| !value.is_truthy()))
            }
            Some(_) | None => Ok(true),
        }
    }

    fn is_direct_object_property_empty(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        let Expr::Variable(name, _) = target else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "empty()",
                        "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name) {
            Some(Value::Object(object)) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                object
                    .is_property_empty_from_context(
                        property,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))
            }
            Some(_) | None => Ok(true),
        }
    }

    fn is_named_static_property_empty(
        &self,
        class_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<bool> {
        Ok(self
            .evaluate_named_static_property_for_null_coalescing(class_name, property, span)?
            .map_or(true, |value| !value.is_truthy()))
    }

    fn is_self_static_property_empty(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_self_static_property_for_null_coalescing(property, span)?
            .map_or(true, |value| !value.is_truthy()))
    }

    fn is_parent_static_property_empty(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_parent_static_property_for_null_coalescing(property, span)?
            .map_or(true, |value| !value.is_truthy()))
    }

    fn is_late_static_property_empty(&self, property: &str, span: Span) -> CompileResult<bool> {
        Ok(self
            .evaluate_late_static_property_for_null_coalescing(property, span)?
            .map_or(true, |value| !value.is_truthy()))
    }

    fn apply_binary(
        &self,
        op: BinaryOp,
        left: Value,
        right: Value,
        span: Span,
    ) -> CompileResult<Value> {
        let result: RuntimeResult<Value> = match op {
            BinaryOp::Add => left.php_add(&right),
            BinaryOp::Sub => left.php_sub(&right),
            BinaryOp::Mul => left.php_mul(&right),
            BinaryOp::Div => left.php_div(&right),
            BinaryOp::Mod => left.php_mod(&right),
            BinaryOp::Concat => left.php_concat(&right),
            BinaryOp::Eq => left
                .php_cmp_checked(&right, Comparison::Eq)
                .map(Value::Bool),
            BinaryOp::Ne => left
                .php_cmp_checked(&right, Comparison::Ne)
                .map(Value::Bool),
            BinaryOp::StrictEq => left.php_identical_checked(&right).map(Value::Bool),
            BinaryOp::StrictNe => left
                .php_identical_checked(&right)
                .map(|identical| Value::Bool(!identical)),
            BinaryOp::NullCoalesce => unreachable!("null coalescing is evaluated lazily"),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => {
                unreachable!("logical operators are evaluated before binary application")
            }
            BinaryOp::BitwiseAnd => left.php_bitwise_and(&right),
            BinaryOp::BitwiseOr => left.php_bitwise_or(&right),
            BinaryOp::BitwiseXor => left.php_bitwise_xor(&right),
            BinaryOp::ShiftLeft => left.php_shift_left(&right),
            BinaryOp::ShiftRight => left.php_shift_right(&right),
            BinaryOp::Lt => left
                .php_cmp_checked(&right, Comparison::Lt)
                .map(Value::Bool),
            BinaryOp::Le => left
                .php_cmp_checked(&right, Comparison::Le)
                .map(Value::Bool),
            BinaryOp::Gt => left
                .php_cmp_checked(&right, Comparison::Gt)
                .map(Value::Bool),
            BinaryOp::Ge => left
                .php_cmp_checked(&right, Comparison::Ge)
                .map(Value::Bool),
        };

        result.map_err(|error| runtime_error(span, error))
    }

    fn apply_unary(&self, op: UnaryOp, value: Value, span: Span) -> CompileResult<Value> {
        let result: RuntimeResult<Value> = match op {
            UnaryOp::Negate => value.php_negate(),
            UnaryOp::Not => Ok(Value::Bool(!value.is_truthy())),
            UnaryOp::BitwiseNot => value.php_bitwise_not(),
        };

        result.map_err(|error| runtime_error(span, error))
    }

    fn apply_cast(&self, kind: CastKind, value: Value, span: Span) -> CompileResult<Value> {
        match kind {
            CastKind::String => match value {
                Value::Null
                | Value::Bool(_)
                | Value::Int(_)
                | Value::Float(_)
                | Value::String(_) => Ok(Value::String(value.echo_string())),
                Value::Array(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(string)",
                        "array-to-string cast warning behavior is not implemented",
                    ),
                )),
                Value::Object(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(string)",
                        "object __toString() and cast error behavior are not implemented",
                    ),
                )),
            },
            CastKind::Int => match value {
                Value::Null => Ok(Value::Int(0)),
                Value::Bool(value) => Ok(Value::Int(if value { 1 } else { 0 })),
                Value::Int(value) => Ok(Value::Int(value)),
                Value::Float(value) => cast_float_to_int(value, "(int)", span),
                Value::String(value) => cast_string_to_int(&value, span),
                Value::Array(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(int)",
                        "array-to-int cast behavior is not implemented",
                    ),
                )),
                Value::Object(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(int)",
                        "object-to-int cast behavior is not implemented",
                    ),
                )),
            },
            CastKind::Bool => Ok(Value::Bool(value.is_truthy())),
        }
    }
}

fn cast_string_to_int(value: &str, span: Span) -> CompileResult<Value> {
    let trimmed = value.trim_matches(|ch: char| ch.is_ascii_whitespace());
    if trimmed.is_empty() {
        return Ok(Value::Int(0));
    }
    if !starts_with_numeric_prefix(trimmed) {
        return Ok(Value::Int(0));
    }

    if let Ok(value) = trimmed.parse::<i64>() {
        return Ok(Value::Int(value));
    }
    if let Ok(value) = trimmed.parse::<f64>() {
        return cast_float_to_int(value, "(int)", span);
    }

    Err(runtime_error(
        span,
        RuntimeError::unsupported_call(
            "(int)",
            "leading-numeric string cast behavior is not implemented",
        ),
    ))
}

fn starts_with_numeric_prefix(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some('+' | '-') => matches!(chars.next(), Some('0'..='9') | Some('.')),
        Some('0'..='9') | Some('.') => true,
        _ => false,
    }
}

fn cast_float_to_int(value: f64, callable: &'static str, span: Span) -> CompileResult<Value> {
    if !value.is_finite() || value < i64::MIN as f64 || value >= 9_223_372_036_854_775_808.0 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable,
                "non-finite or out-of-range float-to-int cast behavior is not implemented",
            ),
        ));
    }

    Ok(Value::Int(value.trunc() as i64))
}

fn register_class_name(classes: &mut PhpClassTable, class: &ClassDecl) -> CompileResult<ClassId> {
    classes
        .declare_class(&class.name)
        .map_err(|error| runtime_error(class.span, error))
}

fn register_class_member_runtime_tables(
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: &mut HashMap<(ClassId, String), Value>,
    methods: &mut HashMap<(ClassId, String), Rc<FunctionDecl>>,
    class_id: ClassId,
    class: &ClassDecl,
) {
    for member in &class.members {
        match member {
            ClassMember::Constant(constant) => {
                class_constants.insert((class_id, constant.name.clone()), constant.clone());
            }
            ClassMember::Property(property) if property.is_static => {
                static_properties.insert((class_id, property.name.clone()), Value::Null);
            }
            ClassMember::Method(method) => {
                methods.insert(
                    (class_id, method.function.name.to_ascii_lowercase()),
                    Rc::new(method.function.clone()),
                );
            }
            ClassMember::Property(_) => {}
        }
    }
}

fn remove_class_member_runtime_tables(
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: &mut HashMap<(ClassId, String), Value>,
    methods: &mut HashMap<(ClassId, String), Rc<FunctionDecl>>,
    class_id: ClassId,
) {
    class_constants.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    static_properties.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    methods.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
}

fn register_class_members(
    classes: &mut PhpClassTable,
    class: &ClassDecl,
) -> CompileResult<ClassId> {
    let id = classes
        .lookup_class_id(&class.name)
        .expect("class name pass should declare class id");

    if let Some(parent_name) = &class.parent {
        let parent_id = classes
            .lookup_class_id(parent_name)
            .ok_or_else(|| runtime_error(class.span, RuntimeError::undefined_class(parent_name)))?;
        classes
            .set_parent(id, parent_id)
            .map_err(|error| runtime_error(class.span, error))?;
    }

    for member in &class.members {
        match member {
            ClassMember::Property(property) => {
                let visibility = runtime_visibility(property.visibility);
                validate_inherited_property_compatibility(classes, id, &class.name, property)
                    .map_err(|error| runtime_error(property.span, error))?;

                let metadata_property = if property.is_static {
                    PhpPropertyMetadata::static_property(&property.name, visibility)
                } else {
                    PhpPropertyMetadata::instance(&property.name, visibility)
                };
                classes
                    .get_mut(id)
                    .expect("declared class id should resolve to class metadata")
                    .add_property(metadata_property)
                    .map_err(|error| runtime_error(property.span, error))?;
            }
            ClassMember::Constant(constant) => {
                let visibility = runtime_visibility(constant.visibility);
                let metadata_constant = PhpClassConstantMetadata::new(&constant.name, visibility);
                classes
                    .get_mut(id)
                    .expect("declared class id should resolve to class metadata")
                    .add_constant(metadata_constant)
                    .map_err(|error| runtime_error(constant.span, error))?;
            }
            ClassMember::Method(method) => {
                let visibility = runtime_visibility(method.visibility);
                let metadata_method = if method.is_static {
                    PhpMethodMetadata::static_method(&method.function.name, visibility)
                } else {
                    PhpMethodMetadata::instance(&method.function.name, visibility)
                };
                classes
                    .get_mut(id)
                    .expect("declared class id should resolve to class metadata")
                    .add_method(metadata_method)
                    .map_err(|error| runtime_error(method.span, error))?;
            }
        }
    }

    Ok(id)
}

fn validate_inherited_property_compatibility(
    classes: &PhpClassTable,
    class_id: ClassId,
    class_name: &str,
    property: &ClassPropertyDecl,
) -> RuntimeResult<()> {
    let visibility = runtime_visibility(property.visibility);
    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        if let Some(parent_property) = parent.property(&property.name) {
            if parent_property.visibility() == Visibility::Private {
                current = parent.parent_id();
                continue;
            }

            if parent_property.is_static() != property.is_static {
                let parent_static = if parent_property.is_static() {
                    "static"
                } else {
                    "non static"
                };
                let child_static = if property.is_static {
                    "static"
                } else {
                    "non static"
                };
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "cannot redeclare {parent_static} property {}::${} as {child_static} {}::${}",
                        parent.name(),
                        property.name,
                        class_name,
                        property.name
                    ),
                ));
            }

            if property_visibility_is_more_restrictive(visibility, parent_property.visibility()) {
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "property {}::${} cannot reduce visibility of inherited {} property {}::${}",
                        class_name,
                        property.name,
                        visibility_name(parent_property.visibility()),
                        parent.name(),
                        property.name
                    ),
                ));
            }

            return Ok(());
        }

        current = parent.parent_id();
    }

    Ok(())
}

fn property_visibility_is_more_restrictive(child: Visibility, parent: Visibility) -> bool {
    visibility_rank(child) > visibility_rank(parent)
}

fn visibility_rank(visibility: Visibility) -> u8 {
    match visibility {
        Visibility::Public => 0,
        Visibility::Protected => 1,
        Visibility::Private => 2,
    }
}

fn visibility_name(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "public",
        Visibility::Protected => "protected",
        Visibility::Private => "private",
    }
}

fn runtime_visibility(visibility: ClassVisibility) -> Visibility {
    match visibility {
        ClassVisibility::Public => Visibility::Public,
        ClassVisibility::Protected => Visibility::Protected,
        ClassVisibility::Private => Visibility::Private,
    }
}

fn runtime_error(span: Span, error: RuntimeError) -> Diagnostic {
    Diagnostic::new(Phase::Runtime, span.line, span.column, error.message())
}

fn undefined_goto_label_error(span: Span, label: &str) -> Diagnostic {
    Diagnostic::new(
        Phase::Runtime,
        span.line,
        span.column,
        format!("undefined goto label '{label}'"),
    )
}

fn metadata_exists_autoload_flag(
    function_name: &'static str,
    value: &Value,
    span: Span,
) -> CompileResult<bool> {
    match value {
        Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
            Ok(value.is_truthy())
        }
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function_name,
                format!(
                    "autoload argument must be bool-like scalar in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn is_array_callable_syntax_shape(array: &PhpArray) -> bool {
    array_callable_parts(array).is_some()
}

fn is_array_callable_resolved(classes: &PhpClassTable, array: &PhpArray) -> bool {
    let Some((target, method_name)) = array_callable_parts(array) else {
        return false;
    };

    match target {
        Value::Object(object) => classes
            .get(object.class_id())
            .and_then(|class| class.method(method_name))
            .is_some_and(|method| method.visibility() == Visibility::Public),
        Value::String(class_name) => classes
            .lookup_class(class_name)
            .and_then(|class| class.method(method_name))
            .is_some_and(|method| method.visibility() == Visibility::Public && method.is_static()),
        _ => false,
    }
}

fn array_callable_parts(array: &PhpArray) -> Option<(&Value, &str)> {
    let entries = array.entries();
    if entries.len() != 2 {
        return None;
    }

    if !matches!(entries[0].key, ArrayKey::Int(0)) || !matches!(entries[1].key, ArrayKey::Int(1)) {
        return None;
    }

    let Value::String(method_name) = &entries[1].value else {
        return None;
    };

    match &entries[0].value {
        Value::String(_) | Value::Object(_) => Some((&entries[0].value, method_name)),
        _ => None,
    }
}

impl From<RuntimeError> for Diagnostic {
    fn from(value: RuntimeError) -> Self {
        Diagnostic::new(Phase::Runtime, 0, 0, value.message())
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "define"
            | "strlen"
            | "dirname"
            | "count"
            | "constant"
            | "defined"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "array_is_list"
            | "array_keys"
            | "array_change_key_case"
            | "array_column"
            | "array_reverse"
            | "array_slice"
            | "array_chunk"
            | "array_pad"
            | "array_merge"
            | "array_replace"
            | "array_flip"
            | "array_fill_keys"
            | "array_combine"
            | "array_intersect_key"
            | "array_diff_key"
            | "array_diff"
            | "array_intersect"
            | "array_unique"
            | "array_count_values"
            | "array_sum"
            | "array_product"
            | "array_reduce"
            | "array_filter"
            | "array_map"
            | "in_array"
            | "array_search"
            | "gettype"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_array"
            | "is_scalar"
            | "is_numeric"
            | "is_countable"
            | "is_iterable"
            | "is_callable"
            | "function_exists"
            | "extension_loaded"
            | "get_class"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "get_declared_classes"
            | "get_declared_interfaces"
            | "get_declared_traits"
            | "get_called_class"
            | "spl_object_id"
            | "spl_object_hash"
            | "spl_autoload_register"
            | "property_exists"
            | "method_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_object_vars"
            | "get_mangled_object_vars"
            | "is_a"
            | "is_subclass_of"
            | "get_parent_class"
            | "var_dump"
            | "print_r"
    )
}

fn dirname_path(path: &str, levels: i64) -> String {
    let mut current = path.to_string();
    for _ in 0..levels {
        current = dirname_once(&current);
    }
    current
}

fn dirname_once(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > 1 && bytes[end - 1] == b'/' {
        end -= 1;
    }

    if end == 1 && bytes[0] == b'/' {
        return "/".to_string();
    }

    let trimmed = &path[..end];
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(position) => {
            let mut parent_end = position;
            let parent_bytes = trimmed.as_bytes();
            while parent_end > 1 && parent_bytes[parent_end - 1] == b'/' {
                parent_end -= 1;
            }
            if parent_end == 0 {
                ".".to_string()
            } else {
                trimmed[..parent_end].to_string()
            }
        }
        None => ".".to_string(),
    }
}

fn builtin_global_constant_value(name: &str) -> Option<i64> {
    match name {
        "PHP_VERSION_ID" => Some(80300),
        "CASE_LOWER" => Some(0),
        "CASE_UPPER" => Some(1),
        "ARRAY_FILTER_USE_BOTH" => Some(1),
        "ARRAY_FILTER_USE_KEY" => Some(2),
        "SORT_REGULAR" => Some(0),
        "SORT_NUMERIC" => Some(1),
        "SORT_STRING" => Some(2),
        _ => None,
    }
}

fn is_supported_runtime_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|char| char == '_' || char.is_ascii_alphanumeric())
}

fn unsupported_runtime_constant_value_type(value: &Value) -> Option<&'static str> {
    match value {
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => None,
        Value::Array(array) => array
            .entries()
            .iter()
            .find_map(|entry| unsupported_runtime_constant_value_type(&entry.value)),
        Value::Object(_) => Some("object"),
    }
}

fn expect_arity(name: &str, args: &[Value], expected: usize, span: Span) -> CompileResult<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                callable_name(name),
                ArityExpectation::Exactly(expected),
                args.len(),
            ),
        ))
    }
}

fn positional_argument_label(index: usize) -> String {
    match index {
        0 => "first argument".to_string(),
        1 => "second argument".to_string(),
        2 => "third argument".to_string(),
        3 => "fourth argument".to_string(),
        4 => "fifth argument".to_string(),
        _ => format!("argument #{}", index + 1),
    }
}

fn ensure_user_function_arity(
    function: &FunctionDecl,
    actual: usize,
    span: Span,
) -> CompileResult<()> {
    let required = required_param_count(function);
    if actual < required || actual > function.params.len() {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                callable_name(&function.name),
                arity_expectation(required, function.params.len()),
                actual,
            ),
        ));
    }

    Ok(())
}

fn ensure_supported_function_signature(function: &FunctionDecl, span: Span) -> CompileResult<()> {
    if function.params.iter().any(|param| param.by_reference) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "reference parameter invocation is not implemented",
            ),
        ));
    }

    if function.return_type.is_some()
        || function
            .params
            .iter()
            .any(|param| param.type_decl.is_some())
    {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "parameter and return type enforcement is not implemented",
            ),
        ));
    }

    Ok(())
}

fn required_param_count(function: &FunctionDecl) -> usize {
    function
        .params
        .iter()
        .filter(|param| param.default.is_none())
        .count()
}

fn arity_expectation(required: usize, total: usize) -> ArityExpectation {
    if required == total {
        ArityExpectation::Exactly(total)
    } else {
        ArityExpectation::Between {
            min: required,
            max: total,
        }
    }
}

fn callable_name(name: &str) -> String {
    format!("{name}()")
}

fn value_from_array_key(key: &ArrayKey) -> Value {
    match key {
        ArrayKey::Int(value) => Value::Int(*value),
        ArrayKey::String(value) => Value::String(value.clone()),
    }
}

fn format_var_dump(value: &Value) -> String {
    format_var_dump_with_indent(value, 0)
}

fn format_var_dump_with_indent(value: &Value, indent: usize) -> String {
    let padding = "  ".repeat(indent);
    match value {
        Value::Null => format!("{padding}NULL\n"),
        Value::Bool(value) => format!("{padding}bool({})\n", if *value { "true" } else { "false" }),
        Value::Int(value) => format!("{padding}int({value})\n"),
        Value::Float(value) => format!("{padding}float({})\n", value),
        Value::String(value) => format!("{padding}string({}) \"{}\"\n", value.len(), value),
        Value::Array(value) => {
            let mut output = format!("{padding}array({}) {{\n", value.len());
            for entry in value.entries() {
                output.push_str(&format!(
                    "{padding}  [{}]=>\n",
                    format_var_dump_key(&entry.key)
                ));
                output.push_str(&format_var_dump_with_indent(&entry.value, indent + 1));
            }
            output.push_str(&format!("{padding}}}\n"));
            output
        }
        Value::Object(value) => {
            let mut output = format!(
                "{padding}object({}) ({}) {{\n",
                value.class_name(),
                value.properties().len()
            );
            for property in value.properties() {
                output.push_str(&format!(
                    "{padding}  [{}]=>\n",
                    format_var_dump_object_property(&property)
                ));
                output.push_str(&format_var_dump_with_indent(property.value(), indent + 1));
            }
            output.push_str(&format!("{padding}}}\n"));
            output
        }
    }
}

fn format_var_dump_key(key: &ArrayKey) -> String {
    match key {
        ArrayKey::Int(value) => value.to_string(),
        ArrayKey::String(value) => format!("\"{value}\""),
    }
}

fn format_print_r(value: &Value) -> String {
    format_print_r_with_indent(value, 0)
}

fn format_print_r_with_indent(value: &Value, indent: usize) -> String {
    match value {
        Value::Array(array) => format_print_r_array(array, indent),
        Value::Object(object) => format_print_r_object(object, indent),
        _ => value.echo_string(),
    }
}

fn format_print_r_array(array: &PhpArray, indent: usize) -> String {
    let padding = "    ".repeat(indent);
    let child_padding = "    ".repeat(indent + 1);
    let mut output = String::new();

    output.push_str("Array\n");
    output.push_str(&format!("{padding}(\n"));
    for entry in array.entries() {
        output.push_str(&format!("{child_padding}[{}] => ", entry.key.display_key()));
        match &entry.value {
            Value::Array(value) => {
                output.push_str(&format_print_r_array(value, indent + 1));
            }
            Value::Object(value) => {
                output.push_str(&format_print_r_object(value, indent + 1));
            }
            value => {
                output.push_str(&value.echo_string());
                output.push('\n');
            }
        }
    }
    output.push_str(&format!("{padding})\n"));
    output
}

fn format_print_r_object(object: &PhpObject, indent: usize) -> String {
    let padding = "    ".repeat(indent);
    let child_padding = "    ".repeat(indent + 1);
    let mut output = String::new();

    output.push_str(&format!("{} Object\n", object.class_name()));
    output.push_str(&format!("{padding}(\n"));
    for property in object.properties() {
        output.push_str(&format!(
            "{child_padding}[{}] => ",
            format_print_r_object_property(&property)
        ));
        match property.value() {
            Value::Array(value) => {
                output.push_str(&format_print_r_array(value, indent + 1));
            }
            Value::Object(value) => {
                output.push_str(&format_print_r_object(value, indent + 1));
            }
            value => {
                output.push_str(&value.echo_string());
                output.push('\n');
            }
        }
    }
    output.push_str(&format!("{padding})\n"));
    output
}

fn format_print_r_object_property(property: &ObjectProperty) -> String {
    match property.visibility() {
        Visibility::Public => property.name().to_string(),
        Visibility::Protected => format!("{}:protected", property.name()),
        Visibility::Private => {
            format!(
                "{}:{}:private",
                property.name(),
                property.declaring_class_name()
            )
        }
    }
}

fn format_var_dump_object_property(property: &ObjectProperty) -> String {
    match property.visibility() {
        Visibility::Public => format!("\"{}\"", property.name()),
        Visibility::Protected => format!("\"{}\":protected", property.name()),
        Visibility::Private => format!(
            "\"{}\":\"{}\":private",
            property.name(),
            property.declaring_class_name()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Phase;

    #[test]
    fn symbol_table_static_reads_and_writes_use_named_storage() {
        let mut symbols = SymbolTable::new();
        let span = Span::new(7, 3);

        symbols.write_static("name", Value::String("Ada".to_string()));

        assert_eq!(
            symbols.read_static("name", span).unwrap(),
            Value::String("Ada".to_string())
        );
        assert!(symbols.is_set_static("name"));

        symbols.write_static("name", Value::Null);

        assert_eq!(symbols.read_static("name", span).unwrap(), Value::Null);
        assert!(!symbols.is_set_static("name"));
    }

    #[test]
    fn symbol_table_static_unset_removes_existing_symbol_and_ignores_missing_names() {
        let mut symbols = SymbolTable::new();
        let span = Span::new(7, 3);

        symbols.write_static("name", Value::String("Ada".to_string()));
        symbols.unset_static("name");
        symbols.unset_static("missing");

        assert!(!symbols.is_set_static("name"));
        assert!(symbols.read_static("name", span).is_err());
    }

    #[test]
    fn symbol_table_missing_static_read_keeps_undefined_variable_diagnostic() {
        let symbols = SymbolTable::new();
        let error = symbols
            .read_static("missing", Span::new(4, 12))
            .unwrap_err();

        assert_eq!(error.phase, Phase::Runtime);
        assert_eq!(error.line, 4);
        assert_eq!(error.column, 12);
        assert_eq!(error.message, "undefined variable '$missing'");
    }

    #[test]
    fn symbol_table_array_write_slot_materializes_undefined_static_variable() {
        let mut symbols = SymbolTable::new();

        let slot = symbols.array_slot_for_static_write("items");
        match slot {
            Value::Array(array) => {
                array.append(Value::String("first".to_string())).unwrap();
            }
            other => panic!("expected materialized array slot, got {other:?}"),
        }

        let value = symbols.read_static("items", Span::new(1, 1)).unwrap();
        let Value::Array(array) = value else {
            panic!("expected stored array");
        };

        assert_eq!(array.len(), 1);
        assert_eq!(
            array.get(0).cloned(),
            Some(Value::String("first".to_string()))
        );
    }
}
