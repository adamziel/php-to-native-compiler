use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use php_runtime::{
    ArityExpectation, ArrayColumnKey, ArrayKey, ArrayKeyCase, ClassId, Comparison, ObjectProperty,
    PhpArray, PhpClassConstantMetadata, PhpClassTable, PhpClosure, PhpClosureCapture,
    PhpMethodMetadata, PhpObject, PhpObjectPropertyInitializer, PhpPropertyMetadata, RuntimeError,
    RuntimeResult, Value, Visibility,
};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, ClassConstantDecl, ClassDecl, ClassMember,
    ClassPropertyDecl, ClassVisibility, ClosureCapture, CompoundAssignOp, EnumDecl, Expr,
    ForAction, FunctionDecl, IncrementDecrementOp, IncrementDecrementPosition, InterfaceDecl,
    InterpolatedAccessSegment, InterpolatedArrayKey, InterpolatedStringPart, NewClassName, Program,
    ReferenceSource, Span, StaticLocalDeclarator, Stmt, SwitchCase, TraitDecl, UnaryOp,
    UnsetTarget,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunOptions {
    pub max_execution_steps: Option<usize>,
    pub trace_includes: bool,
}

pub fn run_program(program: &Program) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program, None, RunOptions::default())?;
    interpreter.run(program)
}

pub fn run_program_with_source_file(
    program: &Program,
    source_file: impl Into<String>,
) -> CompileResult<Execution> {
    let mut interpreter =
        Interpreter::from_program(program, Some(source_file.into()), RunOptions::default())?;
    interpreter.run(program)
}

pub fn run_program_with_options(
    program: &Program,
    options: RunOptions,
) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program, None, options)?;
    interpreter.run(program)
}

pub fn run_program_with_source_file_and_options(
    program: &Program,
    source_file: impl Into<String>,
    options: RunOptions,
) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program, Some(source_file.into()), options)?;
    interpreter.run(program)
}

pub fn class_metadata(program: &Program) -> CompileResult<PhpClassTable> {
    Interpreter::from_program(program, None, RunOptions::default())
        .map(|interpreter| interpreter.classes)
}

struct Interpreter {
    functions: HashMap<String, Rc<FunctionDecl>>,
    methods: HashMap<(ClassId, String), Rc<FunctionDecl>>,
    abstract_classes: HashSet<ClassId>,
    abstract_methods: HashSet<(ClassId, String)>,
    interfaces: Vec<Rc<InterfaceDecl>>,
    interface_lookup: HashMap<String, Rc<InterfaceDecl>>,
    traits: Vec<Rc<TraitDecl>>,
    trait_lookup: HashMap<String, Rc<TraitDecl>>,
    enums: Vec<Rc<EnumDecl>>,
    enum_lookup: HashMap<String, Rc<EnumDecl>>,
    class_constants: HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: HashMap<(ClassId, String), Value>,
    instance_property_defaults: HashMap<(ClassId, String), Value>,
    classes: PhpClassTable,
    constants: ConstantTable,
    required_once: HashSet<PathBuf>,
    static_locals: HashMap<(String, String), Value>,
    active_static_locals: Vec<Vec<String>>,
    global_symbols: Rc<RefCell<HashMap<String, Value>>>,
    error_reporting_mask: i64,
    error_handler: Option<Value>,
    error_handler_mask: Option<i64>,
    mysqli_report_mode: i64,
    source_file: Option<String>,
    max_execution_steps: Option<usize>,
    trace_includes: bool,
    execution_steps: usize,
    call_depth: usize,
    next_object_id: i64,
    next_closure_id: i64,
    function_context: Vec<String>,
    class_context: Vec<ClassId>,
    called_class_context: Vec<ClassId>,
    stdout: String,
    exit_signal: Option<i32>,
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

fn is_auto_global_name(name: &str) -> bool {
    name == "_SERVER"
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
    let candidate = repo_root_relative_candidate(path)?;
    candidate.exists().then_some(candidate)
}

fn repo_root_relative_candidate(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return None;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent()?;
    Some(repo_root.join(path))
}

fn local_filesystem_metadata_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() || path.exists() {
        return path;
    }
    repo_root_relative_candidate(&path).unwrap_or(path)
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
    symbols: Rc<RefCell<HashMap<String, Value>>>,
    global_symbols: Option<Rc<RefCell<HashMap<String, Value>>>>,
    imported_globals: HashSet<String>,
}

impl SymbolTable {
    fn new() -> Self {
        Self::default()
    }

    fn new_child(global_symbols: Rc<RefCell<HashMap<String, Value>>>) -> Self {
        Self {
            symbols: Rc::new(RefCell::new(HashMap::new())),
            global_symbols: Some(global_symbols),
            imported_globals: HashSet::new(),
        }
    }

    fn from_root(symbols: Rc<RefCell<HashMap<String, Value>>>) -> Self {
        Self {
            symbols,
            global_symbols: None,
            imported_globals: HashSet::new(),
        }
    }

    fn import_global(&mut self, name: &str) {
        if let Some(global_symbols) = &self.global_symbols {
            let value = global_symbols
                .borrow()
                .get(name)
                .cloned()
                .unwrap_or(Value::Null);
            global_symbols.borrow_mut().insert(name.to_string(), value);
            self.imported_globals.insert(name.to_string());
        }
    }

    fn read_static(&self, name: &str, span: Span) -> CompileResult<Value> {
        self.read_named(name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_variable(name)))
    }

    fn write_static(&mut self, name: &str, value: Value) {
        self.write_named(name, value);
    }

    fn is_set_static(&self, name: &str) -> bool {
        matches!(self.read_named(name), Some(value) if !matches!(value, Value::Null))
    }

    fn unset_static(&mut self, name: &str) {
        if is_auto_global_name(name) {
            if let Some(global_symbols) = &self.global_symbols {
                global_symbols.borrow_mut().remove(name);
                return;
            }
        }
        if self.imported_globals.contains(name) {
            self.imported_globals.remove(name);
            self.symbols.borrow_mut().remove(name);
            return;
        }
        self.symbols.borrow_mut().remove(name);
    }

    fn read_named(&self, name: &str) -> Option<Value> {
        if self.imported_globals.contains(name)
            || (is_auto_global_name(name) && self.global_symbols.is_some())
        {
            return self
                .global_symbols
                .as_ref()
                .and_then(|symbols| symbols.borrow().get(name).cloned());
        }
        self.symbols.borrow().get(name).cloned()
    }

    fn write_named(&mut self, name: &str, value: Value) {
        if self.imported_globals.contains(name)
            || (is_auto_global_name(name) && self.global_symbols.is_some())
        {
            if let Some(global_symbols) = &self.global_symbols {
                global_symbols.borrow_mut().insert(name.to_string(), value);
                return;
            }
        }
        self.symbols.borrow_mut().insert(name.to_string(), value);
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
        let canonical_name = normalize_runtime_constant_lookup_name(name).unwrap_or(name);
        if builtin_global_constant_value(canonical_name).is_some()
            || self.values.contains_key(canonical_name)
        {
            return Err(RuntimeError::duplicate_constant(canonical_name));
        }

        self.values.insert(canonical_name.to_string(), value);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<Value> {
        let canonical_name = normalize_runtime_constant_lookup_name(name)?;
        self.values
            .get(canonical_name)
            .cloned()
            .or_else(|| builtin_global_constant_value(canonical_name))
    }

    fn contains(&self, name: &str) -> bool {
        normalize_runtime_constant_lookup_name(name).is_some_and(|canonical_name| {
            self.values.contains_key(canonical_name)
                || builtin_global_constant_value(canonical_name).is_some()
        })
    }
}

enum Flow {
    Normal,
    Break { depth: usize, span: Span },
    Continue { depth: usize, span: Span },
    Return(Value),
    Exit(i32),
    Goto { label: String, span: Span },
}

