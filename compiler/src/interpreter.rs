use std::collections::HashMap;

use php_runtime::{
    ArityExpectation, ArrayKey, Comparison, PhpArray, RuntimeError, RuntimeResult, Value,
};

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, Expr, FunctionDecl, Program, Span, Stmt, UnaryOp,
};
use crate::error::{CompileResult, Diagnostic, Phase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execution {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub fn run_program(program: &Program) -> CompileResult<Execution> {
    let mut interpreter = Interpreter::from_program(program)?;
    interpreter.run(program)
}

struct Interpreter {
    functions: HashMap<String, FunctionDecl>,
    stdout: String,
}

type Scope = HashMap<String, Value>;

enum Flow {
    Continue,
    Return(Value),
}

impl Interpreter {
    fn from_program(program: &Program) -> CompileResult<Self> {
        let mut functions = HashMap::new();
        for stmt in &program.statements {
            if let Stmt::Function(function) = stmt {
                let key = function.name.to_ascii_lowercase();
                if functions.contains_key(&key) {
                    return Err(runtime_error(
                        function.span,
                        RuntimeError::duplicate_function(callable_name(&function.name)),
                    ));
                }
                functions.insert(key, function.clone());
            }
        }

        Ok(Self {
            functions,
            stdout: String::new(),
        })
    }

    fn run(&mut self, program: &Program) -> CompileResult<Execution> {
        let mut scope = Scope::new();
        match self.execute_statements(&program.statements, &mut scope)? {
            Flow::Continue | Flow::Return(_) => Ok(Execution {
                stdout: self.stdout.clone(),
                stderr: String::new(),
                exit_code: 0,
            }),
        }
    }

    fn execute_statements(
        &mut self,
        statements: &[Stmt],
        scope: &mut Scope,
    ) -> CompileResult<Flow> {
        for stmt in statements {
            match self.execute_statement(stmt, scope)? {
                Flow::Continue => {}
                flow @ Flow::Return(_) => return Ok(flow),
            }
        }
        Ok(Flow::Continue)
    }

    fn execute_statement(&mut self, stmt: &Stmt, scope: &mut Scope) -> CompileResult<Flow> {
        match stmt {
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.evaluate(expr, scope)?;
                    self.stdout.push_str(&value.echo_string());
                }
                Ok(Flow::Continue)
            }
            Stmt::Print { expr, .. } => {
                let value = self.evaluate(expr, scope)?;
                self.stdout.push_str(&value.echo_string());
                Ok(Flow::Continue)
            }
            Stmt::Assign { target, expr, .. } => {
                self.execute_assignment(target, expr, scope)?;
                Ok(Flow::Continue)
            }
            Stmt::Expr { expr, .. } => {
                self.evaluate(expr, scope)?;
                Ok(Flow::Continue)
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
                        Flow::Continue => {}
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                }
                Ok(Flow::Continue)
            }
            Stmt::Function(_) => Ok(Flow::Continue),
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.evaluate(expr, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(value))
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr, scope: &mut Scope) -> CompileResult<Value> {
        match expr {
            Expr::Null(_) => Ok(Value::Null),
            Expr::Bool(value, _) => Ok(Value::Bool(*value)),
            Expr::Int(value, _) => Ok(Value::Int(*value)),
            Expr::Float(value, _) => Ok(Value::Float(*value)),
            Expr::String(value, _) => Ok(Value::String(value.clone())),
            Expr::Variable(name, span) => scope
                .get(name)
                .cloned()
                .ok_or_else(|| runtime_error(*span, RuntimeError::undefined_variable(name))),
            Expr::Array { items, span } => self.evaluate_array(items, *span, scope),
            Expr::Index {
                target,
                index,
                span,
            } => self.evaluate_array_index(target, index, *span, scope),
            Expr::Call { name, args, span } => self.call_function(name, args, *span, scope),
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

    fn execute_assignment(
        &mut self,
        target: &AssignTarget,
        expr: &Expr,
        scope: &mut Scope,
    ) -> CompileResult<()> {
        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.evaluate(expr, scope)?;
                scope.insert(name.clone(), value);
                Ok(())
            }
            AssignTarget::ArrayIndex { name, index, span } => {
                let key = match index {
                    Some(index) => Some(self.evaluate_array_key(index, scope)?),
                    None => None,
                };
                let value = self.evaluate(expr, scope)?;
                let slot = scope
                    .entry(name.clone())
                    .or_insert_with(|| Value::Array(PhpArray::new()));

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
        }
    }

    fn evaluate_array(
        &mut self,
        items: &[ArrayItem],
        span: Span,
        scope: &mut Scope,
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

    fn evaluate_array_index(
        &mut self,
        target: &Expr,
        index: &Expr,
        span: Span,
        scope: &mut Scope,
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

    fn evaluate_array_key(&mut self, expr: &Expr, scope: &mut Scope) -> CompileResult<ArrayKey> {
        let key = self.evaluate(expr, scope)?;
        ArrayKey::from_value(&key).map_err(|error| runtime_error(expr.span(), error))
    }

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut Scope,
    ) -> CompileResult<Value> {
        let key = name.to_ascii_lowercase();
        if key == "isset" {
            return self.call_isset(args, span, caller_scope);
        }

        if is_builtin(&key) {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(self.evaluate(arg, caller_scope)?);
            }
            return self.call_builtin(&key, values, span);
        }

        let function = self.functions.get(&key).cloned().ok_or_else(|| {
            runtime_error(span, RuntimeError::undefined_function(callable_name(name)))
        })?;

        if args.len() != function.params.len() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    callable_name(&function.name),
                    ArityExpectation::Exactly(function.params.len()),
                    args.len(),
                ),
            ));
        }

        let mut local_scope = Scope::new();
        for (param, arg) in function.params.iter().zip(args) {
            let value = self.evaluate(arg, caller_scope)?;
            local_scope.insert(param.clone(), value);
        }

        match self.execute_statements(&function.body, &mut local_scope)? {
            Flow::Continue => Ok(Value::Null),
            Flow::Return(value) => Ok(value),
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        match name {
            "strlen" => {
                expect_arity(name, &args, 1, span)?;
                if matches!(&args[0], Value::Array(_)) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call("strlen()", "arrays are not supported"),
                    ));
                }
                Ok(Value::Int(args[0].echo_string().as_bytes().len() as i64))
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

    fn call_isset(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut Scope,
    ) -> CompileResult<Value> {
        if args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch("isset()", ArityExpectation::AtLeast(1), args.len()),
            ));
        }

        for arg in args {
            match arg {
                Expr::Variable(name, _) => match caller_scope.get(name) {
                    Some(value) if !matches!(value, Value::Null) => {}
                    _ => return Ok(Value::Bool(false)),
                },
                _ => {
                    return Err(runtime_error(
                        arg.span(),
                        RuntimeError::unsupported_call(
                            "isset()",
                            "only direct variable operands are supported",
                        ),
                    ));
                }
            }
        }

        Ok(Value::Bool(true))
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
            BinaryOp::Concat => Ok(left.php_concat(&right)),
            BinaryOp::Eq => Ok(Value::Bool(left.php_cmp(&right, Comparison::Eq))),
            BinaryOp::Ne => Ok(Value::Bool(left.php_cmp(&right, Comparison::Ne))),
            BinaryOp::Lt => Ok(Value::Bool(left.php_cmp(&right, Comparison::Lt))),
            BinaryOp::Le => Ok(Value::Bool(left.php_cmp(&right, Comparison::Le))),
            BinaryOp::Gt => Ok(Value::Bool(left.php_cmp(&right, Comparison::Gt))),
            BinaryOp::Ge => Ok(Value::Bool(left.php_cmp(&right, Comparison::Ge))),
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

fn runtime_error(span: Span, error: RuntimeError) -> Diagnostic {
    Diagnostic::new(Phase::Runtime, span.line, span.column, error.message())
}

impl From<RuntimeError> for Diagnostic {
    fn from(value: RuntimeError) -> Self {
        Diagnostic::new(Phase::Runtime, 0, 0, value.message())
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "strlen" | "count" | "var_dump" | "print_r")
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

fn callable_name(name: &str) -> String {
    format!("{name}()")
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
            value => {
                output.push_str(&value.echo_string());
                output.push('\n');
            }
        }
    }
    output.push_str(&format!("{padding})\n"));
    output
}
