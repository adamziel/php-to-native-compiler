use std::cmp::Ordering;
use std::collections::HashMap;
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
    DuplicateClass {
        class_name: String,
    },
    UndefinedClass {
        class_name: String,
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
        RuntimeErrorKind::DuplicateClass { class_name } => {
            format!("class {class_name} is already defined")
        }
        RuntimeErrorKind::UndefinedClass { class_name } => {
            format!("undefined class {class_name}")
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
            let value = match &entry.key {
                ArrayKey::Int(value) => Value::Int(*value),
                ArrayKey::String(value) => Value::String(value.clone()),
            };
            array.insert(key, value);
        }
        array
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

    pub fn classes(&self) -> &[PhpClassMetadata] {
        &self.classes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhpClassMetadata {
    id: ClassId,
    name: String,
    properties: Vec<PhpPropertyMetadata>,
    property_lookup: HashMap<String, usize>,
    methods: Vec<PhpMethodMetadata>,
    method_lookup: HashMap<String, usize>,
}

impl PhpClassMetadata {
    fn new(id: ClassId, name: String) -> Self {
        Self {
            id,
            name,
            properties: Vec::new(),
            property_lookup: HashMap::new(),
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

    pub fn properties(&self) -> &[PhpPropertyMetadata] {
        &self.properties
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
    Method,
}

impl fmt::Display for ClassMemberKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClassMemberKind::Property => write!(f, "property"),
            ClassMemberKind::Method => write!(f, "method"),
        }
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

#[derive(Debug, Clone, PartialEq)]
pub struct PhpObject {
    class_id: ClassId,
    class_name: String,
    properties: Vec<ObjectProperty>,
}

impl PhpObject {
    pub fn from_class(class: &PhpClassMetadata) -> Self {
        let properties = class
            .properties()
            .iter()
            .filter(|property| !property.is_static())
            .map(|property| ObjectProperty {
                name: property.name().to_string(),
                visibility: property.visibility(),
                value: Value::Null,
            })
            .collect();

        Self {
            class_id: class.id(),
            class_name: class.name().to_string(),
            properties,
        }
    }

    pub fn class_id(&self) -> ClassId {
        self.class_id
    }

    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    pub fn properties(&self) -> &[ObjectProperty] {
        &self.properties
    }

    pub fn read_public_property(&self, name: &str) -> RuntimeResult<&Value> {
        let property = self
            .properties
            .iter()
            .find(|property| property.name == name)
            .ok_or_else(|| RuntimeError::undefined_property(self.class_name.clone(), name))?;

        if property.visibility != Visibility::Public {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires visibility enforcement, which is not implemented",
                self.class_name, name
            )));
        }

        Ok(&property.value)
    }

    pub fn is_public_property_set(&self, name: &str) -> RuntimeResult<bool> {
        let Some(property) = self
            .properties
            .iter()
            .find(|property| property.name == name)
        else {
            return Ok(false);
        };

        if property.visibility != Visibility::Public {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires visibility enforcement, which is not implemented",
                self.class_name, name
            )));
        }

        Ok(!matches!(property.value, Value::Null))
    }

    pub fn write_public_property(&mut self, name: &str, value: Value) -> RuntimeResult<()> {
        let property = self
            .properties
            .iter_mut()
            .find(|property| property.name == name)
            .ok_or_else(|| RuntimeError::undefined_property(self.class_name.clone(), name))?;

        if property.visibility != Visibility::Public {
            return Err(RuntimeError::unsupported_property_access(format!(
                "non-public property {}::${} requires visibility enforcement, which is not implemented",
                self.class_name, name
            )));
        }

        property.value = value;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectProperty {
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

    pub fn value(&self) -> &Value {
        &self.value
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
            Value::Object(_) => "Object".to_string(),
        }
    }

    pub fn try_echo_string(&self) -> RuntimeResult<String> {
        match self {
            Value::Object(object) => Err(RuntimeError::invalid_string_conversion(format!(
                "object of class {} cannot be converted to string",
                object.class_name()
            ))),
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

    pub fn php_concat(&self, other: &Value) -> RuntimeResult<Value> {
        Ok(Value::String(format!(
            "{}{}",
            self.try_echo_string()?,
            other.try_echo_string()?
        )))
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
            (Value::Object(_), _) | (_, Value::Object(_)) => {
                Err(RuntimeError::unsupported_comparison(
                    "strict identity for objects is not implemented",
                ))
            }
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
    fn strict_identity_rejects_non_scalar_values() {
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
        let error = object.php_identical_checked(&Value::Null).unwrap_err();

        assert_eq!(
            error.kind(),
            &RuntimeErrorKind::UnsupportedComparison {
                reason: "strict identity for objects is not implemented".to_string(),
            }
        );
        assert_eq!(
            error.message(),
            "unsupported comparison: strict identity for objects is not implemented"
        );
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

        let mut object = PhpObject::from_class(class);

        assert_eq!(object.read_public_property("id").unwrap(), &Value::Null);
        assert!(!object.is_public_property_set("id").unwrap());
        object
            .write_public_property("id", Value::Int(42))
            .expect("public property write should update the slot");
        assert_eq!(object.read_public_property("id").unwrap(), &Value::Int(42));
        assert!(object.is_public_property_set("id").unwrap());
        assert!(!object.is_public_property_set("ID").unwrap());

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
            "unsupported object property access: non-public property Packet::$secret requires visibility enforcement, which is not implemented"
        );

        let private_isset = object.is_public_property_set("secret").unwrap_err();
        assert_eq!(
            private_isset.message(),
            "unsupported object property access: non-public property Packet::$secret requires visibility enforcement, which is not implemented"
        );
    }
}
