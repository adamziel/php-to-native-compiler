use std::fmt;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    kind: RuntimeErrorKind,
    message: String,
}

impl RuntimeError {
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedVariable { name: name.into() })
    }

    pub fn undefined_function(callable: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedFunction {
            callable: callable.into(),
        })
    }

    pub fn duplicate_function(callable: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::DuplicateFunction {
            callable: callable.into(),
        })
    }

    pub fn arity_mismatch(
        callable: impl Into<String>,
        expected: ArityExpectation,
        actual: usize,
    ) -> Self {
        Self::from_kind(RuntimeErrorKind::ArityMismatch {
            callable: callable.into(),
            expected,
            actual,
        })
    }

    pub fn unsupported_call(callable: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedCall {
            callable: callable.into(),
            reason: reason.into(),
        })
    }

    pub fn invalid_arithmetic(operation: ArithmeticOp, reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidArithmetic {
            operation,
            reason: reason.into(),
        })
    }

    pub fn kind(&self) -> &RuntimeErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    fn from_kind(kind: RuntimeErrorKind) -> Self {
        let message = format_runtime_error(&kind);
        Self { kind, message }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    UndefinedVariable {
        name: String,
    },
    UndefinedFunction {
        callable: String,
    },
    DuplicateFunction {
        callable: String,
    },
    ArityMismatch {
        callable: String,
        expected: ArityExpectation,
        actual: usize,
    },
    UnsupportedCall {
        callable: String,
        reason: String,
    },
    InvalidArithmetic {
        operation: ArithmeticOp,
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArityExpectation {
    Exactly(usize),
    AtLeast(usize),
    Between { min: usize, max: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOp {
    Divide,
}

impl fmt::Display for ArithmeticOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOp::Divide => write!(f, "/"),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

fn format_runtime_error(kind: &RuntimeErrorKind) -> String {
    match kind {
        RuntimeErrorKind::UndefinedVariable { name } => format!("undefined variable '${name}'"),
        RuntimeErrorKind::UndefinedFunction { callable } => {
            format!("undefined function {callable}")
        }
        RuntimeErrorKind::DuplicateFunction { callable } => {
            format!("function {callable} is already defined")
        }
        RuntimeErrorKind::ArityMismatch {
            callable,
            expected,
            actual,
        } => format!(
            "arity mismatch for {callable}: {}, got {actual}",
            format_arity_expectation(*expected)
        ),
        RuntimeErrorKind::UnsupportedCall { callable, reason } => {
            format!("unsupported call {callable}: {reason}")
        }
        RuntimeErrorKind::InvalidArithmetic { operation, reason } => {
            format!("invalid arithmetic for {operation}: {reason}")
        }
    }
}

fn format_arity_expectation(expected: ArityExpectation) -> String {
    match expected {
        ArityExpectation::Exactly(count) => format!("expected {count} argument(s)"),
        ArityExpectation::AtLeast(count) => format!("expected at least {count} argument(s)"),
        ArityExpectation::Between { min, max } => {
            format!("expected {min} to {max} argument(s)")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
        }
    }

    pub fn echo_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(false) => String::new(),
            Value::Bool(true) => "1".to_string(),
            Value::Int(value) => value.to_string(),
            Value::Float(value) => format_php_float(*value),
            Value::String(value) => value.clone(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Int(value) => *value != 0,
            Value::Float(value) => *value != 0.0,
            Value::String(value) => !value.is_empty() && value != "0",
        }
    }

    pub fn php_add(&self, other: &Value) -> RuntimeResult<Value> {
        match (self.to_number(), other.to_number()) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            (a, b) => Ok(Value::Float(a.as_float() + b.as_float())),
        }
    }

    pub fn php_sub(&self, other: &Value) -> RuntimeResult<Value> {
        match (self.to_number(), other.to_number()) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            (a, b) => Ok(Value::Float(a.as_float() - b.as_float())),
        }
    }

    pub fn php_mul(&self, other: &Value) -> RuntimeResult<Value> {
        match (self.to_number(), other.to_number()) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            (a, b) => Ok(Value::Float(a.as_float() * b.as_float())),
        }
    }

    pub fn php_div(&self, other: &Value) -> RuntimeResult<Value> {
        let left = self.to_number();
        let right = other.to_number();
        if right.as_float() == 0.0 {
            return Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::Divide,
                "division by zero",
            ));
        }

        match (left, right) {
            (Number::Int(a), Number::Int(b)) if a % b == 0 => Ok(Value::Int(a / b)),
            (a, b) => Ok(Value::Float(a.as_float() / b.as_float())),
        }
    }

    pub fn php_negate(&self) -> RuntimeResult<Value> {
        match self.to_number() {
            Number::Int(value) => Ok(Value::Int(value.wrapping_neg())),
            Number::Float(value) => Ok(Value::Float(-value)),
        }
    }

    pub fn php_concat(&self, other: &Value) -> Value {
        Value::String(format!("{}{}", self.echo_string(), other.echo_string()))
    }

    pub fn php_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(_), _) | (_, Value::Bool(_)) => self.is_truthy() == other.is_truthy(),
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::String(_), Value::Int(_))
            | (Value::String(_), Value::Float(_))
            | (Value::Int(_), Value::String(_))
            | (Value::Float(_), Value::String(_)) => {
                self.to_number().as_float() == other.to_number().as_float()
            }
            (Value::Null, _) | (_, Value::Null) => self.is_truthy() == other.is_truthy(),
        }
    }

    pub fn php_cmp(&self, other: &Value, op: Comparison) -> bool {
        let ordering = match (self, other) {
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            _ => self
                .to_number()
                .as_float()
                .partial_cmp(&other.to_number().as_float()),
        };

        match (ordering, op) {
            (Some(std::cmp::Ordering::Less), Comparison::Lt | Comparison::Le | Comparison::Ne) => {
                true
            }
            (Some(std::cmp::Ordering::Equal), Comparison::Eq | Comparison::Le | Comparison::Ge) => {
                true
            }
            (
                Some(std::cmp::Ordering::Greater),
                Comparison::Gt | Comparison::Ge | Comparison::Ne,
            ) => true,
            _ => false,
        }
    }

    fn to_number(&self) -> Number {
        match self {
            Value::Null => Number::Int(0),
            Value::Bool(false) => Number::Int(0),
            Value::Bool(true) => Number::Int(1),
            Value::Int(value) => Number::Int(*value),
            Value::Float(value) => Number::Float(*value),
            Value::String(value) => parse_numeric_string(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    fn as_float(&self) -> f64 {
        match self {
            Number::Int(value) => *value as f64,
            Number::Float(value) => *value,
        }
    }
}

fn parse_numeric_string(value: &str) -> Number {
    let trimmed = value.trim_start();
    if trimmed.is_empty() {
        return Number::Int(0);
    }

    if let Ok(parsed) = trimmed.parse::<i64>() {
        return Number::Int(parsed);
    }

    if let Ok(parsed) = trimmed.parse::<f64>() {
        return Number::Float(parsed);
    }

    Number::Int(0)
}

fn format_php_float(value: f64) -> String {
    if value.is_nan() {
        return "NAN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "INF".to_string()
        } else {
            "-INF".to_string()
        };
    }

    let formatted = format!("{}", value);
    if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_conversions_match_php_scalars_for_supported_values() {
        assert_eq!(Value::Null.echo_string(), "");
        assert_eq!(Value::Bool(false).echo_string(), "");
        assert_eq!(Value::Bool(true).echo_string(), "1");
        assert_eq!(Value::Int(42).echo_string(), "42");
        assert_eq!(Value::Float(1.5).echo_string(), "1.5");
        assert_eq!(Value::String("x".to_string()).echo_string(), "x");
    }

    #[test]
    fn scalar_arithmetic_works() {
        assert_eq!(
            Value::Int(2).php_add(&Value::Int(3)).unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            Value::String("2".to_string())
                .php_mul(&Value::Int(3))
                .unwrap(),
            Value::Int(6)
        );
        assert_eq!(
            Value::Int(7).php_div(&Value::Int(2)).unwrap(),
            Value::Float(3.5)
        );
    }

    #[test]
    fn runtime_errors_keep_structured_kind_and_stable_message() {
        let error = RuntimeError::arity_mismatch("strlen()", ArityExpectation::Exactly(1), 2);

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::ArityMismatch {
                callable: "strlen()".to_string(),
                expected: ArityExpectation::Exactly(1),
                actual: 2,
            }
        );
        assert_eq!(
            error.message(),
            "arity mismatch for strlen(): expected 1 argument(s), got 2"
        );
    }

    #[test]
    fn division_by_zero_is_invalid_arithmetic() {
        let error = Value::Int(1).php_div(&Value::Int(0)).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::Divide,
                reason: "division by zero".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for /: division by zero"
        );
    }
}