impl Interpreter {
    fn from_program(
        program: &Program,
        source_file: Option<String>,
        options: RunOptions,
    ) -> CompileResult<Self> {
        let mut functions = HashMap::new();
        let mut interfaces = Vec::new();
        let mut interface_lookup = HashMap::new();
        let mut traits = Vec::new();
        let mut trait_lookup = HashMap::new();
        let mut enums = Vec::new();
        let mut enum_lookup = HashMap::new();
        let mut methods = HashMap::new();
        let mut abstract_classes = HashSet::new();
        let mut abstract_methods = HashSet::new();
        let mut class_constants = HashMap::new();
        let mut static_properties = HashMap::new();
        let instance_property_defaults = HashMap::new();
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
                    let key = class.name.to_ascii_lowercase();
                    if interface_lookup.contains_key(&key)
                        || trait_lookup.contains_key(&key)
                        || enum_lookup.contains_key(&key)
                    {
                        return Err(runtime_error(
                            class.span,
                            RuntimeError::duplicate_class(&class.name),
                        ));
                    }
                    register_class_name(&mut classes, class)?;
                }
                Stmt::Interface(interface) => {
                    register_interface_name(
                        &classes,
                        &trait_lookup,
                        &enum_lookup,
                        &mut interfaces,
                        &mut interface_lookup,
                        interface,
                    )?;
                }
                Stmt::Trait(trait_decl) => {
                    register_trait_name(
                        &classes,
                        &interface_lookup,
                        &enum_lookup,
                        &mut traits,
                        &mut trait_lookup,
                        trait_decl,
                    )?;
                }
                Stmt::Enum(enum_decl) => {
                    register_enum_name(
                        &classes,
                        &interface_lookup,
                        &trait_lookup,
                        &mut enums,
                        &mut enum_lookup,
                        enum_decl,
                    )?;
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
                    &mut abstract_methods,
                    class_id,
                    class,
                );
                if class.is_abstract {
                    abstract_classes.insert(class_id);
                }
            }
        }

        let mut interpreter = Self {
            functions,
            methods,
            abstract_classes,
            abstract_methods,
            interfaces,
            interface_lookup,
            traits,
            trait_lookup,
            enums,
            enum_lookup,
            class_constants,
            static_properties,
            instance_property_defaults,
            classes,
            constants: ConstantTable::new(),
            required_once: HashSet::new(),
            static_locals: HashMap::new(),
            active_static_locals: Vec::new(),
            global_symbols: Rc::new(RefCell::new(HashMap::new())),
            error_reporting_mask: PHP_E_ALL,
            error_handler: None,
            error_handler_mask: None,
            mysqli_report_mode: PHP_MYSQLI_REPORT_ERROR | PHP_MYSQLI_REPORT_STRICT,
            source_file,
            max_execution_steps: options.max_execution_steps,
            trace_includes: options.trace_includes,
            execution_steps: 0,
            call_depth: 0,
            next_object_id: 1,
            next_closure_id: 1,
            function_context: Vec::new(),
            class_context: Vec::new(),
            called_class_context: Vec::new(),
            stdout: String::new(),
            exit_signal: None,
        };
        interpreter.initialize_superglobals();
        interpreter.initialize_static_property_defaults(program)?;
        interpreter.initialize_instance_property_defaults(program)?;
        Ok(interpreter)
    }

    fn initialize_superglobals(&mut self) {
        let mut server = PhpArray::new();
        server.insert("SERVER_SOFTWARE", Value::String("phpc".to_string()));
        server.insert("REQUEST_URI", Value::String("/".to_string()));
        server.insert("PHP_SELF", Value::String("/index.php".to_string()));
        server.insert("SCRIPT_NAME", Value::String("/index.php".to_string()));
        server.insert("QUERY_STRING", Value::String(String::new()));

        self.global_symbols
            .borrow_mut()
            .insert("_SERVER".to_string(), Value::Array(server));
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

    fn initialize_instance_property_defaults(&mut self, program: &Program) -> CompileResult<()> {
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

            self.initialize_instance_property_defaults_for_class(class_id, class)?;
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
                    let key = class.name.to_ascii_lowercase();
                    if self.interface_lookup.contains_key(&key)
                        || self.trait_lookup.contains_key(&key)
                        || self.enum_lookup.contains_key(&key)
                    {
                        return Err(runtime_error(
                            class.span,
                            RuntimeError::duplicate_class(&class.name),
                        ));
                    }
                    let class_id = register_class_name(&mut self.classes, class)?;
                    if class.is_abstract {
                        self.abstract_classes.insert(class_id);
                    }
                }
                Stmt::Interface(interface) => {
                    register_interface_name(
                        &self.classes,
                        &self.trait_lookup,
                        &self.enum_lookup,
                        &mut self.interfaces,
                        &mut self.interface_lookup,
                        interface,
                    )?;
                }
                Stmt::Trait(trait_decl) => {
                    register_trait_name(
                        &self.classes,
                        &self.interface_lookup,
                        &self.enum_lookup,
                        &mut self.traits,
                        &mut self.trait_lookup,
                        trait_decl,
                    )?;
                }
                Stmt::Enum(enum_decl) => {
                    register_enum_name(
                        &self.classes,
                        &self.interface_lookup,
                        &self.trait_lookup,
                        &mut self.enums,
                        &mut self.enum_lookup,
                        enum_decl,
                    )?;
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
                &mut self.abstract_methods,
                class_id,
                class,
            );
        }

        self.initialize_static_property_defaults(program)
            .and_then(|_| self.initialize_instance_property_defaults(program))
    }

    fn register_nested_class_declaration(&mut self, class: &ClassDecl) -> CompileResult<()> {
        let key = class.name.to_ascii_lowercase();
        if self.interface_lookup.contains_key(&key)
            || self.trait_lookup.contains_key(&key)
            || self.enum_lookup.contains_key(&key)
        {
            return Err(runtime_error(
                class.span,
                RuntimeError::duplicate_class(&class.name),
            ));
        }
        let class_id = register_class_name(&mut self.classes, class)?;
        if class.is_abstract {
            self.abstract_classes.insert(class_id);
        }
        if let Err(error) = register_class_members(&mut self.classes, class) {
            self.abstract_classes.remove(&class_id);
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }
        register_class_member_runtime_tables(
            &mut self.class_constants,
            &mut self.static_properties,
            &mut self.methods,
            &mut self.abstract_methods,
            class_id,
            class,
        );
        if let Err(error) = self.initialize_static_property_defaults_for_class(class_id, class) {
            remove_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                &mut self.abstract_methods,
                class_id,
            );
            self.abstract_classes.remove(&class_id);
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }
        if let Err(error) = self.initialize_instance_property_defaults_for_class(class_id, class) {
            remove_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                &mut self.abstract_methods,
                class_id,
            );
            self.instance_property_defaults
                .retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
            self.abstract_classes.remove(&class_id);
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

    fn initialize_instance_property_defaults_for_class(
        &mut self,
        class_id: ClassId,
        class: &ClassDecl,
    ) -> CompileResult<()> {
        self.instance_property_defaults
            .retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);

        for member in &class.members {
            let ClassMember::Property(property) = member else {
                continue;
            };
            if property.is_static {
                continue;
            }
            let Some(default) = &property.default else {
                continue;
            };

            let mut default_scope = SymbolTable::new();
            let value = self.evaluate(default, &mut default_scope)?;
            self.instance_property_defaults
                .insert((class_id, property.name.clone()), value);
        }

        Ok(())
    }

    fn run(&mut self, program: &Program) -> CompileResult<Execution> {
        let mut scope = SymbolTable::from_root(self.global_symbols.clone());
        match self.execute_statements(&program.statements, &mut scope)? {
            Flow::Normal | Flow::Return(_) => Ok(Execution {
                stdout: self.stdout.clone(),
                stderr: String::new(),
                exit_code: 0,
            }),
            Flow::Exit(code) => Ok(Execution {
                stdout: self.stdout.clone(),
                stderr: String::new(),
                exit_code: code,
            }),
            Flow::Break { span, .. } => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
            )),
            Flow::Continue { span, .. } => Err(runtime_error(
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
                flow @ (Flow::Break { .. }
                | Flow::Continue { .. }
                | Flow::Return(_)
                | Flow::Exit(_)) => return Ok(flow),
            }
            if let Some(code) = self.exit_signal {
                return Ok(Flow::Exit(code));
            }
            index += 1;
        }
        Ok(Flow::Normal)
    }

    fn execute_statement(&mut self, stmt: &Stmt, scope: &mut SymbolTable) -> CompileResult<Flow> {
        self.tick(stmt.span())?;
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
            Stmt::ReferenceAssign {
                target,
                source,
                span,
            } => {
                self.execute_reference_assignment(target, source, *span, scope)?;
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
                condition,
                body,
                span,
            } => {
                loop {
                    self.tick(*span)?;
                    if !self.evaluate(condition, scope)?.is_truthy() {
                        break;
                    }
                    match self.execute_statements(body, scope)? {
                        Flow::Normal => {}
                        Flow::Continue { depth, .. } if depth <= 1 => {}
                        Flow::Continue { depth, span } => {
                            return Ok(Flow::Continue {
                                depth: depth - 1,
                                span,
                            });
                        }
                        Flow::Break { depth, .. } if depth <= 1 => break,
                        Flow::Break { depth, span } => {
                            return Ok(Flow::Break {
                                depth: depth - 1,
                                span,
                            });
                        }
                        flow @ (Flow::Return(_) | Flow::Goto { .. } | Flow::Exit(_)) => {
                            return Ok(flow);
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::DoWhile {
                body,
                condition,
                span,
            } => {
                loop {
                    self.tick(*span)?;
                    match self.execute_statements(body, scope)? {
                        Flow::Normal => {}
                        Flow::Continue { depth, .. } if depth <= 1 => {}
                        Flow::Continue { depth, span } => {
                            return Ok(Flow::Continue {
                                depth: depth - 1,
                                span,
                            });
                        }
                        Flow::Break { depth, .. } if depth <= 1 => break,
                        Flow::Break { depth, span } => {
                            return Ok(Flow::Break {
                                depth: depth - 1,
                                span,
                            });
                        }
                        flow @ (Flow::Return(_) | Flow::Goto { .. } | Flow::Exit(_)) => {
                            return Ok(flow);
                        }
                    }

                    if !self.evaluate(condition, scope)?.is_truthy() {
                        break;
                    }
                }
                Ok(Flow::Normal)
            }
            Stmt::For {
                initializers,
                conditions,
                increments,
                body,
                span,
            } => {
                for initializer in initializers {
                    self.execute_for_action(initializer, scope)?;
                }

                loop {
                    self.tick(*span)?;
                    if !conditions.is_empty() {
                        let mut keep_running = true;
                        for condition in conditions {
                            keep_running = self.evaluate(condition, scope)?.is_truthy();
                        }
                        if !keep_running {
                            break;
                        }
                    }

                    match self.execute_statements(body, scope)? {
                        Flow::Normal => {}
                        Flow::Continue { depth, .. } if depth <= 1 => {}
                        Flow::Continue { depth, span } => {
                            return Ok(Flow::Continue {
                                depth: depth - 1,
                                span,
                            });
                        }
                        Flow::Break { depth, .. } if depth <= 1 => break,
                        Flow::Break { depth, span } => {
                            return Ok(Flow::Break {
                                depth: depth - 1,
                                span,
                            });
                        }
                        flow @ (Flow::Return(_) | Flow::Goto { .. } | Flow::Exit(_)) => {
                            return Ok(flow);
                        }
                    }

                    for increment in increments {
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
                by_reference,
                body,
                span,
            } => {
                if *by_reference {
                    return Err(runtime_error(
                        *span,
                        RuntimeError::unsupported_call(
                            "foreach",
                            "by-reference iteration is not implemented; only by-value iteration is supported",
                        ),
                    ));
                }
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
                    self.tick(*span)?;
                    if let Some(key) = key {
                        scope.write_static(key, value_from_array_key(&entry.key));
                    }
                    scope.write_static(value, entry.value.clone());
                    match self.execute_statements(body, scope)? {
                        Flow::Normal => {}
                        Flow::Continue { depth, .. } if depth <= 1 => {}
                        Flow::Continue { depth, span } => {
                            return Ok(Flow::Continue {
                                depth: depth - 1,
                                span,
                            });
                        }
                        Flow::Break { depth, .. } if depth <= 1 => break,
                        Flow::Break { depth, span } => {
                            return Ok(Flow::Break {
                                depth: depth - 1,
                                span,
                            });
                        }
                        flow @ (Flow::Return(_) | Flow::Goto { .. } | Flow::Exit(_)) => {
                            return Ok(flow);
                        }
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
            Stmt::UnsetNestedArrayIndex {
                name,
                indices,
                span,
            } => {
                self.execute_unset_nested_array_index(name, indices, *span, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetObjectProperty {
                object,
                property,
                span,
            } => {
                self.execute_unset_object_property(object, property, *span, scope)?;
                Ok(Flow::Normal)
            }
            Stmt::UnsetDynamicObjectProperty {
                object,
                property,
                span,
            } => {
                self.execute_unset_dynamic_object_property(object, property, *span, scope)?;
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
            Stmt::Function(_) | Stmt::Interface(_) | Stmt::Trait(_) | Stmt::Enum(_) => {
                Ok(Flow::Normal)
            }
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
            Stmt::Try {
                body, finally_body, ..
            } => self.execute_try_statement(body, finally_body.as_deref(), scope),
            Stmt::Break { depth, span } => Ok(Flow::Break {
                depth: *depth,
                span: *span,
            }),
            Stmt::Continue { depth, span } => Ok(Flow::Continue {
                depth: *depth,
                span: *span,
            }),
            Stmt::Global { names, .. } => {
                if self.function_context.is_empty() {
                    for name in names {
                        if scope.read_named(name).is_none() {
                            scope.write_static(name, Value::Null);
                        }
                    }
                    Ok(Flow::Normal)
                } else {
                    for name in names {
                        scope.import_global(name);
                    }
                    Ok(Flow::Normal)
                }
            }
            Stmt::StaticLocal { declarations, span } => {
                self.execute_static_local_declaration(declarations, *span, scope)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn tick(&mut self, span: Span) -> CompileResult<()> {
        if let Some(max_steps) = self.max_execution_steps {
            if self.execution_steps >= max_steps {
                let source = self.source_file.as_deref().unwrap_or("<unknown>");
                let function_context = self
                    .function_context
                    .last()
                    .map(|name| format!("; function {name}()"))
                    .unwrap_or_default();
                return Err(Diagnostic::new(
                    Phase::Runtime,
                    span.line,
                    span.column,
                    format!(
                        "maximum execution step budget exceeded after {max_steps} step(s); last location {source}:{}:{}{function_context}",
                        span.line, span.column
                    ),
                ));
            }
        }
        self.execution_steps += 1;
        Ok(())
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

    fn execute_try_statement(
        &mut self,
        body: &[Stmt],
        finally_body: Option<&[Stmt]>,
        scope: &mut SymbolTable,
    ) -> CompileResult<Flow> {
        let try_flow = self.execute_statements(body, scope)?;
        if let Some(finally_body) = finally_body {
            let finally_flow = self.execute_statements(finally_body, scope)?;
            if !matches!(finally_flow, Flow::Normal) {
                return Ok(finally_flow);
            }
        }
        Ok(try_flow)
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
        let (flow, _) = self.evaluate_file_include(path, once, required, span, scope)?;
        match flow {
            Flow::Normal | Flow::Return(_) => Ok(Flow::Normal),
            Flow::Exit(code) => Ok(Flow::Exit(code)),
            Flow::Break { depth, span } => Ok(Flow::Break { depth, span }),
            Flow::Continue { depth, span } => Ok(Flow::Continue { depth, span }),
            Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
        }
    }

    fn evaluate_file_include(
        &mut self,
        path: &Expr,
        once: bool,
        required: bool,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<(Flow, Value)> {
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
        if self.trace_includes {
            eprintln!("phpc trace include: {}", path.source_file.display());
        }
        let include_key =
            fs::canonicalize(&path.read_path).unwrap_or_else(|_| path.read_path.clone());
        if once && self.required_once.contains(&include_key) {
            return Ok((Flow::Normal, Value::Bool(true)));
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
        self.required_once.insert(include_key);

        let previous_source_file = self.source_file.clone();
        self.source_file = Some(path.source_file.display().to_string());
        let flow = (|| {
            self.register_included_declarations(&program)?;
            self.execute_statements(&program.statements, scope)
        })();
        self.source_file = previous_source_file;

        match flow? {
            Flow::Normal => Ok((Flow::Normal, Value::Int(1))),
            Flow::Return(value) => Ok((Flow::Normal, value)),
            Flow::Exit(code) => Ok((Flow::Exit(code), Value::Null)),
            Flow::Break { depth, span } => Ok((Flow::Break { depth, span }, Value::Null)),
            Flow::Continue { depth, span } => Ok((Flow::Continue { depth, span }, Value::Null)),
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
                Flow::Break { depth, .. } if depth <= 1 => return Ok(Flow::Normal),
                Flow::Break { depth, span } => {
                    return Ok(Flow::Break {
                        depth: depth - 1,
                        span,
                    });
                }
                Flow::Continue { depth, span } if depth <= 1 => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control(
                            "continue inside switch is not implemented; use break for switch cases in the current subset",
                        ),
                    ));
                }
                Flow::Continue { depth, span } => {
                    return Ok(Flow::Continue {
                        depth: depth - 1,
                        span,
                    });
                }
                flow @ (Flow::Return(_) | Flow::Goto { .. } | Flow::Exit(_)) => return Ok(flow),
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
            UnsetTarget::NestedArrayIndex { name, indices, .. } => {
                self.execute_unset_nested_array_index(name, indices, span, scope)
            }
            UnsetTarget::ObjectProperty {
                object, property, ..
            } => self.execute_unset_object_property(object, property, span, scope),
            UnsetTarget::DynamicObjectProperty {
                object, property, ..
            } => self.execute_unset_dynamic_object_property(object, property, span, scope),
            UnsetTarget::ObjectPropertyArrayIndex {
                object,
                property,
                indices,
                ..
            } => self.execute_unset_object_property_nested_array_index(
                object, property, indices, span, scope,
            ),
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

    fn execute_unset_object_property(
        &mut self,
        object_name: &str,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        let object = match scope.read_static(object_name, span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot unset property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        if object
            .read_property_for_isset_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?
            .is_none()
        {
            return Ok(());
        }

        object
            .write_property_from_context(
                property,
                Value::Null,
                current_class_id,
                &protected_class_ids,
            )
            .map_err(|error| runtime_error(span, error))
    }

    fn execute_unset_dynamic_object_property(
        &mut self,
        object_name: &str,
        property: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let property_name = self.evaluate_dynamic_property_name(property, span, scope)?;
        self.execute_unset_object_property(object_name, &property_name, span, scope)
    }

    fn execute_unset_array_index(
        &mut self,
        name: &str,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let key = self.evaluate_array_key(index, scope)?;

        match scope.read_named(name) {
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

    fn execute_unset_nested_array_index(
        &mut self,
        name: &str,
        indices: &[Expr],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let keys = indices
            .iter()
            .map(|index| self.evaluate_array_key(index, scope))
            .collect::<CompileResult<Vec<_>>>()?;

        match scope.read_named(name) {
            Some(Value::Array(mut array)) => {
                Self::unset_nested_array_value(&mut array, &keys, span)?;
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

    fn unset_nested_array_value(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        span: Span,
    ) -> CompileResult<()> {
        let Some((key, rest)) = keys.split_first() else {
            return Ok(());
        };

        if rest.is_empty() {
            array.remove(key.clone());
            return Ok(());
        }

        let mut child = match array.get(key.clone()).cloned() {
            Some(Value::Array(child)) => child,
            Some(Value::Null) | None => return Ok(()),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(format!(
                        "cannot unset offset on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        Self::unset_nested_array_value(&mut child, rest, span)?;
        array.insert(key.clone(), Value::Array(child));
        Ok(())
    }

    fn execute_unset_object_property_nested_array_index(
        &mut self,
        object_name: &str,
        property: &str,
        indices: &[Expr],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let keys = indices
            .iter()
            .map(|index| self.evaluate_array_key(index, scope))
            .collect::<CompileResult<Vec<_>>>()?;
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        let object = match scope.read_static(object_name, span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot unset property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let Some(mut slot) = object
            .read_property_for_isset_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?
        else {
            return Ok(());
        };

        match &mut slot {
            Value::Array(array) => {
                Self::unset_nested_array_value(array, &keys, span)?;
                object
                    .write_property_from_context(
                        property,
                        slot,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))
            }
            Value::Null => Ok(()),
            other => Err(runtime_error(
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
            Expr::InterpolatedString { parts, span } => {
                self.evaluate_interpolated_string(parts, *span, scope)
            }
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
            Expr::MagicClass { span } => Ok(Value::String(self.magic_class_value(*span)?)),
            Expr::MagicMethod { span } => Ok(Value::String(self.magic_method_value(*span)?)),
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
            Expr::DynamicProperty {
                target,
                property,
                span,
            } => self.evaluate_dynamic_property_read(target, property, *span, scope),
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
            Expr::Closure {
                captures,
                span,
                is_arrow,
                ..
            } => self.evaluate_closure_expression(captures, *is_arrow, *span, scope),
            Expr::New {
                class_name,
                args,
                span,
            } => self.instantiate_object(class_name, args, *span, scope),
            Expr::Clone { expr, span } => self.evaluate_clone_expression(expr, *span, scope),
            Expr::Unary { op, expr, span } => {
                let value = self.evaluate(expr, scope)?;
                self.apply_unary(*op, value, *span)
            }
            Expr::ErrorControl { expr, .. } => self.evaluate(expr, scope),
            Expr::Include { path, once, span } => {
                let (flow, value) = self.evaluate_file_include(path, *once, false, *span, scope)?;
                match flow {
                    Flow::Normal | Flow::Return(_) => Ok(value),
                    Flow::Exit(_) => Ok(Value::Null),
                    Flow::Break { span, .. } => Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
                    )),
                    Flow::Continue { span, .. } => Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control(
                            "continue cannot be used outside a loop",
                        ),
                    )),
                    Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
                }
            }
            Expr::Require { path, once, span } => {
                let (flow, value) = self.evaluate_file_include(path, *once, true, *span, scope)?;
                match flow {
                    Flow::Normal | Flow::Return(_) => Ok(value),
                    Flow::Exit(_) => Ok(Value::Null),
                    Flow::Break { span, .. } => Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
                    )),
                    Flow::Continue { span, .. } => Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control(
                            "continue cannot be used outside a loop",
                        ),
                    )),
                    Flow::Goto { label, span } => Err(undefined_goto_label_error(span, &label)),
                }
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
        class_name: &NewClassName,
        args: &[Expr],
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let class_name = self.resolve_new_class_name(class_name, span)?;
        let (class_id, declared_class_name) = {
            let class = self
                .classes
                .lookup_class(&class_name)
                .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(&class_name)))?;
            (class.id(), class.name().to_string())
        };
        if self.abstract_classes.contains(&class_id) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(
                    declared_class_name,
                    "abstract classes are not instantiable in the current subset",
                ),
            ));
        }

        let constructor = self.resolve_instance_method(class_id, "__construct");

        let object_id = self.allocate_object_id();

        let inherited_properties = self.inherited_instance_properties(class_id);
        let class = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata");
        let object = PhpObject::from_class_with_inherited_properties_with_id(
            class,
            &inherited_properties,
            object_id,
        );
        self.apply_instance_property_defaults(&object, class_id)?;
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

        let function = self.method_function(
            constructor_class_id,
            &constructor_class_name,
            &constructor_name,
            span,
        )?;
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

    fn resolve_new_class_name(
        &self,
        class_name: &NewClassName,
        span: Span,
    ) -> CompileResult<String> {
        match class_name {
            NewClassName::Named(name) => Ok(name.clone()),
            NewClassName::SelfClass => {
                let Some(current_class_id) = self.class_context.last().copied() else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_object_instantiation(
                            "self",
                            "self requires active class context",
                        ),
                    ));
                };
                Ok(self
                    .classes
                    .get(current_class_id)
                    .expect("active class context should resolve to class metadata")
                    .name()
                    .to_string())
            }
            NewClassName::ParentClass => {
                let Some(current_class_id) = self.class_context.last().copied() else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_object_instantiation(
                            "parent",
                            "parent requires active class context",
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
                        RuntimeError::unsupported_object_instantiation(
                            "parent",
                            "parent requires a parent class",
                        ),
                    ));
                };
                Ok(self
                    .classes
                    .get(parent_class_id)
                    .expect("parent class id should resolve to class metadata")
                    .name()
                    .to_string())
            }
            NewClassName::StaticClass => {
                let Some(called_class_id) = self.called_class_context.last().copied() else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_object_instantiation(
                            "static",
                            "static requires method or static class context",
                        ),
                    ));
                };
                Ok(self
                    .classes
                    .get(called_class_id)
                    .expect("called class context should resolve to class metadata")
                    .name()
                    .to_string())
            }
        }
    }

    fn allocate_object_id(&mut self) -> i64 {
        let object_id = self.next_object_id;
        self.next_object_id = self
            .next_object_id
            .checked_add(1)
            .expect("object id counter fits in i64");
        object_id
    }

    fn evaluate_clone_expression(
        &mut self,
        expr: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let value = self.evaluate(expr, scope)?;
        let Value::Object(object) = value else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "clone",
                    format!(
                        "clone operand must be object in the current subset, got {}",
                        value.type_name()
                    ),
                ),
            ));
        };

        if self
            .resolve_instance_method(object.class_id(), "__clone")
            .is_some()
        {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call("clone", "__clone dispatch is not implemented"),
            ));
        }

        let object_id = self.allocate_object_id();
        Ok(Value::Object(object.shallow_clone_with_id(object_id)))
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

    fn apply_instance_property_defaults(
        &self,
        object: &PhpObject,
        class_id: ClassId,
    ) -> CompileResult<()> {
        for declaring_class_id in self.instance_property_default_class_order(class_id) {
            let Some(class) = self.classes.get(declaring_class_id) else {
                continue;
            };
            for property in class.properties().iter().filter(|property| {
                !property.is_static()
                    && self
                        .instance_property_defaults
                        .contains_key(&(declaring_class_id, property.name().to_string()))
            }) {
                let value = self
                    .instance_property_defaults
                    .get(&(declaring_class_id, property.name().to_string()))
                    .expect("default existence checked")
                    .clone();
                object
                    .write_property_from_context(
                        property.name(),
                        value,
                        Some(declaring_class_id),
                        &[declaring_class_id],
                    )
                    .map_err(|error| runtime_error(Span::new(0, 0), error))?;
            }
        }

        Ok(())
    }

    fn instance_property_default_class_order(&self, class_id: ClassId) -> Vec<ClassId> {
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
        ancestors.push(class_id);
        ancestors
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

    fn execute_reference_assignment(
        &mut self,
        target: &AssignTarget,
        source: &ReferenceSource,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let unsupported = || {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "references and aliasing are not implemented",
                ),
            )
        };

        match target {
            AssignTarget::Variable { name, .. } => {
                let value =
                    self.evaluate_direct_variable_reference_source_value(source, span, scope)?;
                scope.write_static(name, value);
                Ok(())
            }
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                ..
            } => {
                let key = self.evaluate_array_key(index, scope)?;
                let value = self.evaluate_container_reference_source_value(source, span, scope)?;
                let mut slot = scope
                    .read_named(name)
                    .unwrap_or_else(|| Value::Array(PhpArray::new()));

                if matches!(slot, Value::Null) {
                    slot = Value::Array(PhpArray::new());
                }

                match &mut slot {
                    Value::Array(array) => {
                        array.insert(key, value);
                        scope.write_static(name, slot);
                        Ok(())
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::invalid_array_access(format!(
                            "cannot write offset on {}",
                            other.type_name()
                        )),
                    )),
                }
            }
            _ => Err(unsupported()),
        }
    }

    fn evaluate_container_reference_source_value(
        &mut self,
        source: &ReferenceSource,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let value = match source {
            ReferenceSource::Variable { name, .. } => scope.read_static(name, span)?,
            ReferenceSource::Property { expr, .. } => self.evaluate(expr, scope)?,
            ReferenceSource::ArrayIndex { .. } | ReferenceSource::MethodCall { .. } => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "references and aliasing are not implemented",
                    ),
                ));
            }
        };

        match value {
            Value::Object(_) | Value::Array(_) => Ok(value),
            _ => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "references and aliasing are not implemented",
                ),
            )),
        }
    }

    fn evaluate_direct_variable_reference_source_value(
        &mut self,
        source: &ReferenceSource,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let value = match source {
            ReferenceSource::Variable { name, .. } => scope.read_static(name, span)?,
            ReferenceSource::Property { expr, .. } => self.evaluate(expr, scope)?,
            ReferenceSource::ArrayIndex { .. } | ReferenceSource::MethodCall { .. } => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "references and aliasing are not implemented",
                    ),
                ));
            }
        };

        match value {
            Value::Object(_) | Value::Array(_) => Ok(value),
            _ => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "references and aliasing are not implemented",
                ),
            )),
        }
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
                let mut slot = scope
                    .read_named(name)
                    .unwrap_or_else(|| Value::Array(PhpArray::new()));

                if matches!(slot, Value::Null) {
                    slot = Value::Array(PhpArray::new());
                }

                match &mut slot {
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
                scope.write_static(name, slot);

                Ok(value)
            }
            AssignTarget::NestedArrayIndex {
                name,
                indices,
                span,
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                let value = self.evaluate(expr, scope)?;
                Self::write_nested_array_assignment(name, &keys, value.clone(), *span, scope)?;
                Ok(value)
            }
            AssignTarget::NestedArrayAppend {
                name,
                indices,
                span,
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                let value = self.evaluate(expr, scope)?;
                Self::write_nested_array_append(name, &keys, value.clone(), *span, scope)?;
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
                match scope.read_static(object, *span)? {
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
            AssignTarget::ObjectPropertyArrayIndex {
                object,
                property,
                indices,
                span,
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                let value = self.evaluate(expr, scope)?;
                self.write_object_property_nested_array_assignment(
                    object,
                    property,
                    &keys,
                    value.clone(),
                    *span,
                    scope,
                )?;
                Ok(value)
            }
            AssignTarget::ObjectPropertyArrayAppend {
                object,
                property,
                indices,
                span,
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                let value = self.evaluate(expr, scope)?;
                self.write_object_property_nested_array_append(
                    object,
                    property,
                    &keys,
                    value.clone(),
                    *span,
                    scope,
                )?;
                Ok(value)
            }
            AssignTarget::DynamicProperty {
                object,
                property,
                span,
            } => {
                let property = self.evaluate_dynamic_property_name(property, *span, scope)?;
                let value = self.evaluate(expr, scope)?;
                match scope.read_static(object, *span)? {
                    Value::Object(object) => object
                        .write_dynamic_public_property(&property, value.clone())
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

    fn write_nested_array_assignment(
        name: &str,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let mut slot = scope
            .read_named(name)
            .unwrap_or_else(|| Value::Array(PhpArray::new()));

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::write_nested_array_value(array, keys, value, span)?;
                scope.write_static(name, slot);
                Ok(())
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot write offset on {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn write_object_property_nested_array_assignment(
        &mut self,
        object_name: &str,
        property: &str,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        let object = match scope.read_static(object_name, span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot write property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let mut slot = object
            .read_property_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?;

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::write_nested_array_value(array, keys, value, span)?;
                object
                    .write_property_from_context(
                        property,
                        slot,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot write offset on {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn write_object_property_nested_array_append(
        &mut self,
        object_name: &str,
        property: &str,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        let object = match scope.read_static(object_name, span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot write property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let mut slot = object
            .read_property_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?;

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::append_nested_array_value(array, keys, value, span)?;
                object
                    .write_property_from_context(
                        property,
                        slot,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot write offset on {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn write_nested_array_value(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
    ) -> CompileResult<()> {
        let Some((key, rest)) = keys.split_first() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array assignment",
                    "nested array assignment requires at least one key",
                ),
            ));
        };

        if rest.is_empty() {
            array.insert(key.clone(), value);
            return Ok(());
        }

        let mut child = match array.get(key.clone()).cloned() {
            Some(Value::Array(child)) => child,
            Some(Value::Null) | None => PhpArray::new(),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(format!(
                        "cannot write offset on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        Self::write_nested_array_value(&mut child, rest, value, span)?;
        array.insert(key.clone(), Value::Array(child));
        Ok(())
    }

    fn write_nested_array_append(
        name: &str,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let mut slot = scope
            .read_named(name)
            .unwrap_or_else(|| Value::Array(PhpArray::new()));

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::append_nested_array_value(array, keys, value, span)?;
                scope.write_static(name, slot);
                Ok(())
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot write offset on {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn append_nested_array_value(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
    ) -> CompileResult<()> {
        let Some((key, rest)) = keys.split_first() else {
            array
                .append(value)
                .map_err(|error| runtime_error(span, error))?;
            return Ok(());
        };

        let mut child = match array.get(key.clone()).cloned() {
            Some(Value::Array(child)) => child,
            Some(Value::Null) | None => PhpArray::new(),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(format!(
                        "cannot write offset on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        Self::append_nested_array_value(&mut child, rest, value, span)?;
        array.insert(key.clone(), Value::Array(child));
        Ok(())
    }

    fn evaluate_list_assignment(
        &mut self,
        names: &[Option<String>],
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
            .filter_map(|(index, name)| {
                name.as_ref().map(|name| {
                    let element = array
                        .get(ArrayKey::Int(index as i64))
                        .cloned()
                        .unwrap_or(Value::Null);
                    (name.clone(), element)
                })
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
            AssignTarget::NestedArrayIndex { .. }
            | AssignTarget::NestedArrayAppend { .. }
            | AssignTarget::ObjectPropertyArrayIndex { .. }
            | AssignTarget::ObjectPropertyArrayAppend { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "nested array targets are not implemented",
                ),
            )),
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
            AssignTarget::DynamicProperty { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "dynamic property targets are not implemented",
                ),
            )),
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
            CompoundAssignmentPlace::ArrayIndex { name, key } => match scope.read_named(&name) {
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
            },
            CompoundAssignmentPlace::ObjectProperty { object, property } => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                match scope.read_static(&object, span)? {
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
            AssignTarget::NestedArrayIndex { .. }
            | AssignTarget::NestedArrayAppend { .. }
            | AssignTarget::ObjectPropertyArrayIndex { .. }
            | AssignTarget::ObjectPropertyArrayAppend { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "nested array targets are not implemented",
                ),
            )),
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
            AssignTarget::DynamicProperty { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "dynamic property targets are not implemented",
                ),
            )),
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
                    let mut slot = scope
                        .read_named(name)
                        .unwrap_or_else(|| Value::Array(PhpArray::new()));

                    if matches!(slot, Value::Null) {
                        slot = Value::Array(PhpArray::new());
                    }

                    match &mut slot {
                        Value::Array(array) => {
                            array.insert(key, value.clone());
                            scope.write_static(name, slot);
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
            AssignTarget::NestedArrayIndex { span, .. }
            | AssignTarget::NestedArrayAppend { span, .. }
            | AssignTarget::ObjectPropertyArrayIndex { span, .. }
            | AssignTarget::ObjectPropertyArrayAppend { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call("??=", "nested array targets are not implemented"),
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
                    match scope.read_static(object, *span)? {
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
            AssignTarget::DynamicProperty { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "??=",
                    "dynamic property targets are not implemented",
                ),
            )),
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

    fn magic_method_value(&self, span: Span) -> CompileResult<String> {
        let Some(function_name) = self.function_context.last() else {
            return Ok(String::new());
        };
        let Some(class_id) = self.class_context.last().copied() else {
            return Ok(function_name.clone());
        };
        let class = self.classes.get(class_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_class(format!("class id {}", class_id.index())),
            )
        })?;
        Ok(format!("{}::{function_name}", class.name()))
    }

    fn magic_class_value(&self, span: Span) -> CompileResult<String> {
        let Some(class_id) = self.class_context.last().copied() else {
            return Ok(String::new());
        };
        let class = self.classes.get(class_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_class(format!("class id {}", class_id.index())),
            )
        })?;
        Ok(class.name().to_string())
    }

    fn create_mysqli_placeholder(&mut self, span: Span) -> CompileResult<Value> {
        let class_id = self.classes.lookup_class_id("mysqli").ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_class("mysqli core placeholder"),
            )
        })?;
        let object_id = self.allocate_object_id();
        let class = self
            .classes
            .get(class_id)
            .expect("core mysqli class id should resolve");
        let object = PhpObject::from_class_with_id(class, object_id);
        object
            .write_public_property("connect_errno", Value::Int(0))
            .map_err(|error| runtime_error(span, error))?;
        object
            .write_public_property("connect_error", Value::Null)
            .map_err(|error| runtime_error(span, error))?;
        Ok(Value::Object(object))
    }

    fn call_mysqli_real_connect(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(1..=8).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_real_connect()",
                    ArityExpectation::Between { min: 1, max: 8 },
                    args.len(),
                ),
            ));
        }

        let Value::Object(handle) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_connect()",
                    format!(
                        "first argument must be mysqli object in the current subset, got {}",
                        args[0].type_name()
                    ),
                ),
            ));
        };
        if !handle.class_name().eq_ignore_ascii_case("mysqli") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_connect()",
                    format!(
                        "first argument must be mysqli object in the current subset, got {} object",
                        handle.class_name()
                    ),
                ),
            ));
        }

        for (index, label) in ["hostname", "username", "password", "database"]
            .iter()
            .enumerate()
        {
            let arg_index = index + 1;
            if let Some(value) = args.get(arg_index) {
                if !matches!(value, Value::String(_) | Value::Null) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_real_connect()",
                            format!(
                                "{label} argument must be string or null in the current subset, got {}",
                                value.type_name()
                            ),
                        ),
                    ));
                }
            }
        }

        if let Some(port) = args.get(5) {
            if !matches!(port, Value::Int(_) | Value::Null) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_connect()",
                        format!(
                            "port argument must be int or null in the current subset, got {}",
                            port.type_name()
                        ),
                    ),
                ));
            }
        }

        if let Some(socket) = args.get(6) {
            if !matches!(socket, Value::String(_) | Value::Null) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_connect()",
                        format!(
                            "socket argument must be string or null in the current subset, got {}",
                            socket.type_name()
                        ),
                    ),
                ));
            }
        }

        if let Some(flags) = args.get(7) {
            if !matches!(flags, Value::Int(_)) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_connect()",
                        format!(
                            "flags argument must be int in the current subset, got {}",
                            flags.type_name()
                        ),
                    ),
                ));
            }
        }

        handle
            .write_public_property("connect_errno", Value::Int(0))
            .map_err(|error| runtime_error(span, error))?;
        handle
            .write_public_property("connect_error", Value::Null)
            .map_err(|error| runtime_error(span, error))?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_get_server_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_server_info", args, 1, span)?;
        let Value::Object(handle) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_get_server_info()",
                    format!(
                        "argument must be mysqli object in the current subset, got {}",
                        args[0].type_name()
                    ),
                ),
            ));
        };
        if !handle.class_name().eq_ignore_ascii_case("mysqli") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_get_server_info()",
                    format!(
                        "argument must be mysqli object in the current subset, got {} object",
                        handle.class_name()
                    ),
                ),
            ));
        }

        Ok(Value::String("8.0.0-phpc-placeholder".to_string()))
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
        match scope.read_named(name) {
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

    fn evaluate_dynamic_property_read(
        &mut self,
        target: &Expr,
        property: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let property = self.evaluate_dynamic_property_name(property, span, scope)?;
        let target_value = self.evaluate(target, scope)?;

        match target_value {
            Value::Object(object) => object
                .read_public_property(&property)
                .map_err(|error| runtime_error(span, error)),
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_property_access(format!(
                    "cannot read property ${property} from {}",
                    other.type_name()
                )),
            )),
        }
    }

    fn evaluate_dynamic_property_name(
        &mut self,
        property: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<String> {
        match self.evaluate(property, scope)? {
            Value::String(value) => Ok(value),
            Value::Int(value) => Ok(value.to_string()),
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_property_access(format!(
                    "dynamic property names only support strings and integers in the current subset, got {}",
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

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
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

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
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

            let function = self.method_function(
                declaring_class_id,
                &declaring_class_name,
                &_resolved_method_name,
                span,
            )?;
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

        let function = self.method_function(
            declaring_class_id,
            &declaring_class_name,
            &resolved_method_name,
            span,
        )?;
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

    fn evaluate_interpolated_string(
        &self,
        parts: &[InterpolatedStringPart],
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<Value> {
        let mut output = String::new();
        for part in parts {
            match part {
                InterpolatedStringPart::Literal(value) => output.push_str(value),
                InterpolatedStringPart::Variable(name) => {
                    let value = scope.read_static(name, span)?;
                    let text = value
                        .try_echo_string()
                        .map_err(|error| runtime_error(span, error))?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::ArrayOffset { variable, key } => {
                    let value =
                        self.evaluate_interpolated_array_offset(variable, key, span, scope)?;
                    let text = value
                        .try_echo_string()
                        .map_err(|error| runtime_error(span, error))?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::ObjectProperty { variable, property } => {
                    let value = self
                        .evaluate_interpolated_object_property(variable, property, span, scope)?;
                    let text = value
                        .try_echo_string()
                        .map_err(|error| runtime_error(span, error))?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::AccessChain { variable, segments } => {
                    let value =
                        self.evaluate_interpolated_access_chain(variable, segments, span, scope)?;
                    let text = value
                        .try_echo_string()
                        .map_err(|error| runtime_error(span, error))?;
                    output.push_str(&text);
                }
            }
        }
        Ok(Value::String(output))
    }

    fn evaluate_interpolated_access_chain(
        &self,
        variable: &str,
        segments: &[InterpolatedAccessSegment],
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<Value> {
        let mut value = scope.read_static(variable, span)?;

        for segment in segments {
            value = match segment {
                InterpolatedAccessSegment::ArrayOffset(key) => {
                    let key = self.evaluate_interpolated_array_key(key, span, scope)?;
                    self.read_interpolated_array_offset(value, key, span)?
                }
                InterpolatedAccessSegment::ObjectProperty(property) => {
                    self.read_interpolated_object_property(value, property, span)?
                }
            };
        }

        Ok(value)
    }

    fn evaluate_interpolated_array_offset(
        &self,
        variable: &str,
        key: &InterpolatedArrayKey,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<Value> {
        let array = scope.read_static(variable, span)?;
        let key = self.evaluate_interpolated_array_key(key, span, scope)?;
        self.read_interpolated_array_offset(array, key, span)
    }

    fn evaluate_interpolated_array_key(
        &self,
        key: &InterpolatedArrayKey,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<ArrayKey> {
        Ok(match key {
            InterpolatedArrayKey::Int(value) => ArrayKey::Int(*value),
            InterpolatedArrayKey::String(value) => ArrayKey::String(value.clone()),
            InterpolatedArrayKey::Variable(name) => {
                let value = scope.read_static(name, span)?;
                ArrayKey::from_value(&value).map_err(|error| runtime_error(span, error))?
            }
        })
    }

    fn read_interpolated_array_offset(
        &self,
        array: Value,
        key: ArrayKey,
        span: Span,
    ) -> CompileResult<Value> {
        match array {
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

    fn evaluate_interpolated_object_property(
        &self,
        variable: &str,
        property: &str,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<Value> {
        let object = scope.read_static(variable, span)?;
        self.read_interpolated_object_property(object, property, span)
    }

    fn read_interpolated_object_property(
        &self,
        object: Value,
        property: &str,
        span: Span,
    ) -> CompileResult<Value> {
        match object {
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

    fn class_constant_lookup_string_is_defined(&self, class_name: &str, constant: &str) -> bool {
        let Some(class_id) = self.classes.lookup_class_id(class_name) else {
            return false;
        };

        matches!(
            self.resolve_class_constant(class_id, constant),
            Some((_, _, Visibility::Public, _))
        )
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

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
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

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
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

    fn method_function(
        &self,
        class_id: ClassId,
        class_name: &str,
        method_name: &str,
        span: Span,
    ) -> CompileResult<Rc<FunctionDecl>> {
        let key = (class_id, method_name.to_ascii_lowercase());
        if self.abstract_methods.contains(&key) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "abstract methods are not executable in the current subset",
                ),
            ));
        }

        self.methods.get(&key).cloned().ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_function(format!("{class_name}::{method_name}()")),
            )
        })
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
                let value = if property.is_static() {
                    self.static_properties
                        .get(&(class.id(), property.name().to_string()))
                        .cloned()
                        .unwrap_or(Value::Null)
                } else {
                    self.instance_property_defaults
                        .get(&(class.id(), property.name().to_string()))
                        .cloned()
                        .unwrap_or(Value::Null)
                };
                properties.insert(ArrayKey::from(property.name()), value);
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

    fn evaluate_closure_expression(
        &mut self,
        captures: &[ClosureCapture],
        is_arrow: bool,
        _span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let mut captured_values = Vec::with_capacity(captures.len());
        for capture in captures {
            let value = scope.read_static(&capture.name, capture.span)?;
            captured_values.push(PhpClosureCapture::new(
                capture.name.clone(),
                capture.by_reference,
                value,
            ));
        }

        let id = self.next_closure_id;
        self.next_closure_id += 1;
        Ok(Value::Closure(PhpClosure::new(
            id,
            is_arrow,
            captured_values,
        )))
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let key = name.to_ascii_lowercase();
        let fallback_key = name
            .rsplit_once('\\')
            .map(|(_, suffix)| suffix.to_ascii_lowercase());
        if key == "isset" || fallback_key.as_deref() == Some("isset") {
            return self.call_isset(args, span, caller_scope);
        }
        if key == "empty" || fallback_key.as_deref() == Some("empty") {
            return self.call_empty(args, span, caller_scope);
        }

        self.call_direct_named_function(name, args, span, caller_scope)
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
            Value::Closure(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "closure",
                        "closure invocation is not implemented",
                    ),
                ));
            }
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
        match self.lookup_function_exact(name).ok_or_else(|| {
            runtime_error(span, RuntimeError::undefined_function(callable_name(name)))
        })? {
            Callable::Builtin(key) => {
                if key == "spl_autoload_register" {
                    return self.call_spl_autoload_register(args, span, caller_scope);
                }
                if key == "preg_match" {
                    return self.call_preg_match_with_optional_matches(args, span, caller_scope);
                }
                if key == "compact" {
                    return self.call_compact(args, span, caller_scope);
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

    fn call_direct_named_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if matches!(name.to_ascii_lowercase().as_str(), "exit" | "die") {
            return self.call_exit_construct(name, args, span, caller_scope);
        }

        match self.lookup_direct_function_call(name).ok_or_else(|| {
            runtime_error(span, RuntimeError::undefined_function(callable_name(name)))
        })? {
            Callable::Builtin(key) => {
                if key == "spl_autoload_register" {
                    return self.call_spl_autoload_register(args, span, caller_scope);
                }
                if key == "preg_match" {
                    return self.call_preg_match_with_optional_matches(args, span, caller_scope);
                }
                if key == "compact" {
                    return self.call_compact(args, span, caller_scope);
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

    fn call_preg_match_with_optional_matches(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if !(2..=5).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "preg_match()",
                    ArityExpectation::Between { min: 2, max: 5 },
                    args.len(),
                ),
            ));
        }

        if args.len() > 3 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_match()",
                    "flags and offset arguments are not implemented; pass at most a direct matches variable in the current subset",
                ),
            ));
        }

        let pattern = self.evaluate(&args[0], caller_scope)?;
        let subject = self.evaluate(&args[1], caller_scope)?;
        let values = vec![pattern, subject];
        if args.len() == 2 {
            return call_preg_match(&values, span);
        }

        let Expr::Variable(matches_name, _) = &args[2] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_match()",
                    "matches output must be a direct variable in the current subset",
                ),
            ));
        };

        let pattern = string_contains_argument("preg_match()", "pattern", &values[0], span)?;
        let subject = string_contains_argument("preg_match()", "subject", &values[1], span)?;
        let pattern = BoundedPregPattern::parse(&pattern).map_err(|message| {
            runtime_error(
                span,
                RuntimeError::unsupported_call("preg_match()", message),
            )
        })?;

        if let Some(matches) = pattern.captures(&subject) {
            caller_scope.write_static(matches_name, Value::Array(matches));
            Ok(Value::Int(1))
        } else {
            caller_scope.write_static(matches_name, Value::Array(PhpArray::new()));
            Ok(Value::Int(0))
        }
    }

    fn call_compact(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch("compact()", ArityExpectation::AtLeast(1), args.len()),
            ));
        }

        let mut compacted = PhpArray::new();
        for arg in args {
            let value = self.evaluate(arg, caller_scope)?;
            let Value::String(name) = value else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "compact()",
                        format!(
                            "variable names must be direct strings in the current subset, got {}",
                            value.type_name()
                        ),
                    ),
                ));
            };
            if !is_compact_variable_name(&name) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "compact()",
                        "variable names must be non-empty simple identifiers in the current subset",
                    ),
                ));
            }
            if let Some(value) = caller_scope.read_named(&name) {
                compacted.insert(name, value);
            }
        }

        Ok(Value::Array(compacted))
    }

    fn call_exit_construct(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let construct = if name.eq_ignore_ascii_case("die") {
            "die()"
        } else {
            "exit()"
        };

        if args.len() > 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    construct,
                    ArityExpectation::Between { min: 0, max: 1 },
                    args.len(),
                ),
            ));
        }

        let mut code = 0;
        if let Some(arg) = args.first() {
            match self.evaluate(arg, caller_scope)? {
                Value::Null => {}
                Value::Int(value) => {
                    code = i32::try_from(value).map_err(|_| {
                        runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                construct,
                                "integer status must fit in i32 in the current subset",
                            ),
                        )
                    })?;
                }
                Value::String(value) => {
                    self.stdout.push_str(&value);
                }
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            construct,
                            format!(
                                "argument must be null, int, or string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }
            }
        }

        self.exit_signal = Some(code);
        Ok(Value::Null)
    }

    fn lookup_function(&self, name: &str) -> Option<Callable> {
        self.lookup_function_exact(name)
    }

    fn lookup_function_exact(&self, name: &str) -> Option<Callable> {
        let key = name.to_ascii_lowercase();
        if is_builtin(&key) {
            return Some(Callable::Builtin(key));
        }

        self.functions.get(&key).cloned().map(Callable::User)
    }

    fn lookup_direct_function_call(&self, name: &str) -> Option<Callable> {
        if let Some(callable) = self.lookup_function_exact(name) {
            return Some(callable);
        }

        let (_, suffix) = name.rsplit_once('\\')?;
        self.lookup_function_exact(suffix)
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
        let mut local_scope = SymbolTable::new_child(self.global_symbols.clone());
        if let Some(this_object) = this_object {
            local_scope.write_static("this", Value::Object(this_object));
        }
        for (index, param) in function.params.iter().enumerate() {
            if param.is_variadic {
                let mut rest = PhpArray::new();
                for arg in args.iter().skip(index) {
                    rest.append(arg.clone())
                        .map_err(|error| runtime_error(param.span, error))?;
                }
                local_scope.write_static(&param.name, Value::Array(rest));
                break;
            }

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
            Flow::Break { span, .. } => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
            )),
            Flow::Continue { span, .. } => Err(runtime_error(
                span,
                RuntimeError::invalid_loop_control("continue cannot be used outside a loop"),
            )),
            Flow::Return(value) => Ok(value),
            Flow::Exit(_) => Ok(Value::Null),
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
        let Some(candidate_id) = self.value_class_id(object_or_class, allow_string) else {
            return false;
        };

        if self.classes.implements_interface(candidate_id, class_name) {
            return true;
        }

        let Some(target_class) = self.classes.lookup_class(class_name) else {
            return false;
        };

        candidate_id == target_class.id()
            || self.classes.is_subclass_of(candidate_id, target_class.id())
    }

    fn value_instanceof(&self, value: &Value, class_name: &str) -> bool {
        let Value::Object(object) = value else {
            return false;
        };
        if self
            .classes
            .implements_interface(object.class_id(), class_name)
        {
            return true;
        }
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
        let Some(candidate_id) = self.value_class_id(object_or_class, allow_string) else {
            return false;
        };

        if self.classes.implements_interface(candidate_id, class_name) {
            return true;
        }

        let Some(target_class) = self.classes.lookup_class(class_name) else {
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
                    Value::String(name) if is_supported_qualified_runtime_constant_name(name) => {
                        name.clone()
                    }
                    Value::String(name) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "define()",
                                format!(
                                    "constant name must be a non-empty supported identifier or qualified name in the current subset, got {name}"
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
            "strtolower" => call_strtolower(&args, span),
            "trim" => call_trim(&args, span),
            "strcasecmp" => call_strcasecmp(&args, span),
            "str_contains" => call_str_contains(&args, span),
            "strpos" => call_strpos(&args, span),
            "substr_count" => call_substr_count(&args, span),
            "str_replace" => call_str_replace(&args, span),
            "preg_match" => call_preg_match(&args, span),
            "preg_replace" => call_preg_replace(&args, span),
            "compact" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compact()",
                    "caller-scope variable lookup is only implemented for direct and dynamic compact() calls in the current subset",
                ),
            )),
            "error_reporting" => self.call_error_reporting(args, span),
            "sprintf" => call_sprintf(&args, span),
            "call_user_func" => self.call_user_func_builtin(args, span),
            "implode" => call_implode(&args, span),
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
            "abs" => call_abs(&args, span),
            "version_compare" => call_version_compare(&args, span),
            "microtime" => call_microtime(&args, span),
            "date_default_timezone_set" => call_date_default_timezone_set(&args, span),
            "ini_get" => call_ini_get(&args, span),
            "min" => call_min(&args, span),
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
                    Value::String(name) => {
                        if let Some((class_name, constant)) =
                            normalize_runtime_class_constant_lookup_name(name)
                        {
                            return self.evaluate_named_class_constant(class_name, constant, span);
                        }

                        let Some(normalized) = normalize_runtime_constant_lookup_name(name) else {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "constant()",
                                    format!(
                                        "constant name must be a non-empty supported identifier or qualified name in the current subset, got {name}"
                                    ),
                                ),
                            ));
                        };

                        self.constants.get(normalized).ok_or_else(|| {
                            runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "constant()",
                                    format!(
                                        "constant {normalized} is not defined in the current runtime-defined or built-in constant subset"
                                    ),
                                ),
                            )
                        })
                    }
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
                    Value::String(name) => {
                        if let Some((class_name, constant)) =
                            normalize_runtime_class_constant_lookup_name(name)
                        {
                            return Ok(Value::Bool(
                                self.class_constant_lookup_string_is_defined(class_name, constant),
                            ));
                        }

                        let Some(normalized) = normalize_runtime_constant_lookup_name(name) else {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "defined()",
                                    format!(
                                        "constant name must be a non-empty supported identifier or qualified name in the current subset, got {name}"
                                    ),
                                ),
                            ));
                        };
                        Ok(Value::Bool(self.constants.contains(normalized)))
                    }
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
                    Value::String(name) => Ok(Value::Bool(is_compat_loaded_extension_name(name))),
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
            "mysqli_connect" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_connect()",
                    "mysqli/database connections are not implemented in the current subset",
                ),
            )),
            "mysqli_real_connect" => self.call_mysqli_real_connect(&args, span),
            "mysqli_get_server_info" => self.call_mysqli_get_server_info(&args, span),
            "mysqli_report" => {
                expect_arity(name, &args, 1, span)?;
                let Value::Int(mode) = args[0] else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_report()",
                            format!(
                                "report mode must be int in the current subset, got {}",
                                args[0].type_name()
                            ),
                        ),
                    ));
                };
                if mode != PHP_MYSQLI_REPORT_OFF
                    && mode != (PHP_MYSQLI_REPORT_ERROR | PHP_MYSQLI_REPORT_STRICT)
                {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_report()",
                            "only MYSQLI_REPORT_OFF and MYSQLI_REPORT_ERROR|MYSQLI_REPORT_STRICT are supported in the current subset",
                        ),
                    ));
                }
                self.mysqli_report_mode = mode;
                Ok(Value::Bool(true))
            }
            "mysqli_init" => {
                expect_arity(name, &args, 0, span)?;
                self.create_mysqli_placeholder(span)
            }
            "file_exists" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "file_exists()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        Ok(Value::Bool(metadata_path.try_exists().map_err(|error| {
                            runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "file_exists()",
                                    format!("filesystem metadata lookup failed: {error}"),
                                ),
                            )
                        })?))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "file_exists()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "is_dir" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "is_dir()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        Ok(Value::Bool(
                            fs::metadata(&metadata_path)
                                .map(|metadata| metadata.is_dir())
                                .unwrap_or(false),
                        ))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_dir()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "is_readable" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "is_readable()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        let Ok(metadata) = fs::metadata(&metadata_path) else {
                            return Ok(Value::Bool(false));
                        };
                        let readable = if metadata.is_dir() {
                            fs::read_dir(&metadata_path).is_ok()
                        } else {
                            fs::File::open(&metadata_path).is_ok()
                        };
                        Ok(Value::Bool(readable))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_readable()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "register_shutdown_function" => {
                if args.is_empty() {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "register_shutdown_function()",
                            ArityExpectation::AtLeast(1),
                            args.len(),
                        ),
                    ));
                }

                match &args[0] {
                    Value::String(name) if self.lookup_function(name).is_some() => {
                        Ok(Value::Null)
                    }
                    Value::String(_) => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "register_shutdown_function()",
                            "callback must be a valid callable in the current subset",
                        ),
                    )),
                    Value::Array(array) if is_array_callable_resolved(&self.classes, array) => {
                        Ok(Value::Null)
                    }
                    Value::Array(_) => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "register_shutdown_function()",
                            "callback must be a valid callable in the current subset",
                        ),
                    )),
                    Value::Closure(_) => Ok(Value::Null),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "register_shutdown_function()",
                            format!(
                                "callback argument must be string, array callable, or closure in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "set_error_handler" => self.call_set_error_handler(args, span),
            "restore_error_handler" => self.call_restore_error_handler(args, span),
            "header" => call_header(&args, span),
            "header_remove" => call_header_remove(&args, span),
            "headers_sent" => call_headers_sent(&args, span),
            "assert" => {
                if !(1..=2).contains(&args.len()) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "assert()",
                            ArityExpectation::Between { min: 1, max: 2 },
                            args.len(),
                        ),
                    ));
                }

                if let Some(other) = args
                    .get(1)
                    .filter(|value| {
                        !matches!(
                            value,
                            Value::Null
                                | Value::Bool(_)
                                | Value::Int(_)
                                | Value::Float(_)
                                | Value::String(_)
                        )
                    })
                {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "assert()",
                            format!(
                                "description argument must be null, bool, int, float, or string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }

                if args[0].is_truthy() {
                    Ok(Value::Bool(true))
                } else {
                    Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "assert()",
                            "assertion failures are not implemented in the current subset",
                        ),
                    ))
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
                        Ok(Value::Bool(
                            self.classes.lookup_class(class_name).is_some()
                                || self
                                    .enum_lookup
                                    .contains_key(&class_name.to_ascii_lowercase()),
                        ))
                    }
                    [Value::String(class_name), autoload] => {
                        let _autoload =
                            metadata_exists_autoload_flag("class_exists()", autoload, span)?;
                        Ok(Value::Bool(
                            self.classes.lookup_class(class_name).is_some()
                                || self
                                    .enum_lookup
                                    .contains_key(&class_name.to_ascii_lowercase()),
                        ))
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
                [Value::String(interface_name)] => Ok(Value::Bool(
                    self.interface_lookup
                        .contains_key(&interface_name.to_ascii_lowercase()),
                )),
                [Value::String(interface_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("interface_exists()", autoload, span)?;
                    Ok(Value::Bool(
                        self.interface_lookup
                            .contains_key(&interface_name.to_ascii_lowercase()),
                    ))
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
                [Value::String(trait_name)] => Ok(Value::Bool(
                    self.trait_lookup
                        .contains_key(&trait_name.to_ascii_lowercase()),
                )),
                [Value::String(trait_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("trait_exists()", autoload, span)?;
                    Ok(Value::Bool(
                        self.trait_lookup
                            .contains_key(&trait_name.to_ascii_lowercase()),
                    ))
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
                [Value::String(enum_name)] => Ok(Value::Bool(
                    self.enum_lookup
                        .contains_key(&enum_name.to_ascii_lowercase()),
                )),
                [Value::String(enum_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("enum_exists()", autoload, span)?;
                    Ok(Value::Bool(
                        self.enum_lookup
                            .contains_key(&enum_name.to_ascii_lowercase()),
                    ))
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
                for enum_decl in &self.enums {
                    classes
                        .append(Value::String(enum_decl.name.clone()))
                        .expect("declared class-like enum count fits in array keys");
                }
                Ok(Value::Array(classes))
            }
            "get_declared_interfaces" => {
                expect_arity(name, &args, 0, span)?;
                let mut interfaces = PhpArray::new();
                for interface in &self.interfaces {
                    interfaces
                        .append(Value::String(interface.name.clone()))
                        .expect("declared interface count fits in array keys");
                }
                Ok(Value::Array(interfaces))
            }
            "get_declared_traits" => {
                expect_arity(name, &args, 0, span)?;
                let mut traits = PhpArray::new();
                for trait_decl in &self.traits {
                    traits
                        .append(Value::String(trait_decl.name.clone()))
                        .expect("declared trait count fits in array keys");
                }
                Ok(Value::Array(traits))
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

    fn call_user_func_builtin(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        if args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "call_user_func()",
                    ArityExpectation::AtLeast(1),
                    args.len(),
                ),
            ));
        }

        let callback_name = match &args[0] {
            Value::String(name) => name,
            Value::Array(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "call_user_func()",
                        "array callables are not implemented in the current subset",
                    ),
                ));
            }
            Value::Closure(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "call_user_func()",
                        "closure invocation is not implemented",
                    ),
                ));
            }
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "call_user_func()",
                        format!(
                            "callback must evaluate to string in the current subset, got {}",
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

        self.call_callable_with_values(callable, args[1..].to_vec(), span)
    }

    fn call_set_error_handler(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "set_error_handler()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        match &args[0] {
            Value::String(name) if self.lookup_function(name).is_some() => {}
            Value::String(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "set_error_handler()",
                        "callback must be a valid callable in the current subset",
                    ),
                ));
            }
            Value::Array(array) if is_array_callable_resolved(&self.classes, array) => {}
            Value::Array(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "set_error_handler()",
                        "callback must be a valid callable in the current subset",
                    ),
                ));
            }
            Value::Closure(_) => {}
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "set_error_handler()",
                        format!(
                            "callback argument must be string, array callable, or closure in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        }

        let mask = if let Some(mask) = args.get(1) {
            match mask {
                Value::Int(mask) => Some(*mask),
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "set_error_handler()",
                            format!(
                                "error levels argument must be int in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }
            }
        } else {
            None
        };

        let previous = self.error_handler.clone().unwrap_or(Value::Null);
        self.error_handler = Some(args[0].clone());
        self.error_handler_mask = mask;
        Ok(previous)
    }

    fn call_restore_error_handler(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        expect_arity("restore_error_handler", &args, 0, span)?;
        self.error_handler = None;
        self.error_handler_mask = None;
        Ok(Value::Bool(true))
    }

    fn call_error_reporting(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        if args.len() > 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "error_reporting()",
                    ArityExpectation::Between { min: 0, max: 1 },
                    args.len(),
                ),
            ));
        }

        let previous = self.error_reporting_mask;
        if let Some(mask) = args.first() {
            let Value::Int(mask) = mask else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "error_reporting()",
                        format!(
                            "mask must be int in the current subset, got {}",
                            mask.type_name()
                        ),
                    ),
                ));
            };
            self.error_reporting_mask = *mask;
        }

        Ok(Value::Int(previous))
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
            } => self.is_direct_array_offset_path_set(target, index, caller_scope),
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

    fn is_direct_array_offset_path_set(
        &mut self,
        target: &Expr,
        index: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        let Some((name, indices)) = Self::collect_direct_variable_array_index_path(target, index)
        else {
            return Err(runtime_error(
                target.span(),
                RuntimeError::unsupported_call(
                    "isset()",
                    "only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported",
                ),
            ));
        };

        let mut keys = Vec::with_capacity(indices.len());
        for index in indices {
            keys.push(self.evaluate_array_key(index, caller_scope)?);
        }

        match caller_scope.read_named(name) {
            Some(value) => Ok(Self::array_path_isset(&value, &keys)),
            None => Ok(false),
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

    fn collect_direct_variable_array_index_path<'a>(
        target: &'a Expr,
        index: &'a Expr,
    ) -> Option<(&'a str, Vec<&'a Expr>)> {
        let mut indices = vec![index];
        let mut current = target;

        loop {
            match current {
                Expr::Variable(name, _) => {
                    indices.reverse();
                    return Some((name, indices));
                }
                Expr::Index { target, index, .. } => {
                    indices.push(index);
                    current = target;
                }
                _ => return None,
            }
        }
    }

    fn array_path_isset(value: &Value, keys: &[ArrayKey]) -> bool {
        let mut current = value;

        for key in keys {
            let Value::Array(array) = current else {
                return false;
            };
            let Some(next) = array.get(key.clone()) else {
                return false;
            };
            current = next;
        }

        !matches!(current, Value::Null)
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

        match caller_scope.read_named(name) {
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
                Value::Closure(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(string)",
                        "Closure __toString() and cast error behavior are not implemented",
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
                Value::Closure(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(int)",
                        "Closure object-to-int cast behavior is not implemented",
                    ),
                )),
            },
            CastKind::Bool => Ok(Value::Bool(value.is_truthy())),
            CastKind::Float => match value {
                Value::Null => Ok(Value::Float(0.0)),
                Value::Bool(value) => Ok(Value::Float(if value { 1.0 } else { 0.0 })),
                Value::Int(value) => Ok(Value::Float(value as f64)),
                Value::Float(value) => Ok(Value::Float(value)),
                Value::String(value) => cast_string_to_float(&value, span),
                Value::Array(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(float)",
                        "array-to-float cast behavior is not implemented",
                    ),
                )),
                Value::Object(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(float)",
                        "object-to-float cast behavior is not implemented",
                    ),
                )),
                Value::Closure(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(float)",
                        "Closure object-to-float cast behavior is not implemented",
                    ),
                )),
            },
            CastKind::Array => match value {
                Value::Null => Ok(Value::Array(PhpArray::new())),
                Value::Array(value) => Ok(Value::Array(value)),
                Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                    let mut array = PhpArray::new();
                    array
                        .append(value)
                        .expect("append into a fresh array should not fail");
                    Ok(Value::Array(array))
                }
                Value::Object(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(array)",
                        "object-to-array cast property materialization is not implemented",
                    ),
                )),
                Value::Closure(_) => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "(array)",
                        "Closure object-to-array cast behavior is not implemented",
                    ),
                )),
            },
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

    if let Some(prefix) = leading_numeric_prefix(trimmed) {
        if let Ok(value) = prefix.parse::<i64>() {
            return Ok(Value::Int(value));
        }
        if let Ok(value) = prefix.parse::<f64>() {
            return cast_float_to_int(value, "(int)", span);
        }
    }

    Err(runtime_error(
        span,
        RuntimeError::unsupported_call(
            "(int)",
            "leading-numeric string cast behavior is outside the current bounded prefix subset",
        ),
    ))
}

