use std::cmp::Ordering;
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

    pub fn unsupported_global(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedGlobal {
            reason: reason.into(),
        })
    }

    pub fn invalid_arithmetic(operation: ArithmeticOp, reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidArithmetic {
            operation,
            reason: reason.into(),
        })
    }

    pub fn invalid_array_key(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidArrayKey {
            reason: reason.into(),
        })
    }

    pub fn undefined_array_key(key: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedArrayKey { key: key.into() })
    }

    pub fn invalid_array_access(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidArrayAccess {
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
    UnsupportedGlobal {
        reason: String,
    },
    InvalidArithmetic {
        operation: ArithmeticOp,
        reason: String,
    },
    InvalidArrayKey {
        reason: String,
    },
    UndefinedArrayKey {
        key: String,
    },
    InvalidArrayAccess {
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
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
}

impl fmt::Display for ArithmeticOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOp::Add => write!(f, "+"),
            ArithmeticOp::Subtract => write!(f, "-"),
            ArithmeticOp::Multiply => write!(f, "*"),
            ArithmeticOp::Divide => write!(f, "/"),
            ArithmeticOp::Negate => write!(f, "unary -"),
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
        RuntimeErrorKind::UnsupportedGlobal { reason } => {
            format!("unsupported global declaration: {reason}")
        }
        RuntimeErrorKind::InvalidArithmetic { operation, reason } => {
            format!("invalid arithmetic for {operation}: {reason}")
        }
        RuntimeErrorKind::InvalidArrayKey { reason } => {
            format!("invalid array key: {reason}")
        }
        RuntimeErrorKind::UndefinedArrayKey { key } => {
            format!("undefined array key {key}")
        }
        RuntimeErrorKind::InvalidArrayAccess { reason } => {
            format!("invalid array access: {reason}")
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
pub struct PhpArray {
    entries: Vec<ArrayEntry>,
    next_auto_index: i64,
    auto_index_exhausted: bool,
}

impl PhpArray {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_auto_index: 0,
            auto_index_exhausted: false,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[ArrayEntry] {
        &self.entries
    }

    pub fn get(&self, key: impl Into<ArrayKey>) -> Option<&Value> {
        let key = key.into().normalized();
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }

    pub fn insert(&mut self, key: impl Into<ArrayKey>, value: Value) -> ArrayKey {
        let key = key.into().normalized();
        self.bump_next_auto_index(&key);

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            entry.value = value;
            return key;
        }

        self.entries.push(ArrayEntry {
            key: key.clone(),
            value,
        });
        key
    }

    pub fn append(&mut self, value: Value) -> RuntimeResult<ArrayKey> {
        if self.auto_index_exhausted {
            return Err(RuntimeError::invalid_array_key(
                "cannot append after maximum integer key",
            ));
        }

        let key = ArrayKey::Int(self.next_auto_index);
        self.insert(key.clone(), value);
        Ok(key)
    }

    fn bump_next_auto_index(&mut self, key: &ArrayKey) {
        let ArrayKey::Int(value) = key else {
            return;
        };
        if *value < 0 || self.auto_index_exhausted || *value < self.next_auto_index {
            return;
        }

        match value.checked_add(1) {
            Some(next) => self.next_auto_index = next,
            None => self.auto_index_exhausted = true,
        }
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayEntry {
    pub key: ArrayKey,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayKey {
    Int(i64),
    String(String),
}

impl ArrayKey {
    pub fn int(value: i64) -> Self {
        Self::Int(value)
    }

    pub fn string(value: impl Into<String>) -> Self {
        normalize_string_key(value.into())
    }

    pub fn from_value(value: &Value) -> RuntimeResult<Self> {
        match value {
            Value::Int(value) => Ok(Self::Int(*value)),
            Value::String(value) => Ok(Self::string(value.clone())),
            other => Err(RuntimeError::invalid_array_key(format!(
                "{} keys are not supported; only int and string keys are implemented",
                other.type_name()
            ))),
        }
    }

    pub fn display_key(&self) -> String {
        match self {
            ArrayKey::Int(value) => value.to_string(),
            ArrayKey::String(value) => value.clone(),
        }
    }

    pub fn diagnostic_key(&self) -> String {
        match self {
            ArrayKey::Int(value) => value.to_string(),
            ArrayKey::String(value) => format!("\"{value}\""),
        }
    }

    fn normalized(self) -> Self {
        match self {
            ArrayKey::String(value) => normalize_string_key(value),
            key => key,
        }
    }
}

impl From<i64> for ArrayKey {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<String> for ArrayKey {
    fn from(value: String) -> Self {
        Self::string(value)
    }
}

impl From<&str> for ArrayKey {
    fn from(value: &str) -> Self {
        Self::string(value)
    }
}

fn normalize_string_key(value: String) -> ArrayKey {
    if is_php_integer_array_key(&value) {
        if let Ok(parsed) = value.parse::<i64>() {
            return ArrayKey::Int(parsed);
        }
    }

    ArrayKey::String(value)
}

fn is_php_integer_array_key(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let (negative, digits) = if bytes[0] == b'-' {
        if bytes.len() == 1 {
            return false;
        }
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };

    if !digits.iter().all(u8::is_ascii_digit) {
        return false;
    }

    if digits == b"0" {
        return !negative;
    }

    digits[0] != b'0'
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(PhpArray),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
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
            Value::Array(_) => "Array".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Int(value) => *value != 0,
            Value::Float(value) => *value != 0.0,
            Value::String(value) => !value.is_empty() && value != "0",
            Value::Array(value) => !value.is_empty(),
        }
    }

    pub fn php_add(&self, other: &Value) -> RuntimeResult<Value> {
        match (
            self.to_arithmetic_number(ArithmeticOp::Add)?,
            other.to_arithmetic_number(ArithmeticOp::Add)?,
        ) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_add(b))),
            (a, b) => Ok(Value::Float(a.as_float() + b.as_float())),
        }
    }

    pub fn php_sub(&self, other: &Value) -> RuntimeResult<Value> {
        match (
            self.to_arithmetic_number(ArithmeticOp::Subtract)?,
            other.to_arithmetic_number(ArithmeticOp::Subtract)?,
        ) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_sub(b))),
            (a, b) => Ok(Value::Float(a.as_float() - b.as_float())),
        }
    }

    pub fn php_mul(&self, other: &Value) -> RuntimeResult<Value> {
        match (
            self.to_arithmetic_number(ArithmeticOp::Multiply)?,
            other.to_arithmetic_number(ArithmeticOp::Multiply)?,
        ) {
            (Number::Int(a), Number::Int(b)) => Ok(Value::Int(a.wrapping_mul(b))),
            (a, b) => Ok(Value::Float(a.as_float() * b.as_float())),
        }
    }

    pub fn php_div(&self, other: &Value) -> RuntimeResult<Value> {
        let left = self.to_arithmetic_number(ArithmeticOp::Divide)?;
        let right = other.to_arithmetic_number(ArithmeticOp::Divide)?;
        if right.as_float() == 0.0 {
            return Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::Divide,
                "division by zero",
            ));
        }

        match (left, right) {
            (Number::Int(i64::MIN), Number::Int(-1)) => Ok(Value::Float(i64::MIN as f64 / -1.0)),
            (Number::Int(a), Number::Int(b)) if a % b == 0 => Ok(Value::Int(a / b)),
            (a, b) => Ok(Value::Float(a.as_float() / b.as_float())),
        }
    }

    pub fn php_negate(&self) -> RuntimeResult<Value> {
        match self.to_arithmetic_number(ArithmeticOp::Negate)? {
            Number::Int(value) => Ok(Value::Int(value.wrapping_neg())),
            Number::Float(value) => Ok(Value::Float(-value)),
        }
    }

    pub fn php_concat(&self, other: &Value) -> Value {
        Value::String(format!("{}{}", self.echo_string(), other.echo_string()))
    }

    pub fn php_eq(&self, other: &Value) -> bool {
        self.php_cmp(other, Comparison::Eq)
    }

    pub fn php_cmp(&self, other: &Value, op: Comparison) -> bool {
        match (self.php_ordering(other), op) {
            (Some(Ordering::Less), Comparison::Lt | Comparison::Le | Comparison::Ne) => true,
            (Some(Ordering::Equal), Comparison::Eq | Comparison::Le | Comparison::Ge) => true,
            (Some(Ordering::Greater), Comparison::Gt | Comparison::Ge | Comparison::Ne) => true,
            (None, Comparison::Ne) => true,
            _ => false,
        }
    }

    fn php_ordering(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Bool(_), _) | (_, Value::Bool(_)) => {
                Some(self.is_truthy().cmp(&other.is_truthy()))
            }
            (Value::Array(_), _) | (_, Value::Array(_)) => None,
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Null, Value::String(right)) => compare_binary_strings("", right),
            (Value::String(left), Value::Null) => compare_binary_strings(left, ""),
            (Value::Null, _) => compare_numbers(Number::Int(0), other.numeric_value()?),
            (_, Value::Null) => compare_numbers(self.numeric_value()?, Number::Int(0)),
            (Value::String(left), Value::String(right)) => compare_php_strings(left, right),
            (Value::String(left), Value::Int(right)) => {
                compare_string_and_number(left, Number::Int(*right))
            }
            (Value::String(left), Value::Float(right)) => {
                compare_string_and_number(left, Number::Float(*right))
            }
            (Value::Int(left), Value::String(right)) => {
                compare_number_and_string(Number::Int(*left), right)
            }
            (Value::Float(left), Value::String(right)) => {
                compare_number_and_string(Number::Float(*left), right)
            }
            _ => compare_numbers(self.numeric_value()?, other.numeric_value()?),
        }
    }

    fn numeric_value(&self) -> Option<Number> {
        match self {
            Value::Int(value) => Some(Number::Int(*value)),
            Value::Float(value) => Some(Number::Float(*value)),
            Value::Null => Some(Number::Int(0)),
            Value::Bool(false) => Some(Number::Int(0)),
            Value::Bool(true) => Some(Number::Int(1)),
            Value::String(value) => parse_numeric_string(value),
            Value::Array(_) => None,
        }
    }

    fn to_arithmetic_number(&self, operation: ArithmeticOp) -> RuntimeResult<Number> {
        match self {
            Value::Null => Ok(Number::Int(0)),
            Value::Bool(false) => Ok(Number::Int(0)),
            Value::Bool(true) => Ok(Number::Int(1)),
            Value::Int(value) => Ok(Number::Int(*value)),
            Value::Float(value) => Ok(Number::Float(*value)),
            Value::String(value) => parse_numeric_string(value).ok_or_else(|| {
                RuntimeError::invalid_arithmetic(operation, "string is not numeric")
            }),
            Value::Array(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "arrays are not numeric",
            )),
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

#[derive(Debug, Clone, Copy)]
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

    fn to_php_string(self) -> String {
        match self {
            Number::Int(value) => value.to_string(),
            Number::Float(value) => format_php_float(value),
        }
    }
}

