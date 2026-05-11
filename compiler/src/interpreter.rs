use std::collections::HashMap;

use php_runtime::{Comparison, RuntimeError, RuntimeResult, Value};

use crate::ast::{BinaryOp, Expr, FunctionDecl, Program, Span, Stmt, UnaryOp};
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
                        format!("function '{}' is already defined", function.name),
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
            Stmt::Assign { name, expr, .. } => {
                let value = self.evaluate(expr, scope)?;
                scope.insert(name.clone(), value);
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
            Expr::Variable(name, _) => Ok(scope.get(name).cloned().unwrap_or(Value::Null)),
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

    fn call_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut Scope,
    ) -> CompileResult<Value> {
        let key = name.to_ascii_lowercase();
        if is_builtin(&key) {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(self.evaluate(arg, caller_scope)?);
            }
            return self.call_builtin(&key, values, span);
        }

        let function = self
            .functions
            .get(&key)
            .cloned()
            .ok_or_else(|| runtime_error(span, format!("undefined function '{name}'")))?;

        if args.len() != function.params.len() {
            return Err(runtime_error(
                span,
                format!(
                    "function '{}' expects {} argument(s), got {}",
                    function.name,
                    function.params.len(),
                    args.len()
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
                Ok(Value::Int(args[0].echo_string().as_bytes().len() as i64))
            }
            "isset" => Ok(Value::Bool(
                args.iter().all(|value| !matches!(value, Value::Null)),
            )),
            "var_dump" => {
                for value in &args {
                    self.stdout.push_str(&format_var_dump(value));
                }
                Ok(Value::Null)
            }
            "print_r" => match args.as_slice() {
                [value] => {
                    self.stdout.push_str(&value.echo_string());
                    Ok(Value::Bool(true))
                }
                [value, return_output] if return_output.is_truthy() => {
                    Ok(Value::String(value.echo_string()))
                }
                [value, _] => {
                    self.stdout.push_str(&value.echo_string());
                    Ok(Value::Bool(true))
                }
                _ => Err(runtime_error(
                    span,
                    format!("print_r() expects 1 or 2 argument(s), got {}", args.len()),
                )),
            },
            "count" => Err(runtime_error(
                span,
                "count() is reserved for array support and is not implemented for scalars",
            )),
            _ => unreachable!("is_builtin keeps this match exhaustive for callers"),
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
            BinaryOp::Concat => Ok(left.php_concat(&right)),
            BinaryOp::Eq => Ok(Value::Bool(left.php_cmp(&right, Comparison::Eq))),
            BinaryOp::Ne => Ok(Value::Bool(left.php_cmp(&right, Comparison::Ne))),
            BinaryOp::Lt => Ok(Value::Bool(left.php_cmp(&right, Comparison::Lt))),
            BinaryOp::Le => Ok(Value::Bool(left.php_cmp(&right, Comparison::Le))),
            BinaryOp::Gt => Ok(Value::Bool(left.php_cmp(&right, Comparison::Gt))),
            BinaryOp::Ge => Ok(Value::Bool(left.php_cmp(&right, Comparison::Ge))),
        };

        result.map_err(|error| runtime_error(span, error.message()))
    }

    fn apply_unary(&self, op: UnaryOp, value: Value, span: Span) -> CompileResult<Value> {
        let result: RuntimeResult<Value> = match op {
            UnaryOp::Negate => value.php_negate(),
            UnaryOp::Not => Ok(Value::Bool(!value.is_truthy())),
        };

        result.map_err(|error| runtime_error(span, error.message()))
    }
}

fn runtime_error(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Phase::Runtime, span.line, span.column, message)
}

impl From<RuntimeError> for Diagnostic {
    fn from(value: RuntimeError) -> Self {
        Diagnostic::new(Phase::Runtime, 0, 0, value.message())
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "strlen" | "isset" | "var_dump" | "print_r" | "count")
}

fn expect_arity(name: &str, args: &[Value], expected: usize, span: Span) -> CompileResult<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(runtime_error(
            span,
            format!(
                "{name}() expects {expected} argument(s), got {}",
                args.len()
            ),
        ))
    }
}

fn format_var_dump(value: &Value) -> String {
    match value {
        Value::Null => "NULL\n".to_string(),
        Value::Bool(value) => format!("bool({})\n", if *value { "true" } else { "false" }),
        Value::Int(value) => format!("int({value})\n"),
        Value::Float(value) => format!("float({})\n", value),
        Value::String(value) => format!("string({}) \"{}\"\n", value.len(), value),
    }
}