fn starts_with_numeric_prefix(value: &str) -> bool {
    leading_numeric_prefix(value).is_some()
}

fn leading_numeric_prefix(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }

    let digits_before_decimal = consume_ascii_digits_bytes(bytes, &mut index);
    if matches!(bytes.get(index), Some(b'.')) {
        index += 1;
        let digits_after_decimal = consume_ascii_digits_bytes(bytes, &mut index);
        if digits_before_decimal == 0 && digits_after_decimal == 0 {
            return None;
        }
    } else if digits_before_decimal == 0 {
        return None;
    }

    let end_before_exponent = index;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_marker = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if consume_ascii_digits_bytes(bytes, &mut index) == 0 {
            index = exponent_marker;
        }
    }

    if index == 0 {
        None
    } else {
        let end = if index == end_before_exponent {
            end_before_exponent
        } else {
            index
        };
        Some(&value[..end])
    }
}

fn consume_ascii_digits_bytes(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while matches!(bytes.get(*index), Some(b'0'..=b'9')) {
        *index += 1;
    }
    *index - start
}

fn cast_string_to_float(value: &str, span: Span) -> CompileResult<Value> {
    let trimmed = value.trim_matches(|ch: char| ch.is_ascii_whitespace());
    if trimmed.is_empty() {
        return Ok(Value::Float(0.0));
    }

    if let Ok(value) = trimmed.parse::<f64>() {
        if value.is_finite() {
            return Ok(Value::Float(value));
        }
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "(float)",
                "non-finite float string cast behavior is not implemented",
            ),
        ));
    }

    if starts_with_numeric_prefix(trimmed) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "(float)",
                "leading-numeric string cast behavior is not implemented",
            ),
        ));
    }

    Ok(Value::Float(0.0))
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