fn compare_numbers(left: Number, right: Number) -> Option<Ordering> {
    match (left, right) {
        (Number::Int(left), Number::Int(right)) => Some(left.cmp(&right)),
        (left, right) => left.as_float().partial_cmp(&right.as_float()),
    }
}

fn compare_php_strings(left: &str, right: &str) -> Option<Ordering> {
    match (parse_numeric_string(left), parse_numeric_string(right)) {
        (Some(left), Some(right)) => compare_numbers(left, right),
        _ => compare_binary_strings(left, right),
    }
}

fn compare_number_and_string(left: Number, right: &str) -> Option<Ordering> {
    if let Some(right) = parse_numeric_string(right) {
        compare_numbers(left, right)
    } else {
        compare_binary_strings(&left.to_php_string(), right)
    }
}

fn compare_string_and_number(left: &str, right: Number) -> Option<Ordering> {
    if let Some(left) = parse_numeric_string(left) {
        compare_numbers(left, right)
    } else {
        compare_binary_strings(left, &right.to_php_string())
    }
}

fn compare_binary_strings(left: &str, right: &str) -> Option<Ordering> {
    Some(left.as_bytes().cmp(right.as_bytes()))
}

fn parse_numeric_string(value: &str) -> Option<Number> {
    let trimmed = value.trim_matches(|ch: char| ch.is_ascii_whitespace());
    if trimmed.is_empty() || !is_well_formed_numeric_string(trimmed) {
        return None;
    }

    let has_float_syntax = trimmed
        .bytes()
        .any(|byte| matches!(byte, b'.' | b'e' | b'E'));
    if !has_float_syntax {
        if let Ok(parsed) = trimmed.parse::<i64>() {
            return Some(Number::Int(parsed));
        }
    }

    trimmed.parse::<f64>().ok().map(Number::Float)
}

