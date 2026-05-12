use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use php_runtime::{
    ArityExpectation, ArrayKey, Comparison, ObjectProperty, PhpArray, PhpClassTable,
    PhpMethodMetadata, PhpObject, PhpPropertyMetadata, RuntimeError, RuntimeResult, Value,
    Visibility,
};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, ClassDecl, ClassMember, ClassVisibility, Expr, ForAction,
    FunctionDecl, Program, Span, Stmt, SwitchCase, UnaryOp, UnsetTarget,
};
use crate::error::{CompileResult, Diagnostic, Phase};

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
    classes: PhpClassTable,
    constants: ConstantTable,
    source_file: Option<String>,
    call_depth: usize,
    function_context: Vec<String>,
    stdout: String,
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
}

impl Interpreter {
    fn from_program(program: &Program, source_file: Option<String>) -> CompileResult<Self> {
        let mut functions = HashMap::new();
        let mut classes = PhpClassTable::new();
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
                Stmt::Class(class) => register_class(&mut classes, class)?,
                _ => {}
            }
        }

        Ok(Self {
            functions,
            classes,
            constants: ConstantTable::new(),
            source_file,
            call_depth: 0,
            function_context: Vec::new(),
            stdout: String::new(),
        })
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
        }
    }

    fn execute_statements(
        &mut self,
        statements: &[Stmt],
        scope: &mut SymbolTable,
    ) -> CompileResult<Flow> {
        for stmt in statements {
            match self.execute_statement(stmt, scope)? {
                Flow::Normal => {}
                flow @ (Flow::Break(_) | Flow::Continue(_) | Flow::Return(_)) => return Ok(flow),
            }
        }
        Ok(Flow::Normal)
    }

    fn execute_statement(&mut self, stmt: &Stmt, scope: &mut SymbolTable) -> CompileResult<Flow> {
        match stmt {
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
            Stmt::Expr { expr, .. } => {
                self.evaluate(expr, scope)?;
                Ok(Flow::Normal)
            }
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
                        flow @ Flow::Return(_) => return Ok(flow),
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
                        flow @ Flow::Return(_) => return Ok(flow),
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
                        flow @ Flow::Return(_) => return Ok(flow),
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
                        flow @ Flow::Return(_) => return Ok(flow),
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
            Stmt::Function(_) => Ok(Flow::Normal),
            Stmt::Class(_) => Ok(Flow::Normal),
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.evaluate(expr, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(value))
            }
            Stmt::Break { span } => Ok(Flow::Break(*span)),
            Stmt::Continue { span } => Ok(Flow::Continue(*span)),
            Stmt::Global { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_global(
                    "importing globals into function scope is not implemented",
                ),
            )),
        }
    }

    fn execute_for_action(
        &mut self,
        action: &ForAction,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        match action {
            ForAction::Assign { target, expr } => self.execute_assignment(target, expr, scope),
            ForAction::Expr { expr } => {
                self.evaluate(expr, scope)?;
                Ok(())
            }
        }
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
                flow @ Flow::Return(_) => return Ok(flow),
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
            Expr::Variable(name, span) => scope.read_static(name, *span),
            Expr::MagicLine { span } => Ok(Value::Int(span.line as i64)),
            Expr::MagicFile { .. } => {
                Ok(Value::String(self.source_file.clone().unwrap_or_default()))
            }
            Expr::MagicDir { .. } => Ok(Value::String(self.magic_dir_value())),
            Expr::MagicFunction { .. } => Ok(Value::String(
                self.function_context.last().cloned().unwrap_or_default(),
            )),
            Expr::GlobalConstant { name, span } => self.evaluate_global_constant(name, *span),
            Expr::Array { items, span } => self.evaluate_array(items, *span, scope),
            Expr::Index {
                target,
                index,
                span,
            } => self.evaluate_array_index(target, index, *span, scope),
            Expr::Property {
                target,
                property,
                span,
            } => self.evaluate_property_read(target, property, *span, scope),
            Expr::Call { name, args, span } => self.call_function(name, args, *span, scope),
            Expr::DynamicCall { callee, args, span } => {
                self.call_dynamic_function(callee, args, *span, scope)
            }
            Expr::New {
                class_name,
                args,
                span,
            } => self.instantiate_object(class_name, args, *span),
            Expr::Unary { op, expr, span } => {
                let value = self.evaluate(expr, scope)?;
                self.apply_unary(*op, value, *span)
            }
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                let left = self.evaluate(left, scope)?;
                let right = self.evaluate(right, scope)?;
                self.apply_binary(*op, left, right, *span)
            }
        }
    }

    fn instantiate_object(
        &self,
        class_name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<Value> {
        let class = self
            .classes
            .lookup_class(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;

        if !args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(
                    class.name(),
                    "constructor arguments are not implemented",
                ),
            ));
        }

        if class.method("__construct").is_some() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(
                    class.name(),
                    "constructors are not implemented",
                ),
            ));
        }

        Ok(Value::Object(PhpObject::from_class(class)))
    }

    fn execute_assignment(
        &mut self,
        target: &AssignTarget,
        expr: &Expr,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.evaluate(expr, scope)?;
                scope.write_static(name, value);
                Ok(())
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
                            array.insert(key, value);
                        }
                        None => {
                            array
                                .append(value)
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

                Ok(())
            }
            AssignTarget::Property {
                object,
                property,
                span,
            } => {
                let value = self.evaluate(expr, scope)?;
                let slot = scope.object_slot_for_static_write(object, *span)?;

                match slot {
                    Value::Object(object) => object
                        .write_public_property(property, value)
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

    fn evaluate_property_read(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let target_value = self.evaluate(target, scope)?;

        match target_value {
            Value::Object(object) => object
                .read_public_property(property)
                .cloned()
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
        self.ensure_user_function_call_depth(function, span)?;

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        self.call_user_function_with_checked_values(function, values)
    }

    fn call_user_function_with_values(
        &mut self,
        function: Rc<FunctionDecl>,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        self.ensure_user_function_call_depth(function, span)?;
        self.call_user_function_with_checked_values(function, args)
    }

    fn call_user_function_with_checked_values(
        &mut self,
        function: &FunctionDecl,
        args: Vec<Value>,
    ) -> CompileResult<Value> {
        self.function_context.push(function.name.clone());
        let mut local_scope = SymbolTable::new();
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
                        return Err(error);
                    }
                }
            };
            local_scope.write_static(&param.name, value);
        }

        self.call_depth += 1;
        let flow = self.execute_statements(&function.body, &mut local_scope);
        self.call_depth -= 1;
        self.function_context.pop();

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
        }
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
                let key =
                    ArrayKey::from_value(&args[0]).map_err(|error| runtime_error(span, error))?;
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
                [Value::Array(_), _] => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "array_unique()",
                        "sort flags are not supported in the current subset",
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
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_filter()",
                    format!(
                        "mode flag must be integer 0, 1, or 2 in the current subset, got {}",
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
            _ => Err(runtime_error(
                arg.span(),
                RuntimeError::unsupported_call(
                    "isset()",
                    "only direct variables, direct array offset operands, and direct object property operands are supported",
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
                    "only direct variables, direct array offset operands, and direct object property operands are supported",
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
                    "only direct variables, direct array offset operands, and direct object property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name) {
            Some(Value::Object(object)) => object
                .is_public_property_set(property)
                .map_err(|error| runtime_error(span, error)),
            Some(_) | None => Ok(false),
        }
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
            _ => Err(runtime_error(
                arg.span(),
                RuntimeError::unsupported_call(
                    "empty()",
                    "only direct variables and direct array offset operands are supported",
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
                    "only direct variables and direct array offset operands are supported",
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
        };

        result.map_err(|error| runtime_error(span, error))
    }
}

fn register_class(classes: &mut PhpClassTable, class: &ClassDecl) -> CompileResult<()> {
    let id = classes
        .declare_class(&class.name)
        .map_err(|error| runtime_error(class.span, error))?;
    let metadata = classes
        .get_mut(id)
        .expect("declared class id should resolve to class metadata");

    for member in &class.members {
        match member {
            ClassMember::Property(property) => {
                let visibility = runtime_visibility(property.visibility);
                let metadata_property = if property.is_static {
                    PhpPropertyMetadata::static_property(&property.name, visibility)
                } else {
                    PhpPropertyMetadata::instance(&property.name, visibility)
                };
                metadata
                    .add_property(metadata_property)
                    .map_err(|error| runtime_error(property.span, error))?;
            }
            ClassMember::Method(method) => {
                let visibility = runtime_visibility(method.visibility);
                let metadata_method = if method.is_static {
                    PhpMethodMetadata::static_method(&method.function.name, visibility)
                } else {
                    PhpMethodMetadata::instance(&method.function.name, visibility)
                };
                metadata
                    .add_method(metadata_method)
                    .map_err(|error| runtime_error(method.span, error))?;
            }
        }
    }

    Ok(())
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
            | "count"
            | "constant"
            | "defined"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "array_is_list"
            | "array_keys"
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
            | "get_class"
            | "is_object"
            | "var_dump"
            | "print_r"
    )
}

fn builtin_global_constant_value(name: &str) -> Option<i64> {
    match name {
        "ARRAY_FILTER_USE_BOTH" => Some(1),
        "ARRAY_FILTER_USE_KEY" => Some(2),
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
                    format_var_dump_object_property(value.class_name(), property)
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
            format_print_r_object_property(object.class_name(), property)
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

fn format_print_r_object_property(class_name: &str, property: &ObjectProperty) -> String {
    match property.visibility() {
        Visibility::Public => property.name().to_string(),
        Visibility::Protected => format!("{}:protected", property.name()),
        Visibility::Private => format!("{}:{class_name}:private", property.name()),
    }
}

fn format_var_dump_object_property(class_name: &str, property: &ObjectProperty) -> String {
    match property.visibility() {
        Visibility::Public => format!("\"{}\"", property.name()),
        Visibility::Protected => format!("\"{}\":protected", property.name()),
        Visibility::Private => format!("\"{}\":\"{class_name}\":private", property.name()),
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