fn register_interface_name(
    classes: &PhpClassTable,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    enum_lookup: &HashMap<String, Rc<EnumDecl>>,
    interfaces: &mut Vec<Rc<InterfaceDecl>>,
    interface_lookup: &mut HashMap<String, Rc<InterfaceDecl>>,
    interface: &InterfaceDecl,
) -> CompileResult<()> {
    let key = interface.name.to_ascii_lowercase();
    if classes.lookup_class_id(&interface.name).is_some()
        || interface_lookup.contains_key(&key)
        || trait_lookup.contains_key(&key)
        || enum_lookup.contains_key(&key)
    {
        return Err(runtime_error(
            interface.span,
            RuntimeError::duplicate_class(&interface.name),
        ));
    }

    let interface = Rc::new(interface.clone());
    interfaces.push(interface.clone());
    interface_lookup.insert(key, interface);
    Ok(())
}

fn register_trait_name(
    classes: &PhpClassTable,
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    enum_lookup: &HashMap<String, Rc<EnumDecl>>,
    traits: &mut Vec<Rc<TraitDecl>>,
    trait_lookup: &mut HashMap<String, Rc<TraitDecl>>,
    trait_decl: &TraitDecl,
) -> CompileResult<()> {
    let key = trait_decl.name.to_ascii_lowercase();
    if classes.lookup_class_id(&trait_decl.name).is_some()
        || interface_lookup.contains_key(&key)
        || trait_lookup.contains_key(&key)
        || enum_lookup.contains_key(&key)
    {
        return Err(runtime_error(
            trait_decl.span,
            RuntimeError::duplicate_class(&trait_decl.name),
        ));
    }

    let trait_decl = Rc::new(trait_decl.clone());
    traits.push(trait_decl.clone());
    trait_lookup.insert(key, trait_decl);
    Ok(())
}

