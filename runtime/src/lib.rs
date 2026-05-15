use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeScalarTag {
    Null = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeScalarValue {
    tag: NativeScalarTag,
    bool_value: u8,
    int_value: i64,
    float_value: f64,
}

impl NativeScalarValue {
    pub const fn null() -> Self {
        Self {
            tag: NativeScalarTag::Null,
            bool_value: 0,
            int_value: 0,
            float_value: 0.0,
        }
    }

    pub const fn bool(value: bool) -> Self {
        Self {
            tag: NativeScalarTag::Bool,
            bool_value: value as u8,
            int_value: 0,
            float_value: 0.0,
        }
    }

    pub const fn int(value: i64) -> Self {
        Self {
            tag: NativeScalarTag::Int,
            bool_value: 0,
            int_value: value,
            float_value: 0.0,
        }
    }

    pub const fn float(value: f64) -> Self {
        Self {
            tag: NativeScalarTag::Float,
            bool_value: 0,
            int_value: 0,
            float_value: value,
        }
    }

    pub fn tag(&self) -> NativeScalarTag {
        self.tag
    }

    pub fn to_value(self) -> Value {
        match self.tag {
            NativeScalarTag::Null => Value::Null,
            NativeScalarTag::Bool => Value::Bool(self.bool_value != 0),
            NativeScalarTag::Int => Value::Int(self.int_value),
            NativeScalarTag::Float => Value::Float(self.float_value),
        }
    }

    pub fn echo_string(self) -> String {
        self.to_value().echo_string()
    }
}

#[no_mangle]
pub extern "C" fn phpc_native_null() -> NativeScalarValue {
    NativeScalarValue::null()
}

#[no_mangle]
pub extern "C" fn phpc_native_bool(value: bool) -> NativeScalarValue {
    NativeScalarValue::bool(value)
}

#[no_mangle]
pub extern "C" fn phpc_native_int(value: i64) -> NativeScalarValue {
    NativeScalarValue::int(value)
}

#[no_mangle]
pub extern "C" fn phpc_native_float(value: f64) -> NativeScalarValue {
    NativeScalarValue::float(value)
}

#[no_mangle]
pub extern "C" fn phpc_native_scalar_echo_len(value: NativeScalarValue) -> usize {
    value.echo_string().len()
}

/// # Safety
///
/// `buffer` must either be null or point to at least `capacity` writable bytes.
/// The function returns the total byte length required for the scalar echo
/// string, even when the provided buffer is too small.
#[no_mangle]
pub unsafe extern "C" fn phpc_native_scalar_echo_write(
    value: NativeScalarValue,
    buffer: *mut u8,
    capacity: usize,
) -> usize {
    let output = value.echo_string();
    let bytes = output.as_bytes();
    let required = bytes.len();

    if buffer.is_null() || capacity == 0 {
        return required;
    }

    let written = required.min(capacity);
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, written);
    required
}

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

    pub fn duplicate_constant(name: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::DuplicateConstant { name: name.into() })
    }

    pub fn undefined_constant(name: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedConstant { name: name.into() })
    }

    pub fn duplicate_class(class_name: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::DuplicateClass {
            class_name: class_name.into(),
        })
    }

    pub fn undefined_class(class_name: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedClass {
            class_name: class_name.into(),
        })
    }

    pub fn unsupported_class_inheritance(
        class_name: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedClassInheritance {
            class_name: class_name.into(),
            reason: reason.into(),
        })
    }

    pub fn unsupported_object_instantiation(
        class_name: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedObjectInstantiation {
            class_name: class_name.into(),
            reason: reason.into(),
        })
    }

    pub fn duplicate_class_member(
        class_name: impl Into<String>,
        member_kind: ClassMemberKind,
        member_name: impl Into<String>,
    ) -> Self {
        Self::from_kind(RuntimeErrorKind::DuplicateClassMember {
            class_name: class_name.into(),
            member_kind,
            member_name: member_name.into(),
        })
    }

    pub fn undefined_property(
        class_name: impl Into<String>,
        property_name: impl Into<String>,
    ) -> Self {
        Self::from_kind(RuntimeErrorKind::UndefinedProperty {
            class_name: class_name.into(),
            property_name: property_name.into(),
        })
    }

    pub fn invalid_property_access(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidPropertyAccess {
            reason: reason.into(),
        })
    }

    pub fn unsupported_property_access(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedPropertyAccess {
            reason: reason.into(),
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

    pub fn invalid_loop_control(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidLoopControl {
            reason: reason.into(),
        })
    }

    pub fn invalid_foreach(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidForeach {
            reason: reason.into(),
        })
    }

    pub fn call_depth_exceeded(callable: impl Into<String>, limit: usize) -> Self {
        Self::from_kind(RuntimeErrorKind::CallDepthExceeded {
            callable: callable.into(),
            limit,
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

    pub fn invalid_string_conversion(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::InvalidStringConversion {
            reason: reason.into(),
        })
    }

    pub fn unsupported_comparison(reason: impl Into<String>) -> Self {
        Self::from_kind(RuntimeErrorKind::UnsupportedComparison {
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
    DuplicateConstant {
        name: String,
    },
    UndefinedConstant {
        name: String,
    },
    DuplicateClass {
        class_name: String,
    },
    UndefinedClass {
        class_name: String,
    },
    UnsupportedClassInheritance {
        class_name: String,
        reason: String,
    },
    UnsupportedObjectInstantiation {
        class_name: String,
        reason: String,
    },
    DuplicateClassMember {
        class_name: String,
        member_kind: ClassMemberKind,
        member_name: String,
    },
    UndefinedProperty {
        class_name: String,
        property_name: String,
    },
    InvalidPropertyAccess {
        reason: String,
    },
    UnsupportedPropertyAccess {
        reason: String,
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
    InvalidLoopControl {
        reason: String,
    },
    InvalidForeach {
        reason: String,
    },
    CallDepthExceeded {
        callable: String,
        limit: usize,
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
    InvalidStringConversion {
        reason: String,
    },
    UnsupportedComparison {
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
    Modulo,
    Negate,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    ShiftLeft,
    ShiftRight,
}

impl fmt::Display for ArithmeticOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOp::Add => write!(f, "+"),
            ArithmeticOp::Subtract => write!(f, "-"),
            ArithmeticOp::Multiply => write!(f, "*"),
            ArithmeticOp::Divide => write!(f, "/"),
            ArithmeticOp::Modulo => write!(f, "%"),
            ArithmeticOp::Negate => write!(f, "unary -"),
            ArithmeticOp::BitwiseAnd => write!(f, "&"),
            ArithmeticOp::BitwiseOr => write!(f, "|"),
            ArithmeticOp::BitwiseXor => write!(f, "^"),
            ArithmeticOp::BitwiseNot => write!(f, "~"),
            ArithmeticOp::ShiftLeft => write!(f, "<<"),
            ArithmeticOp::ShiftRight => write!(f, ">>"),
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
        RuntimeErrorKind::DuplicateConstant { name } => {
            format!("constant {name} is already defined")
        }
        RuntimeErrorKind::UndefinedConstant { name } => {
            format!("undefined constant {name}")
        }
        RuntimeErrorKind::DuplicateClass { class_name } => {
            format!("class {class_name} is already defined")
        }
        RuntimeErrorKind::UndefinedClass { class_name } => {
            format!("undefined class {class_name}")
        }
        RuntimeErrorKind::UnsupportedClassInheritance { class_name, reason } => {
            format!("unsupported class inheritance for {class_name}: {reason}")
        }
        RuntimeErrorKind::UnsupportedObjectInstantiation { class_name, reason } => {
            format!("unsupported object instantiation for {class_name}: {reason}")
        }
        RuntimeErrorKind::DuplicateClassMember {
            class_name,
            member_kind,
            member_name,
        } => {
            format!("class {class_name} already defines {member_kind} {member_name}")
        }
        RuntimeErrorKind::UndefinedProperty {
            class_name,
            property_name,
        } => {
            format!("undefined property {class_name}::${property_name}")
        }
        RuntimeErrorKind::InvalidPropertyAccess { reason } => {
            format!("invalid property access: {reason}")
        }
        RuntimeErrorKind::UnsupportedPropertyAccess { reason } => {
            format!("unsupported object property access: {reason}")
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
        RuntimeErrorKind::InvalidLoopControl { reason } => {
            format!("invalid loop control: {reason}")
        }
        RuntimeErrorKind::InvalidForeach { reason } => {
            format!("invalid foreach: {reason}")
        }
        RuntimeErrorKind::CallDepthExceeded { callable, limit } => {
            format!("maximum user function call depth exceeded for {callable}: limit {limit}")
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
        RuntimeErrorKind::InvalidStringConversion { reason } => {
            format!("invalid string conversion: {reason}")
        }
        RuntimeErrorKind::UnsupportedComparison { reason } => {
            format!("unsupported comparison: {reason}")
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

const ARRAY_PAD_MAX_PADDING: u64 = 1_048_576;

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

    pub fn contains_key(&self, key: impl Into<ArrayKey>) -> bool {
        let key = key.into().normalized();
        self.entries.iter().any(|entry| entry.key == key)
    }

    pub fn remove(&mut self, key: impl Into<ArrayKey>) -> bool {
        let key = key.into().normalized();
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            self.entries.remove(index);
            return true;
        }

        false
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

    pub fn values_reindexed(&self) -> Self {
        let mut array = Self::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let key = i64::try_from(index).expect("array length fits in i64");
            array.insert(key, entry.value.clone());
        }
        array
    }

    pub fn keys_reindexed(&self) -> Self {
        let mut array = Self::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let key = i64::try_from(index).expect("array length fits in i64");
            array.insert(key, array_key_to_value(&entry.key));
        }
        array
    }

    pub fn first_key_value(&self) -> Value {
        self.entries
            .first()
            .map(|entry| array_key_to_value(&entry.key))
            .unwrap_or(Value::Null)
    }

    pub fn last_key_value(&self) -> Value {
        self.entries
            .last()
            .map(|entry| array_key_to_value(&entry.key))
            .unwrap_or(Value::Null)
    }

    pub fn is_list(&self) -> bool {
        self.entries
            .iter()
            .enumerate()
            .all(|(index, entry)| entry.key == ArrayKey::Int(index as i64))
    }

    pub fn keys_matching_loose_scalar(&self, search_value: &Value) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            ensure_array_keys_filter_values_supported(search_value, &entry.value)?;
            if search_value.php_cmp_checked(&entry.value, Comparison::Eq)? {
                let key = i64::try_from(array.entries.len()).expect("array length fits in i64");
                array.insert(key, array_key_to_value(&entry.key));
            }
        }

        Ok(array)
    }

    pub fn keys_matching_strict_scalar(&self, search_value: &Value) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            ensure_array_keys_filter_values_supported(search_value, &entry.value)?;
            if search_value.php_identical_checked(&entry.value)? {
                let key = i64::try_from(array.entries.len()).expect("array length fits in i64");
                array.insert(key, array_key_to_value(&entry.key));
            }
        }

        Ok(array)
    }

    pub fn column_values(
        &self,
        column_key: Option<ArrayColumnKey>,
        index_key: Option<ArrayColumnKey>,
    ) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            let value = match &column_key {
                None => Some(entry.value.clone()),
                Some(column_key) => array_column_row_value(&entry.value, column_key),
            };

            if let Some(value) = value {
                match &index_key {
                    Some(index_key) => match array_column_row_value(&entry.value, index_key) {
                        Some(index_value) => {
                            let key = array_column_index_key_from_value(&index_value)?;
                            array.insert(key, value.clone());
                        }
                        None => {
                            array.append(value.clone())?;
                        }
                    },
                    None => {
                        array.append(value.clone())?;
                    }
                }
            }
        }

        Ok(array)
    }

    pub fn reversed_reindexed(&self) -> Self {
        let mut array = Self::new();
        for entry in self.entries.iter().rev() {
            match &entry.key {
                ArrayKey::Int(_) => {
                    array
                        .append(entry.value.clone())
                        .expect("array length fits in i64");
                }
                ArrayKey::String(key) => {
                    array.insert(key.clone(), entry.value.clone());
                }
            }
        }
        array
    }

    pub fn reversed_preserving_keys(&self) -> Self {
        let mut array = Self::new();
        for entry in self.entries.iter().rev() {
            array.insert(entry.key.clone(), entry.value.clone());
        }
        array
    }

    pub fn sliced_from_offset(&self, offset: i64) -> Self {
        self.sliced(offset, None)
    }

    pub fn sliced(&self, offset: i64, length: Option<i64>) -> Self {
        self.sliced_with_key_mode(offset, length, false)
    }

    pub fn sliced_preserving_keys(&self, offset: i64, length: Option<i64>) -> Self {
        self.sliced_with_key_mode(offset, length, true)
    }

    pub fn chunked_reindexed(&self, length: usize) -> Self {
        self.chunked_with_key_mode(length, false)
    }

    pub fn chunked_preserving_keys(&self, length: usize) -> Self {
        self.chunked_with_key_mode(length, true)
    }

    pub fn padded(&self, length: i64, value: Value) -> RuntimeResult<Self> {
        let requested_len = length.unsigned_abs();
        let current_len = u64::try_from(self.entries.len()).expect("array length fits in u64");
        if requested_len <= current_len {
            return Ok(self.clone());
        }

        let padding = requested_len - current_len;
        if padding > ARRAY_PAD_MAX_PADDING {
            return Err(RuntimeError::unsupported_call(
                "array_pad()",
                format!(
                    "padding length must be at most {ARRAY_PAD_MAX_PADDING} in the current subset, got {padding}"
                ),
            ));
        }

        let mut array = Self::new();
        if length < 0 {
            for _ in 0..padding {
                array.append(value.clone())?;
            }
            array.merge_entries_from(self);
        } else {
            array.merge_entries_from(self);
            for _ in 0..padding {
                array.append(value.clone())?;
            }
        }

        Ok(array)
    }

    fn chunked_with_key_mode(&self, length: usize, preserve_keys: bool) -> Self {
        assert!(length > 0, "array_chunk length must be positive");

        let mut chunks = Self::new();
        for entries in self.entries.chunks(length) {
            let mut chunk = Self::new();
            for entry in entries {
                if preserve_keys {
                    chunk.insert(entry.key.clone(), entry.value.clone());
                } else {
                    chunk
                        .append(entry.value.clone())
                        .expect("array length fits in i64");
                }
            }
            chunks
                .append(Value::Array(chunk))
                .expect("array length fits in i64");
        }
        chunks
    }

    fn sliced_with_key_mode(&self, offset: i64, length: Option<i64>, preserve_keys: bool) -> Self {
        let len = i64::try_from(self.entries.len()).expect("array length fits in i64");
        let start = if offset >= 0 {
            offset.min(len)
        } else {
            len.saturating_add(offset).max(0)
        };
        let end = match length {
            Some(length) if length >= 0 => start.saturating_add(length).min(len),
            Some(length) => len.saturating_add(length).max(0).min(len),
            None => len,
        }
        .max(start);

        let start = usize::try_from(start).expect("non-negative slice start fits in usize");
        let end = usize::try_from(end).expect("non-negative slice end fits in usize");

        let mut array = Self::new();
        for entry in self.entries[start..end].iter() {
            if preserve_keys {
                array.insert(entry.key.clone(), entry.value.clone());
            } else {
                match &entry.key {
                    ArrayKey::Int(_) => {
                        array
                            .append(entry.value.clone())
                            .expect("array length fits in i64");
                    }
                    ArrayKey::String(key) => {
                        array.insert(key.clone(), entry.value.clone());
                    }
                }
            }
        }
        array
    }

    pub fn merged_with(&self, right: &Self) -> Self {
        Self::merged_from([self, right])
    }

    pub fn merged_from<'a>(arrays: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut array = Self::new();
        for source in arrays {
            array.merge_entries_from(source);
        }
        array
    }

    pub fn replaced_with(&self, replacement: &Self) -> Self {
        self.replaced_with_all([replacement])
    }

    pub fn replaced_with_all<'a>(&self, replacements: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut array = self.clone();
        for replacement in replacements {
            for entry in &replacement.entries {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array
    }

    pub fn flipped(&self) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            let key = array_flip_key_from_value(&entry.value)?;
            array.insert(key, array_key_to_value(&entry.key));
        }
        Ok(array)
    }

    pub fn keys_with_ascii_case(&self, case: ArrayKeyCase) -> Self {
        let mut array = Self::new();
        for entry in &self.entries {
            let key = match &entry.key {
                ArrayKey::Int(value) => ArrayKey::Int(*value),
                ArrayKey::String(value) => match case {
                    ArrayKeyCase::Lower => ArrayKey::String(value.to_ascii_lowercase()),
                    ArrayKeyCase::Upper => ArrayKey::String(value.to_ascii_uppercase()),
                },
            };
            array.insert(key, entry.value.clone());
        }
        array
    }

    pub fn filled_keys(&self, value: Value) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            let key = array_fill_key_from_value(&entry.value)?;
            array.insert(key, value.clone());
        }
        Ok(array)
    }

    pub fn combined_with(&self, values: &Self) -> RuntimeResult<Self> {
        if self.entries.len() != values.entries.len() {
            return Err(RuntimeError::unsupported_call(
                "array_combine()",
                format!(
                    "keys and values must have the same number of elements in the current subset, got {} and {}",
                    self.entries.len(),
                    values.entries.len()
                ),
            ));
        }

        let mut array = Self::new();
        for (key_entry, value_entry) in self.entries.iter().zip(values.entries.iter()) {
            let key = array_combine_key_from_value(&key_entry.value)?;
            array.insert(key, value_entry.value.clone());
        }
        Ok(array)
    }

    pub fn intersect_keys_with(&self, right: &Self) -> Self {
        self.intersect_keys_with_all([right])
    }

    pub fn intersect_keys_with_all<'a>(&self, others: impl IntoIterator<Item = &'a Self>) -> Self {
        let others = others.into_iter().collect::<Vec<_>>();
        let mut array = Self::new();
        for entry in &self.entries {
            if others
                .iter()
                .all(|other| other.contains_key(entry.key.clone()))
            {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array
    }

    pub fn diff_keys_with(&self, right: &Self) -> Self {
        self.diff_keys_with_all([right])
    }

    pub fn diff_keys_with_all<'a>(&self, others: impl IntoIterator<Item = &'a Self>) -> Self {
        let others = others.into_iter().collect::<Vec<_>>();
        let mut array = Self::new();
        for entry in &self.entries {
            if others
                .iter()
                .all(|other| !other.contains_key(entry.key.clone()))
            {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array
    }

    pub fn diff_values_with(&self, right: &Self) -> RuntimeResult<Self> {
        self.diff_values_with_all([right])
    }

    pub fn diff_values_with_all<'a>(
        &self,
        others: impl IntoIterator<Item = &'a Self>,
    ) -> RuntimeResult<Self> {
        let left_values = self
            .entries
            .iter()
            .map(|entry| array_scalar_string_comparison_value("array_diff()", &entry.value))
            .collect::<RuntimeResult<Vec<_>>>()?;
        let other_values = others
            .into_iter()
            .map(|other| {
                other
                    .entries
                    .iter()
                    .map(|entry| array_scalar_string_comparison_value("array_diff()", &entry.value))
                    .collect::<RuntimeResult<Vec<_>>>()
            })
            .collect::<RuntimeResult<Vec<_>>>()?;

        let mut array = Self::new();
        for (entry, left_value) in self.entries.iter().zip(left_values.iter()) {
            if other_values
                .iter()
                .all(|values| !values.iter().any(|right_value| right_value == left_value))
            {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array.inherit_append_cursor_from(self);

        Ok(array)
    }

    pub fn intersect_values_with(&self, right: &Self) -> RuntimeResult<Self> {
        self.intersect_values_with_all([right])
    }

    pub fn intersect_values_with_all<'a>(
        &self,
        others: impl IntoIterator<Item = &'a Self>,
    ) -> RuntimeResult<Self> {
        let left_values = self
            .entries
            .iter()
            .map(|entry| array_scalar_string_comparison_value("array_intersect()", &entry.value))
            .collect::<RuntimeResult<Vec<_>>>()?;
        let other_values = others
            .into_iter()
            .map(|other| {
                other
                    .entries
                    .iter()
                    .map(|entry| {
                        array_scalar_string_comparison_value("array_intersect()", &entry.value)
                    })
                    .collect::<RuntimeResult<Vec<_>>>()
            })
            .collect::<RuntimeResult<Vec<_>>>()?;

        let mut array = Self::new();
        for (entry, left_value) in self.entries.iter().zip(left_values.iter()) {
            if other_values
                .iter()
                .all(|values| values.iter().any(|right_value| right_value == left_value))
            {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array.inherit_append_cursor_from(self);

        Ok(array)
    }

    pub fn unique_values_by_string(&self) -> RuntimeResult<Self> {
        let mut seen = Vec::new();
        let mut array = Self::new();
        for entry in &self.entries {
            let comparison_value =
                array_scalar_string_comparison_value("array_unique()", &entry.value)?;
            if seen
                .iter()
                .any(|seen_value| seen_value == &comparison_value)
            {
                continue;
            }

            seen.push(comparison_value);
            array.insert(entry.key.clone(), entry.value.clone());
        }

        Ok(array)
    }

    pub fn unique_values_regular(&self) -> RuntimeResult<Self> {
        let mut seen = Vec::new();
        let mut array = Self::new();
        for entry in &self.entries {
            array_scalar_value_supported("array_unique()", &entry.value)?;

            let mut duplicate = false;
            for seen_value in &seen {
                if entry.value.php_cmp_checked(seen_value, Comparison::Eq)? {
                    duplicate = true;
                    break;
                }
            }
            if duplicate {
                continue;
            }

            seen.push(entry.value.clone());
            array.insert(entry.key.clone(), entry.value.clone());
        }

        Ok(array)
    }

    pub fn unique_values_by_numeric(&self) -> RuntimeResult<Self> {
        let mut seen = Vec::new();
        let mut array = Self::new();
        for entry in &self.entries {
            let comparison_value = array_numeric_number_from_value("array_unique()", &entry.value)?;
            if seen.iter().any(|seen_value| {
                compare_numbers(*seen_value, comparison_value) == Some(Ordering::Equal)
            }) {
                continue;
            }

            seen.push(comparison_value);
            array.insert(entry.key.clone(), entry.value.clone());
        }

        Ok(array)
    }

    pub fn count_values(&self) -> RuntimeResult<Self> {
        let mut array = Self::new();
        for entry in &self.entries {
            let key = array_count_values_key_from_value(&entry.value)?;
            if let Some(count_entry) = array.entries.iter_mut().find(|entry| entry.key == key) {
                let Value::Int(count) = &mut count_entry.value else {
                    unreachable!("array_count_values stores integer counts")
                };
                *count = count.checked_add(1).expect("array value count fits in i64");
            } else {
                array.insert(key, Value::Int(1));
            }
        }
        Ok(array)
    }

    pub fn sum_values(&self) -> RuntimeResult<Value> {
        let mut sum = Number::Int(0);
        for entry in &self.entries {
            let value = array_sum_number_from_value(&entry.value)?;
            sum = add_array_sum_numbers(sum, value);
        }

        Ok(value_from_number(sum))
    }

    pub fn product_values(&self) -> RuntimeResult<Value> {
        let mut product = Number::Int(1);
        for entry in &self.entries {
            let value = array_product_number_from_value(&entry.value)?;
            product = multiply_array_product_numbers(product, value);
        }

        Ok(value_from_number(product))
    }

    pub fn filtered_without_callback(&self) -> Self {
        let mut array = Self::new();
        for entry in &self.entries {
            if entry.value.is_truthy() {
                array.insert(entry.key.clone(), entry.value.clone());
            }
        }
        array
    }

    fn merge_entries_from(&mut self, source: &Self) {
        for entry in &source.entries {
            match &entry.key {
                ArrayKey::Int(_) => {
                    self.append(entry.value.clone())
                        .expect("array length fits in i64");
                }
                ArrayKey::String(key) => {
                    self.insert(key.clone(), entry.value.clone());
                }
            }
        }
    }

    pub fn contains_value_loose_scalar(&self, needle: &Value) -> RuntimeResult<bool> {
        for entry in &self.entries {
            ensure_array_search_values_supported("in_array()", needle, &entry.value)?;
            if needle.php_cmp_checked(&entry.value, Comparison::Eq)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn contains_value_strict_scalar(&self, needle: &Value) -> RuntimeResult<bool> {
        for entry in &self.entries {
            ensure_array_search_values_supported("in_array()", needle, &entry.value)?;
            if needle.php_identical_checked(&entry.value)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn search_value_loose_scalar(&self, needle: &Value) -> RuntimeResult<Option<ArrayKey>> {
        for entry in &self.entries {
            ensure_array_search_values_supported("array_search()", needle, &entry.value)?;
            if needle.php_cmp_checked(&entry.value, Comparison::Eq)? {
                return Ok(Some(entry.key.clone()));
            }
        }

        Ok(None)
    }

    pub fn search_value_strict_scalar(&self, needle: &Value) -> RuntimeResult<Option<ArrayKey>> {
        for entry in &self.entries {
            ensure_array_search_values_supported("array_search()", needle, &entry.value)?;
            if needle.php_identical_checked(&entry.value)? {
                return Ok(Some(entry.key.clone()));
            }
        }

        Ok(None)
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

    fn inherit_append_cursor_from(&mut self, source: &Self) {
        self.next_auto_index = source.next_auto_index;
        self.auto_index_exhausted = source.auto_index_exhausted;
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        Self::new()
    }
}

fn ensure_array_search_values_supported(
    callable: &str,
    needle: &Value,
    value: &Value,
) -> RuntimeResult<()> {
    match (needle, value) {
        (Value::Array(_), _) | (_, Value::Array(_)) => Err(RuntimeError::unsupported_call(
            callable,
            "array needles and array values are not implemented",
        )),
        (Value::Object(_), _) | (_, Value::Object(_)) => Err(RuntimeError::unsupported_call(
            callable,
            "object needles and object values are not implemented",
        )),
        _ => Ok(()),
    }
}

fn array_scalar_string_comparison_value(callable: &str, value: &Value) -> RuntimeResult<String> {
    array_scalar_value_supported(callable, value)?;
    Ok(value.echo_string())
}

fn array_scalar_value_supported(callable: &str, value: &Value) -> RuntimeResult<()> {
    match value {
        Value::Array(_) | Value::Object(_) | Value::Closure(_) => {
            Err(RuntimeError::unsupported_call(
                callable,
                format!(
                    "values must be scalar in the current subset, got {}",
                    value.type_name()
                ),
            ))
        }
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => Ok(()),
    }
}

fn ensure_array_keys_filter_values_supported(
    search_value: &Value,
    value: &Value,
) -> RuntimeResult<()> {
    match (search_value, value) {
        (Value::Array(_), _) | (_, Value::Array(_)) => Err(RuntimeError::unsupported_call(
            "array_keys()",
            "array search values and array values are not implemented",
        )),
        (Value::Object(_), _) | (_, Value::Object(_)) => Err(RuntimeError::unsupported_call(
            "array_keys()",
            "object search values and object values are not implemented",
        )),
        _ => Ok(()),
    }
}

fn array_key_to_value(key: &ArrayKey) -> Value {
    match key {
        ArrayKey::Int(value) => Value::Int(*value),
        ArrayKey::String(value) => Value::String(value.clone()),
    }
}

fn array_flip_key_from_value(value: &Value) -> RuntimeResult<ArrayKey> {
    match value {
        Value::Int(value) => Ok(ArrayKey::Int(*value)),
        Value::String(value) => Ok(ArrayKey::string(value.clone())),
        other => Err(RuntimeError::unsupported_call(
            "array_flip()",
            format!(
                "values must be int or string in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
}

fn array_fill_key_from_value(value: &Value) -> RuntimeResult<ArrayKey> {
    match value {
        Value::Null => Ok(ArrayKey::String(String::new())),
        Value::Bool(false) => Ok(ArrayKey::String(String::new())),
        Value::Bool(true) => Ok(ArrayKey::string("1")),
        Value::Int(value) => Ok(ArrayKey::Int(*value)),
        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= i64::MIN as f64
                && *value < i64::MAX as f64 =>
        {
            Ok(ArrayKey::Int(*value as i64))
        }
        Value::Float(_) => Err(RuntimeError::unsupported_call(
            "array_fill_keys()",
            "lossy or non-finite float key values are not supported; only null, bool, int, string, and integral finite float key values are implemented",
        )),
        Value::String(value) => Ok(ArrayKey::string(value.clone())),
        other => Err(RuntimeError::unsupported_call(
            "array_fill_keys()",
            format!(
                "key values must be null, bool, int, string, or integral finite float in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
}

fn array_combine_key_from_value(value: &Value) -> RuntimeResult<ArrayKey> {
    match value {
        Value::Null => Ok(ArrayKey::String(String::new())),
        Value::Bool(false) => Ok(ArrayKey::String(String::new())),
        Value::Bool(true) => Ok(ArrayKey::string("1")),
        Value::Int(value) => Ok(ArrayKey::Int(*value)),
        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= i64::MIN as f64
                && *value < i64::MAX as f64 =>
        {
            Ok(ArrayKey::Int(*value as i64))
        }
        Value::Float(_) => Err(RuntimeError::unsupported_call(
            "array_combine()",
            "lossy or non-finite float key values are not supported; only null, bool, int, string, and integral finite float key values are implemented",
        )),
        Value::String(value) => Ok(ArrayKey::string(value.clone())),
        other => Err(RuntimeError::unsupported_call(
            "array_combine()",
            format!(
                "key values must be null, bool, int, string, or integral finite float in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
}

fn array_count_values_key_from_value(value: &Value) -> RuntimeResult<ArrayKey> {
    match value {
        Value::Int(value) => Ok(ArrayKey::Int(*value)),
        Value::String(value) => Ok(ArrayKey::string(value.clone())),
        other => Err(RuntimeError::unsupported_call(
            "array_count_values()",
            format!(
                "values must be int or string in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
}

fn array_sum_number_from_value(value: &Value) -> RuntimeResult<Number> {
    array_numeric_number_from_value("array_sum()", value)
}

fn array_product_number_from_value(value: &Value) -> RuntimeResult<Number> {
    array_numeric_number_from_value("array_product()", value)
}

fn array_numeric_number_from_value(callable: &'static str, value: &Value) -> RuntimeResult<Number> {
    match value {
        Value::Null => Ok(Number::Int(0)),
        Value::Bool(false) => Ok(Number::Int(0)),
        Value::Bool(true) => Ok(Number::Int(1)),
        Value::Int(value) => Ok(Number::Int(*value)),
        Value::Float(value) => Ok(Number::Float(*value)),
        Value::String(value) => parse_numeric_string(value).ok_or_else(|| {
            RuntimeError::unsupported_call(
                callable,
                "values must be numeric in the current subset, got non-numeric string",
            )
        }),
        other => Err(RuntimeError::unsupported_call(
            callable,
            format!(
                "values must be numeric scalar in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
}

fn add_array_sum_numbers(left: Number, right: Number) -> Number {
    match (left, right) {
        (Number::Int(left), Number::Int(right)) => left
            .checked_add(right)
            .map(Number::Int)
            .unwrap_or_else(|| Number::Float(left as f64 + right as f64)),
        (left, right) => Number::Float(left.as_float() + right.as_float()),
    }
}

fn multiply_array_product_numbers(left: Number, right: Number) -> Number {
    match (left, right) {
        (Number::Int(left), Number::Int(right)) => left
            .checked_mul(right)
            .map(Number::Int)
            .unwrap_or_else(|| Number::Float(left as f64 * right as f64)),
        (left, right) => Number::Float(left.as_float() * right.as_float()),
    }
}

fn value_from_number(number: Number) -> Value {
    match number {
        Number::Int(value) => Value::Int(value),
        Number::Float(value) => Value::Float(value),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayKeyCase {
    Lower,
    Upper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayColumnKey {
    Int(i64),
    String(String),
}

impl ArrayKeyCase {
    pub fn from_flag(flag: i64) -> Self {
        if flag == 0 {
            Self::Lower
        } else {
            Self::Upper
        }
    }
}

impl ArrayColumnKey {
    pub fn from_value(value: &Value) -> RuntimeResult<Option<Self>> {
        Self::from_value_named(value, "column key")
    }

    pub fn index_from_value(value: &Value) -> RuntimeResult<Option<Self>> {
        Self::from_value_named(value, "index key")
    }

    fn from_value_named(value: &Value, name: &str) -> RuntimeResult<Option<Self>> {
        match value {
            Value::Null => Ok(None),
            Value::Int(value) => Ok(Some(Self::Int(*value))),
            Value::String(value) => Ok(Some(Self::String(value.clone()))),
            other => Err(RuntimeError::unsupported_call(
                "array_column()",
                format!(
                    "{name} must be int, string, or null in the current subset, got {}",
                    other.type_name()
                ),
            )),
        }
    }

    fn array_key(&self) -> ArrayKey {
        match self {
            Self::Int(value) => ArrayKey::Int(*value),
            Self::String(value) => ArrayKey::string(value.clone()),
        }
    }

    fn property_name(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            Self::Int(_) => None,
        }
    }
}

fn array_column_row_value(row: &Value, key: &ArrayColumnKey) -> Option<Value> {
    match row {
        Value::Array(row) => row.get(key.array_key()).cloned(),
        Value::Object(row) => key
            .property_name()
            .and_then(|name| row.read_current_public_property(name)),
        _ => None,
    }
}

fn array_column_index_key_from_value(value: &Value) -> RuntimeResult<ArrayKey> {
    match value {
        Value::Null => Ok(ArrayKey::String(String::new())),
        Value::Bool(false) => Ok(ArrayKey::Int(0)),
        Value::Bool(true) => Ok(ArrayKey::Int(1)),
        Value::Int(value) => Ok(ArrayKey::Int(*value)),
        Value::Float(value)
            if value.is_finite()
                && value.fract() == 0.0
                && *value >= i64::MIN as f64
                && *value < i64::MAX as f64 =>
        {
            Ok(ArrayKey::Int(*value as i64))
        }
        Value::Float(_) => Err(RuntimeError::unsupported_call(
            "array_column()",
            "lossy or non-finite float index values are not supported; only null, bool, int, string, and integral finite float index values are implemented",
        )),
        Value::String(value) => Ok(ArrayKey::string(value.clone())),
        other => Err(RuntimeError::unsupported_call(
            "array_column()",
            format!(
                "index values must be null, bool, int, string, or integral finite float in the current subset, got {}",
                other.type_name()
            ),
        )),
    }
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

    pub fn from_array_key_exists_value(value: &Value) -> RuntimeResult<Self> {
        match value {
            Value::Null => Ok(Self::String(String::new())),
            Value::Bool(false) => Ok(Self::Int(0)),
            Value::Bool(true) => Ok(Self::Int(1)),
            Value::Int(value) => Ok(Self::Int(*value)),
            Value::Float(value)
                if value.is_finite()
                    && value.fract() == 0.0
                    && *value >= i64::MIN as f64
                    && *value < i64::MAX as f64 =>
            {
                Ok(Self::Int(*value as i64))
            }
            Value::Float(_) => Err(RuntimeError::invalid_array_key(
                "lossy or non-finite float keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented",
            )),
            Value::String(value) => Ok(Self::string(value.clone())),
            other => Err(RuntimeError::invalid_array_key(format!(
                "{} keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassId(usize);

impl ClassId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PhpClassTable {
    classes: Vec<PhpClassMetadata>,
    lookup: HashMap<String, ClassId>,
}

impl PhpClassTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_core_classes() -> Self {
        let mut classes = Self::new();
        classes
            .declare_class("Exception")
            .expect("core class table should start empty");
        classes
            .declare_class("stdClass")
            .expect("core class table should contain only Exception before stdClass");
        let mysqli_id = classes
            .declare_class("mysqli")
            .expect("core class table should contain Exception and stdClass before mysqli");
        let mysqli = classes
            .get_mut(mysqli_id)
            .expect("declared mysqli class id should resolve");
        mysqli
            .add_property(PhpPropertyMetadata::instance(
                "connect_errno",
                Visibility::Public,
            ))
            .expect("mysqli core metadata should not duplicate connect_errno");
        mysqli
            .add_property(PhpPropertyMetadata::instance(
                "connect_error",
                Visibility::Public,
            ))
            .expect("mysqli core metadata should not duplicate connect_error");
        classes
    }

    pub fn declare_class(&mut self, name: impl Into<String>) -> RuntimeResult<ClassId> {
        let name = name.into();
        let lookup_name = normalize_class_lookup_name(&name);
        if self.lookup.contains_key(&lookup_name) {
            return Err(RuntimeError::duplicate_class(name));
        }

        let id = ClassId(self.classes.len());
        self.lookup.insert(lookup_name, id);
        self.classes.push(PhpClassMetadata::new(id, name));
        Ok(id)
    }

    pub fn get(&self, id: ClassId) -> Option<&PhpClassMetadata> {
        self.classes.get(id.index())
    }

    pub fn get_mut(&mut self, id: ClassId) -> Option<&mut PhpClassMetadata> {
        self.classes.get_mut(id.index())
    }

    pub fn lookup_class(&self, name: &str) -> Option<&PhpClassMetadata> {
        let id = self.lookup.get(&normalize_class_lookup_name(name))?;
        self.get(*id)
    }

    pub fn lookup_class_id(&self, name: &str) -> Option<ClassId> {
        self.lookup.get(&normalize_class_lookup_name(name)).copied()
    }

    pub fn classes(&self) -> &[PhpClassMetadata] {
        &self.classes
    }

    pub fn remove_last_declared_class(&mut self, id: ClassId) {
        if id.index() + 1 != self.classes.len() {
            return;
        }

        if let Some(class) = self.classes.pop() {
            self.lookup
                .remove(&normalize_class_lookup_name(class.name()));
        }
    }

    pub fn set_parent(&mut self, child_id: ClassId, parent_id: ClassId) -> RuntimeResult<()> {
        if child_id == parent_id || self.is_subclass_of(parent_id, child_id) {
            let child_name = self
                .get(child_id)
                .map(|class| class.name().to_string())
                .unwrap_or_else(|| format!("#{}", child_id.index()));
            return Err(RuntimeError::unsupported_class_inheritance(
                child_name,
                "cyclic inheritance is not implemented",
            ));
        }

        let child = self
            .get_mut(child_id)
            .expect("declared child class id should resolve to metadata");
        child.parent_id = Some(parent_id);
        Ok(())
    }

    pub fn set_interfaces(
        &mut self,
        class_id: ClassId,
        interfaces: Vec<String>,
    ) -> RuntimeResult<()> {
        let class = self
            .get_mut(class_id)
            .expect("declared class id should resolve to metadata");
        class.interfaces = interfaces;
        Ok(())
    }

    pub fn is_subclass_of(&self, child_id: ClassId, ancestor_id: ClassId) -> bool {
        let mut current = self.get(child_id).and_then(|class| class.parent_id());
        let mut visited = HashSet::new();
        while let Some(class_id) = current {
            if class_id == ancestor_id {
                return true;
            }
            if !visited.insert(class_id) {
                return false;
            }
            current = self.get(class_id).and_then(|class| class.parent_id());
        }
        false
    }

    pub fn implements_interface(&self, class_id: ClassId, interface_name: &str) -> bool {
        let interface_name = normalize_class_lookup_name(interface_name);
        let mut current = Some(class_id);
        let mut visited = HashSet::new();
        while let Some(class_id) = current {
            if !visited.insert(class_id) {
                return false;
            }
            let Some(class) = self.get(class_id) else {
                return false;
            };
            if class
                .interfaces
                .iter()
                .any(|name| normalize_class_lookup_name(name) == interface_name)
            {
                return true;
            }
            current = class.parent_id();
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpClassMetadata {
    id: ClassId,
    name: String,
    parent_id: Option<ClassId>,
    properties: Vec<PhpPropertyMetadata>,
    property_lookup: HashMap<String, usize>,
    interfaces: Vec<String>,
    constants: Vec<PhpClassConstantMetadata>,
    constant_lookup: HashMap<String, usize>,
    methods: Vec<PhpMethodMetadata>,
    method_lookup: HashMap<String, usize>,
}

impl PhpClassMetadata {
    fn new(id: ClassId, name: String) -> Self {
        Self {
            id,
            name,
            parent_id: None,
            properties: Vec::new(),
            property_lookup: HashMap::new(),
            interfaces: Vec::new(),
            constants: Vec::new(),
            constant_lookup: HashMap::new(),
            methods: Vec::new(),
            method_lookup: HashMap::new(),
        }
    }

    pub fn id(&self) -> ClassId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent_id(&self) -> Option<ClassId> {
        self.parent_id
    }

    pub fn properties(&self) -> &[PhpPropertyMetadata] {
        &self.properties
    }

    pub fn interfaces(&self) -> &[String] {
        &self.interfaces
    }

    pub fn constants(&self) -> &[PhpClassConstantMetadata] {
        &self.constants
    }

    pub fn methods(&self) -> &[PhpMethodMetadata] {
        &self.methods
    }

    pub fn add_property(&mut self, property: PhpPropertyMetadata) -> RuntimeResult<()> {
        let name = property.name.clone();
        if self.property_lookup.contains_key(&name) {
            return Err(RuntimeError::duplicate_class_member(
                self.name.clone(),
                ClassMemberKind::Property,
                name,
            ));
        }

        self.property_lookup.insert(name, self.properties.len());
        self.properties.push(property);
        Ok(())
    }

    pub fn add_constant(&mut self, constant: PhpClassConstantMetadata) -> RuntimeResult<()> {
        let name = constant.name.clone();
        if self.constant_lookup.contains_key(&name) {
            return Err(RuntimeError::duplicate_class_member(
                self.name.clone(),
                ClassMemberKind::Constant,
                name,
            ));
        }

        self.constant_lookup.insert(name, self.constants.len());
        self.constants.push(constant);
        Ok(())
    }

    pub fn add_method(&mut self, method: PhpMethodMetadata) -> RuntimeResult<()> {
        let name = method.name.clone();
        let lookup_name = normalize_class_lookup_name(&name);
        if self.method_lookup.contains_key(&lookup_name) {
            return Err(RuntimeError::duplicate_class_member(
                self.name.clone(),
                ClassMemberKind::Method,
                name,
            ));
        }

        self.method_lookup.insert(lookup_name, self.methods.len());
        self.methods.push(method);
        Ok(())
    }

    pub fn property(&self, name: &str) -> Option<&PhpPropertyMetadata> {
        let index = self.property_lookup.get(name)?;
        self.properties.get(*index)
    }

    pub fn constant(&self, name: &str) -> Option<&PhpClassConstantMetadata> {
        let index = self.constant_lookup.get(name)?;
        self.constants.get(*index)
    }

    pub fn method(&self, name: &str) -> Option<&PhpMethodMetadata> {
        let index = self.method_lookup.get(&normalize_class_lookup_name(name))?;
        self.methods.get(*index)
    }

    pub fn object_shape(&self) -> PhpObjectShape {
        let instance_properties = self
            .properties
            .iter()
            .filter(|property| !property.is_static)
            .map(|property| property.name.clone())
            .collect();

        PhpObjectShape {
            class_id: self.id,
            instance_properties,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassMemberKind {
    Property,
    Constant,
    Method,
}

impl fmt::Display for ClassMemberKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassMemberKind::Property => write!(f, "property"),
            ClassMemberKind::Constant => write!(f, "constant"),
            ClassMemberKind::Method => write!(f, "method"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpClassConstantMetadata {
    name: String,
    visibility: Visibility,
}

impl PhpClassConstantMetadata {
    pub fn new(name: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            visibility,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpPropertyMetadata {
    name: String,
    visibility: Visibility,
    is_static: bool,
}

impl PhpPropertyMetadata {
    pub fn instance(name: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            visibility,
            is_static: false,
        }
    }

    pub fn static_property(name: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            visibility,
            is_static: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpObjectPropertyInitializer {
    declaring_class_id: ClassId,
    declaring_class_name: String,
    property: PhpPropertyMetadata,
}

impl PhpObjectPropertyInitializer {
    pub fn new(
        declaring_class_id: ClassId,
        declaring_class_name: impl Into<String>,
        property: PhpPropertyMetadata,
    ) -> Self {
        Self {
            declaring_class_id,
            declaring_class_name: declaring_class_name.into(),
            property,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpMethodMetadata {
    name: String,
    visibility: Visibility,
    is_static: bool,
}

impl PhpMethodMetadata {
    pub fn instance(name: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            visibility,
            is_static: false,
        }
    }

    pub fn static_method(name: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            name: name.into(),
            visibility,
            is_static: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpObjectShape {
    class_id: ClassId,
    instance_properties: Vec<String>,
}

impl PhpObjectShape {
    pub fn class_id(&self) -> ClassId {
        self.class_id
    }

    pub fn instance_properties(&self) -> &[String] {
        &self.instance_properties
    }
}

static NEXT_OBJECT_ID: AtomicI64 = AtomicI64::new(1);

#[derive(Debug, Clone)]
pub struct PhpObject {
    id: i64,
    class_id: ClassId,
    class_name: String,
    properties: Rc<RefCell<Vec<ObjectProperty>>>,
}

impl PartialEq for PhpObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PhpObject {
    pub fn from_class(class: &PhpClassMetadata) -> Self {
        Self::from_class_with_id(class, NEXT_OBJECT_ID.fetch_add(1, AtomicOrdering::Relaxed))
    }

    pub fn from_class_with_id(class: &PhpClassMetadata, id: i64) -> Self {
        Self::from_class_with_inherited_properties_with_id(class, &[], id)
    }

    pub fn from_class_with_inherited_properties_with_id(
        class: &PhpClassMetadata,
        inherited_properties: &[PhpObjectPropertyInitializer],
        id: i64,
    ) -> Self {
        let mut properties = Vec::new();
        for initializer in inherited_properties
            .iter()
            .filter(|initializer| !initializer.property.is_static())
        {
            Self::push_or_update_instance_property(
                &mut properties,
                initializer.declaring_class_id,
                initializer.declaring_class_name.clone(),
                initializer.property.name(),
                initializer.property.visibility(),
            );
        }

        for property in class
            .properties()
            .iter()
            .filter(|property| !property.is_static())
        {
            Self::push_or_update_instance_property(
                &mut properties,
                class.id(),
                class.name().to_string(),
                property.name(),
                property.visibility(),
            );
        }

        Self {
            id,
            class_id: class.id(),
            class_name: class.name().to_string(),
            properties: Rc::new(RefCell::new(properties)),
        }
    }

    fn push_or_update_instance_property(
        properties: &mut Vec<ObjectProperty>,
        declaring_class_id: ClassId,
        declaring_class_name: String,
        name: &str,
        visibility: Visibility,
    ) {
        if visibility != Visibility::Private {
            if let Some(property) = properties.iter_mut().find(|property| {
                property.name == name && property.visibility != Visibility::Private
            }) {
                property.visibility = visibility;
                return;
            }
        }

        properties.push(ObjectProperty {
            declaring_class_id,
            declaring_class_name,
            name: name.to_string(),
            visibility,
            value: Value::Null,
        });
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn hash(&self) -> String {
        format!("{:032x}", self.id)
    }

    pub fn class_id(&self) -> ClassId {
        self.class_id
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn properties(&self) -> Vec<ObjectProperty> {
        self.properties.borrow().clone()
    }

    pub fn shallow_clone_with_id(&self, id: i64) -> Self {
        Self {
            id,
            class_id: self.class_id,
            class_name: self.class_name.clone(),
            properties: Rc::new(RefCell::new(self.properties.borrow().clone())),
        }
    }

    pub fn read_public_property(&self, name: &str) -> RuntimeResult<Value> {
        let properties = self.properties.borrow();
        let property = self.public_property_or_error(&properties, name)?;

        Ok(property.value.clone())
    }

    pub fn read_property_from_context(
        &self,
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<Value> {
        let properties = self.properties.borrow();
        let property =
            self.context_property(&properties, name, current_class_id, protected_class_ids)?;

        Ok(property.value.clone())
    }

    pub fn is_public_property_set(&self, name: &str) -> RuntimeResult<bool> {
        let properties = self.properties.borrow();
        let Some(property) = self.public_property_or_none(&properties, name)? else {
            return Ok(false);
        };

        Ok(!matches!(property.value, Value::Null))
    }

    pub fn is_property_set_from_context(
        &self,
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<bool> {
        let properties = self.properties.borrow();
        let Some(property) = self.context_property_or_none(
            &properties,
            name,
            current_class_id,
            protected_class_ids,
        )?
        else {
            return Ok(false);
        };

        Ok(!matches!(property.value, Value::Null))
    }

    pub fn read_public_property_for_isset(&self, name: &str) -> RuntimeResult<Option<Value>> {
        let properties = self.properties.borrow();
        let Some(property) = self.public_property_or_none(&properties, name)? else {
            return Ok(None);
        };

        Ok(Some(property.value.clone()))
    }

    pub fn read_property_for_isset_from_context(
        &self,
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<Option<Value>> {
        let properties = self.properties.borrow();
        let Some(property) = self.context_property_or_none(
            &properties,
            name,
            current_class_id,
            protected_class_ids,
        )?
        else {
            return Ok(None);
        };

        Ok(Some(property.value.clone()))
    }

    pub fn read_current_public_property(&self, name: &str) -> Option<Value> {
        self.properties
            .borrow()
            .iter()
            .find(|property| property.name == name && property.visibility == Visibility::Public)
            .map(|property| property.value.clone())
    }

    pub fn is_public_property_empty(&self, name: &str) -> RuntimeResult<bool> {
        let properties = self.properties.borrow();
        let Some(property) = self.public_property_or_none(&properties, name)? else {
            return Ok(true);
        };

        Ok(!property.value.is_truthy())
    }

    pub fn is_property_empty_from_context(
        &self,
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<bool> {
        let properties = self.properties.borrow();
        let Some(property) = self.context_property_or_none(
            &properties,
            name,
            current_class_id,
            protected_class_ids,
        )?
        else {
            return Ok(true);
        };

        Ok(!property.value.is_truthy())
    }

    pub fn write_public_property(&self, name: &str, value: Value) -> RuntimeResult<()> {
        let mut properties = self.properties.borrow_mut();
        let property = self.public_property_mut_or_error(&mut properties, name)?;

        property.value = value;
        Ok(())
    }

    pub fn write_dynamic_public_property(&self, name: &str, value: Value) -> RuntimeResult<()> {
        let mut properties = self.properties.borrow_mut();
        if let Some(index) = properties.iter().rposition(|property| {
            property.name == name && property.visibility == Visibility::Public
        }) {
            properties[index].value = value;
            return Ok(());
        }

        if properties
            .iter()
            .any(|property| property.name == name && property.visibility != Visibility::Public)
        {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires same-class method context in the current subset",
                self.class_name, name
            )));
        }

        if !self.class_name.eq_ignore_ascii_case("stdClass") {
            return Err(RuntimeError::undefined_property(
                self.class_name.clone(),
                name,
            ));
        }

        properties.push(ObjectProperty {
            declaring_class_id: self.class_id,
            declaring_class_name: self.class_name.clone(),
            name: name.to_string(),
            visibility: Visibility::Public,
            value,
        });
        Ok(())
    }

    pub fn write_property_from_context(
        &self,
        name: &str,
        value: Value,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<()> {
        let mut properties = self.properties.borrow_mut();
        let property = self.context_property_mut(
            &mut properties,
            name,
            current_class_id,
            protected_class_ids,
        )?;

        property.value = value;
        Ok(())
    }

    fn public_property_or_error<'a>(
        &self,
        properties: &'a [ObjectProperty],
        name: &str,
    ) -> RuntimeResult<&'a ObjectProperty> {
        self.public_property_or_none(properties, name)?
            .ok_or_else(|| RuntimeError::undefined_property(self.class_name.clone(), name))
    }

    fn public_property_or_none<'a>(
        &self,
        properties: &'a [ObjectProperty],
        name: &str,
    ) -> RuntimeResult<Option<&'a ObjectProperty>> {
        if let Some(property) = properties
            .iter()
            .rev()
            .find(|property| property.name == name && property.visibility == Visibility::Public)
        {
            return Ok(Some(property));
        }

        self.unsupported_non_public_property(properties, name)
    }

    fn public_property_mut_or_error<'a>(
        &self,
        properties: &'a mut [ObjectProperty],
        name: &str,
    ) -> RuntimeResult<&'a mut ObjectProperty> {
        if let Some(index) = properties.iter().rposition(|property| {
            property.name == name && property.visibility == Visibility::Public
        }) {
            return Ok(&mut properties[index]);
        }

        self.unsupported_non_public_property_mut(properties, name)
    }

    fn context_property<'a>(
        &self,
        properties: &'a [ObjectProperty],
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<&'a ObjectProperty> {
        self.context_property_or_none(properties, name, current_class_id, protected_class_ids)?
            .ok_or_else(|| RuntimeError::undefined_property(self.class_name.clone(), name))
    }

    fn context_property_or_none<'a>(
        &self,
        properties: &'a [ObjectProperty],
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<Option<&'a ObjectProperty>> {
        if current_class_id.is_some() {
            if let Some(property) = properties.iter().rev().find(|property| {
                property.name == name
                    && property.visibility != Visibility::Public
                    && property.is_visible_in_context(current_class_id, protected_class_ids)
            }) {
                return Ok(Some(property));
            }
        }

        self.public_property_or_none(properties, name)
    }

    fn context_property_mut<'a>(
        &self,
        properties: &'a mut [ObjectProperty],
        name: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> RuntimeResult<&'a mut ObjectProperty> {
        if current_class_id.is_some() {
            if let Some(index) = properties.iter().rposition(|property| {
                property.name == name
                    && property.visibility != Visibility::Public
                    && property.is_visible_in_context(current_class_id, protected_class_ids)
            }) {
                return Ok(&mut properties[index]);
            }
        }

        self.public_property_mut_or_error(properties, name)
    }

    fn unsupported_non_public_property<'a>(
        &self,
        properties: &'a [ObjectProperty],
        name: &str,
    ) -> RuntimeResult<Option<&'a ObjectProperty>> {
        if properties
            .iter()
            .any(|property| property.name == name && property.visibility != Visibility::Public)
        {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires same-class method context in the current subset",
                self.class_name, name
            )));
        }

        Ok(None)
    }

    fn unsupported_non_public_property_mut<'a>(
        &self,
        properties: &'a mut [ObjectProperty],
        name: &str,
    ) -> RuntimeResult<&'a mut ObjectProperty> {
        if properties
            .iter()
            .any(|property| property.name == name && property.visibility != Visibility::Public)
        {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires same-class method context in the current subset",
                self.class_name, name
            )));
        }

        Err(RuntimeError::undefined_property(
            self.class_name.clone(),
            name,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
    declaring_class_id: ClassId,
    declaring_class_name: String,
    name: String,
    visibility: Visibility,
    value: Value,
}

impl ObjectProperty {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn declaring_class_id(&self) -> ClassId {
        self.declaring_class_id
    }

    pub fn declaring_class_name(&self) -> &str {
        &self.declaring_class_name
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn mangled_name(&self) -> String {
        match self.visibility {
            Visibility::Public => self.name.clone(),
            Visibility::Protected => format!("\0*\0{}", self.name),
            Visibility::Private => format!("\0{}\0{}", self.declaring_class_name, self.name),
        }
    }

    fn is_visible_in_context(
        &self,
        current_class_id: Option<ClassId>,
        protected_class_ids: &[ClassId],
    ) -> bool {
        match self.visibility {
            Visibility::Public => true,
            Visibility::Private => current_class_id == Some(self.declaring_class_id),
            Visibility::Protected => protected_class_ids.contains(&self.declaring_class_id),
        }
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

fn normalize_class_lookup_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(PhpArray),
    Object(PhpObject),
    Closure(PhpClosure),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhpClosure {
    id: i64,
    is_arrow: bool,
    captures: Vec<PhpClosureCapture>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhpClosureCapture {
    name: String,
    by_reference: bool,
    value: Value,
}

impl PhpClosure {
    pub fn new(id: i64, is_arrow: bool, captures: Vec<PhpClosureCapture>) -> Self {
        Self {
            id,
            is_arrow,
            captures,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn is_arrow(&self) -> bool {
        self.is_arrow
    }

    pub fn captures(&self) -> &[PhpClosureCapture] {
        &self.captures
    }
}

impl PhpClosureCapture {
    pub fn new(name: impl Into<String>, by_reference: bool, value: Value) -> Self {
        Self {
            name: name.into(),
            by_reference,
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn by_reference(&self) -> bool {
        self.by_reference
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
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
            Value::Object(_) => "object",
            Value::Closure(_) => "closure",
        }
    }

    pub fn gettype_name(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Bool(_) => "boolean",
            Value::Int(_) => "integer",
            Value::Float(_) => "double",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Closure(_) => "object",
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
        )
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            Value::Int(_) | Value::Float(_) => true,
            Value::String(value) => parse_numeric_string(value).is_some(),
            Value::Null
            | Value::Bool(_)
            | Value::Array(_)
            | Value::Object(_)
            | Value::Closure(_) => false,
        }
    }

    pub fn is_countable(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_iterable(&self) -> bool {
        matches!(self, Value::Array(_))
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
            Value::Object(_) => "Object".to_string(),
            Value::Closure(_) => "Object".to_string(),
        }
    }

    pub fn try_echo_string(&self) -> RuntimeResult<String> {
        match self {
            Value::Object(object) => Err(RuntimeError::invalid_string_conversion(format!(
                "object of class {} cannot be converted to string",
                object.class_name()
            ))),
            Value::Closure(_) => Err(RuntimeError::invalid_string_conversion(
                "object of class Closure cannot be converted to string",
            )),
            _ => Ok(self.echo_string()),
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
            Value::Object(_) => true,
            Value::Closure(_) => true,
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

    pub fn php_mod(&self, other: &Value) -> RuntimeResult<Value> {
        let left = self.to_arithmetic_int(ArithmeticOp::Modulo)?;
        let right = other.to_arithmetic_int(ArithmeticOp::Modulo)?;
        if right == 0 {
            return Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::Modulo,
                "modulo by zero",
            ));
        }
        if left == i64::MIN && right == -1 {
            return Ok(Value::Int(0));
        }

        Ok(Value::Int(left % right))
    }

    pub fn php_negate(&self) -> RuntimeResult<Value> {
        match self.to_arithmetic_number(ArithmeticOp::Negate)? {
            Number::Int(value) => Ok(Value::Int(value.wrapping_neg())),
            Number::Float(value) => Ok(Value::Float(-value)),
        }
    }

    pub fn php_concat(&self, other: &Value) -> RuntimeResult<Value> {
        Ok(Value::String(format!(
            "{}{}",
            self.try_echo_string()?,
            other.try_echo_string()?
        )))
    }

    pub fn php_bitwise_and(&self, other: &Value) -> RuntimeResult<Value> {
        self.php_bitwise(
            other,
            ArithmeticOp::BitwiseAnd,
            |left, right| left & right,
            |left, right| left & right,
        )
    }

    pub fn php_bitwise_or(&self, other: &Value) -> RuntimeResult<Value> {
        self.php_bitwise(
            other,
            ArithmeticOp::BitwiseOr,
            |left, right| left | right,
            |left, right| left | right,
        )
    }

    pub fn php_bitwise_xor(&self, other: &Value) -> RuntimeResult<Value> {
        self.php_bitwise(
            other,
            ArithmeticOp::BitwiseXor,
            |left, right| left ^ right,
            |left, right| left ^ right,
        )
    }

    pub fn php_bitwise_not(&self) -> RuntimeResult<Value> {
        match self {
            Value::Int(value) => Ok(Value::Int(!value)),
            Value::String(value) => bitwise_not_string(value).map(Value::String),
            Value::Null => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "null cannot be used with unary bitwise not",
            )),
            Value::Bool(_) => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "booleans cannot be used with unary bitwise not",
            )),
            Value::Float(_) => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "floats cannot be used with unary bitwise not",
            )),
            Value::Array(_) => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "arrays cannot be used with unary bitwise not",
            )),
            Value::Object(_) => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "objects cannot be used with unary bitwise not",
            )),
            Value::Closure(_) => Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::BitwiseNot,
                "closures cannot be used with unary bitwise not",
            )),
        }
    }

    pub fn php_shift_left(&self, other: &Value) -> RuntimeResult<Value> {
        let left = self.to_bitwise_int(ArithmeticOp::ShiftLeft)?;
        let right = other.to_bitwise_int(ArithmeticOp::ShiftLeft)?;
        if right < 0 {
            return Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::ShiftLeft,
                "bit shift by negative number",
            ));
        }
        let right = right as u32;
        if right >= i64::BITS {
            return Ok(Value::Int(0));
        }
        Ok(Value::Int((left as u64).wrapping_shl(right) as i64))
    }

    pub fn php_shift_right(&self, other: &Value) -> RuntimeResult<Value> {
        let left = self.to_bitwise_int(ArithmeticOp::ShiftRight)?;
        let right = other.to_bitwise_int(ArithmeticOp::ShiftRight)?;
        if right < 0 {
            return Err(RuntimeError::invalid_arithmetic(
                ArithmeticOp::ShiftRight,
                "bit shift by negative number",
            ));
        }
        let right = right as u32;
        if right >= i64::BITS - 1 {
            return Ok(Value::Int(if left < 0 { -1 } else { 0 }));
        }
        Ok(Value::Int(left >> right))
    }

    pub fn php_eq(&self, other: &Value) -> bool {
        self.php_cmp(other, Comparison::Eq)
    }

    pub fn php_identical_checked(&self, other: &Value) -> RuntimeResult<bool> {
        match (self, other) {
            (Value::Array(_), _) | (_, Value::Array(_)) => {
                Err(RuntimeError::unsupported_comparison(
                    "strict identity for arrays is not implemented",
                ))
            }
            (Value::Object(left), Value::Object(right)) => Ok(left.id() == right.id()),
            (Value::Object(_), _) | (_, Value::Object(_)) => Ok(false),
            _ => Ok(self.php_identical_scalar(other)),
        }
    }

    fn php_identical_scalar(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Int(left), Value::Int(right)) => left == right,
            (Value::Float(left), Value::Float(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            _ => false,
        }
    }

    pub fn php_cmp_checked(&self, other: &Value, op: Comparison) -> RuntimeResult<bool> {
        match (self, other) {
            (Value::Object(_), _) | (_, Value::Object(_)) => Err(
                RuntimeError::unsupported_comparison("object comparisons are not implemented"),
            ),
            _ => Ok(self.php_cmp(other, op)),
        }
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
            (Value::Object(_), _) | (_, Value::Object(_)) => None,
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
            Value::Object(_) => None,
            Value::Closure(_) => None,
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
            Value::Object(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "objects are not numeric",
            )),
            Value::Closure(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "closures are not numeric",
            )),
        }
    }

    fn to_arithmetic_int(&self, operation: ArithmeticOp) -> RuntimeResult<i64> {
        match self.to_arithmetic_number(operation)? {
            Number::Int(value) => Ok(value),
            Number::Float(value) => Ok(value as i64),
        }
    }

    fn php_bitwise(
        &self,
        other: &Value,
        operation: ArithmeticOp,
        byte_op: fn(u8, u8) -> u8,
        int_op: fn(i64, i64) -> i64,
    ) -> RuntimeResult<Value> {
        if let (Value::String(left), Value::String(right)) = (self, other) {
            return bitwise_strings(left, right, operation, byte_op).map(Value::String);
        }

        let left = self.to_bitwise_int(operation)?;
        let right = other.to_bitwise_int(operation)?;
        Ok(Value::Int(int_op(left, right)))
    }

    fn to_bitwise_int(&self, operation: ArithmeticOp) -> RuntimeResult<i64> {
        match self {
            Value::Null => Ok(0),
            Value::Bool(false) => Ok(0),
            Value::Bool(true) => Ok(1),
            Value::Int(value) => Ok(*value),
            Value::Float(value) => Ok(*value as i64),
            Value::String(value) => match parse_numeric_string(value) {
                Some(Number::Int(value)) => Ok(value),
                Some(Number::Float(value)) => Ok(value as i64),
                None => Err(RuntimeError::invalid_arithmetic(
                    operation,
                    "string is not numeric",
                )),
            },
            Value::Array(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "arrays cannot be used with bitwise operators",
            )),
            Value::Object(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "objects cannot be used with bitwise operators",
            )),
            Value::Closure(_) => Err(RuntimeError::invalid_arithmetic(
                operation,
                "closures cannot be used with bitwise operators",
            )),
        }
    }
}

fn bitwise_strings(
    left: &str,
    right: &str,
    operation: ArithmeticOp,
    op: fn(u8, u8) -> u8,
) -> RuntimeResult<String> {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let output = match operation {
        ArithmeticOp::BitwiseAnd | ArithmeticOp::BitwiseXor => left
            .iter()
            .zip(right.iter())
            .map(|(left, right)| op(*left, *right))
            .collect(),
        ArithmeticOp::BitwiseOr => {
            let mut output = Vec::with_capacity(left.len().max(right.len()));
            let common_len = left.len().min(right.len());
            for index in 0..common_len {
                output.push(op(left[index], right[index]));
            }
            if left.len() > common_len {
                output.extend_from_slice(&left[common_len..]);
            } else if right.len() > common_len {
                output.extend_from_slice(&right[common_len..]);
            }
            output
        }
        _ => unreachable!("caller provided a bitwise operation"),
    };

    String::from_utf8(output).map_err(|_| {
        RuntimeError::invalid_arithmetic(
            operation,
            "binary string results outside UTF-8 are not supported",
        )
    })
}

fn bitwise_not_string(value: &str) -> RuntimeResult<String> {
    let output: Vec<u8> = value.as_bytes().iter().map(|byte| !byte).collect();
    String::from_utf8(output).map_err(|_| {
        RuntimeError::invalid_arithmetic(
            ArithmeticOp::BitwiseNot,
            "binary string results outside UTF-8 are not supported",
        )
    })
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
    fn native_scalar_abi_tags_are_stable() {
        assert_eq!(NativeScalarTag::Null as u8, 0);
        assert_eq!(NativeScalarTag::Bool as u8, 1);
        assert_eq!(NativeScalarTag::Int as u8, 2);
        assert_eq!(NativeScalarTag::Float as u8, 3);
    }

    #[test]
    fn native_scalar_abi_converts_to_runtime_values() {
        assert_eq!(phpc_native_null().to_value(), Value::Null);
        assert_eq!(phpc_native_bool(false).to_value(), Value::Bool(false));
        assert_eq!(phpc_native_bool(true).to_value(), Value::Bool(true));
        assert_eq!(phpc_native_int(-42).to_value(), Value::Int(-42));
        assert_eq!(phpc_native_float(1.5).to_value(), Value::Float(1.5));
    }

    #[test]
    fn native_scalar_abi_normalizes_bool_payloads() {
        let mut value = NativeScalarValue::bool(false);
        value.bool_value = 7;

        assert_eq!(value.tag(), NativeScalarTag::Bool);
        assert_eq!(value.to_value(), Value::Bool(true));
    }

    #[test]
    fn native_scalar_echo_helper_reports_required_lengths() {
        assert_eq!(phpc_native_scalar_echo_len(phpc_native_null()), 0);
        assert_eq!(phpc_native_scalar_echo_len(phpc_native_bool(false)), 0);
        assert_eq!(phpc_native_scalar_echo_len(phpc_native_bool(true)), 1);
        assert_eq!(phpc_native_scalar_echo_len(phpc_native_int(-42)), 3);
        assert_eq!(phpc_native_scalar_echo_len(phpc_native_float(1.5)), 3);
    }

    #[test]
    fn native_scalar_echo_helper_writes_bytes() {
        let mut buffer = [0_u8; 8];
        let required = unsafe {
            phpc_native_scalar_echo_write(phpc_native_int(12345), buffer.as_mut_ptr(), 3)
        };

        assert_eq!(required, 5);
        assert_eq!(&buffer[..3], b"123");
        assert_eq!(&buffer[3..], &[0, 0, 0, 0, 0]);

        let required = unsafe {
            phpc_native_scalar_echo_write(phpc_native_bool(true), buffer.as_mut_ptr(), buffer.len())
        };

        assert_eq!(required, 1);
        assert_eq!(buffer[0], b'1');
    }

    #[test]
    fn native_scalar_echo_helper_accepts_null_buffers_for_sizing() {
        let required =
            unsafe { phpc_native_scalar_echo_write(phpc_native_int(123), std::ptr::null_mut(), 0) };

        assert_eq!(required, 3);
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
    fn is_numeric_matches_current_numeric_scalar_subset() {
        assert!(Value::Int(7).is_numeric());
        assert!(Value::Float(3.5).is_numeric());
        assert!(Value::String(" 42 ".to_string()).is_numeric());
        assert!(Value::String("-.5".to_string()).is_numeric());
        assert!(Value::String("5.".to_string()).is_numeric());
        assert!(Value::String("8e2".to_string()).is_numeric());

        assert!(!Value::String(String::new()).is_numeric());
        assert!(!Value::String(" ".to_string()).is_numeric());
        assert!(!Value::String("8foo".to_string()).is_numeric());
        assert!(!Value::String("0x10".to_string()).is_numeric());
        assert!(!Value::Bool(true).is_numeric());
        assert!(!Value::Null.is_numeric());
        assert!(!Value::Array(PhpArray::new()).is_numeric());
    }

    #[test]
    fn is_countable_matches_current_array_only_subset() {
        assert!(Value::Array(PhpArray::new()).is_countable());
        assert!(!Value::Null.is_countable());
        assert!(!Value::Bool(false).is_countable());
        assert!(!Value::Int(0).is_countable());
        assert!(!Value::Float(3.5).is_countable());
        assert!(!Value::String(String::new()).is_countable());
    }

    #[test]
    fn is_iterable_matches_current_array_only_subset() {
        assert!(Value::Array(PhpArray::new()).is_iterable());
        assert!(!Value::Null.is_iterable());
        assert!(!Value::Bool(false).is_iterable());
        assert!(!Value::Int(0).is_iterable());
        assert!(!Value::Float(3.5).is_iterable());
        assert!(!Value::String(String::new()).is_iterable());
    }

    #[test]
    fn modulo_coerces_supported_operands_to_integers() {
        assert_eq!(
            Value::Int(7).php_mod(&Value::Int(3)).unwrap(),
            Value::Int(1)
        );
        assert_eq!(
            Value::Int(-7).php_mod(&Value::Int(3)).unwrap(),
            Value::Int(-1)
        );
        assert_eq!(
            Value::Float(7.9).php_mod(&Value::Int(3)).unwrap(),
            Value::Int(1)
        );
        assert_eq!(
            Value::String(" 8 ".to_string())
                .php_mod(&Value::Bool(true))
                .unwrap(),
            Value::Int(0)
        );
        assert_eq!(Value::Null.php_mod(&Value::Int(3)).unwrap(), Value::Int(0));
        assert_eq!(
            Value::Int(i64::MIN).php_mod(&Value::Int(-1)).unwrap(),
            Value::Int(0)
        );
    }

    #[test]
    fn modulo_reports_zero_divisor_with_stable_error() {
        let error = Value::Int(5).php_mod(&Value::Int(0)).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::Modulo,
                reason: "modulo by zero".to_string(),
            }
        );
        assert_eq!(error.message(), "invalid arithmetic for %: modulo by zero");
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
    fn bitwise_not_handles_current_int_and_string_subset() {
        assert_eq!(Value::Int(0).php_bitwise_not().unwrap(), Value::Int(-1));
        assert_eq!(Value::Int(5).php_bitwise_not().unwrap(), Value::Int(-6));
        assert_eq!(Value::Int(-1).php_bitwise_not().unwrap(), Value::Int(0));
        assert_eq!(
            Value::String(String::new()).php_bitwise_not().unwrap(),
            Value::String(String::new())
        );
    }

    #[test]
    fn bitwise_not_reports_unsupported_operands_with_stable_errors() {
        let error = Value::Bool(true).php_bitwise_not().unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::BitwiseNot,
                reason: "booleans cannot be used with unary bitwise not".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for ~: booleans cannot be used with unary bitwise not"
        );

        let error = Value::String("A".to_string())
            .php_bitwise_not()
            .unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::BitwiseNot,
                reason: "binary string results outside UTF-8 are not supported".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for ~: binary string results outside UTF-8 are not supported"
        );
    }

    #[test]
    fn shift_operators_handle_current_int_coercion_subset() {
        assert_eq!(
            Value::Int(8).php_shift_left(&Value::Int(1)).unwrap(),
            Value::Int(16)
        );
        assert_eq!(
            Value::Int(8).php_shift_right(&Value::Int(1)).unwrap(),
            Value::Int(4)
        );
        assert_eq!(
            Value::Int(-8).php_shift_right(&Value::Int(1)).unwrap(),
            Value::Int(-4)
        );
        assert_eq!(
            Value::String("8".to_string())
                .php_shift_left(&Value::Bool(true))
                .unwrap(),
            Value::Int(16)
        );
        assert_eq!(
            Value::Null.php_shift_right(&Value::Int(1)).unwrap(),
            Value::Int(0)
        );
        assert_eq!(
            Value::Int(8).php_shift_left(&Value::Int(64)).unwrap(),
            Value::Int(0)
        );
        assert_eq!(
            Value::Int(-1).php_shift_right(&Value::Int(64)).unwrap(),
            Value::Int(-1)
        );
    }

    #[test]
    fn shift_operators_report_negative_counts_with_stable_errors() {
        let error = Value::Int(8).php_shift_left(&Value::Int(-1)).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::ShiftLeft,
                reason: "bit shift by negative number".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for <<: bit shift by negative number"
        );

        let error = Value::Int(8).php_shift_right(&Value::Int(-1)).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::InvalidArithmetic {
                operation: ArithmeticOp::ShiftRight,
                reason: "bit shift by negative number".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "invalid arithmetic for >>: bit shift by negative number"
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
    fn call_depth_errors_keep_structured_kind_and_stable_message() {
        let error = RuntimeError::call_depth_exceeded("loop()", 128);

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::CallDepthExceeded {
                callable: "loop()".to_string(),
                limit: 128,
            }
        );
        assert_eq!(
            error.message(),
            "maximum user function call depth exceeded for loop(): limit 128"
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

    #[test]
    fn strict_identity_matches_php_scalar_subset() {
        let cases = [
            ("null|null", Value::Null, Value::Null, true),
            ("null|false", Value::Null, Value::Bool(false), false),
            ("false|false", Value::Bool(false), Value::Bool(false), true),
            ("false|int0", Value::Bool(false), Value::Int(0), false),
            ("true|int1", Value::Bool(true), Value::Int(1), false),
            ("int1|int1", Value::Int(1), Value::Int(1), true),
            ("int1|float1", Value::Int(1), Value::Float(1.0), false),
            ("float1|float1", Value::Float(1.0), Value::Float(1.0), true),
            (
                "str1|int1",
                Value::String("1".to_string()),
                Value::Int(1),
                false,
            ),
            (
                "str1|str1",
                Value::String("1".to_string()),
                Value::String("1".to_string()),
                true,
            ),
        ];

        for (label, left, right, expected) in cases {
            let actual = left.php_identical_checked(&right).unwrap();
            assert_eq!(actual, expected, "strict identity mismatch for {label}");
        }
    }

    #[test]
    fn strict_identity_rejects_arrays_and_checks_object_handles() {
        let error = Value::Array(PhpArray::new())
            .php_identical_checked(&Value::Array(PhpArray::new()))
            .unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedComparison {
                reason: "strict identity for arrays is not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported comparison: strict identity for arrays is not implemented"
        );

        let mut classes = PhpClassTable::new();
        let class_id = classes.declare_class("Box").unwrap();
        let class = classes.get(class_id).unwrap();
        let object = Value::Object(PhpObject::from_class(class));
        let alias = object.clone();
        let other = Value::Object(PhpObject::from_class(class));

        assert!(object.php_identical_checked(&alias).unwrap());
        assert!(!object.php_identical_checked(&other).unwrap());
        assert!(!object.php_identical_checked(&Value::Null).unwrap());
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

    fn array_key_values(array: &PhpArray) -> Vec<Value> {
        array
            .entries()
            .iter()
            .map(|entry| entry.value.clone())
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
        assert!(array.contains_key("2"));
        assert!(array.contains_key(2));
        assert!(array.contains_key("02"));
        assert!(!array.contains_key("missing"));
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
    fn array_remove_preserves_order_and_does_not_reuse_auto_index() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.append(Value::String("three".to_string())).unwrap();

        assert!(array.remove("2"));
        assert!(!array.remove("missing"));
        assert_eq!(
            array.append(Value::String("four".to_string())).unwrap(),
            ArrayKey::Int(4)
        );

        let keys: Vec<ArrayKey> = array
            .entries()
            .iter()
            .map(|entry| entry.key.clone())
            .collect();
        assert_eq!(
            keys,
            vec![
                ArrayKey::String("name".to_string()),
                ArrayKey::Int(3),
                ArrayKey::Int(4),
            ]
        );
        assert!(!array.contains_key(2));
    }

    #[test]
    fn array_values_reindexes_entries_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert("2", Value::String("two updated".to_string()));

        let values = array.values_reindexed();
        let entries = values.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("five".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::String("two updated".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(3));
        assert_eq!(entries[3].value, Value::String("zero two".to_string()));
        assert!(values.contains_key(0));
        assert!(values.contains_key(3));
        assert!(!values.contains_key("name"));
        assert!(!values.contains_key(5));
        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_values must not mutate the original array"
        );
    }

    #[test]
    fn array_keys_reindexes_integer_and_string_keys_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert("2", Value::String("two updated".to_string()));
        array.insert(-1, Value::String("negative".to_string()));

        let keys = array.keys_reindexed();
        let entries = keys.entries();

        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("name".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::Int(5));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::Int(2));
        assert_eq!(entries[3].key, ArrayKey::Int(3));
        assert_eq!(entries[3].value, Value::String("02".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(4));
        assert_eq!(entries[4].value, Value::Int(-1));
        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_keys must not mutate the original array"
        );
    }

    #[test]
    fn array_key_first_returns_first_integer_or_string_key_or_null() {
        let empty = PhpArray::new();
        assert_eq!(empty.first_key_value(), Value::Null);

        let mut string_first = PhpArray::new();
        string_first.insert("name", Value::String("Ada".to_string()));
        string_first.insert(5, Value::String("five".to_string()));
        string_first.insert("2", Value::String("two".to_string()));
        string_first.insert("02", Value::String("zero two".to_string()));
        assert_eq!(
            string_first.first_key_value(),
            Value::String("name".to_string())
        );

        let mut int_first = PhpArray::new();
        int_first.insert("2", Value::String("two".to_string()));
        int_first.insert("name", Value::String("Ada".to_string()));
        int_first.insert("02", Value::String("zero two".to_string()));
        assert_eq!(int_first.first_key_value(), Value::Int(2));
    }

    #[test]
    fn array_key_last_returns_last_integer_or_string_key_or_null() {
        let empty = PhpArray::new();
        assert_eq!(empty.last_key_value(), Value::Null);

        let mut string_last = PhpArray::new();
        string_last.insert("name", Value::String("Ada".to_string()));
        string_last.insert(5, Value::String("five".to_string()));
        string_last.insert("2", Value::String("two".to_string()));
        string_last.insert("02", Value::String("zero two".to_string()));
        string_last.insert("2", Value::String("two updated".to_string()));
        assert_eq!(
            string_last.last_key_value(),
            Value::String("02".to_string())
        );

        let mut int_last = PhpArray::new();
        int_last.insert("name", Value::String("Ada".to_string()));
        int_last.insert("02", Value::String("zero two".to_string()));
        int_last.insert("2", Value::String("two".to_string()));
        assert_eq!(int_last.last_key_value(), Value::Int(2));
    }

    #[test]
    fn array_is_list_detects_zero_based_integer_keys_in_insertion_order() {
        let empty = PhpArray::new();
        assert!(empty.is_list());

        let mut list = PhpArray::new();
        list.append(Value::String("zero".to_string())).unwrap();
        list.append(Value::String("one".to_string())).unwrap();
        assert!(list.is_list());

        let mut normalized = PhpArray::new();
        normalized.insert("0", Value::String("zero".to_string()));
        normalized.insert("1", Value::String("one".to_string()));
        assert!(normalized.is_list());

        let mut out_of_order = PhpArray::new();
        out_of_order.insert(1, Value::String("one".to_string()));
        out_of_order.insert(0, Value::String("zero".to_string()));
        assert!(!out_of_order.is_list());

        let mut gap = PhpArray::new();
        gap.insert(0, Value::String("zero".to_string()));
        gap.insert(2, Value::String("two".to_string()));
        assert!(!gap.is_list());

        let mut string_key = PhpArray::new();
        string_key.insert(0, Value::String("zero".to_string()));
        string_key.insert("01", Value::String("one".to_string()));
        assert!(!string_key.is_list());

        let mut negative_key = PhpArray::new();
        negative_key.insert(-1, Value::String("negative".to_string()));
        negative_key.insert(0, Value::String("zero".to_string()));
        assert!(!negative_key.is_list());

        let mut after_unset = PhpArray::new();
        after_unset
            .append(Value::String("zero".to_string()))
            .unwrap();
        after_unset
            .append(Value::String("one".to_string()))
            .unwrap();
        after_unset
            .append(Value::String("two".to_string()))
            .unwrap();
        after_unset.remove(1);
        assert!(!after_unset.is_list());
        assert!(after_unset.values_reindexed().is_list());
    }

    #[test]
    fn array_keys_filters_with_loose_scalar_comparison_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert("int-zero", Value::Int(0));
        array.insert("string-zero", Value::String("0".to_string()));
        array.insert("empty", Value::String(String::new()));
        array.insert("int-ten", Value::Int(10));
        array.insert("string-ten", Value::String("10".to_string()));
        array.insert("numeric-string", Value::String("10.0".to_string()));
        array.insert("text", Value::String("abc".to_string()));

        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_loose_scalar(&Value::String(String::new()))
                    .unwrap()
            ),
            vec![
                Value::String("null".to_string()),
                Value::String("false".to_string()),
                Value::String("empty".to_string()),
            ]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_loose_scalar(&Value::String("0".to_string()))
                    .unwrap()
            ),
            vec![
                Value::String("false".to_string()),
                Value::String("int-zero".to_string()),
                Value::String("string-zero".to_string()),
            ]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_loose_scalar(&Value::String("10".to_string()))
                    .unwrap()
            ),
            vec![
                Value::String("int-ten".to_string()),
                Value::String("string-ten".to_string()),
                Value::String("numeric-string".to_string()),
            ]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_loose_scalar(&Value::String("abc".to_string()))
                    .unwrap()
            ),
            vec![Value::String("text".to_string())]
        );
        assert!(array
            .keys_matching_loose_scalar(&Value::String("missing".to_string()))
            .unwrap()
            .entries()
            .is_empty());
    }

    #[test]
    fn array_keys_filters_with_strict_scalar_identity_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert("int-zero", Value::Int(0));
        array.insert("string-zero", Value::String("0".to_string()));
        array.insert("empty", Value::String(String::new()));
        array.insert("int-ten", Value::Int(10));
        array.insert("string-ten", Value::String("10".to_string()));
        array.insert("numeric-string", Value::String("10.0".to_string()));
        array.insert("text", Value::String("abc".to_string()));

        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_strict_scalar(&Value::String(String::new()))
                    .unwrap()
            ),
            vec![Value::String("empty".to_string())]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_strict_scalar(&Value::Bool(false))
                    .unwrap()
            ),
            vec![Value::String("false".to_string())]
        );
        assert_eq!(
            array_key_values(&array.keys_matching_strict_scalar(&Value::Int(0)).unwrap()),
            vec![Value::String("int-zero".to_string())]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_strict_scalar(&Value::String("0".to_string()))
                    .unwrap()
            ),
            vec![Value::String("string-zero".to_string())]
        );
        assert!(array
            .keys_matching_strict_scalar(&Value::Float(10.0))
            .unwrap()
            .entries()
            .is_empty());
        assert_eq!(
            array_key_values(&array.keys_matching_strict_scalar(&Value::Int(10)).unwrap()),
            vec![Value::String("int-ten".to_string())]
        );
        assert_eq!(
            array_key_values(
                &array
                    .keys_matching_strict_scalar(&Value::String("10".to_string()))
                    .unwrap()
            ),
            vec![Value::String("string-ten".to_string())]
        );
        assert_eq!(
            array_key_values(&array.keys_matching_strict_scalar(&Value::Null).unwrap()),
            vec![Value::String("null".to_string())]
        );
        assert!(array
            .keys_matching_strict_scalar(&Value::String("missing".to_string()))
            .unwrap()
            .entries()
            .is_empty());
    }

    #[test]
    fn array_reverse_reindexes_integer_keys_and_preserves_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert("2", Value::String("two updated".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut reversed = array.reversed_reindexed();
        let entries = reversed.entries();

        assert_eq!(entries.len(), 6);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("next".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("negative".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("zero two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("two updated".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(3));
        assert_eq!(entries[4].value, Value::String("five".to_string()));
        assert_eq!(entries[5].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[5].value, Value::String("Ada".to_string()));
        assert_eq!(
            reversed.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(4)
        );
        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_reverse must not mutate the original array"
        );
    }

    #[test]
    fn array_reverse_can_preserve_integer_and_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert("2", Value::String("two updated".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut reversed = array.reversed_preserving_keys();
        let entries = reversed.entries();

        assert_eq!(entries.len(), 6);
        assert_eq!(entries[0].key, ArrayKey::Int(6));
        assert_eq!(entries[0].value, Value::String("next".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(-1));
        assert_eq!(entries[1].value, Value::String("negative".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("zero two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("two updated".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(5));
        assert_eq!(entries[4].value, Value::String("five".to_string()));
        assert_eq!(entries[5].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[5].value, Value::String("Ada".to_string()));
        assert_eq!(
            reversed.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(7)
        );
        assert_eq!(
            array.get(6),
            Some(&Value::String("next".to_string())),
            "array_reverse preserve_keys must not mutate the original array"
        );
    }

    #[test]
    fn array_slice_from_positive_offset_reindexes_integer_keys_and_preserves_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut sliced = array.sliced_from_offset(2);
        let entries = sliced.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("two".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[1].value, Value::String("zero two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(1));
        assert_eq!(entries[2].value, Value::String("negative".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("next".to_string()));
        assert_eq!(
            sliced.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_slice must not mutate the original array"
        );
    }

    #[test]
    fn array_slice_supports_negative_and_out_of_range_offsets() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let tail = array.sliced_from_offset(-3);
        let tail_entries = tail.entries();
        assert_eq!(tail_entries.len(), 3);
        assert_eq!(tail_entries[0].key, ArrayKey::String("02".to_string()));
        assert_eq!(tail_entries[0].value, Value::String("zero two".to_string()));
        assert_eq!(tail_entries[1].key, ArrayKey::Int(0));
        assert_eq!(tail_entries[1].value, Value::String("negative".to_string()));
        assert_eq!(tail_entries[2].key, ArrayKey::Int(1));
        assert_eq!(tail_entries[2].value, Value::String("next".to_string()));

        assert!(array.sliced_from_offset(99).entries().is_empty());

        let whole = array.sliced_from_offset(-99);
        assert_eq!(whole.entries().len(), array.entries().len());
        assert_eq!(whole.entries()[0].key, ArrayKey::String("name".to_string()));
    }

    #[test]
    fn array_slice_supports_positive_zero_and_negative_lengths() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let middle = array.sliced(1, Some(3));
        let middle_entries = middle.entries();
        assert_eq!(middle_entries.len(), 3);
        assert_eq!(middle_entries[0].key, ArrayKey::Int(0));
        assert_eq!(middle_entries[0].value, Value::String("five".to_string()));
        assert_eq!(middle_entries[1].key, ArrayKey::Int(1));
        assert_eq!(middle_entries[1].value, Value::String("two".to_string()));
        assert_eq!(middle_entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            middle_entries[2].value,
            Value::String("zero two".to_string())
        );

        assert!(array.sliced(1, Some(0)).entries().is_empty());

        let without_tail = array.sliced(1, Some(-2));
        assert_eq!(without_tail.entries(), middle_entries);

        let empty = array.sliced(4, Some(-3));
        assert!(empty.entries().is_empty());

        let negative_offset_with_length = array.sliced(-4, Some(2));
        let entries = negative_offset_with_length.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("two".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[1].value, Value::String("zero two".to_string()));

        let null_length = array.sliced(1, None);
        let entries = null_length.entries();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("five".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("zero two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("negative".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(3));
        assert_eq!(entries[4].value, Value::String("next".to_string()));
    }

    #[test]
    fn array_slice_can_preserve_integer_and_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut middle = array.sliced_preserving_keys(1, Some(3));
        let entries = middle.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::Int(5));
        assert_eq!(entries[0].value, Value::String("five".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("zero two".to_string()));
        assert_eq!(
            middle.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(6)
        );

        let mut tail = array.sliced_preserving_keys(-3, None);
        let entries = tail.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[0].value, Value::String("zero two".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(-1));
        assert_eq!(entries[1].value, Value::String("negative".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(6));
        assert_eq!(entries[2].value, Value::String("next".to_string()));
        assert_eq!(
            tail.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(7)
        );
        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_slice preserve_keys must not mutate the original array"
        );
    }

    #[test]
    fn array_chunk_splits_values_into_reindexed_nested_arrays() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut chunks = array.chunked_reindexed(2);
        let entries = chunks.entries();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        let Value::Array(first) = &entries[0].value else {
            panic!("first chunk is an array");
        };
        assert_eq!(first.entries().len(), 2);
        assert_eq!(first.entries()[0].key, ArrayKey::Int(0));
        assert_eq!(first.entries()[0].value, Value::String("Ada".to_string()));
        assert_eq!(first.entries()[1].key, ArrayKey::Int(1));
        assert_eq!(first.entries()[1].value, Value::String("five".to_string()));

        assert_eq!(entries[1].key, ArrayKey::Int(1));
        let Value::Array(second) = &entries[1].value else {
            panic!("second chunk is an array");
        };
        assert_eq!(second.entries().len(), 2);
        assert_eq!(second.entries()[0].key, ArrayKey::Int(0));
        assert_eq!(second.entries()[0].value, Value::String("two".to_string()));
        assert_eq!(second.entries()[1].key, ArrayKey::Int(1));
        assert_eq!(
            second.entries()[1].value,
            Value::String("zero two".to_string())
        );

        assert_eq!(entries[2].key, ArrayKey::Int(2));
        let Value::Array(third) = &entries[2].value else {
            panic!("third chunk is an array");
        };
        assert_eq!(third.entries().len(), 1);
        assert_eq!(third.entries()[0].key, ArrayKey::Int(0));
        assert_eq!(third.entries()[0].value, Value::String("next".to_string()));

        assert_eq!(
            chunks.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            array.get("02"),
            Some(&Value::String("zero two".to_string())),
            "array_chunk must not mutate the original array"
        );
        assert!(PhpArray::new().chunked_reindexed(2).entries().is_empty());
    }

    #[test]
    fn array_chunk_can_preserve_integer_and_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let chunks = array.chunked_preserving_keys(2);
        let entries = chunks.entries();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        let Value::Array(first) = &entries[0].value else {
            panic!("first chunk is an array");
        };
        assert_eq!(first.entries().len(), 2);
        assert_eq!(first.entries()[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(first.entries()[0].value, Value::String("Ada".to_string()));
        assert_eq!(first.entries()[1].key, ArrayKey::Int(5));
        assert_eq!(first.entries()[1].value, Value::String("five".to_string()));

        let Value::Array(second) = &entries[1].value else {
            panic!("second chunk is an array");
        };
        assert_eq!(second.entries().len(), 2);
        assert_eq!(second.entries()[0].key, ArrayKey::Int(2));
        assert_eq!(second.entries()[0].value, Value::String("two".to_string()));
        assert_eq!(second.entries()[1].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            second.entries()[1].value,
            Value::String("zero two".to_string())
        );

        let Value::Array(third) = &entries[2].value else {
            panic!("third chunk is an array");
        };
        assert_eq!(third.entries().len(), 2);
        assert_eq!(third.entries()[0].key, ArrayKey::Int(-1));
        assert_eq!(
            third.entries()[0].value,
            Value::String("negative".to_string())
        );
        assert_eq!(third.entries()[1].key, ArrayKey::Int(6));
        assert_eq!(third.entries()[1].value, Value::String("next".to_string()));

        assert_eq!(
            array.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_chunk preserve_keys must not mutate the original array"
        );
        assert!(PhpArray::new()
            .chunked_preserving_keys(2)
            .entries()
            .is_empty());
    }

    #[test]
    fn array_pad_right_and_left_reindexes_integer_keys_and_preserves_string_keys() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut right = array
            .padded(8, Value::String("pad".to_string()))
            .expect("right padding succeeds");
        let entries = right.entries();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(0));
        assert_eq!(entries[1].value, Value::String("five".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(1));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[3].value, Value::String("zero two".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(2));
        assert_eq!(entries[4].value, Value::String("negative".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(3));
        assert_eq!(entries[5].value, Value::String("next".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(4));
        assert_eq!(entries[6].value, Value::String("pad".to_string()));
        assert_eq!(entries[7].key, ArrayKey::Int(5));
        assert_eq!(entries[7].value, Value::String("pad".to_string()));
        assert_eq!(
            right.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(6)
        );

        let mut left = array
            .padded(-8, Value::String("pad".to_string()))
            .expect("left padding succeeds");
        let entries = left.entries();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].key, ArrayKey::Int(0));
        assert_eq!(entries[0].value, Value::String("pad".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("pad".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[2].value, Value::String("Ada".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("five".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(3));
        assert_eq!(entries[4].value, Value::String("two".to_string()));
        assert_eq!(entries[5].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[5].value, Value::String("zero two".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(4));
        assert_eq!(entries[6].value, Value::String("negative".to_string()));
        assert_eq!(entries[7].key, ArrayKey::Int(5));
        assert_eq!(entries[7].value, Value::String("next".to_string()));
        assert_eq!(
            left.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(6)
        );

        assert_eq!(
            array.get(5),
            Some(&Value::String("five".to_string())),
            "array_pad must not mutate the original array"
        );
    }

    #[test]
    fn array_pad_noop_preserves_original_array_shape_and_append_index() {
        let mut array = PhpArray::new();

        array.insert("name", Value::String("Ada".to_string()));
        array.insert(5, Value::String("five".to_string()));
        array.insert("2", Value::String("two".to_string()));
        array.insert("02", Value::String("zero two".to_string()));
        array.insert(-1, Value::String("negative".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut noop = array
            .padded(3, Value::String("pad".to_string()))
            .expect("no-op padding succeeds");
        assert_eq!(noop.entries(), array.entries());
        assert_eq!(
            noop.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(7)
        );

        let empty = PhpArray::new()
            .padded(0, Value::String("pad".to_string()))
            .expect("zero-length empty padding succeeds");
        assert!(empty.entries().is_empty());
    }

    #[test]
    fn array_pad_rejects_padding_larger_than_current_limit() {
        let error = PhpArray::new()
            .padded(1_048_577, Value::String("pad".to_string()))
            .expect_err("padding over the current limit fails");

        assert_eq!(
            error.message(),
            "unsupported call array_pad(): padding length must be at most 1048576 in the current subset, got 1048577"
        );
    }

    #[test]
    fn array_merge_reindexes_integer_keys_and_overwrites_string_keys() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.append(Value::String("left next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.insert("name", Value::String("Bea".to_string()));
        right.insert(7, Value::String("seven".to_string()));
        right.insert("02", Value::String("zero two right".to_string()));
        right
            .append(Value::String("right next".to_string()))
            .unwrap();
        right.insert("extra", Value::String("extra".to_string()));

        let mut merged = left.merged_with(&right);
        let entries = merged.entries();

        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Bea".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(0));
        assert_eq!(entries[1].value, Value::String("five".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(1));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            entries[3].value,
            Value::String("zero two right".to_string())
        );
        assert_eq!(entries[4].key, ArrayKey::Int(2));
        assert_eq!(entries[4].value, Value::String("left next".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(3));
        assert_eq!(entries[5].value, Value::String("seven".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(4));
        assert_eq!(entries[6].value, Value::String("right next".to_string()));
        assert_eq!(entries[7].key, ArrayKey::String("extra".to_string()));
        assert_eq!(entries[7].value, Value::String("extra".to_string()));
        assert_eq!(
            merged.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(5)
        );
        assert_eq!(
            left.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_merge must not mutate the left array"
        );
        assert_eq!(
            right.get("02"),
            Some(&Value::String("zero two right".to_string())),
            "array_merge must not mutate the right array"
        );
    }

    #[test]
    fn array_merge_accepts_zero_one_and_variadic_arrays() {
        let empty = PhpArray::merged_from(std::iter::empty::<&PhpArray>());
        assert!(empty.entries().is_empty());

        let mut one = PhpArray::new();
        one.insert("name", Value::String("Ada".to_string()));
        one.insert(5, Value::String("five".to_string()));
        one.insert("2", Value::String("two".to_string()));
        one.insert("02", Value::String("zero two".to_string()));

        let single = PhpArray::merged_from([&one]);
        let single_entries = single.entries();
        assert_eq!(single_entries.len(), 4);
        assert_eq!(single_entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(single_entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(single_entries[1].key, ArrayKey::Int(0));
        assert_eq!(single_entries[1].value, Value::String("five".to_string()));
        assert_eq!(single_entries[2].key, ArrayKey::Int(1));
        assert_eq!(single_entries[2].value, Value::String("two".to_string()));
        assert_eq!(single_entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            single_entries[3].value,
            Value::String("zero two".to_string())
        );

        let mut two = PhpArray::new();
        two.insert("name", Value::String("Bea".to_string()));
        two.insert(7, Value::String("seven".to_string()));
        two.insert("extra", Value::String("two extra".to_string()));

        let mut three = PhpArray::new();
        three.insert("name", Value::String("Cy".to_string()));
        three.insert(11, Value::String("eleven".to_string()));
        three.insert("extra", Value::String("three extra".to_string()));

        let merged = PhpArray::merged_from([&one, &two, &three]);
        let entries = merged.entries();

        assert_eq!(entries.len(), 7);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Cy".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(0));
        assert_eq!(entries[1].value, Value::String("five".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(1));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[3].value, Value::String("zero two".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(2));
        assert_eq!(entries[4].value, Value::String("seven".to_string()));
        assert_eq!(entries[5].key, ArrayKey::String("extra".to_string()));
        assert_eq!(entries[5].value, Value::String("three extra".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(3));
        assert_eq!(entries[6].value, Value::String("eleven".to_string()));
    }

    #[test]
    fn array_replace_overwrites_matching_keys_and_appends_new_keys() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.append(Value::String("left next".to_string())).unwrap();

        let mut replacement = PhpArray::new();
        replacement.insert("name", Value::String("Bea".to_string()));
        replacement.insert("5", Value::String("five right".to_string()));
        replacement.insert(7, Value::String("seven".to_string()));
        replacement.insert("02", Value::String("zero two right".to_string()));
        replacement
            .append(Value::String("right next".to_string()))
            .unwrap();
        replacement.insert("extra", Value::String("extra".to_string()));

        let mut replaced = left.replaced_with(&replacement);
        let entries = replaced.entries();

        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Bea".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(5));
        assert_eq!(entries[1].value, Value::String("five right".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            entries[3].value,
            Value::String("zero two right".to_string())
        );
        assert_eq!(entries[4].key, ArrayKey::Int(6));
        assert_eq!(entries[4].value, Value::String("left next".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(7));
        assert_eq!(entries[5].value, Value::String("seven".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(8));
        assert_eq!(entries[6].value, Value::String("right next".to_string()));
        assert_eq!(entries[7].key, ArrayKey::String("extra".to_string()));
        assert_eq!(entries[7].value, Value::String("extra".to_string()));
        assert_eq!(
            replaced.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(9)
        );
        assert_eq!(
            left.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_replace must not mutate the left array"
        );
        assert_eq!(
            replacement.get(5),
            Some(&Value::String("five right".to_string())),
            "array_replace must not mutate the replacement array"
        );
    }

    #[test]
    fn array_replace_accepts_one_array_and_variadic_replacements() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.append(Value::String("left next".to_string())).unwrap();
        left.insert("keep", Value::String("keep".to_string()));

        let single = left.replaced_with_all(std::iter::empty::<&PhpArray>());
        assert_eq!(single.entries(), left.entries());

        let mut first = PhpArray::new();
        first.insert("name", Value::String("Bea".to_string()));
        first.insert(7, Value::String("seven".to_string()));
        first.insert("keep", Value::String("first keep".to_string()));
        first
            .append(Value::String("first next".to_string()))
            .unwrap();

        let mut second = PhpArray::new();
        second.insert("name", Value::String("Cy".to_string()));
        second.insert("7", Value::String("seven second".to_string()));
        second.insert(9, Value::String("nine".to_string()));
        second.insert("extra", Value::String("extra".to_string()));
        second.insert(5, Value::String("five second".to_string()));

        let mut third = PhpArray::new();
        third.insert("name", Value::String("Di".to_string()));
        third.insert("extra", Value::String("extra third".to_string()));
        third
            .append(Value::String("third zero".to_string()))
            .unwrap();
        third.insert(10, Value::String("ten".to_string()));

        let mut replaced = left.replaced_with_all([&first, &second, &third]);
        let entries = replaced.entries();

        assert_eq!(entries.len(), 11);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Di".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(5));
        assert_eq!(entries[1].value, Value::String("five second".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(6));
        assert_eq!(entries[3].value, Value::String("left next".to_string()));
        assert_eq!(entries[4].key, ArrayKey::String("keep".to_string()));
        assert_eq!(entries[4].value, Value::String("first keep".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(7));
        assert_eq!(entries[5].value, Value::String("seven second".to_string()));
        assert_eq!(entries[6].key, ArrayKey::Int(8));
        assert_eq!(entries[6].value, Value::String("first next".to_string()));
        assert_eq!(entries[7].key, ArrayKey::Int(9));
        assert_eq!(entries[7].value, Value::String("nine".to_string()));
        assert_eq!(entries[8].key, ArrayKey::String("extra".to_string()));
        assert_eq!(entries[8].value, Value::String("extra third".to_string()));
        assert_eq!(entries[9].key, ArrayKey::Int(0));
        assert_eq!(entries[9].value, Value::String("third zero".to_string()));
        assert_eq!(entries[10].key, ArrayKey::Int(10));
        assert_eq!(entries[10].value, Value::String("ten".to_string()));
        assert_eq!(
            replaced.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(11)
        );
        assert_eq!(
            first.get("name"),
            Some(&Value::String("Bea".to_string())),
            "array_replace must not mutate replacement arrays"
        );
        assert_eq!(
            third.get("extra"),
            Some(&Value::String("extra third".to_string())),
            "array_replace must not mutate later replacement arrays"
        );
    }

    #[test]
    fn array_flip_uses_int_string_values_as_keys_and_overwrites_duplicates() {
        let mut array = PhpArray::new();

        array.insert("first", Value::String("name".to_string()));
        array.insert(5, Value::String("2".to_string()));
        array.insert("two", Value::Int(2));
        array.insert("02", Value::String("02".to_string()));
        array.append(Value::Int(-1)).unwrap();
        array.insert("dup-string", Value::String("name".to_string()));

        let mut flipped = array.flipped().unwrap();
        let entries = flipped.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("dup-string".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("02".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(-1));
        assert_eq!(entries[3].value, Value::Int(6));
        assert_eq!(
            flipped.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            array.get("dup-string"),
            Some(&Value::String("name".to_string())),
            "array_flip must not mutate the original array"
        );
    }

    #[test]
    fn array_flip_rejects_unsupported_value_types() {
        let mut array = PhpArray::new();
        array.insert("ok", Value::String("name".to_string()));
        array.insert("bad", Value::Bool(true));

        let error = array.flipped().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_flip()".to_string(),
                reason: "values must be int or string in the current subset, got bool".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_flip(): values must be int or string in the current subset, got bool"
        );
    }

    #[test]
    fn array_change_key_case_changes_string_keys_and_preserves_integer_keys() {
        let mut array = PhpArray::new();
        array.insert("Name", Value::String("Ada".to_string()));
        array.insert("name", Value::String("lower".to_string()));
        array.insert(7, Value::String("seven".to_string()));
        array.insert("MiXeD", Value::String("mixed".to_string()));
        array.insert("02", Value::String("numeric string".to_string()));

        let mut lower = array.keys_with_ascii_case(ArrayKeyCase::Lower);
        let entries = lower.entries();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("lower".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(7));
        assert_eq!(entries[1].value, Value::String("seven".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("mixed".to_string()));
        assert_eq!(entries[2].value, Value::String("mixed".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            entries[3].value,
            Value::String("numeric string".to_string())
        );
        assert_eq!(
            lower.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(8)
        );

        let upper = array.keys_with_ascii_case(ArrayKeyCase::Upper);
        let entries = upper.entries();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("NAME".to_string()));
        assert_eq!(entries[0].value, Value::String("lower".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(7));
        assert_eq!(entries[1].value, Value::String("seven".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("MIXED".to_string()));
        assert_eq!(entries[2].value, Value::String("mixed".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(
            array.get("Name"),
            Some(&Value::String("Ada".to_string())),
            "array_change_key_case must not mutate the original array"
        );
    }

    #[test]
    fn array_column_can_index_results_by_int_and_string_row_values() {
        let mut first = PhpArray::new();
        first.insert("id", Value::Int(10));
        first.insert("name", Value::String("Ada".to_string()));

        let mut duplicate = PhpArray::new();
        duplicate.insert("id", Value::String("10".to_string()));
        duplicate.insert("name", Value::String("Grace".to_string()));

        let mut missing_index = PhpArray::new();
        missing_index.insert("name", Value::String("NoId".to_string()));

        let mut string_index = PhpArray::new();
        string_index.insert("id", Value::String("code".to_string()));
        string_index.insert("name", Value::Null);

        let mut rows = PhpArray::new();
        rows.append(Value::Array(first)).unwrap();
        rows.append(Value::Array(duplicate)).unwrap();
        rows.append(Value::Array(missing_index)).unwrap();
        rows.append(Value::Array(string_index)).unwrap();

        let result = rows
            .column_values(
                Some(ArrayColumnKey::String("name".to_string())),
                Some(ArrayColumnKey::String("id".to_string())),
            )
            .unwrap();

        let entries = result.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::Int(10));
        assert_eq!(entries[0].value, Value::String("Grace".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(11));
        assert_eq!(entries[1].value, Value::String("NoId".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("code".to_string()));
        assert_eq!(entries[2].value, Value::Null);
    }

    #[test]
    fn array_column_can_index_results_by_scalar_coerced_row_values() {
        let mut true_index = PhpArray::new();
        true_index.insert("id", Value::Bool(true));
        true_index.insert("name", Value::String("true".to_string()));

        let mut false_index = PhpArray::new();
        false_index.insert("id", Value::Bool(false));
        false_index.insert("name", Value::String("false".to_string()));

        let mut null_index = PhpArray::new();
        null_index.insert("id", Value::Null);
        null_index.insert("name", Value::String("null".to_string()));

        let mut float_index = PhpArray::new();
        float_index.insert("id", Value::Float(1.0));
        float_index.insert("name", Value::String("float".to_string()));

        let mut missing_index = PhpArray::new();
        missing_index.insert("name", Value::String("missing".to_string()));

        let mut rows = PhpArray::new();
        rows.append(Value::Array(true_index)).unwrap();
        rows.append(Value::Array(false_index)).unwrap();
        rows.append(Value::Array(null_index)).unwrap();
        rows.append(Value::Array(float_index)).unwrap();
        rows.append(Value::Array(missing_index)).unwrap();

        let result = rows
            .column_values(
                Some(ArrayColumnKey::String("name".to_string())),
                Some(ArrayColumnKey::String("id".to_string())),
            )
            .unwrap();

        let entries = result.entries();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::Int(1));
        assert_eq!(entries[0].value, Value::String("float".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(0));
        assert_eq!(entries[1].value, Value::String("false".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String(String::new()));
        assert_eq!(entries[2].value, Value::String("null".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("missing".to_string()));
    }

    #[test]
    fn array_column_rejects_unsupported_index_values() {
        let mut row = PhpArray::new();
        row.insert("id", Value::Float(1.5));
        row.insert("name", Value::String("Ada".to_string()));

        let mut rows = PhpArray::new();
        rows.append(Value::Array(row)).unwrap();

        let error = rows
            .column_values(
                Some(ArrayColumnKey::String("name".to_string())),
                Some(ArrayColumnKey::String("id".to_string())),
            )
            .unwrap_err();

        assert_eq!(
            error.message(),
            "unsupported call array_column(): lossy or non-finite float index values are not supported; only null, bool, int, string, and integral finite float index values are implemented"
        );
    }

    #[test]
    fn array_fill_keys_uses_int_string_values_as_keys_and_overwrites_duplicates() {
        let mut keys = PhpArray::new();

        keys.insert("first", Value::String("name".to_string()));
        keys.insert(5, Value::String("2".to_string()));
        keys.insert("two", Value::Int(2));
        keys.insert("02", Value::String("02".to_string()));
        keys.append(Value::Int(-1)).unwrap();
        keys.insert("dup-string", Value::String("name".to_string()));

        let mut filled = keys
            .filled_keys(Value::String("filled".to_string()))
            .unwrap();
        let entries = filled.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("filled".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("filled".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("filled".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(-1));
        assert_eq!(entries[3].value, Value::String("filled".to_string()));
        assert_eq!(
            filled.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            keys.get("dup-string"),
            Some(&Value::String("name".to_string())),
            "array_fill_keys must not mutate the original key array"
        );
    }

    #[test]
    fn array_fill_keys_uses_scalar_values_as_keys_and_overwrites_duplicates() {
        let mut keys = PhpArray::new();
        keys.insert("null", Value::Null);
        keys.insert("false", Value::Bool(false));
        keys.insert("true", Value::Bool(true));
        keys.insert("one", Value::Float(1.0));
        keys.insert("two", Value::Float(2.0));
        keys.insert("two-string", Value::String("2".to_string()));
        keys.insert("02", Value::String("02".to_string()));
        keys.insert("minus", Value::Float(-3.0));

        let filled = keys
            .filled_keys(Value::String("filled".to_string()))
            .unwrap();
        let entries = filled.entries();

        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].key, ArrayKey::String(String::new()));
        assert_eq!(entries[0].value, Value::String("filled".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("filled".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::String("filled".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[3].value, Value::String("filled".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(-3));
        assert_eq!(entries[4].value, Value::String("filled".to_string()));
    }

    #[test]
    fn array_fill_keys_rejects_unsupported_key_value_types() {
        let mut keys = PhpArray::new();
        keys.insert("ok", Value::String("name".to_string()));
        keys.insert("bad", Value::Array(PhpArray::new()));

        let error = keys
            .filled_keys(Value::String("filled".to_string()))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_fill_keys()".to_string(),
                reason:
                    "key values must be null, bool, int, string, or integral finite float in the current subset, got array"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_fill_keys(): key values must be null, bool, int, string, or integral finite float in the current subset, got array"
        );

        let mut keys = PhpArray::new();
        keys.append(Value::Float(1.5)).unwrap();
        let error = keys
            .filled_keys(Value::String("filled".to_string()))
            .unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_fill_keys(): lossy or non-finite float key values are not supported; only null, bool, int, string, and integral finite float key values are implemented"
        );
    }

    #[test]
    fn array_combine_uses_scalar_key_values_and_overwrites_duplicates() {
        let mut keys = PhpArray::new();
        keys.insert("empty", Value::Null);
        keys.insert("false", Value::Bool(false));
        keys.insert("true", Value::Bool(true));
        keys.insert("first", Value::String("name".to_string()));
        keys.insert(5, Value::String("2".to_string()));
        keys.insert("two", Value::Int(2));
        keys.insert("02", Value::String("02".to_string()));
        keys.append(Value::Int(-1)).unwrap();
        keys.insert("dup-string", Value::String("name".to_string()));

        let mut values = PhpArray::new();
        values.insert("empty", Value::String("null key".to_string()));
        values.insert("false", Value::String("false key".to_string()));
        values.insert("true", Value::String("true key".to_string()));
        values.insert("a", Value::String("Ada".to_string()));
        values.insert(10, Value::String("two string".to_string()));
        values.append(Value::String("two int".to_string())).unwrap();
        values.insert("d", Value::String("zero two".to_string()));
        values.insert(-3, Value::String("negative".to_string()));
        values
            .append(Value::String("duplicate".to_string()))
            .unwrap();

        let mut combined = keys.combined_with(&values).unwrap();
        let entries = combined.entries();

        assert_eq!(entries.len(), 6);
        assert_eq!(entries[0].key, ArrayKey::String(String::new()));
        assert_eq!(entries[0].value, Value::String("false key".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(1));
        assert_eq!(entries[1].value, Value::String("true key".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[2].value, Value::String("duplicate".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(2));
        assert_eq!(entries[3].value, Value::String("two int".to_string()));
        assert_eq!(entries[4].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[4].value, Value::String("zero two".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(-1));
        assert_eq!(entries[5].value, Value::String("negative".to_string()));
        assert_eq!(
            combined.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            keys.get("dup-string"),
            Some(&Value::String("name".to_string())),
            "array_combine must not mutate the key array"
        );
        assert_eq!(
            values.get(12),
            Some(&Value::String("duplicate".to_string())),
            "array_combine must not mutate the value array"
        );

        assert!(PhpArray::new()
            .combined_with(&PhpArray::new())
            .unwrap()
            .entries()
            .is_empty());
    }

    #[test]
    fn array_combine_accepts_integral_finite_float_key_values() {
        let mut keys = PhpArray::new();
        keys.append(Value::Float(1.0)).unwrap();
        keys.append(Value::Float(2.0)).unwrap();
        keys.append(Value::Float(-3.0)).unwrap();
        keys.append(Value::String("04".to_string())).unwrap();

        let mut values = PhpArray::new();
        values.append(Value::String("one".to_string())).unwrap();
        values.append(Value::String("two".to_string())).unwrap();
        values.append(Value::String("minus".to_string())).unwrap();
        values.append(Value::String("leading".to_string())).unwrap();

        let combined = keys.combined_with(&values).unwrap();
        let entries = combined.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::Int(1));
        assert_eq!(entries[0].value, Value::String("one".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(-3));
        assert_eq!(entries[2].value, Value::String("minus".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("04".to_string()));
        assert_eq!(entries[3].value, Value::String("leading".to_string()));
    }

    #[test]
    fn array_combine_rejects_length_mismatches_and_unsupported_key_value_types() {
        let mut keys = PhpArray::new();
        keys.append(Value::String("name".to_string())).unwrap();
        keys.append(Value::String("extra".to_string())).unwrap();

        let mut values = PhpArray::new();
        values.append(Value::String("Ada".to_string())).unwrap();

        let error = keys.combined_with(&values).unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_combine(): keys and values must have the same number of elements in the current subset, got 2 and 1"
        );

        let mut bad_keys = PhpArray::new();
        bad_keys.append(Value::String("name".to_string())).unwrap();
        bad_keys.append(Value::Array(PhpArray::new())).unwrap();

        let mut values = PhpArray::new();
        values.append(Value::String("Ada".to_string())).unwrap();
        values.append(Value::String("bad".to_string())).unwrap();

        let error = bad_keys.combined_with(&values).unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_combine()".to_string(),
                reason:
                    "key values must be null, bool, int, string, or integral finite float in the current subset, got array"
                        .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_combine(): key values must be null, bool, int, string, or integral finite float in the current subset, got array"
        );

        let mut bad_keys = PhpArray::new();
        bad_keys.append(Value::Float(1.5)).unwrap();
        let mut values = PhpArray::new();
        values.append(Value::String("lossy".to_string())).unwrap();

        let error = bad_keys.combined_with(&values).unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_combine(): lossy or non-finite float key values are not supported; only null, bool, int, string, and integral finite float key values are implemented"
        );
    }

    #[test]
    fn array_intersect_key_preserves_left_entries_with_matching_right_keys() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.insert(-1, Value::String("negative".to_string()));
        left.insert("drop", Value::String("drop".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.insert("name", Value::String("ignored".to_string()));
        right.insert("5", Value::String("ignored".to_string()));
        right.insert(2, Value::String("ignored".to_string()));
        right.insert("02", Value::String("ignored".to_string()));
        right.insert(-1, Value::String("ignored".to_string()));
        right.insert("extra", Value::String("ignored".to_string()));

        let intersected = left.intersect_keys_with(&right);
        let entries = intersected.entries();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(5));
        assert_eq!(entries[1].value, Value::String("five".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(2));
        assert_eq!(entries[2].value, Value::String("two".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[3].value, Value::String("zero two".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(-1));
        assert_eq!(entries[4].value, Value::String("negative".to_string()));
        assert!(!intersected.contains_key("drop"));
        assert!(!intersected.contains_key(6));

        let mut appended = intersected.clone();
        assert_eq!(
            appended.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(6)
        );
        assert_eq!(
            left.get("drop"),
            Some(&Value::String("drop".to_string())),
            "array_intersect_key must not mutate the left array"
        );
        assert!(
            right.contains_key("extra"),
            "array_intersect_key must not mutate the right array"
        );
    }

    #[test]
    fn array_intersect_key_accepts_variadic_arrays() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.insert(-1, Value::String("negative".to_string()));
        left.insert("drop", Value::String("drop".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.insert("name", Value::String("ignored".to_string()));
        right.insert("5", Value::String("ignored".to_string()));
        right.insert(2, Value::String("ignored".to_string()));
        right.insert("02", Value::String("ignored".to_string()));
        right.insert(-1, Value::String("ignored".to_string()));

        let mut third = PhpArray::new();
        third.insert("name", Value::String("third".to_string()));
        third.insert("2", Value::String("third".to_string()));
        third.insert("02", Value::String("third".to_string()));
        third.insert("drop", Value::String("third".to_string()));

        let mut fourth = PhpArray::new();
        fourth.insert("name", Value::String("fourth".to_string()));
        fourth.insert(2, Value::String("fourth".to_string()));
        fourth.insert("02", Value::String("fourth".to_string()));
        fourth.insert(-1, Value::String("fourth".to_string()));

        let mut intersected = left.intersect_keys_with_all([&right, &third, &fourth]);
        let entries = intersected.entries();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::String("zero two".to_string()));
        assert_eq!(
            intersected
                .append(Value::String("after".to_string()))
                .unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            left.get("drop"),
            Some(&Value::String("drop".to_string())),
            "array_intersect_key must not mutate the left array"
        );
        assert!(
            third.contains_key("drop"),
            "array_intersect_key must not mutate variadic operands"
        );
        assert!(
            fourth.contains_key(-1),
            "array_intersect_key must not mutate later variadic operands"
        );
    }

    #[test]
    fn array_diff_key_preserves_left_entries_missing_from_right_keys() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.insert(-1, Value::String("negative".to_string()));
        left.insert("drop", Value::String("drop".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.insert("name", Value::String("ignored".to_string()));
        right.insert("5", Value::String("ignored".to_string()));
        right.insert(2, Value::String("ignored".to_string()));
        right.insert(-1, Value::String("ignored".to_string()));
        right.insert("extra", Value::String("ignored".to_string()));

        let diffed = left.diff_keys_with(&right);
        let entries = diffed.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[0].value, Value::String("zero two".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("drop".to_string()));
        assert_eq!(entries[1].value, Value::String("drop".to_string()));
        assert_eq!(entries[2].key, ArrayKey::Int(6));
        assert_eq!(entries[2].value, Value::String("next".to_string()));
        assert!(!diffed.contains_key("name"));
        assert!(!diffed.contains_key(5));
        assert!(!diffed.contains_key(2));
        assert!(!diffed.contains_key(-1));

        let mut appended = diffed.clone();
        assert_eq!(
            appended.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(7)
        );
        assert_eq!(
            left.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_diff_key must not mutate the left array"
        );
        assert!(
            right.contains_key("extra"),
            "array_diff_key must not mutate the right array"
        );
    }

    #[test]
    fn array_diff_key_can_compare_against_variadic_arrays() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(5, Value::String("five".to_string()));
        left.insert("2", Value::String("two".to_string()));
        left.insert("02", Value::String("zero two".to_string()));
        left.insert(-1, Value::String("negative".to_string()));
        left.insert("drop", Value::String("drop".to_string()));
        left.insert(8, Value::String("eight".to_string()));
        left.insert("keep", Value::String("keep".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.insert("name", Value::Bool(true));
        right.insert("5", Value::Bool(true));
        right.insert(2, Value::Bool(true));
        right.insert(-1, Value::Bool(true));

        let mut third = PhpArray::new();
        third.insert("02", Value::Bool(true));
        third.insert("drop", Value::Bool(true));

        let mut fourth = PhpArray::new();
        fourth.insert(9, Value::Bool(true));

        let mut diffed = left.diff_keys_with_all([&right, &third, &fourth]);
        let entries = diffed.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, ArrayKey::Int(8));
        assert_eq!(entries[0].value, Value::String("eight".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("keep".to_string()));
        assert_eq!(entries[1].value, Value::String("keep".to_string()));
        assert_eq!(
            diffed.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(9)
        );
        assert_eq!(
            left.get("drop"),
            Some(&Value::String("drop".to_string())),
            "array_diff_key must not mutate the left array"
        );
        assert!(
            third.contains_key("drop"),
            "array_diff_key must not mutate variadic operands"
        );
        assert!(
            fourth.contains_key(9),
            "array_diff_key must not mutate later variadic operands"
        );
    }

    #[test]
    fn array_diff_preserves_left_entries_whose_scalar_strings_are_absent_from_right() {
        let mut left = PhpArray::new();
        left.insert("null", Value::Null);
        left.insert("false", Value::Bool(false));
        left.insert("empty", Value::String(String::new()));
        left.insert("true", Value::Bool(true));
        left.insert("one", Value::Int(1));
        left.insert("zero", Value::Int(0));
        left.insert("string-zero", Value::String("0".to_string()));
        left.insert("int-ten", Value::Int(10));
        left.insert("float-ten", Value::Float(10.0));
        left.insert("string-ten-float", Value::String("10.0".to_string()));
        left.insert("text", Value::String("abc".to_string()));
        left.insert(8, Value::String("eight".to_string()));
        left.insert("keep", Value::String("keep".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.append(Value::String(String::new())).unwrap();
        right.append(Value::String("0".to_string())).unwrap();
        right.append(Value::String("1".to_string())).unwrap();
        right.append(Value::String("10".to_string())).unwrap();
        right.append(Value::String("abc".to_string())).unwrap();
        right.append(Value::String("missing".to_string())).unwrap();

        let mut diffed = left.diff_values_with(&right).unwrap();
        let entries = diffed.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(
            entries[0].key,
            ArrayKey::String("string-ten-float".to_string())
        );
        assert_eq!(entries[0].value, Value::String("10.0".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(8));
        assert_eq!(entries[1].value, Value::String("eight".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("keep".to_string()));
        assert_eq!(entries[2].value, Value::String("keep".to_string()));
        assert_eq!(entries[3].key, ArrayKey::Int(9));
        assert_eq!(entries[3].value, Value::String("next".to_string()));
        assert_eq!(
            diffed.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(10)
        );
        assert_eq!(
            left.get("null"),
            Some(&Value::Null),
            "array_diff must not mutate the left array"
        );
        assert_eq!(
            right.get(5),
            Some(&Value::String("missing".to_string())),
            "array_diff must not mutate the right array"
        );
    }

    #[test]
    fn array_diff_accepts_variadic_arrays() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(1, Value::String("1".to_string()));
        left.insert("two", Value::String("two".to_string()));
        left.insert("ten", Value::Int(10));
        left.insert("float-ten", Value::Float(10.0));
        left.insert("drop", Value::String("drop".to_string()));
        left.insert(8, Value::String("eight".to_string()));
        left.insert("keep", Value::String("keep".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut first = PhpArray::new();
        first.append(Value::String("Ada".to_string())).unwrap();
        first.append(Value::String("1".to_string())).unwrap();
        first.append(Value::String("10".to_string())).unwrap();
        first.append(Value::String("extra".to_string())).unwrap();

        let mut second = PhpArray::new();
        second.append(Value::String("drop".to_string())).unwrap();
        second.append(Value::String("eight".to_string())).unwrap();

        let mut third = PhpArray::new();
        third.append(Value::String("two".to_string())).unwrap();

        let mut diffed = left
            .diff_values_with_all([&first, &second, &third])
            .unwrap();
        let entries = diffed.entries();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].key, ArrayKey::String("keep".to_string()));
        assert_eq!(entries[0].value, Value::String("keep".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(9));
        assert_eq!(entries[1].value, Value::String("next".to_string()));
        assert_eq!(
            diffed.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(10)
        );
        assert_eq!(
            left.get("name"),
            Some(&Value::String("Ada".to_string())),
            "array_diff must not mutate the left array"
        );
        assert_eq!(
            first.get(3),
            Some(&Value::String("extra".to_string())),
            "array_diff must not mutate variadic operands"
        );
        assert_eq!(
            second.get(0),
            Some(&Value::String("drop".to_string())),
            "array_diff must not mutate later variadic operands"
        );
        assert_eq!(
            third.get(0),
            Some(&Value::String("two".to_string())),
            "array_diff must not mutate final variadic operands"
        );
    }

    #[test]
    fn array_diff_rejects_non_scalar_value_comparisons() {
        let mut left = PhpArray::new();
        left.insert("nested", Value::Array(PhpArray::new()));

        let right = PhpArray::new();
        let error = left.diff_values_with(&right).unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_diff()".to_string(),
                reason: "values must be scalar in the current subset, got array".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_diff(): values must be scalar in the current subset, got array"
        );

        let mut classes = PhpClassTable::new();
        let class_id = classes.declare_class("Box").unwrap();
        let object = Value::Object(PhpObject::from_class(classes.get(class_id).unwrap()));
        let mut right = PhpArray::new();
        right.insert("object", object);

        let mut left = PhpArray::new();
        left.insert("value", Value::String("value".to_string()));
        let error = left.diff_values_with(&right).unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_diff()".to_string(),
                reason: "values must be scalar in the current subset, got object".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_diff(): values must be scalar in the current subset, got object"
        );
    }

    #[test]
    fn array_intersect_preserves_left_entries_whose_scalar_strings_are_present_in_right() {
        let mut left = PhpArray::new();
        left.insert("null", Value::Null);
        left.insert("false", Value::Bool(false));
        left.insert("empty", Value::String(String::new()));
        left.insert("true", Value::Bool(true));
        left.insert("one", Value::Int(1));
        left.insert("zero", Value::Int(0));
        left.insert("string-zero", Value::String("0".to_string()));
        left.insert("int-ten", Value::Int(10));
        left.insert("float-ten", Value::Float(10.0));
        left.insert("string-ten-float", Value::String("10.0".to_string()));
        left.insert("text", Value::String("abc".to_string()));
        left.insert(8, Value::String("eight".to_string()));
        left.insert("drop", Value::String("drop".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut right = PhpArray::new();
        right.append(Value::String(String::new())).unwrap();
        right.append(Value::String("0".to_string())).unwrap();
        right.append(Value::String("1".to_string())).unwrap();
        right.append(Value::String("10".to_string())).unwrap();
        right.append(Value::String("abc".to_string())).unwrap();
        right.append(Value::String("eight".to_string())).unwrap();
        right.append(Value::String("missing".to_string())).unwrap();

        let mut intersected = left.intersect_values_with(&right).unwrap();
        let entries = intersected.entries();

        assert_eq!(entries.len(), 11);
        assert_eq!(entries[0].key, ArrayKey::String("null".to_string()));
        assert_eq!(entries[0].value, Value::Null);
        assert_eq!(entries[1].key, ArrayKey::String("false".to_string()));
        assert_eq!(entries[1].value, Value::Bool(false));
        assert_eq!(entries[2].key, ArrayKey::String("empty".to_string()));
        assert_eq!(entries[2].value, Value::String(String::new()));
        assert_eq!(entries[3].key, ArrayKey::String("true".to_string()));
        assert_eq!(entries[3].value, Value::Bool(true));
        assert_eq!(entries[4].key, ArrayKey::String("one".to_string()));
        assert_eq!(entries[4].value, Value::Int(1));
        assert_eq!(entries[5].key, ArrayKey::String("zero".to_string()));
        assert_eq!(entries[5].value, Value::Int(0));
        assert_eq!(entries[6].key, ArrayKey::String("string-zero".to_string()));
        assert_eq!(entries[6].value, Value::String("0".to_string()));
        assert_eq!(entries[7].key, ArrayKey::String("int-ten".to_string()));
        assert_eq!(entries[7].value, Value::Int(10));
        assert_eq!(entries[8].key, ArrayKey::String("float-ten".to_string()));
        assert_eq!(entries[8].value, Value::Float(10.0));
        assert_eq!(entries[9].key, ArrayKey::String("text".to_string()));
        assert_eq!(entries[9].value, Value::String("abc".to_string()));
        assert_eq!(entries[10].key, ArrayKey::Int(8));
        assert_eq!(entries[10].value, Value::String("eight".to_string()));
        assert_eq!(
            intersected
                .append(Value::String("after".to_string()))
                .unwrap(),
            ArrayKey::Int(10)
        );
        assert_eq!(
            left.get("drop"),
            Some(&Value::String("drop".to_string())),
            "array_intersect must not mutate the left array"
        );
        assert_eq!(
            right.get(6),
            Some(&Value::String("missing".to_string())),
            "array_intersect must not mutate the right array"
        );
    }

    #[test]
    fn array_intersect_accepts_variadic_arrays() {
        let mut left = PhpArray::new();
        left.insert("name", Value::String("Ada".to_string()));
        left.insert(1, Value::String("1".to_string()));
        left.insert("two", Value::String("two".to_string()));
        left.insert("ten", Value::Int(10));
        left.insert("float-ten", Value::Float(10.0));
        left.insert("drop", Value::String("drop".to_string()));
        left.insert(8, Value::String("eight".to_string()));
        left.insert("keep", Value::String("keep".to_string()));
        left.append(Value::String("next".to_string())).unwrap();

        let mut first = PhpArray::new();
        first.append(Value::String("Ada".to_string())).unwrap();
        first.append(Value::String("1".to_string())).unwrap();
        first.append(Value::String("10".to_string())).unwrap();
        first.append(Value::String("eight".to_string())).unwrap();
        first.append(Value::String("next".to_string())).unwrap();
        first.append(Value::String("extra".to_string())).unwrap();

        let mut second = PhpArray::new();
        second.append(Value::String("Ada".to_string())).unwrap();
        second.append(Value::String("10".to_string())).unwrap();
        second.append(Value::String("eight".to_string())).unwrap();
        second.append(Value::String("drop".to_string())).unwrap();
        second.append(Value::String("next".to_string())).unwrap();

        let mut third = PhpArray::new();
        third.append(Value::String("Ada".to_string())).unwrap();
        third.append(Value::String("10".to_string())).unwrap();
        third.append(Value::String("eight".to_string())).unwrap();
        third.append(Value::String("next".to_string())).unwrap();

        let mut intersected = left
            .intersect_values_with_all([&first, &second, &third])
            .unwrap();
        let entries = intersected.entries();

        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::String("Ada".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("ten".to_string()));
        assert_eq!(entries[1].value, Value::Int(10));
        assert_eq!(entries[2].key, ArrayKey::String("float-ten".to_string()));
        assert_eq!(entries[2].value, Value::Float(10.0));
        assert_eq!(entries[3].key, ArrayKey::Int(8));
        assert_eq!(entries[3].value, Value::String("eight".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(9));
        assert_eq!(entries[4].value, Value::String("next".to_string()));
        assert_eq!(
            intersected
                .append(Value::String("after".to_string()))
                .unwrap(),
            ArrayKey::Int(10)
        );
        assert_eq!(
            left.get("keep"),
            Some(&Value::String("keep".to_string())),
            "array_intersect must not mutate the left array"
        );
        assert_eq!(
            first.get(5),
            Some(&Value::String("extra".to_string())),
            "array_intersect must not mutate variadic operands"
        );
        assert_eq!(
            second.get(3),
            Some(&Value::String("drop".to_string())),
            "array_intersect must not mutate later variadic operands"
        );
    }

    #[test]
    fn array_intersect_rejects_non_scalar_value_comparisons() {
        let mut left = PhpArray::new();
        left.insert("nested", Value::Array(PhpArray::new()));

        let right = PhpArray::new();
        let error = left.intersect_values_with(&right).unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_intersect()".to_string(),
                reason: "values must be scalar in the current subset, got array".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_intersect(): values must be scalar in the current subset, got array"
        );

        let mut classes = PhpClassTable::new();
        let class_id = classes.declare_class("Box").unwrap();
        let object = Value::Object(PhpObject::from_class(classes.get(class_id).unwrap()));
        let mut right = PhpArray::new();
        right.insert("object", object);

        let mut left = PhpArray::new();
        left.insert("value", Value::String("value".to_string()));
        let error = left.intersect_values_with(&right).unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_intersect()".to_string(),
                reason: "values must be scalar in the current subset, got object".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_intersect(): values must be scalar in the current subset, got object"
        );
    }

    #[test]
    fn array_unique_preserves_first_entries_by_scalar_string_form() {
        let mut array = PhpArray::new();

        array.insert(5, Value::String("five".to_string()));
        array.insert(9, Value::String("five".to_string()));
        array.insert(2, Value::String("two".to_string()));
        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert("empty", Value::String(String::new()));
        array.insert("true", Value::Bool(true));
        array.insert("one", Value::Int(1));
        array.insert("string-one", Value::String("1".to_string()));
        array.insert("int-ten", Value::Int(10));
        array.insert("float-ten", Value::Float(10.0));
        array.insert("string-ten-float", Value::String("10.0".to_string()));
        array.insert("text", Value::String("abc".to_string()));
        array.insert("dup-text", Value::String("abc".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut unique = array.unique_values_by_string().unwrap();
        let entries = unique.entries();

        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0].key, ArrayKey::Int(5));
        assert_eq!(entries[0].value, Value::String("five".to_string()));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::String("two".to_string()));
        assert_eq!(entries[2].key, ArrayKey::String("null".to_string()));
        assert_eq!(entries[2].value, Value::Null);
        assert_eq!(entries[3].key, ArrayKey::String("true".to_string()));
        assert_eq!(entries[3].value, Value::Bool(true));
        assert_eq!(entries[4].key, ArrayKey::String("int-ten".to_string()));
        assert_eq!(entries[4].value, Value::Int(10));
        assert_eq!(
            entries[5].key,
            ArrayKey::String("string-ten-float".to_string())
        );
        assert_eq!(entries[5].value, Value::String("10.0".to_string()));
        assert_eq!(entries[6].key, ArrayKey::String("text".to_string()));
        assert_eq!(entries[6].value, Value::String("abc".to_string()));
        assert_eq!(entries[7].key, ArrayKey::Int(10));
        assert_eq!(entries[7].value, Value::String("next".to_string()));
        assert_eq!(
            unique.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(11)
        );
        assert_eq!(
            array.get(9),
            Some(&Value::String("five".to_string())),
            "array_unique must not mutate the original array"
        );
    }

    #[test]
    fn array_unique_sort_regular_uses_loose_scalar_comparison() {
        let mut array = PhpArray::new();

        array.insert("s10", Value::String("10".to_string()));
        array.insert("i10", Value::Int(10));
        array.insert("f10", Value::Float(10.0));
        array.insert("s10f", Value::String("10.0".to_string()));
        array.insert("true", Value::Bool(true));
        array.insert("one", Value::Int(1));
        array.insert("false", Value::Bool(false));
        array.insert("empty", Value::String(String::new()));
        array.insert("null", Value::Null);
        array.insert("zero", Value::Int(0));
        array.insert("s0", Value::String("0".to_string()));
        array.insert("text", Value::String("abc".to_string()));
        array.insert("dup-text", Value::String("abc".to_string()));

        let regular = array.unique_values_regular().unwrap();
        let entries = regular.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("s10".to_string()));
        assert_eq!(entries[0].value, Value::String("10".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("one".to_string()));
        assert_eq!(entries[1].value, Value::Int(1));
        assert_eq!(entries[2].key, ArrayKey::String("false".to_string()));
        assert_eq!(entries[2].value, Value::Bool(false));
        assert_eq!(entries[3].key, ArrayKey::String("text".to_string()));
        assert_eq!(entries[3].value, Value::String("abc".to_string()));
        assert_eq!(
            array.get("i10"),
            Some(&Value::Int(10)),
            "array_unique SORT_REGULAR must not mutate the original array"
        );
    }

    #[test]
    fn array_unique_sort_numeric_uses_numeric_scalar_comparison() {
        let mut array = PhpArray::new();

        array.insert("first", Value::String("10".to_string()));
        array.insert("second", Value::Int(10));
        array.insert("third", Value::String("10.0".to_string()));
        array.insert("fourth", Value::Float(10.5));
        array.insert("fifth", Value::String("010.50".to_string()));
        array.insert("sixth", Value::Int(11));
        array.insert("seventh", Value::String("11.0".to_string()));
        array.insert("eighth", Value::Int(0));
        array.insert("ninth", Value::Bool(false));
        array.insert("tenth", Value::Null);

        let numeric = array.unique_values_by_numeric().unwrap();
        let entries = numeric.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("first".to_string()));
        assert_eq!(entries[0].value, Value::String("10".to_string()));
        assert_eq!(entries[1].key, ArrayKey::String("fourth".to_string()));
        assert_eq!(entries[1].value, Value::Float(10.5));
        assert_eq!(entries[2].key, ArrayKey::String("sixth".to_string()));
        assert_eq!(entries[2].value, Value::Int(11));
        assert_eq!(entries[3].key, ArrayKey::String("eighth".to_string()));
        assert_eq!(entries[3].value, Value::Int(0));
        assert_eq!(
            array.get("second"),
            Some(&Value::Int(10)),
            "array_unique SORT_NUMERIC must not mutate the original array"
        );
    }

    #[test]
    fn array_unique_rejects_non_scalar_value_comparisons() {
        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array.unique_values_by_string().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_unique()".to_string(),
                reason: "values must be scalar in the current subset, got array".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_unique(): values must be scalar in the current subset, got array"
        );

        let mut classes = PhpClassTable::new();
        let class_id = classes.declare_class("Box").unwrap();
        let object = Value::Object(PhpObject::from_class(classes.get(class_id).unwrap()));
        let mut array = PhpArray::new();
        array.insert("object", object);

        let error = array.unique_values_by_string().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_unique()".to_string(),
                reason: "values must be scalar in the current subset, got object".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_unique(): values must be scalar in the current subset, got object"
        );

        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array.unique_values_regular().unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_unique(): values must be scalar in the current subset, got array"
        );

        let mut array = PhpArray::new();
        array.insert("text", Value::String("not numeric".to_string()));

        let error = array.unique_values_by_numeric().unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_unique(): values must be numeric in the current subset, got non-numeric string"
        );

        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array.unique_values_by_numeric().unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_unique(): values must be numeric scalar in the current subset, got array"
        );
    }

    #[test]
    fn array_count_values_counts_int_string_values_and_normalizes_keys() {
        let mut array = PhpArray::new();

        array.insert("first", Value::String("name".to_string()));
        array.insert(5, Value::String("2".to_string()));
        array.insert("two", Value::Int(2));
        array.insert("02", Value::String("02".to_string()));
        array.append(Value::Int(-1)).unwrap();
        array.insert("dup-string", Value::String("name".to_string()));
        array.insert("dup-int", Value::Int(2));

        let mut counted = array.count_values().unwrap();
        let entries = counted.entries();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].key, ArrayKey::String("name".to_string()));
        assert_eq!(entries[0].value, Value::Int(2));
        assert_eq!(entries[1].key, ArrayKey::Int(2));
        assert_eq!(entries[1].value, Value::Int(3));
        assert_eq!(entries[2].key, ArrayKey::String("02".to_string()));
        assert_eq!(entries[2].value, Value::Int(1));
        assert_eq!(entries[3].key, ArrayKey::Int(-1));
        assert_eq!(entries[3].value, Value::Int(1));
        assert_eq!(
            counted.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(3)
        );
        assert_eq!(
            array.get("dup-string"),
            Some(&Value::String("name".to_string())),
            "array_count_values must not mutate the original array"
        );
    }

    #[test]
    fn array_count_values_rejects_unsupported_value_types() {
        let mut array = PhpArray::new();
        array.insert("ok", Value::String("name".to_string()));
        array.insert("bad", Value::Bool(true));

        let error = array.count_values().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_count_values()".to_string(),
                reason: "values must be int or string in the current subset, got bool".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_count_values(): values must be int or string in the current subset, got bool"
        );
    }

    #[test]
    fn array_sum_accumulates_supported_numeric_scalar_values() {
        let mut integers = PhpArray::new();
        integers.insert("null", Value::Null);
        integers.insert("false", Value::Bool(false));
        integers.insert("true", Value::Bool(true));
        integers.insert("int", Value::Int(2));
        integers.insert("string-int", Value::String(" 4 ".to_string()));
        integers.insert("negative", Value::String("-3".to_string()));

        assert_eq!(integers.sum_values().unwrap(), Value::Int(4));

        let mut mixed = PhpArray::new();
        mixed.insert("int", Value::Int(2));
        mixed.insert("float", Value::Float(3.5));
        mixed.insert("exponent", Value::String("6e1".to_string()));
        mixed.insert("decimal", Value::String(".25".to_string()));

        assert_eq!(mixed.sum_values().unwrap(), Value::Float(65.75));

        let empty = PhpArray::new();
        assert_eq!(empty.sum_values().unwrap(), Value::Int(0));

        let mut overflowing = PhpArray::new();
        overflowing.insert("max", Value::Int(i64::MAX));
        overflowing.insert("one", Value::Int(1));
        assert!(matches!(overflowing.sum_values().unwrap(), Value::Float(_)));
    }

    #[test]
    fn array_sum_rejects_values_outside_current_numeric_subset() {
        let mut string = PhpArray::new();
        string.insert("bad", Value::String("abc".to_string()));

        let error = string.sum_values().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_sum()".to_string(),
                reason: "values must be numeric in the current subset, got non-numeric string"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_sum(): values must be numeric in the current subset, got non-numeric string"
        );

        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array.sum_values().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_sum()".to_string(),
                reason: "values must be numeric scalar in the current subset, got array"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_sum(): values must be numeric scalar in the current subset, got array"
        );
    }

    #[test]
    fn array_product_accumulates_supported_numeric_scalar_values() {
        let mut integers = PhpArray::new();
        integers.insert("true", Value::Bool(true));
        integers.insert("int", Value::Int(2));
        integers.insert("string-int", Value::String(" 4 ".to_string()));
        integers.insert("negative", Value::String("-3".to_string()));

        assert_eq!(integers.product_values().unwrap(), Value::Int(-24));

        let mut with_zero = PhpArray::new();
        with_zero.insert("null", Value::Null);
        with_zero.insert("true", Value::Bool(true));
        with_zero.insert("int", Value::Int(2));

        assert_eq!(with_zero.product_values().unwrap(), Value::Int(0));

        let mut mixed = PhpArray::new();
        mixed.insert("int", Value::Int(2));
        mixed.insert("float", Value::Float(3.5));
        mixed.insert("exponent", Value::String("6e1".to_string()));
        mixed.insert("decimal", Value::String(".25".to_string()));

        assert_eq!(mixed.product_values().unwrap(), Value::Float(105.0));

        let empty = PhpArray::new();
        assert_eq!(empty.product_values().unwrap(), Value::Int(1));

        let mut overflowing = PhpArray::new();
        overflowing.insert("max", Value::Int(i64::MAX));
        overflowing.insert("two", Value::Int(2));
        assert!(matches!(
            overflowing.product_values().unwrap(),
            Value::Float(_)
        ));
    }

    #[test]
    fn array_product_rejects_values_outside_current_numeric_subset() {
        let mut string = PhpArray::new();
        string.insert("bad", Value::String("abc".to_string()));

        let error = string.product_values().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_product()".to_string(),
                reason: "values must be numeric in the current subset, got non-numeric string"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_product(): values must be numeric in the current subset, got non-numeric string"
        );

        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array.product_values().unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_product()".to_string(),
                reason: "values must be numeric scalar in the current subset, got array"
                    .to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_product(): values must be numeric scalar in the current subset, got array"
        );
    }

    #[test]
    fn array_filter_without_callback_removes_falsey_values_and_preserves_keys() {
        let mut array = PhpArray::new();
        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert("true", Value::Bool(true));
        array.insert("zero", Value::Int(0));
        array.insert("one", Value::Int(1));
        array.insert("empty-string", Value::String(String::new()));
        array.insert("zero-string", Value::String("0".to_string()));
        array.insert("space", Value::String(" ".to_string()));
        array.insert("empty-array", Value::Array(PhpArray::new()));

        let mut nested = PhpArray::new();
        nested.insert("item", Value::String("kept".to_string()));
        array.insert("nested-array", Value::Array(nested));
        array.insert(7, Value::String("seven".to_string()));
        array.append(Value::String("next".to_string())).unwrap();

        let mut filtered = array.filtered_without_callback();
        let entries = filtered.entries();

        assert_eq!(entries.len(), 6);
        assert_eq!(entries[0].key, ArrayKey::String("true".to_string()));
        assert_eq!(entries[0].value, Value::Bool(true));
        assert_eq!(entries[1].key, ArrayKey::String("one".to_string()));
        assert_eq!(entries[1].value, Value::Int(1));
        assert_eq!(entries[2].key, ArrayKey::String("space".to_string()));
        assert_eq!(entries[2].value, Value::String(" ".to_string()));
        assert_eq!(entries[3].key, ArrayKey::String("nested-array".to_string()));
        assert_eq!(entries[4].key, ArrayKey::Int(7));
        assert_eq!(entries[4].value, Value::String("seven".to_string()));
        assert_eq!(entries[5].key, ArrayKey::Int(8));
        assert_eq!(entries[5].value, Value::String("next".to_string()));
        assert_eq!(
            filtered.append(Value::String("after".to_string())).unwrap(),
            ArrayKey::Int(9)
        );
        assert!(
            !filtered.contains_key("empty-array"),
            "array_filter without a callback removes empty arrays"
        );
        assert!(
            array.contains_key("empty-array"),
            "array_filter must not mutate the original array"
        );
    }

    #[test]
    fn in_array_uses_loose_scalar_comparison_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert("int", Value::Int(10));
        array.insert("numeric-string", Value::String("10.0".to_string()));
        array.insert("text", Value::String("abc".to_string()));

        assert!(array
            .contains_value_loose_scalar(&Value::String(String::new()))
            .unwrap());
        assert!(array
            .contains_value_loose_scalar(&Value::String("0".to_string()))
            .unwrap());
        assert!(array
            .contains_value_loose_scalar(&Value::String("10".to_string()))
            .unwrap());
        assert!(array.contains_value_loose_scalar(&Value::Int(10)).unwrap());
        assert!(!array.contains_value_loose_scalar(&Value::Int(11)).unwrap());
        assert!(!array
            .contains_value_loose_scalar(&Value::String("missing".to_string()))
            .unwrap());
    }

    #[test]
    fn in_array_strict_mode_uses_scalar_identity_in_insertion_order() {
        let mut array = PhpArray::new();

        array.insert("false", Value::Bool(false));
        array.insert("int-zero", Value::Int(0));
        array.insert("string-zero", Value::String("0".to_string()));
        array.insert("int-ten", Value::Int(10));
        array.insert("string-ten", Value::String("10".to_string()));
        array.insert("null", Value::Null);

        assert!(!array
            .contains_value_strict_scalar(&Value::String(String::new()))
            .unwrap());
        assert!(array
            .contains_value_strict_scalar(&Value::Bool(false))
            .unwrap());
        assert!(array.contains_value_strict_scalar(&Value::Int(0)).unwrap());
        assert!(array
            .contains_value_strict_scalar(&Value::String("0".to_string()))
            .unwrap());
        assert!(!array
            .contains_value_strict_scalar(&Value::Float(10.0))
            .unwrap());
        assert!(array.contains_value_strict_scalar(&Value::Int(10)).unwrap());
        assert!(array
            .contains_value_strict_scalar(&Value::String("10".to_string()))
            .unwrap());
        assert!(array.contains_value_strict_scalar(&Value::Null).unwrap());
        assert!(!array
            .contains_value_strict_scalar(&Value::String("missing".to_string()))
            .unwrap());
    }

    #[test]
    fn in_array_rejects_array_comparison_gaps() {
        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array
            .contains_value_loose_scalar(&Value::String("needle".to_string()))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "in_array()".to_string(),
                reason: "array needles and array values are not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call in_array(): array needles and array values are not implemented"
        );
    }

    #[test]
    fn array_search_returns_first_loose_scalar_match_key() {
        let mut array = PhpArray::new();

        array.insert("null", Value::Null);
        array.insert("false", Value::Bool(false));
        array.insert(0, Value::String("zero-key".to_string()));
        array.insert("2", Value::String("two-key".to_string()));
        array.insert("02", Value::String("zero-two-key".to_string()));
        array.append(Value::String("appended".to_string())).unwrap();
        array.insert("numeric", Value::String("10.0".to_string()));
        array.insert("text", Value::String("abc".to_string()));

        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String(String::new()))
                .unwrap(),
            Some(ArrayKey::String("null".to_string()))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("0".to_string()))
                .unwrap(),
            Some(ArrayKey::String("false".to_string()))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("zero-key".to_string()))
                .unwrap(),
            Some(ArrayKey::Int(0))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("two-key".to_string()))
                .unwrap(),
            Some(ArrayKey::Int(2))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("zero-two-key".to_string()))
                .unwrap(),
            Some(ArrayKey::String("02".to_string()))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("appended".to_string()))
                .unwrap(),
            Some(ArrayKey::Int(3))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("10".to_string()))
                .unwrap(),
            Some(ArrayKey::String("numeric".to_string()))
        );
        assert_eq!(
            array
                .search_value_loose_scalar(&Value::String("missing".to_string()))
                .unwrap(),
            None
        );
    }

    #[test]
    fn array_search_strict_mode_returns_first_scalar_identity_match_key() {
        let mut array = PhpArray::new();

        array.insert("false", Value::Bool(false));
        array.insert("int-zero", Value::Int(0));
        array.insert("string-zero", Value::String("0".to_string()));
        array.insert("int-ten", Value::Int(10));
        array.insert("string-ten", Value::String("10".to_string()));
        array.insert("null", Value::Null);
        array.insert(2, Value::String("int-key".to_string()));
        array.insert("text", Value::String("abc".to_string()));

        assert_eq!(
            array
                .search_value_strict_scalar(&Value::String(String::new()))
                .unwrap(),
            None
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::Bool(false))
                .unwrap(),
            Some(ArrayKey::String("false".to_string()))
        );
        assert_eq!(
            array.search_value_strict_scalar(&Value::Int(0)).unwrap(),
            Some(ArrayKey::String("int-zero".to_string()))
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::String("0".to_string()))
                .unwrap(),
            Some(ArrayKey::String("string-zero".to_string()))
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::Float(10.0))
                .unwrap(),
            None
        );
        assert_eq!(
            array.search_value_strict_scalar(&Value::Int(10)).unwrap(),
            Some(ArrayKey::String("int-ten".to_string()))
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::String("10".to_string()))
                .unwrap(),
            Some(ArrayKey::String("string-ten".to_string()))
        );
        assert_eq!(
            array.search_value_strict_scalar(&Value::Null).unwrap(),
            Some(ArrayKey::String("null".to_string()))
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::String("int-key".to_string()))
                .unwrap(),
            Some(ArrayKey::Int(2))
        );
        assert_eq!(
            array
                .search_value_strict_scalar(&Value::String("missing".to_string()))
                .unwrap(),
            None
        );
    }

    #[test]
    fn array_search_rejects_array_comparison_gaps() {
        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array
            .search_value_loose_scalar(&Value::String("needle".to_string()))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_search()".to_string(),
                reason: "array needles and array values are not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_search(): array needles and array values are not implemented"
        );
    }

    #[test]
    fn array_keys_filter_rejects_array_comparison_gaps() {
        let mut array = PhpArray::new();
        array.insert("value", Value::Int(1));

        let error = array
            .keys_matching_loose_scalar(&Value::Array(PhpArray::new()))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_keys()".to_string(),
                reason: "array search values and array values are not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_keys(): array search values and array values are not implemented"
        );

        let error = array
            .keys_matching_strict_scalar(&Value::Array(PhpArray::new()))
            .unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_keys(): array search values and array values are not implemented"
        );

        let mut array = PhpArray::new();
        array.insert("nested", Value::Array(PhpArray::new()));

        let error = array
            .keys_matching_loose_scalar(&Value::String("needle".to_string()))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedCall {
                callable: "array_keys()".to_string(),
                reason: "array search values and array values are not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported call array_keys(): array search values and array values are not implemented"
        );

        let error = array
            .keys_matching_strict_scalar(&Value::String("needle".to_string()))
            .unwrap_err();
        assert_eq!(
            error.message(),
            "unsupported call array_keys(): array search values and array values are not implemented"
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

    #[test]
    fn array_key_exists_keys_accept_current_null_and_bool_coercions() {
        assert_eq!(
            ArrayKey::from_array_key_exists_value(&Value::Null).unwrap(),
            ArrayKey::String(String::new())
        );
        assert_eq!(
            ArrayKey::from_array_key_exists_value(&Value::Bool(false)).unwrap(),
            ArrayKey::Int(0)
        );
        assert_eq!(
            ArrayKey::from_array_key_exists_value(&Value::Bool(true)).unwrap(),
            ArrayKey::Int(1)
        );
        assert_eq!(
            ArrayKey::from_array_key_exists_value(&Value::Float(1.0)).unwrap(),
            ArrayKey::Int(1)
        );
        assert_eq!(
            ArrayKey::from_array_key_exists_value(&Value::Float(-2.0)).unwrap(),
            ArrayKey::Int(-2)
        );

        let float_error = ArrayKey::from_array_key_exists_value(&Value::Float(1.5)).unwrap_err();
        assert_eq!(
            float_error.message(),
            "invalid array key: lossy or non-finite float keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented"
        );

        let error =
            ArrayKey::from_array_key_exists_value(&Value::Array(PhpArray::new())).unwrap_err();
        assert_eq!(
            error.message(),
            "invalid array key: array keys are not supported for array_key_exists(); only null, bool, int, string, and integral finite float keys are implemented"
        );
    }

    #[test]
    fn class_table_preserves_names_and_uses_case_insensitive_lookup() {
        let mut classes = PhpClassTable::new();

        let id = classes.declare_class("Widget").unwrap();

        assert_eq!(id.index(), 0);
        assert_eq!(classes.get(id).unwrap().name(), "Widget");
        assert_eq!(classes.lookup_class("widget").unwrap().id(), id);
        assert_eq!(classes.lookup_class("WIDGET").unwrap().name(), "Widget");

        let error = classes.declare_class("widget").unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::DuplicateClass {
                class_name: "widget".to_string(),
            }
        );
        assert_eq!(error.message(), "class widget is already defined");
    }

    #[test]
    fn class_table_can_bootstrap_core_exception_metadata() {
        let classes = PhpClassTable::with_core_classes();

        let exception = classes.lookup_class("exception").unwrap();
        assert_eq!(exception.name(), "Exception");
        assert_eq!(exception.id().index(), 0);
        assert!(exception.parent_id().is_none());
        assert!(exception.properties().is_empty());
        assert!(exception.methods().is_empty());

        let stdclass = classes.lookup_class("stdclass").unwrap();
        assert_eq!(stdclass.name(), "stdClass");
        assert_eq!(stdclass.id().index(), 1);
        assert!(stdclass.parent_id().is_none());
        assert!(stdclass.properties().is_empty());
        assert!(stdclass.methods().is_empty());

        let mysqli = classes.lookup_class("mysqli").unwrap();
        assert_eq!(mysqli.name(), "mysqli");
        assert_eq!(mysqli.id().index(), 2);
        assert!(mysqli.parent_id().is_none());
        assert_eq!(
            mysqli
                .properties()
                .iter()
                .map(PhpPropertyMetadata::name)
                .collect::<Vec<_>>(),
            vec!["connect_errno", "connect_error"]
        );
        assert!(mysqli.methods().is_empty());
    }

    #[test]
    fn class_table_can_remove_last_declared_class_for_registration_rollback() {
        let mut classes = PhpClassTable::new();

        let first = classes.declare_class("First").unwrap();
        let second = classes.declare_class("Second").unwrap();

        classes.remove_last_declared_class(second);
        assert!(classes.lookup_class("Second").is_none());
        assert_eq!(classes.lookup_class("First").unwrap().id(), first);

        let second = classes.declare_class("Second").unwrap();
        assert_eq!(second.index(), 1);
    }

    #[test]
    fn class_metadata_tracks_php_property_and_method_lookup_rules() {
        let mut classes = PhpClassTable::new();
        let id = classes.declare_class("Counter").unwrap();
        let class = classes.get_mut(id).unwrap();

        class
            .add_property(PhpPropertyMetadata::instance("value", Visibility::Private))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance("Value", Visibility::Public))
            .unwrap();
        class
            .add_method(PhpMethodMetadata::instance("increment", Visibility::Public))
            .unwrap();

        assert_eq!(
            class.property("value").unwrap().visibility(),
            Visibility::Private
        );
        assert_eq!(
            class.property("Value").unwrap().visibility(),
            Visibility::Public
        );
        assert!(class.property("VALUE").is_none());
        assert_eq!(class.method("INCREMENT").unwrap().name(), "increment");

        let error = class
            .add_method(PhpMethodMetadata::instance("Increment", Visibility::Public))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::DuplicateClassMember {
                class_name: "Counter".to_string(),
                member_kind: ClassMemberKind::Method,
                member_name: "Increment".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "class Counter already defines method Increment"
        );

        let error = class
            .add_property(PhpPropertyMetadata::instance("value", Visibility::Public))
            .unwrap_err();
        assert_eq!(
            error.message(),
            "class Counter already defines property value"
        );
    }

    #[test]
    fn object_shape_contains_only_instance_properties_in_declaration_order() {
        let mut classes = PhpClassTable::new();
        let id = classes.declare_class("Packet").unwrap();
        let class = classes.get_mut(id).unwrap();

        class
            .add_property(PhpPropertyMetadata::instance("id", Visibility::Public))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::static_property(
                "nextId",
                Visibility::Private,
            ))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance(
                "payload",
                Visibility::Protected,
            ))
            .unwrap();

        let shape = class.object_shape();

        assert_eq!(shape.class_id(), id);
        assert_eq!(
            shape.instance_properties(),
            &["id".to_string(), "payload".to_string()]
        );
    }

    #[test]
    fn object_values_materialize_instance_properties_as_null() {
        let mut classes = PhpClassTable::new();
        let id = classes.declare_class("Packet").unwrap();
        let class = classes.get_mut(id).unwrap();

        class
            .add_property(PhpPropertyMetadata::instance("id", Visibility::Public))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::static_property(
                "nextId",
                Visibility::Private,
            ))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance(
                "payload",
                Visibility::Protected,
            ))
            .unwrap();

        let object = PhpObject::from_class(class);

        assert_eq!(object.class_id(), id);
        assert_eq!(object.class_name(), "Packet");
        assert_eq!(object.properties().len(), 2);
        assert_eq!(object.properties()[0].name(), "id");
        assert_eq!(object.properties()[0].visibility(), Visibility::Public);
        assert_eq!(object.properties()[0].value(), &Value::Null);
        assert_eq!(object.properties()[1].name(), "payload");
        assert_eq!(object.properties()[1].visibility(), Visibility::Protected);
        assert_eq!(object.properties()[1].value(), &Value::Null);
    }

    #[test]
    fn object_properties_expose_php_mangled_names_for_visibility() {
        let mut classes = PhpClassTable::new();
        let id = classes.declare_class("Packet").unwrap();
        let class = classes.get_mut(id).unwrap();

        class
            .add_property(PhpPropertyMetadata::instance("id", Visibility::Public))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance(
                "payload",
                Visibility::Protected,
            ))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance("secret", Visibility::Private))
            .unwrap();

        let object = PhpObject::from_class(class);
        let properties = object.properties();

        assert_eq!(properties[0].mangled_name(), "id");
        assert_eq!(properties[1].mangled_name(), "\0*\0payload");
        assert_eq!(properties[2].mangled_name(), "\0Packet\0secret");
    }

    #[test]
    fn object_public_property_reads_and_writes_use_exact_slot_names() {
        let mut classes = PhpClassTable::new();
        let id = classes.declare_class("Packet").unwrap();
        let class = classes.get_mut(id).unwrap();

        class
            .add_property(PhpPropertyMetadata::instance("id", Visibility::Public))
            .unwrap();
        class
            .add_property(PhpPropertyMetadata::instance("secret", Visibility::Private))
            .unwrap();

        let object = PhpObject::from_class(class);

        assert_eq!(object.read_public_property("id").unwrap(), Value::Null);
        assert!(!object.is_public_property_set("id").unwrap());
        assert!(object.is_public_property_empty("id").unwrap());
        object
            .write_public_property("id", Value::Int(42))
            .expect("public property write should update the slot");
        assert_eq!(object.read_public_property("id").unwrap(), Value::Int(42));
        assert!(object.is_public_property_set("id").unwrap());
        assert!(!object.is_public_property_empty("id").unwrap());
        assert!(!object.is_public_property_set("ID").unwrap());
        assert!(object.is_public_property_empty("ID").unwrap());

        object
            .write_public_property("id", Value::String("0".to_string()))
            .expect("public property write should update the slot");
        assert!(object.is_public_property_empty("id").unwrap());

        let missing = object.read_public_property("ID").unwrap_err();
        assert_eq!(
            missing.kind(),
            &RuntimeErrorKind::UndefinedProperty {
                class_name: "Packet".to_string(),
                property_name: "ID".to_string(),
            }
        );
        assert_eq!(missing.message(), "undefined property Packet::$ID");

        let private = object
            .write_public_property("secret", Value::String("x".to_string()))
            .unwrap_err();
        assert_eq!(
            private.message(),
            "unsupported object property access: non-public property Packet::$secret requires same-class method context in the current subset"
        );

        let private_isset = object.is_public_property_set("secret").unwrap_err();
        assert_eq!(
            private_isset.message(),
            "unsupported object property access: non-public property Packet::$secret requires same-class method context in the current subset"
        );

        let private_empty = object.is_public_property_empty("secret").unwrap_err();
        assert_eq!(
            private_empty.message(),
            "unsupported object property access: non-public property Packet::$secret requires same-class method context in the current subset"
        );
    }
}