fn is_well_formed_numeric_string(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }

    let digits_before_decimal = consume_ascii_digits(bytes, &mut index);
    if matches!(bytes.get(index), Some(b'.')) {
        index += 1;
        let digits_after_decimal = consume_ascii_digits(bytes, &mut index);
        if digits_before_decimal == 0 && digits_after_decimal == 0 {
            return false;
        }
    } else if digits_before_decimal == 0 {
        return false;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) == 0 {
            return false;
        }
    }

    index == bytes.len()
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while matches!(bytes.get(*index), Some(b'0'..=b'9')) {
        *index += 1;
    }
    *index - start
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
    fn scalar_arithmetic_coerces_supported_scalar_operands() {
        assert_eq!(Value::Null.php_add(&Value::Int(5)).unwrap(), Value::Int(5));
        assert_eq!(
            Value::Bool(false).php_mul(&Value::Int(9)).unwrap(),
            Value::Int(0)
        );
        assert_eq!(
            Value::Bool(true).php_div(&Value::Int(2)).unwrap(),
            Value::Float(0.5)
        );
        assert_eq!(
            Value::Int(2).php_add(&Value::Float(3.5)).unwrap(),
            Value::Float(5.5)
        );
        assert_eq!(
            Value::String(" 4 ".to_string())
                .php_add(&Value::Int(1))
                .unwrap(),
            Value::Int(5)
        );
        assert_eq!(
            Value::String("+5".to_string())
                .php_sub(&Value::Int(2))
                .unwrap(),
            Value::Int(3)
        );
        assert_eq!(
            Value::String("-6".to_string())
                .php_mul(&Value::Int(2))
                .unwrap(),
            Value::Int(-12)
        );
        assert_eq!(
            Value::String("3e2".to_string())
                .php_div(&Value::Int(2))
                .unwrap(),
            Value::Float(150.0)
        );
        assert_eq!(
            Value::String(".5".to_string())
                .php_add(&Value::Float(0.25))
                .unwrap(),
            Value::Float(0.75)
        );
    }

    #[test]
    fn non_numeric_strings_fail_arithmetic_with_stable_errors() {
        let error = Value::String("abc".to_string())
            .php_add(&Value::Int(1))
            .unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::Add,
                reason: "string is not numeric".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for +: string is not numeric"
        );

        let error = Value::Int(1)
            .php_mul(&Value::String(String::new()))
            .unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::Multiply,
                reason: "string is not numeric".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for *: string is not numeric"
        );

        let error = Value::String("10 apples".to_string())
            .php_negate()
            .unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::Negate,
                reason: "string is not numeric".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for unary -: string is not numeric"
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

    #[test]
    fn scalar_comparison_matrix_matches_php_8_scalar_subset() {
        let labels = [
            "null", "false", "true", "int0", "int1", "float1_5", "empty", "str0", "str1_5",
            "strabc",
        ];
        let expected = [
            "100101 100101 011100 100101 011100 011100 100101 011100 011100 011100",
            "100101 100101 011100 100101 011100 011100 100101 100101 011100 011100",
            "010011 010011 100101 010011 100101 100101 010011 010011 100101 100101",
            "100101 100101 011100 100101 011100 011100 010011 100101 011100 011100",
            "010011 010011 100101 010011 100101 011100 010011 010011 011100 011100",
            "010011 010011 100101 010011 010011 100101 010011 010011 100101 011100",
            "100101 100101 011100 011100 011100 011100 100101 011100 011100 011100",
            "010011 100101 011100 100101 011100 011100 010011 100101 011100 011100",
            "010011 010011 100101 010011 010011 100101 010011 010011 100101 011100",
            "010011 010011 100101 010011 010011 010011 010011 010011 010011 100101",
        ];

        for (row_index, left_label) in labels.iter().enumerate() {
            let expected_row: Vec<&str> = expected[row_index].split_whitespace().collect();
            for (column_index, right_label) in labels.iter().enumerate() {
                let left = comparison_matrix_value(left_label);
                let right = comparison_matrix_value(right_label);
                let actual = comparison_bits(&left, &right);
                assert_eq!(
                    actual, expected_row[column_index],
                    "comparison matrix mismatch for {left_label} vs {right_label}",
                );
                assert_eq!(
                    left.php_eq(&right),
                    actual.starts_with('1'),
                    "php_eq mismatch for {left_label} vs {right_label}",
                );
            }
        }
    }

    fn comparison_matrix_value(label: &str) -> Value {
        match label {
            "null" => Value::Null,
            "false" => Value::Bool(false),
            "true" => Value::Bool(true),
            "int0" => Value::Int(0),
            "int1" => Value::Int(1),
            "float1_5" => Value::Float(1.5),
            "empty" => Value::String(String::new()),
            "str0" => Value::String("0".to_string()),
            "str1_5" => Value::String("1.5".to_string()),
            "strabc" => Value::String("abc".to_string()),
            _ => panic!("unknown comparison matrix label {label}"),
        }
    }

    fn comparison_bits(left: &Value, right: &Value) -> String {
        [
            Comparison::Eq,
            Comparison::Ne,
            Comparison::Lt,
            Comparison::Le,
            Comparison::Gt,
            Comparison::Ge,
        ]
        .iter()
        .map(|op| if left.php_cmp(right, *op) { '1' } else { '0' })
        .collect()
    }

    #[test]
    fn array_string_keys_normalize_like_php_integer_keys() {
        let cases = [
            ("0", ArrayKey::Int(0)),
            ("8", ArrayKey::Int(8)),
            ("-8", ArrayKey::Int(-8)),
            ("9223372036854775807", ArrayKey::Int(i64::MAX)),
            ("08", ArrayKey::String("08".to_string())),
            ("+8", ArrayKey::String("+8".to_string())),
            ("-0", ArrayKey::String("-0".to_string())),
            ("00", ArrayKey::String("00".to_string())),
            ("8.0", ArrayKey::String("8.0".to_string())),
            (
                "9223372036854775808",
                ArrayKey::String("9223372036854775808".to_string()),
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(ArrayKey::string(input), expected, "array key {input}");
        }
    }

    #[test]
    fn array_preserves_insertion_order_and_updates_normalized_keys() {
        let mut array = PhpArray::new();

        assert_eq!(
            array.insert("2", Value::String("two".to_string())),
            ArrayKey::Int(2)
        );
        assert_eq!(
            array.insert("02", Value::String("zero two".to_string())),
            ArrayKey::String("02".to_string())
        );
        assert_eq!(
            array.insert(1, Value::String("one".to_string())),
            ArrayKey::Int(1)
        );
        assert_eq!(
            array.insert("2", Value::String("two updated".to_string())),
            ArrayKey::Int(2)
        );

        let entries = array.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::Int(2));
        assert_eq!(entries[0].value, Value::String("two updated".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(1));
        assert_eq!(
            array.get("2"),
            Some(&Value::String("two updated".to_string()))
        );
        assert_eq!(
            array.get("02"),
            Some(&Value::String("zero two".to_string()))
        );
    }

    #[test]
    fn array_append_uses_next_non_negative_integer_key() {
        let mut array = PhpArray::new();

        array.insert(-2, Value::String("negative".to_string()));
        assert_eq!(
            array.append(Value::String("first".to_string())).unwrap(),
            ArrayKey::Int(0)
        );
        array.insert(5, Value::String("five".to_string()));
        assert_eq!(
            array.append(Value::String("six".to_string())).unwrap(),
            ArrayKey::Int(6)
        );

        let keys: Vec<ArrayKey> = array
            .entries()
            .iter()
            .map(|entry| entry.key.clone())
            .collect();
        assert_eq!(
            keys,
            vec![
                ArrayKey::Int(-2),
                ArrayKey::Int(0),
                ArrayKey::Int(5),
                ArrayKey::Int(6),
            ]
        );
    }

    #[test]
    fn non_int_string_array_keys_fail_with_stable_error() {
        let error = ArrayKey::from_value(&Value::Bool(true)).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArrayKey {
                reason: "bool keys are not supported; only int and string keys are implemented"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid array key: bool keys are not supported; only int and string keys are implemented"
        );
    }
}