fn register_enum_name(
    classes: &PhpClassTable,
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    enums: &mut Vec<Rc<EnumDecl>>,
    enum_lookup: &mut HashMap<String, Rc<EnumDecl>>,
    enum_decl: &EnumDecl,
) -> CompileResult<()> {
    let key = enum_decl.name.to_ascii_lowercase();
    if classes.lookup_class_id(&enum_decl.name).is_some()
        || interface_lookup.contains_key(&key)
        || trait_lookup.contains_key(&key)
        || enum_lookup.contains_key(&key)
    {
        return Err(runtime_error(
            enum_decl.span,
            RuntimeError::duplicate_class(&enum_decl.name),
        ));
    }

    let enum_decl = Rc::new(enum_decl.clone());
    enums.push(enum_decl.clone());
    enum_lookup.insert(key, enum_decl);
    Ok(())
}

fn register_class_member_runtime_tables(
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: &mut HashMap<(ClassId, String), Value>,
    methods: &mut HashMap<(ClassId, String), Rc<FunctionDecl>>,
    abstract_methods: &mut HashSet<(ClassId, String)>,
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
                let key = (class_id, method.function.name.to_ascii_lowercase());
                if method.is_abstract {
                    abstract_methods.insert(key);
                } else {
                    methods.insert(key, Rc::new(method.function.clone()));
                }
            }
            ClassMember::Property(_) => {}
        }
    }
}

fn remove_class_member_runtime_tables(
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: &mut HashMap<(ClassId, String), Value>,
    methods: &mut HashMap<(ClassId, String), Rc<FunctionDecl>>,
    abstract_methods: &mut HashSet<(ClassId, String)>,
    class_id: ClassId,
) {
    class_constants.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    static_properties.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    methods.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    abstract_methods.retain(|(declaring_class_id, _)| *declaring_class_id != class_id);
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
    classes
        .set_interfaces(id, class.interfaces.clone())
        .map_err(|error| runtime_error(class.span, error))?;

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
            | "strtolower"
            | "trim"
            | "strcasecmp"
            | "str_contains"
            | "strpos"
            | "substr_count"
            | "str_replace"
            | "preg_match"
            | "preg_replace"
            | "compact"
            | "error_reporting"
            | "sprintf"
            | "call_user_func"
            | "implode"
            | "dirname"
            | "abs"
            | "version_compare"
            | "microtime"
            | "date_default_timezone_set"
            | "ini_get"
            | "min"
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
            | "mysqli_connect"
            | "mysqli_real_connect"
            | "mysqli_get_server_info"
            | "mysqli_report"
            | "mysqli_init"
            | "file_exists"
            | "is_dir"
            | "is_readable"
            | "register_shutdown_function"
            | "set_error_handler"
            | "restore_error_handler"
            | "header"
            | "header_remove"
            | "headers_sent"
            | "assert"
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

const PHP_E_ERROR: i64 = 1;
const PHP_E_WARNING: i64 = 2;
const PHP_E_PARSE: i64 = 4;
const PHP_E_NOTICE: i64 = 8;
const PHP_E_CORE_ERROR: i64 = 16;
const PHP_E_CORE_WARNING: i64 = 32;
const PHP_E_COMPILE_ERROR: i64 = 64;
const PHP_E_COMPILE_WARNING: i64 = 128;
const PHP_E_USER_ERROR: i64 = 256;
const PHP_E_USER_WARNING: i64 = 512;
const PHP_E_USER_NOTICE: i64 = 1024;
const PHP_E_STRICT: i64 = 2048;
const PHP_E_RECOVERABLE_ERROR: i64 = 4096;
const PHP_E_DEPRECATED: i64 = 8192;
const PHP_E_USER_DEPRECATED: i64 = 16384;
const PHP_E_ALL: i64 = 32767;
const PHP_MYSQLI_REPORT_OFF: i64 = 0;
const PHP_MYSQLI_REPORT_ERROR: i64 = 1;
const PHP_MYSQLI_REPORT_STRICT: i64 = 2;

fn builtin_global_constant_value(name: &str) -> Option<Value> {
    match name {
        "PHP_VERSION" => Some(Value::String("8.3.0".to_string())),
        "PHP_VERSION_ID" => Some(Value::Int(80300)),
        "PHP_INT_MAX" => Some(Value::Int(i64::MAX)),
        "PHP_SAPI" => Some(Value::String("cli".to_string())),
        "E_ERROR" => Some(Value::Int(PHP_E_ERROR)),
        "E_WARNING" => Some(Value::Int(PHP_E_WARNING)),
        "E_PARSE" => Some(Value::Int(PHP_E_PARSE)),
        "E_NOTICE" => Some(Value::Int(PHP_E_NOTICE)),
        "E_CORE_ERROR" => Some(Value::Int(PHP_E_CORE_ERROR)),
        "E_CORE_WARNING" => Some(Value::Int(PHP_E_CORE_WARNING)),
        "E_COMPILE_ERROR" => Some(Value::Int(PHP_E_COMPILE_ERROR)),
        "E_COMPILE_WARNING" => Some(Value::Int(PHP_E_COMPILE_WARNING)),
        "E_USER_ERROR" => Some(Value::Int(PHP_E_USER_ERROR)),
        "E_USER_WARNING" => Some(Value::Int(PHP_E_USER_WARNING)),
        "E_USER_NOTICE" => Some(Value::Int(PHP_E_USER_NOTICE)),
        "E_STRICT" => Some(Value::Int(PHP_E_STRICT)),
        "E_RECOVERABLE_ERROR" => Some(Value::Int(PHP_E_RECOVERABLE_ERROR)),
        "E_DEPRECATED" => Some(Value::Int(PHP_E_DEPRECATED)),
        "E_USER_DEPRECATED" => Some(Value::Int(PHP_E_USER_DEPRECATED)),
        "E_ALL" => Some(Value::Int(PHP_E_ALL)),
        "CASE_LOWER" => Some(Value::Int(0)),
        "CASE_UPPER" => Some(Value::Int(1)),
        "ARRAY_FILTER_USE_BOTH" => Some(Value::Int(1)),
        "ARRAY_FILTER_USE_KEY" => Some(Value::Int(2)),
        "SORT_REGULAR" => Some(Value::Int(0)),
        "SORT_NUMERIC" => Some(Value::Int(1)),
        "SORT_STRING" => Some(Value::Int(2)),
        "MYSQLI_REPORT_OFF" => Some(Value::Int(PHP_MYSQLI_REPORT_OFF)),
        "MYSQLI_REPORT_ERROR" => Some(Value::Int(PHP_MYSQLI_REPORT_ERROR)),
        "MYSQLI_REPORT_STRICT" => Some(Value::Int(PHP_MYSQLI_REPORT_STRICT)),
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

fn is_supported_qualified_runtime_constant_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('\\') {
        return false;
    }

    name.split('\\').all(is_supported_runtime_constant_name)
}

fn normalize_runtime_constant_lookup_name(name: &str) -> Option<&str> {
    let normalized = name.strip_prefix('\\').unwrap_or(name);
    if is_supported_qualified_runtime_constant_name(normalized) {
        Some(normalized)
    } else {
        None
    }
}

fn normalize_runtime_class_constant_lookup_name(name: &str) -> Option<(&str, &str)> {
    let normalized = name.strip_prefix('\\').unwrap_or(name);
    let (class_name, constant) = normalized.split_once("::")?;
    if is_supported_qualified_runtime_constant_name(class_name)
        && is_supported_runtime_constant_name(constant)
    {
        Some((class_name, constant))
    } else {
        None
    }
}

fn unsupported_runtime_constant_value_type(value: &Value) -> Option<&'static str> {
    match value {
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => None,
        Value::Array(array) => array
            .entries()
            .iter()
            .find_map(|entry| unsupported_runtime_constant_value_type(&entry.value)),
        Value::Object(_) => Some("object"),
        Value::Closure(_) => Some("closure"),
    }
}

fn is_compat_loaded_extension_name(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "json" | "hash")
}

fn call_strtolower(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("strtolower", args, 1, span)?;

    if matches!(args[0], Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call("strtolower()", "arrays are not supported"),
        ));
    }

    let value = args[0]
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))?;

    Ok(Value::String(value.to_ascii_lowercase()))
}

fn call_trim(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(1..=2).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "trim()",
                ArityExpectation::Between { min: 1, max: 2 },
                args.len(),
            ),
        ));
    }

    if args.len() == 2 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "trim()",
                "custom character masks are not implemented; pass exactly one argument in the current subset",
            ),
        ));
    }

    if matches!(args[0], Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call("trim()", "arrays are not supported"),
        ));
    }

    let value = args[0]
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))?;

    Ok(Value::String(
        value
            .trim_matches(|ch| matches!(ch, ' ' | '\t' | '\n' | '\r' | '\0' | '\u{000B}'))
            .to_string(),
    ))
}

fn call_strcasecmp(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("strcasecmp", args, 2, span)?;

    let left = string_compare_argument("strcasecmp()", "first", &args[0], span)?;
    let right = string_compare_argument("strcasecmp()", "second", &args[1], span)?;

    Ok(Value::Int(ascii_case_insensitive_compare(&left, &right)))
}

fn call_str_contains(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("str_contains", args, 2, span)?;

    let haystack = string_contains_argument("str_contains()", "haystack", &args[0], span)?;
    let needle = string_contains_argument("str_contains()", "needle", &args[1], span)?;

    Ok(Value::Bool(haystack.contains(&needle)))
}

fn call_strpos(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=3).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "strpos()",
                ArityExpectation::Between { min: 2, max: 3 },
                args.len(),
            ),
        ));
    }

    let haystack = string_contains_argument("strpos()", "haystack", &args[0], span)?;
    let needle = string_contains_argument("strpos()", "needle", &args[1], span)?;
    let offset = match args.get(2) {
        Some(Value::Int(offset)) => *offset,
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "strpos()",
                    format!(
                        "offset argument must be int in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => 0,
    };

    let haystack_len = haystack.len() as i64;
    let start = if offset >= 0 {
        offset
    } else {
        haystack_len + offset
    };

    if start < 0 || start > haystack_len {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "strpos()",
                "offset must be within the haystack bounds in the current subset",
            ),
        ));
    }
    let start = start as usize;

    if needle.is_empty() {
        return Ok(Value::Int(start as i64));
    }

    let needle = needle.as_bytes();
    Ok(haystack.as_bytes()[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| Value::Int((start + index) as i64))
        .unwrap_or(Value::Bool(false)))
}

fn call_substr_count(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=4).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "substr_count()",
                ArityExpectation::Between { min: 2, max: 4 },
                args.len(),
            ),
        ));
    }

    let haystack = string_contains_argument("substr_count()", "haystack", &args[0], span)?;
    let needle = string_contains_argument("substr_count()", "needle", &args[1], span)?;
    if needle.is_empty() {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "substr_count()",
                "empty needles are not supported in the current subset",
            ),
        ));
    }

    let offset = match args.get(2) {
        Some(Value::Int(offset)) => *offset,
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "substr_count()",
                    format!(
                        "offset argument must be int in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => 0,
    };
    let haystack_len = haystack.len() as i64;
    let start = if offset >= 0 {
        offset
    } else {
        haystack_len + offset
    };
    if start < 0 || start > haystack_len {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "substr_count()",
                "offset must be within the haystack bounds in the current subset",
            ),
        ));
    }

    let end = match args.get(3) {
        Some(Value::Int(length)) if *length >= 0 => start + *length,
        Some(Value::Int(length)) => haystack_len + *length,
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "substr_count()",
                    format!(
                        "length argument must be int in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => haystack_len,
    };
    if end < start || end > haystack_len {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "substr_count()",
                "length must keep the searched slice within the haystack bounds in the current subset",
            ),
        ));
    }

    let haystack = &haystack.as_bytes()[start as usize..end as usize];
    let needle = needle.as_bytes();
    if needle.len() > haystack.len() {
        return Ok(Value::Int(0));
    }

    let mut count = 0_i64;
    let mut cursor = 0_usize;
    while cursor <= haystack.len().saturating_sub(needle.len()) {
        if &haystack[cursor..cursor + needle.len()] == needle {
            count += 1;
            cursor += needle.len();
        } else {
            cursor += 1;
        }
    }

    Ok(Value::Int(count))
}

fn call_preg_match(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=5).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "preg_match()",
                ArityExpectation::Between { min: 2, max: 5 },
                args.len(),
            ),
        ));
    }

    if args.len() > 2 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_match()",
                "matches output, flags, and offset arguments are not implemented; pass exactly two arguments in the current subset",
            ),
        ));
    }

    let pattern = string_contains_argument("preg_match()", "pattern", &args[0], span)?;
    let subject = string_contains_argument("preg_match()", "subject", &args[1], span)?;
    let pattern = BoundedPregPattern::parse(&pattern).map_err(|message| {
        runtime_error(
            span,
            RuntimeError::unsupported_call("preg_match()", message),
        )
    })?;

    Ok(Value::Int(if pattern.matches(&subject) { 1 } else { 0 }))
}

fn call_preg_replace(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(3..=5).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "preg_replace()",
                ArityExpectation::Between { min: 3, max: 5 },
                args.len(),
            ),
        ));
    }

    if args.len() > 3 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_replace()",
                "limit and count output arguments are not implemented; pass exactly three arguments in the current subset",
            ),
        ));
    }

    let pattern = string_contains_argument("preg_replace()", "pattern", &args[0], span)?;
    let replacement = string_contains_argument("preg_replace()", "replacement", &args[1], span)?;
    let subject = string_contains_argument("preg_replace()", "subject", &args[2], span)?;

    if pattern != "/[^0-9.].*/" {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_replace()",
                "only the WordPress database-version cleanup pattern /[^0-9.].*/ is implemented in the current subset",
            ),
        ));
    }
    if !replacement.is_empty() {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_replace()",
                "only an empty replacement string is implemented in the current subset",
            ),
        ));
    }

    let end = subject
        .bytes()
        .position(|byte| !(byte.is_ascii_digit() || byte == b'.'))
        .unwrap_or(subject.len());
    Ok(Value::String(subject[..end].to_string()))
}

fn is_compact_variable_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first == b'_' || first.is_ascii_alphabetic()) {
        return false;
    }
    bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

enum BoundedPregPattern {
    Contains(String),
    Prefix(String),
    Suffix(String),
    Exact(String),
    WordPressDbHostIpv4,
    WordPressDbHostIpv6,
}

impl BoundedPregPattern {
    fn parse(pattern: &str) -> Result<Self, String> {
        if pattern == "#^(?P<host>[^:/]*)(?::(?P<port>[\\d]+))?#"
            || pattern == "#^(?P<host>[^:/]*)(?::(?P<port>[d]+))?#"
        {
            return Ok(Self::WordPressDbHostIpv4);
        }
        if pattern == "#^(?:\\[)?(?P<host>[0-9a-fA-F:]+)(?:\\]:(?P<port>[\\d]+))?#"
            || pattern == "#^(?:[)?(?P<host>[0-9a-fA-F:]+)(?:]:(?P<port>[d]+))?#"
        {
            return Ok(Self::WordPressDbHostIpv6);
        }

        let Some(body_and_modifiers) = pattern.strip_prefix('/') else {
            return Err(
                "only slash-delimited patterns are implemented in the current subset".to_string(),
            );
        };

        let (body, modifiers) = split_slash_delimited_pattern(body_and_modifiers).ok_or_else(|| {
            "only slash-delimited patterns with a closing delimiter are implemented in the current subset"
                .to_string()
        })?;
        validate_bounded_preg_modifiers(modifiers)?;

        let (starts_with_anchor, body) = match body.strip_prefix('^') {
            Some(rest) => (true, rest),
            None => (false, body),
        };
        let (ends_with_anchor, body) = match body.strip_suffix('$') {
            Some(rest) => (true, rest),
            None => (false, body),
        };

        let literal = decode_bounded_preg_literal(body)?;
        Ok(match (starts_with_anchor, ends_with_anchor) {
            (true, true) => Self::Exact(literal),
            (true, false) => Self::Prefix(literal),
            (false, true) => Self::Suffix(literal),
            (false, false) => Self::Contains(literal),
        })
    }

    fn matches(&self, subject: &str) -> bool {
        match self {
            Self::Contains(literal) => subject.contains(literal),
            Self::Prefix(literal) => subject.starts_with(literal),
            Self::Suffix(literal) => subject.ends_with(literal),
            Self::Exact(literal) => subject == literal,
            Self::WordPressDbHostIpv4 | Self::WordPressDbHostIpv6 => {
                self.captures(subject).is_some()
            }
        }
    }

    fn captures(&self, subject: &str) -> Option<PhpArray> {
        match self {
            Self::Contains(literal) => subject
                .find(literal)
                .map(|start| preg_match_single_capture(&subject[start..start + literal.len()])),
            Self::Prefix(literal) if subject.starts_with(literal) => {
                Some(preg_match_single_capture(literal))
            }
            Self::Suffix(literal) if subject.ends_with(literal) => {
                Some(preg_match_single_capture(literal))
            }
            Self::Exact(literal) if subject == literal => Some(preg_match_single_capture(subject)),
            Self::WordPressDbHostIpv4 => wordpress_db_host_ipv4_captures(subject),
            Self::WordPressDbHostIpv6 => wordpress_db_host_ipv6_captures(subject),
            _ => None,
        }
    }
}

fn preg_match_single_capture(value: &str) -> PhpArray {
    let mut matches = PhpArray::new();
    matches.insert(0, Value::String(value.to_string()));
    matches
}

fn wordpress_db_host_match_array(full: &str, host: &str, port: Option<&str>) -> PhpArray {
    let mut matches = PhpArray::new();
    matches.insert(0, Value::String(full.to_string()));
    matches.insert("host", Value::String(host.to_string()));
    matches.insert(1, Value::String(host.to_string()));
    if let Some(port) = port {
        matches.insert("port", Value::String(port.to_string()));
        matches.insert(2, Value::String(port.to_string()));
    }
    matches
}

fn wordpress_db_host_ipv4_captures(subject: &str) -> Option<PhpArray> {
    let host_end = subject
        .bytes()
        .position(|byte| matches!(byte, b':' | b'/'))
        .unwrap_or(subject.len());
    let host = &subject[..host_end];
    let mut full_end = host_end;
    let mut port = None;

    if subject.as_bytes().get(host_end) == Some(&b':') {
        let port_start = host_end + 1;
        let port_len = subject.as_bytes()[port_start..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if port_len > 0 {
            full_end = port_start + port_len;
            port = Some(&subject[port_start..full_end]);
        }
    }

    Some(wordpress_db_host_match_array(
        &subject[..full_end],
        host,
        port,
    ))
}

fn wordpress_db_host_ipv6_captures(subject: &str) -> Option<PhpArray> {
    let starts_with_bracket = subject.starts_with('[');
    let host_start = usize::from(starts_with_bracket);
    let host_len = subject.as_bytes()[host_start..]
        .iter()
        .take_while(|byte| byte.is_ascii_hexdigit() || **byte == b':')
        .count();
    if host_len == 0 {
        return None;
    }

    let host_end = host_start + host_len;
    let host = &subject[host_start..host_end];
    let mut full_end = host_end;
    let mut port = None;

    if starts_with_bracket && subject.as_bytes().get(host_end..host_end + 2) == Some(b"]:") {
        let port_start = host_end + 2;
        let port_len = subject.as_bytes()[port_start..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if port_len > 0 {
            full_end = port_start + port_len;
            port = Some(&subject[port_start..full_end]);
        }
    }

    Some(wordpress_db_host_match_array(
        &subject[..full_end],
        host,
        port,
    ))
}

fn split_slash_delimited_pattern(pattern: &str) -> Option<(&str, &str)> {
    let index = pattern.rfind('/')?;
    Some((&pattern[..index], &pattern[index + 1..]))
}

fn validate_bounded_preg_modifiers(modifiers: &str) -> Result<(), String> {
    match modifiers {
        "" | "u" => Ok(()),
        _ => Err("only the u pattern modifier is implemented in the current subset".to_string()),
    }
}

fn decode_bounded_preg_literal(body: &str) -> Result<String, String> {
    let mut literal = String::new();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let Some(escaped) = chars.next() else {
                return Err(
                    "trailing pattern escapes are not implemented in the current subset"
                        .to_string(),
                );
            };
            match escaped {
                '/' | '\\' | '.' | '-' | '^' | '$' => literal.push(escaped),
                _ => {
                    return Err(format!(
                        "escape sequence \\{escaped} is not implemented in the current subset"
                    ));
                }
            }
            continue;
        }

        if matches!(
            ch,
            '*' | '+' | '?' | '[' | ']' | '(' | ')' | '{' | '}' | '|'
        ) {
            return Err(format!(
                "regex metacharacter {ch} is not implemented in the current subset"
            ));
        }

        literal.push(ch);
    }

    Ok(literal)
}

fn string_contains_argument(
    function: &str,
    label: &str,
    value: &Value,
    span: Span,
) -> CompileResult<String> {
    if matches!(value, Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!("{label} argument arrays are not implemented in the current subset"),
            ),
        ));
    }

    value
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))
}

fn string_compare_argument(
    function: &str,
    label: &str,
    value: &Value,
    span: Span,
) -> CompileResult<String> {
    if matches!(value, Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!("{label} argument arrays are not implemented in the current subset"),
            ),
        ));
    }

    value
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))
}

fn ascii_case_insensitive_compare(left: &str, right: &str) -> i64 {
    for (left, right) in left.bytes().zip(right.bytes()) {
        let left = left.to_ascii_lowercase();
        let right = right.to_ascii_lowercase();
        if left < right {
            return -1;
        }
        if left > right {
            return 1;
        }
    }

    match left.len().cmp(&right.len()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn call_str_replace(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(3..=4).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "str_replace()",
                ArityExpectation::Between { min: 3, max: 4 },
                args.len(),
            ),
        ));
    }

    if args.len() == 4 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "str_replace()",
                "count output arguments are not implemented; pass exactly three arguments in the current subset",
            ),
        ));
    }

    let search = string_replace_argument("str_replace()", "search", &args[0], span)?;
    let replace = string_replace_argument("str_replace()", "replace", &args[1], span)?;
    let subject = string_replace_argument("str_replace()", "subject", &args[2], span)?;

    if search.is_empty() {
        return Ok(Value::String(subject));
    }

    Ok(Value::String(subject.replace(&search, &replace)))
}

fn string_replace_argument(
    function: &str,
    label: &str,
    value: &Value,
    span: Span,
) -> CompileResult<String> {
    if matches!(value, Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!("{label} argument arrays are not implemented in the current subset"),
            ),
        ));
    }

    value
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))
}

fn call_sprintf(args: &[Value], span: Span) -> CompileResult<Value> {
    if args.is_empty() {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch("sprintf()", ArityExpectation::AtLeast(1), args.len()),
        ));
    }

    let format = match &args[0] {
        Value::String(value) => value,
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "sprintf()",
                    format!(
                        "format argument must be string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    };

    bounded_sprintf(format, &args[1..], span).map(Value::String)
}

fn bounded_sprintf(format: &str, args: &[Value], span: Span) -> CompileResult<String> {
    let mut output = String::new();
    let bytes = format.as_bytes();
    let mut index = 0;
    let mut next_arg = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            let ch = format[index..].chars().next().expect("index is in bounds");
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        index += 1;
        if index < bytes.len() && bytes[index] == b'%' {
            output.push('%');
            index += 1;
            continue;
        }

        let placeholder_start = index - 1;
        let digits_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }

        let positional = if index > digits_start && index < bytes.len() && bytes[index] == b'$' {
            let position = format[digits_start..index].parse::<usize>().ok();
            index += 1;
            position.and_then(|position| position.checked_sub(1))
        } else {
            index = digits_start;
            None
        };

        if index >= bytes.len() || bytes[index] != b's' {
            let placeholder_end = if index < bytes.len() {
                index
                    + format[index..]
                        .chars()
                        .next()
                        .expect("index is in bounds")
                        .len_utf8()
            } else {
                index
            };
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "sprintf()",
                    format!(
                        "unsupported format placeholder {} in the current subset",
                        &format[placeholder_start..placeholder_end.min(bytes.len())]
                    ),
                ),
            ));
        }
        index += 1;

        let arg_index = if let Some(position) = positional {
            position
        } else {
            let position = next_arg;
            next_arg += 1;
            position
        };

        let Some(value) = args.get(arg_index) else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "sprintf()",
                    format!("missing argument for placeholder {}", arg_index + 1),
                ),
            ));
        };
        output.push_str(
            &value
                .try_echo_string()
                .map_err(|error| runtime_error(span, error))?,
        );
    }

    Ok(output)
}

fn call_implode(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(1..=2).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "implode()",
                ArityExpectation::Between { min: 1, max: 2 },
                args.len(),
            ),
        ));
    }

    let (separator, array) = match args {
        [Value::Array(array)] => ("", array),
        [Value::String(separator), Value::Array(array)] => (separator.as_str(), array),
        [other] => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "implode()",
                    format!(
                        "single argument must be array in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        [separator, Value::Array(_)] => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "implode()",
                    format!(
                        "separator argument must be string in the current subset, got {}",
                        separator.type_name()
                    ),
                ),
            ));
        }
        [_, value] => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "implode()",
                    format!(
                        "array argument must be array in the current subset, got {}",
                        value.type_name()
                    ),
                ),
            ));
        }
        _ => unreachable!("implode arity already checked"),
    };

    let mut parts = Vec::with_capacity(array.len());
    for entry in array.entries() {
        match &entry.value {
            Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                parts.push(entry.value.echo_string());
            }
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "implode()",
                        format!(
                            "array values must be null, bool, int, float, or string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        }
    }

    Ok(Value::String(parts.join(separator)))
}

fn call_header(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(1..=3).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "header()",
                ArityExpectation::Between { min: 1, max: 3 },
                args.len(),
            ),
        ));
    }

    match &args[0] {
        Value::String(_) => {}
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "header()",
                    format!(
                        "header argument must be string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    }

    if let Some(other) = args.get(1).filter(|value| !matches!(value, Value::Bool(_))) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "header()",
                format!(
                    "replace argument must be bool in the current subset, got {}",
                    other.type_name()
                ),
            ),
        ));
    }

    if let Some(other) = args.get(2).filter(|value| !matches!(value, Value::Int(_))) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "header()",
                format!(
                    "response_code argument must be int in the current subset, got {}",
                    other.type_name()
                ),
            ),
        ));
    }

    Ok(Value::Null)
}

fn call_header_remove(args: &[Value], span: Span) -> CompileResult<Value> {
    if args.len() > 1 {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "header_remove()",
                ArityExpectation::Between { min: 0, max: 1 },
                args.len(),
            ),
        ));
    }

    if let Some(other) = args
        .first()
        .filter(|value| !matches!(value, Value::String(_)))
    {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "header_remove()",
                format!(
                    "header name argument must be string in the current subset, got {}",
                    other.type_name()
                ),
            ),
        ));
    }

    Ok(Value::Null)
}

fn call_headers_sent(args: &[Value], span: Span) -> CompileResult<Value> {
    if args.len() > 2 {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "headers_sent()",
                ArityExpectation::Between { min: 0, max: 2 },
                args.len(),
            ),
        ));
    }

    if !args.is_empty() {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "headers_sent()",
                "filename and line output arguments are not implemented; call without arguments in the current subset",
            ),
        ));
    }

    Ok(Value::Bool(false))
}

fn call_abs(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("abs", args, 1, span)?;

    match &args[0] {
        Value::Int(value) => value.checked_abs().map(Value::Int).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "abs()",
                    "integer minimum overflow is not implemented in the current subset",
                ),
            )
        }),
        Value::Float(value) if value.is_finite() => Ok(Value::Float(value.abs())),
        Value::Float(_) => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "abs()",
                "NaN and infinity handling is not implemented in the current subset",
            ),
        )),
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "abs()",
                format!(
                    "argument must be int or finite float in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn call_min(args: &[Value], span: Span) -> CompileResult<Value> {
    if matches!(args, [Value::Array(_)]) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "min()",
                "array argument forms are not implemented in the current subset",
            ),
        ));
    }

    if args.len() < 2 {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch("min()", ArityExpectation::AtLeast(2), args.len()),
        ));
    }

    let mut minimum = match &args[0] {
        Value::Int(value) => *value,
        Value::Array(_) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "min()",
                    "array argument forms are not implemented in the current subset",
                ),
            ));
        }
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "min()",
                    format!(
                        "arguments must be integers in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    };

    for value in &args[1..] {
        match value {
            Value::Int(value) => {
                minimum = minimum.min(*value);
            }
            Value::Array(_) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "min()",
                        "array argument forms are not implemented in the current subset",
                    ),
                ));
            }
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "min()",
                        format!(
                            "arguments must be integers in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        }
    }

    Ok(Value::Int(minimum))
}

fn call_microtime(args: &[Value], span: Span) -> CompileResult<Value> {
    if args.len() > 1 {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "microtime()",
                ArityExpectation::Between { min: 0, max: 1 },
                args.len(),
            ),
        ));
    }

    match args.first() {
        Some(Value::Bool(true)) => {
            let duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "microtime()",
                            format!("system clock is before the Unix epoch: {error}"),
                        ),
                    )
                })?;
            Ok(Value::Float(duration.as_secs_f64()))
        }
        Some(Value::Bool(false)) | None => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "microtime()",
                "string return format is not implemented; pass true for float seconds in the current subset",
            ),
        )),
        Some(other) => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "microtime()",
                format!(
                    "as_float argument must be bool in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn call_date_default_timezone_set(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("date_default_timezone_set", args, 1, span)?;

    match &args[0] {
        Value::String(name) if name == "UTC" => Ok(Value::Bool(true)),
        Value::String(_) => Ok(Value::Bool(false)),
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "date_default_timezone_set()",
                format!(
                    "timezone identifier must be string in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn call_ini_get(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("ini_get", args, 1, span)?;

    let name = match &args[0] {
        Value::String(name) => name,
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ini_get()",
                    format!(
                        "option argument must be string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    };

    Ok(compat_ini_value(name)
        .map(|value| Value::String(value.to_string()))
        .unwrap_or(Value::Bool(false)))
}

fn compat_ini_value(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "arg_separator.output" => Some("&"),
        "default_mimetype" => Some("text/html"),
        "disable_functions" => Some(""),
        "display_errors" => Some(""),
        "error_append_string" => Some(""),
        "error_log" => Some(""),
        "error_prepend_string" => Some(""),
        "html_errors" => Some("0"),
        "mail.add_x_header" => Some("0"),
        "max_execution_time" => Some("30"),
        "mbstring.func_overload" => Some("0"),
        "memory_limit" => Some("128M"),
        "open_basedir" => Some(""),
        "output_handler" => Some(""),
        "post_max_size" => Some("8M"),
        "sendmail_from" => Some(""),
        "sendmail_path" => Some(""),
        "upload_max_filesize" => Some("2M"),
        "upload_tmp_dir" => Some(""),
        "user_agent" => Some(""),
        "zlib.output_compression" => Some("0"),
        _ => None,
    }
}

fn call_version_compare(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=3).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "version_compare()",
                ArityExpectation::Between { min: 2, max: 3 },
                args.len(),
            ),
        ));
    }

    let left = version_compare_string_arg(&args[0], "first", span)?;
    let right = version_compare_string_arg(&args[1], "second", span)?;
    let ordering = compare_bounded_versions(left, right).ok_or_else(|| {
        runtime_error(
            span,
            RuntimeError::unsupported_call(
                "version_compare()",
                "version strings must use dot, hyphen, or underscore separated non-negative integer components in the current subset",
            ),
        )
    })?;

    match args.get(2) {
        None => Ok(Value::Int(match ordering {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })),
        Some(Value::String(operator)) => Ok(Value::Bool(version_compare_operator_result(
            ordering, operator, span,
        )?)),
        Some(other) => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "version_compare()",
                format!(
                    "operator argument must be string in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn version_compare_string_arg<'a>(
    value: &'a Value,
    label: &str,
    span: Span,
) -> CompileResult<&'a str> {
    match value {
        Value::String(value) => Ok(value),
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "version_compare()",
                format!(
                    "{label} version argument must be string in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn compare_bounded_versions(left: &str, right: &str) -> Option<Ordering> {
    let left = parse_bounded_version(left)?;
    let right = parse_bounded_version(right)?;
    let len = left.len().min(right.len());

    for index in 0..len {
        let left = left.get(index).copied().unwrap_or(0);
        let right = right.get(index).copied().unwrap_or(0);
        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return Some(ordering),
        }
    }

    Some(left.len().cmp(&right.len()))
}

fn parse_bounded_version(value: &str) -> Option<Vec<i64>> {
    if value.is_empty() {
        return None;
    }

    value
        .split(['.', '-', '_'])
        .map(|part| {
            if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
                None
            } else {
                part.parse::<i64>().ok()
            }
        })
        .collect()
}

fn version_compare_operator_result(
    ordering: Ordering,
    operator: &str,
    span: Span,
) -> CompileResult<bool> {
    match operator.to_ascii_lowercase().as_str() {
        "<" | "lt" => Ok(ordering == Ordering::Less),
        "<=" | "le" => Ok(matches!(ordering, Ordering::Less | Ordering::Equal)),
        ">" | "gt" => Ok(ordering == Ordering::Greater),
        ">=" | "ge" => Ok(matches!(ordering, Ordering::Greater | Ordering::Equal)),
        "==" | "=" | "eq" => Ok(ordering == Ordering::Equal),
        "!=" | "<>" | "ne" => Ok(ordering != Ordering::Equal),
        _ => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "version_compare()",
                format!("unsupported operator {operator} in the current subset"),
            ),
        )),
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
    let variadic = function.params.iter().any(|param| param.is_variadic);
    if actual < required || (!variadic && actual > function.params.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                callable_name(&function.name),
                arity_expectation(required, function.params.len(), variadic),
                actual,
            ),
        ));
    }

    Ok(())
}

fn ensure_supported_function_signature(function: &FunctionDecl, span: Span) -> CompileResult<()> {
    if function.returns_by_reference {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "reference returns are not implemented",
            ),
        ));
    }

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
        .filter(|param| !param.is_variadic && param.default.is_none())
        .count()
}

fn arity_expectation(required: usize, total: usize, variadic: bool) -> ArityExpectation {
    if variadic {
        return ArityExpectation::AtLeast(required);
    }
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
        Value::Closure(value) => {
            format!(
                "{padding}object(Closure)#{} (0) {{\n{padding}}}\n",
                value.id()
            )
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

        let mut array = PhpArray::new();
        array.append(Value::String("first".to_string())).unwrap();
        symbols.write_static("items", Value::Array(array));

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
