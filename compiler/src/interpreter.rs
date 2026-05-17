use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use php_runtime::{
    ArityExpectation, ArrayColumnKey, ArrayKey, ArrayKeyCase, ClassId, Comparison, ObjectProperty,
    PhpArray, PhpClassConstantMetadata, PhpClassTable, PhpClosure, PhpClosureCapture,
    PhpMethodMetadata, PhpObject, PhpObjectPropertyInitializer, PhpPropertyMetadata, RuntimeError,
    RuntimeErrorKind, RuntimeResult, Value, Visibility,
};
use sha2::Sha256;

use crate::ast::{
    ArrayItem, AssignTarget, BinaryOp, CastKind, ClassConstantDecl, ClassDecl, ClassMember,
    ClassMethodDecl, ClassPropertyDecl, ClassVisibility, ClosureCapture, CompoundAssignOp,
    EnumDecl, Expr, ForAction, FunctionDecl, IncrementDecrementOp, IncrementDecrementPosition,
    InterfaceDecl, InterfaceMethodDecl, InterpolatedAccessSegment, InterpolatedArrayKey,
    InterpolatedStringPart, NewClassName, Program, ReferenceSource, Span, StaticLocalDeclarator,
    Stmt, SwitchCase, TraitDecl, UnaryOp, UnsetTarget,
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
    method_signatures: HashMap<(ClassId, String), MethodSignature>,
    abstract_classes: HashSet<ClassId>,
    final_classes: HashSet<ClassId>,
    abstract_methods: HashSet<(ClassId, String)>,
    final_methods: HashMap<(ClassId, String), String>,
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
    global_symbols: SymbolStorage,
    error_reporting_mask: i64,
    ignore_user_abort: bool,
    ini_values: HashMap<String, String>,
    error_handler: Option<Value>,
    error_handler_mask: Option<i64>,
    mysqli_report_mode: i64,
    mysqli_results: HashMap<i64, MysqliResultState>,
    mysqli_pending_results: HashMap<i64, MysqliPendingResultState>,
    mysqli_pending_result_queues: HashMap<i64, VecDeque<MysqliMultiResultSlot>>,
    mysqli_options: HashMap<i64, HashMap<i64, Value>>,
    mysqli_wp_options: HashMap<i64, HashMap<String, WordPressOptionState>>,
    mysqli_transactions: HashMap<i64, MysqliTransactionState>,
    mysqli_affected_rows: HashMap<i64, i64>,
    mysqli_insert_ids: HashMap<i64, i64>,
    mysqli_statements: HashMap<i64, MysqliStatementState>,
    source_file: Option<String>,
    max_execution_steps: Option<usize>,
    trace_includes: bool,
    execution_steps: usize,
    call_depth: usize,
    next_object_id: i64,
    next_closure_id: i64,
    next_foreach_temp_id: i64,
    active_foreach_references: Vec<ActiveForeachReference>,
    function_context: Vec<String>,
    class_context: Vec<ClassId>,
    called_class_context: Vec<ClassId>,
    response_headers: Vec<String>,
    output_buffers: Vec<String>,
    stdout: String,
    exit_signal: Option<i32>,
}

#[derive(Debug, Clone)]
struct MethodSignature {
    required_params: usize,
    params: Vec<ParameterSignature>,
    return_type: Option<String>,
}

#[derive(Debug, Clone)]
struct ParameterSignature {
    name: String,
    type_decl: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveForeachReference {
    array_name: String,
    value_name: String,
    key: ArrayKey,
}

#[derive(Clone)]
struct WordPressOptionState {
    value: String,
    autoload: String,
}

#[derive(Clone)]
struct MysqliTransactionState {
    wp_options_snapshot: Option<HashMap<String, WordPressOptionState>>,
    wp_option_savepoints: HashMap<String, Option<HashMap<String, WordPressOptionState>>>,
}

#[derive(Debug, Clone)]
struct MysqliResultState {
    fields: Vec<String>,
    rows: Vec<Vec<(String, Value)>>,
    row_cursor: usize,
    field_cursor: usize,
    last_lengths: Option<Vec<usize>>,
}

#[derive(Debug, Clone)]
struct MysqliPendingResultState {
    fields: Vec<String>,
    rows: Vec<Vec<(String, Value)>>,
}

#[derive(Debug, Clone)]
enum MysqliMultiResultSlot {
    NoResult,
    Result(MysqliPendingResultState),
}

#[derive(Debug, Clone, Default)]
struct MysqliStatementState {
    connection_handle_id: Option<i64>,
    query: Option<String>,
    param_count: usize,
    bound_parameter_types: Option<String>,
    bound_parameter_variables: Vec<String>,
    bound_parameter_values: Vec<Value>,
    executed_result: Option<MysqliPendingResultState>,
    affected_rows: i64,
    buffered_result: Option<MysqliPendingResultState>,
    buffered_result_cursor: usize,
    bound_result_variables: Vec<String>,
    attributes: HashMap<i64, Value>,
    long_data: HashMap<usize, String>,
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
    matches!(
        name,
        "_SERVER" | "_COOKIE" | "_GET" | "_POST" | "_REQUEST" | "_FILES"
    )
}

fn globals_offset_name(key: &ArrayKey) -> Option<&str> {
    match key {
        ArrayKey::String(name) => Some(name),
        ArrayKey::Int(_) => None,
    }
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
    ArrayAccessOffset {
        name: String,
        key: ArrayKey,
    },
    ObjectPropertyArrayAccessOffset {
        object: String,
        property: String,
        key: ArrayKey,
    },
    ArrayAccessTemporary,
    ObjectProperty {
        object: String,
        property: String,
    },
    ObjectPropertyArrayIndex {
        object: String,
        property: String,
        keys: Vec<ArrayKey>,
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
    symbols: SymbolStorage,
    global_symbols: Option<SymbolStorage>,
    imported_globals: HashSet<String>,
    array_offset_aliases: HashMap<String, Vec<ArrayOffsetAlias>>,
}

type SymbolStorage = Rc<RefCell<HashMap<String, VariableCell>>>;
type VariableCell = Rc<RefCell<Value>>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArrayOffsetAlias {
    root: ArrayOffsetAliasRoot,
    keys: Vec<ArrayKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArrayOffsetAliasRoot {
    StaticArray {
        name: String,
    },
    GlobalArray {
        name: String,
    },
    PublicObjectProperty {
        object: String,
        property: String,
    },
    ContextObjectProperty {
        object: String,
        property: String,
        current_class_id: Option<ClassId>,
        protected_class_ids: Vec<ClassId>,
    },
}

impl ArrayOffsetAliasRoot {
    fn matches_object_property(&self, object_name: &str, property_name: &str) -> bool {
        match self {
            ArrayOffsetAliasRoot::PublicObjectProperty { object, property }
            | ArrayOffsetAliasRoot::ContextObjectProperty {
                object, property, ..
            } => object == object_name && property == property_name,
            _ => false,
        }
    }

    fn matches_object(&self, object_name: &str) -> bool {
        match self {
            ArrayOffsetAliasRoot::PublicObjectProperty { object, .. }
            | ArrayOffsetAliasRoot::ContextObjectProperty { object, .. } => object == object_name,
            _ => false,
        }
    }
}

const CORE_INTERFACE_NAMES: &[&str] = &[
    "Traversable",
    "IteratorAggregate",
    "Iterator",
    "Serializable",
    "ArrayAccess",
    "Countable",
    "Stringable",
];

fn is_core_interface_name(name: &str) -> bool {
    CORE_INTERFACE_NAMES
        .iter()
        .any(|interface| interface.eq_ignore_ascii_case(name))
}

impl SymbolTable {
    fn new() -> Self {
        Self::default()
    }

    fn new_child(global_symbols: SymbolStorage) -> Self {
        Self {
            symbols: Rc::new(RefCell::new(HashMap::new())),
            global_symbols: Some(global_symbols),
            imported_globals: HashSet::new(),
            array_offset_aliases: HashMap::new(),
        }
    }

    fn from_root(symbols: SymbolStorage) -> Self {
        Self {
            symbols,
            global_symbols: None,
            imported_globals: HashSet::new(),
            array_offset_aliases: HashMap::new(),
        }
    }

    fn import_global(&mut self, name: &str) {
        if let Some(global_symbols) = &self.global_symbols {
            if global_symbols.borrow().get(name).is_none() {
                global_symbols
                    .borrow_mut()
                    .insert(name.to_string(), value_cell(Value::Null));
            }
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
        self.array_offset_aliases.remove(name);
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
        if let Some(aliases) = self.array_offset_aliases.get(name) {
            return aliases
                .first()
                .and_then(|alias| self.read_array_offset_alias(alias));
        }

        self.read_storage_named(name)
    }

    fn read_global_name(&self, name: &str) -> Option<Value> {
        self.global_storage()
            .borrow()
            .get(name)
            .map(|cell| cell.borrow().clone())
    }

    fn write_named(&mut self, name: &str, value: Value) {
        if let Some(aliases) = self.array_offset_aliases.get(name).cloned() {
            if self.write_array_offset_aliases(&aliases, value.clone()) {
                return;
            }
            self.array_offset_aliases.remove(name);
        }

        self.write_storage_named(name, value);
        if self.name_routes_to_global_storage(name) {
            self.sync_array_offset_aliases_for_global_root(name);
        }
    }

    fn write_global_name(&mut self, name: &str, value: Value) {
        let storage = self.global_storage().clone();
        let cell = storage.borrow().get(name).cloned();
        if let Some(cell) = cell {
            *cell.borrow_mut() = value;
        } else {
            storage
                .borrow_mut()
                .insert(name.to_string(), value_cell(value));
        }
    }

    fn read_storage_named(&self, name: &str) -> Option<Value> {
        self.read_cell(name).map(|cell| cell.borrow().clone())
    }

    fn write_storage_named(&mut self, name: &str, value: Value) {
        let storage = self.routed_storage(name).clone();
        let cell = storage.borrow().get(name).cloned();
        if let Some(cell) = cell {
            *cell.borrow_mut() = value;
        } else {
            storage
                .borrow_mut()
                .insert(name.to_string(), value_cell(value));
        }
    }

    fn bind_static_to_static(
        &mut self,
        target: &str,
        source: &str,
        span: Span,
    ) -> CompileResult<()> {
        if let Some(aliases) = self.array_offset_aliases.get(source).cloned() {
            for alias in &aliases {
                self.materialize_array_offset_alias(alias, span)?;
            }
            self.bind_static_to_array_offset_aliases(target, aliases);
            return Ok(());
        }

        let source_cell = self
            .read_cell(source)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_variable(source)))?;
        self.array_offset_aliases.remove(target);
        self.routed_storage(target)
            .borrow_mut()
            .insert(target.to_string(), source_cell);
        Ok(())
    }

    fn bind_global_name_to_static_source(
        &mut self,
        global_name: &str,
        source: &str,
        span: Span,
    ) -> CompileResult<()> {
        if let Some(existing_aliases) = self.array_offset_aliases.get(source).cloned() {
            for alias in &existing_aliases {
                self.materialize_array_offset_alias(alias, span)?;
            }
            let source_value = self.read_named(source).unwrap_or(Value::Null);
            let alias = ArrayOffsetAlias {
                root: ArrayOffsetAliasRoot::GlobalArray {
                    name: global_name.to_string(),
                },
                keys: Vec::new(),
            };
            self.materialize_array_offset_alias(&alias, span)?;
            if !self.write_array_offset_alias(&alias, source_value) {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access("cannot bind global reference".to_string()),
                ));
            }
            self.bind_direct_alias_group_to_array_offset_alias(source, alias);
            return Ok(());
        }

        let source_cell = self
            .read_cell(source)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_variable(source)))?;
        self.global_storage()
            .borrow_mut()
            .insert(global_name.to_string(), source_cell);
        Ok(())
    }

    fn bind_static_to_cell(&mut self, target: &str, cell: VariableCell) {
        self.array_offset_aliases.remove(target);
        self.routed_storage(target)
            .borrow_mut()
            .insert(target.to_string(), cell);
    }

    fn bind_static_to_existing_array_offset(
        &mut self,
        target: &str,
        array_name: &str,
        key: ArrayKey,
        span: Span,
    ) -> CompileResult<()> {
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::StaticArray {
                name: array_name.to_string(),
            },
            keys: vec![key],
        };
        self.materialize_array_offset_alias(&alias, span)?;
        self.bind_static_to_array_offset_alias(target, alias);
        Ok(())
    }

    fn bind_static_to_existing_nested_array_offset(
        &mut self,
        target: &str,
        array_name: &str,
        keys: Vec<ArrayKey>,
        span: Span,
    ) -> CompileResult<()> {
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::StaticArray {
                name: array_name.to_string(),
            },
            keys,
        };
        self.materialize_array_offset_alias(&alias, span)?;
        self.bind_static_to_array_offset_alias(target, alias);
        Ok(())
    }

    fn bind_static_to_appended_array_offset(
        &mut self,
        target: &str,
        array_name: &str,
        keys: Vec<ArrayKey>,
        span: Span,
    ) -> CompileResult<()> {
        if array_name == "GLOBALS" {
            return self.bind_static_to_appended_global_array_offset(target, keys, span);
        }

        let mut array = match self.read_storage_named(array_name) {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            Self::append_nested_array_offset_alias(&mut array, &keys, Value::Null, span)?;
        self.write_storage_named(array_name, Value::Array(array));
        self.bind_static_to_array_offset_alias(
            target,
            ArrayOffsetAlias {
                root: ArrayOffsetAliasRoot::StaticArray {
                    name: array_name.to_string(),
                },
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn bind_static_to_appended_global_array_offset(
        &mut self,
        target: &str,
        keys: Vec<ArrayKey>,
        span: Span,
    ) -> CompileResult<()> {
        let (global_name, keys) = Self::split_globals_reference_path(keys, span)?;
        let root = ArrayOffsetAliasRoot::GlobalArray { name: global_name };
        let root_alias = ArrayOffsetAlias {
            root: root.clone(),
            keys: Vec::new(),
        };
        let mut array = match self.read_alias_root_value(&root_alias, span)? {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            Self::append_nested_array_offset_alias(&mut array, &keys, Value::Null, span)?;
        self.write_alias_root_value(&root_alias, Value::Array(array), span)?;
        self.bind_static_to_array_offset_alias(
            target,
            ArrayOffsetAlias {
                root,
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn bind_static_to_existing_context_object_property(
        &mut self,
        target: &str,
        object_name: &str,
        property: &str,
        current_class_id: Option<ClassId>,
        protected_class_ids: Vec<ClassId>,
        span: Span,
    ) -> CompileResult<()> {
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::ContextObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
                current_class_id,
                protected_class_ids,
            },
            keys: Vec::new(),
        };
        self.read_alias_root_value(&alias, span)?;
        self.bind_static_to_array_offset_alias(target, alias);
        Ok(())
    }

    fn bind_static_to_dynamic_object_property(
        &mut self,
        target: &str,
        object_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<()> {
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::PublicObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
            },
            keys: Vec::new(),
        };

        match self.read_storage_named(object_name) {
            Some(Value::Object(object)) => match object.read_public_property(property) {
                Ok(_) => {}
                Err(error)
                    if matches!(error.kind(), RuntimeErrorKind::UndefinedProperty { .. }) =>
                {
                    object
                        .write_dynamic_public_property(property, Value::Null)
                        .map_err(|error| runtime_error(span, error))?;
                }
                Err(error) => return Err(runtime_error(span, error)),
            },
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
            None => {
                return Err(runtime_error(
                    span,
                    RuntimeError::undefined_variable(object_name),
                ));
            }
        }

        self.bind_static_to_array_offset_alias(target, alias);
        Ok(())
    }

    fn bind_static_to_array_offset_alias(&mut self, target: &str, alias: ArrayOffsetAlias) {
        self.bind_static_to_array_offset_aliases(target, vec![alias]);
    }

    fn bind_static_to_array_offset_aliases(
        &mut self,
        target: &str,
        aliases: Vec<ArrayOffsetAlias>,
    ) {
        self.routed_storage(target).borrow_mut().remove(target);
        self.array_offset_aliases
            .insert(target.to_string(), aliases);
    }

    fn is_static_bound_to_array_offset(
        &self,
        target: &str,
        array_name: &str,
        key: &ArrayKey,
    ) -> bool {
        matches!(
            self.array_offset_aliases.get(target),
            Some(aliases) if aliases.iter().any(|alias| matches!(
                alias,
                ArrayOffsetAlias {
                    root: ArrayOffsetAliasRoot::StaticArray { name },
                    keys,
                } if name == array_name && keys.as_slice() == std::slice::from_ref(key)
            ))
        )
    }

    fn is_array_offset_alias_name(&self, name: &str) -> bool {
        self.array_offset_aliases.contains_key(name)
    }

    fn remove_static_root_from_array_offset_aliases(&mut self, root_name: &str) {
        let alias_names: Vec<String> = self.array_offset_aliases.keys().cloned().collect();

        for alias_name in alias_names {
            let Some(existing_aliases) = self.array_offset_aliases.get(&alias_name).cloned() else {
                continue;
            };
            let fallback_value = existing_aliases
                .iter()
                .find(|alias| {
                    matches!(
                        &alias.root,
                        ArrayOffsetAliasRoot::StaticArray { name } if name == root_name
                    )
                })
                .and_then(|alias| self.read_array_offset_alias(alias));
            let aliases: Vec<_> = existing_aliases
                .into_iter()
                .filter(|alias| {
                    !matches!(
                        &alias.root,
                        ArrayOffsetAliasRoot::StaticArray { name } if name == root_name
                    )
                })
                .collect();

            if aliases.is_empty() {
                self.array_offset_aliases.remove(&alias_name);
                if let Some(value) = fallback_value {
                    self.write_storage_named(&alias_name, value);
                }
            } else {
                self.array_offset_aliases.insert(alias_name, aliases);
            }
        }
    }

    fn public_object_property_root_alias_fallbacks(
        &self,
        object_name: &str,
        property: &str,
    ) -> HashMap<String, Value> {
        self.array_offset_aliases
            .iter()
            .filter_map(|(alias_name, aliases)| {
                let root_alias = aliases
                    .iter()
                    .find(|alias| alias.root.matches_object_property(object_name, property))?;
                self.read_array_offset_alias(root_alias)
                    .map(|value| (alias_name.clone(), value))
            })
            .collect()
    }

    fn public_object_roots_alias_fallbacks(&self, object_name: &str) -> HashMap<String, Value> {
        self.array_offset_aliases
            .iter()
            .filter_map(|(alias_name, aliases)| {
                let root_alias = aliases
                    .iter()
                    .find(|alias| alias.root.matches_object(object_name))?;
                self.read_array_offset_alias(root_alias)
                    .map(|value| (alias_name.clone(), value))
            })
            .collect()
    }

    fn remove_public_object_property_root_from_array_offset_aliases(
        &mut self,
        object_name: &str,
        property: &str,
        fallbacks: &HashMap<String, Value>,
    ) {
        let alias_names: Vec<String> = self.array_offset_aliases.keys().cloned().collect();

        for alias_name in alias_names {
            let Some(existing_aliases) = self.array_offset_aliases.get(&alias_name).cloned() else {
                continue;
            };
            let aliases: Vec<_> = existing_aliases
                .into_iter()
                .filter(|alias| {
                    !(alias.root.matches_object_property(object_name, property)
                        && !alias.keys.is_empty())
                })
                .collect();

            if aliases.is_empty() {
                self.array_offset_aliases.remove(&alias_name);
                if let Some(value) = fallbacks.get(&alias_name) {
                    self.write_storage_named(&alias_name, value.clone());
                }
            } else {
                self.array_offset_aliases.insert(alias_name, aliases);
            }
        }
    }

    fn remove_public_object_roots_from_array_offset_aliases(
        &mut self,
        object_name: &str,
        fallbacks: &HashMap<String, Value>,
    ) {
        let alias_names: Vec<String> = self.array_offset_aliases.keys().cloned().collect();

        for alias_name in alias_names {
            let Some(existing_aliases) = self.array_offset_aliases.get(&alias_name).cloned() else {
                continue;
            };
            let aliases: Vec<_> = existing_aliases
                .into_iter()
                .filter(|alias| !alias.root.matches_object(object_name))
                .collect();

            if aliases.is_empty() {
                self.array_offset_aliases.remove(&alias_name);
                if let Some(value) = fallbacks.get(&alias_name) {
                    self.write_storage_named(&alias_name, value.clone());
                }
            } else {
                self.array_offset_aliases.insert(alias_name, aliases);
            }
        }
    }

    fn mirror_static_array_offset_aliases_from_copy(
        &mut self,
        target_name: &str,
        source_name: &str,
    ) {
        if target_name == source_name {
            return;
        }

        let additions: Vec<(String, ArrayOffsetAlias)> = self
            .array_offset_aliases
            .iter()
            .flat_map(|(alias_name, aliases)| {
                aliases.iter().filter_map(move |alias| match &alias.root {
                    ArrayOffsetAliasRoot::StaticArray { name } if name == source_name => Some((
                        alias_name.clone(),
                        ArrayOffsetAlias {
                            root: ArrayOffsetAliasRoot::StaticArray {
                                name: target_name.to_string(),
                            },
                            keys: alias.keys.clone(),
                        },
                    )),
                    _ => None,
                })
            })
            .collect();

        for (alias_name, alias) in additions {
            let aliases = self.array_offset_aliases.entry(alias_name).or_default();
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }
    }

    fn mirror_public_object_property_array_offset_aliases_from_copy(
        &mut self,
        target_name: &str,
        object_name: &str,
        property: &str,
    ) {
        let additions: Vec<(String, ArrayOffsetAlias)> = self
            .array_offset_aliases
            .iter()
            .flat_map(|(alias_name, aliases)| {
                aliases.iter().filter_map(move |alias| match &alias.root {
                    root if root.matches_object_property(object_name, property) => Some((
                        alias_name.clone(),
                        ArrayOffsetAlias {
                            root: ArrayOffsetAliasRoot::StaticArray {
                                name: target_name.to_string(),
                            },
                            keys: alias.keys.clone(),
                        },
                    )),
                    _ => None,
                })
            })
            .collect();

        for (alias_name, alias) in additions {
            let aliases = self.array_offset_aliases.entry(alias_name).or_default();
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }
    }

    fn mirror_object_property_aliases_from_clone(
        &mut self,
        target_object_name: &str,
        source_object_name: &str,
    ) {
        if target_object_name == source_object_name {
            return;
        }

        let additions: Vec<(String, ArrayOffsetAlias)> = self
            .array_offset_aliases
            .iter()
            .flat_map(|(alias_name, aliases)| {
                aliases.iter().filter_map(move |alias| match &alias.root {
                    ArrayOffsetAliasRoot::PublicObjectProperty { object, property }
                        if object == source_object_name =>
                    {
                        Some((
                            alias_name.clone(),
                            ArrayOffsetAlias {
                                root: ArrayOffsetAliasRoot::PublicObjectProperty {
                                    object: target_object_name.to_string(),
                                    property: property.clone(),
                                },
                                keys: alias.keys.clone(),
                            },
                        ))
                    }
                    ArrayOffsetAliasRoot::ContextObjectProperty {
                        object,
                        property,
                        current_class_id,
                        protected_class_ids,
                    } if object == source_object_name => Some((
                        alias_name.clone(),
                        ArrayOffsetAlias {
                            root: ArrayOffsetAliasRoot::ContextObjectProperty {
                                object: target_object_name.to_string(),
                                property: property.clone(),
                                current_class_id: *current_class_id,
                                protected_class_ids: protected_class_ids.clone(),
                            },
                            keys: alias.keys.clone(),
                        },
                    )),
                    _ => None,
                })
            })
            .collect();

        for (alias_name, alias) in additions {
            let aliases = self.array_offset_aliases.entry(alias_name).or_default();
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }
        }
    }

    fn sync_array_offset_aliases_for_static_root(&mut self, root_name: &str) {
        let syncs: Vec<(String, Value)> = self
            .array_offset_aliases
            .iter()
            .filter_map(|(alias_name, aliases)| {
                let touched_alias = aliases.iter().find(|alias| {
                    matches!(
                        &alias.root,
                        ArrayOffsetAliasRoot::StaticArray { name } if name == root_name
                    )
                })?;
                self.read_array_offset_alias(touched_alias)
                    .map(|value| (alias_name.clone(), value))
            })
            .collect();

        for (alias_name, value) in syncs {
            if let Some(aliases) = self.array_offset_aliases.get(&alias_name).cloned() {
                if !self.write_array_offset_aliases(&aliases, value) {
                    self.array_offset_aliases.remove(&alias_name);
                }
            }
        }
    }

    fn sync_array_offset_aliases_for_object_property_root(
        &mut self,
        object_name: &str,
        property: &str,
    ) {
        let syncs: Vec<(String, Value)> = self
            .array_offset_aliases
            .iter()
            .filter_map(|(alias_name, aliases)| {
                let touched_alias = aliases
                    .iter()
                    .find(|alias| alias.root.matches_object_property(object_name, property))?;
                self.read_array_offset_alias(touched_alias)
                    .map(|value| (alias_name.clone(), value))
            })
            .collect();

        for (alias_name, value) in syncs {
            if let Some(aliases) = self.array_offset_aliases.get(&alias_name).cloned() {
                if !self.write_array_offset_aliases(&aliases, value) {
                    self.array_offset_aliases.remove(&alias_name);
                }
            }
        }
    }

    fn sync_array_offset_aliases_for_global_root(&mut self, global_name: &str) {
        let syncs: Vec<(String, Value)> = self
            .array_offset_aliases
            .iter()
            .filter_map(|(alias_name, aliases)| {
                let touched_alias = aliases.iter().find(|alias| {
                    matches!(
                        &alias.root,
                        ArrayOffsetAliasRoot::GlobalArray { name } if name == global_name
                    )
                })?;
                self.read_array_offset_alias(touched_alias)
                    .map(|value| (alias_name.clone(), value))
            })
            .collect();

        for (alias_name, value) in syncs {
            if let Some(aliases) = self.array_offset_aliases.get(&alias_name).cloned() {
                if !self.write_array_offset_aliases(&aliases, value) {
                    self.array_offset_aliases.remove(&alias_name);
                }
            }
        }
    }

    fn bind_array_offset_to_static_source(
        &mut self,
        array_name: &str,
        key: ArrayKey,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(array_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::StaticArray {
                name: array_name.to_string(),
            },
            keys: vec![key],
        };
        self.materialize_array_offset_alias(&alias, span)?;
        if !self.write_array_offset_alias(&alias, source_value) {
            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot bind missing array offset".to_string()),
            ));
        }
        self.bind_direct_alias_group_to_array_offset_alias(source_name, alias);
        Ok(())
    }

    fn bind_nested_array_offset_to_static_source(
        &mut self,
        array_name: &str,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(array_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::StaticArray {
                name: array_name.to_string(),
            },
            keys,
        };
        self.materialize_array_offset_alias(&alias, span)?;
        if !self.write_array_offset_alias(&alias, source_value) {
            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot bind missing array offset".to_string()),
            ));
        }
        self.bind_direct_alias_group_to_array_offset_alias(source_name, alias);
        Ok(())
    }

    fn bind_global_nested_array_offset_to_static_source(
        &mut self,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source("GLOBALS", source_name, span)?;
        let (global_name, keys) = Self::split_globals_reference_path(keys, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::GlobalArray { name: global_name },
            keys,
        };
        self.materialize_array_offset_alias(&alias, span)?;
        if !self.write_array_offset_alias(&alias, source_value) {
            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot bind missing array offset".to_string()),
            ));
        }
        self.bind_direct_alias_group_to_array_offset_alias(source_name, alias);
        Ok(())
    }

    fn append_array_offset_to_static_source(
        &mut self,
        array_name: &str,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(array_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let mut array = match self.read_storage_named(array_name) {
            Some(Value::Array(array)) => array,
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
        let key = array
            .append(source_value)
            .map_err(|error| runtime_error(span, error))?;
        self.write_storage_named(array_name, Value::Array(array));
        self.bind_direct_alias_group_to_array_offset_alias(
            source_name,
            ArrayOffsetAlias {
                root: ArrayOffsetAliasRoot::StaticArray {
                    name: array_name.to_string(),
                },
                keys: vec![key],
            },
        );
        Ok(())
    }

    fn bind_object_property_array_offset_to_static_source(
        &mut self,
        object_name: &str,
        property: &str,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(object_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::PublicObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
            },
            keys,
        };
        self.materialize_array_offset_alias(&alias, span)?;
        if !self.write_array_offset_alias(&alias, source_value) {
            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot bind missing array offset".to_string()),
            ));
        }
        self.bind_direct_alias_group_to_array_offset_alias(source_name, alias);
        Ok(())
    }

    fn append_object_property_array_offset_to_static_source(
        &mut self,
        object_name: &str,
        property: &str,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(object_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let root = ArrayOffsetAliasRoot::PublicObjectProperty {
            object: object_name.to_string(),
            property: property.to_string(),
        };
        let root_alias = ArrayOffsetAlias {
            root: root.clone(),
            keys: Vec::new(),
        };
        let mut array = match self.read_alias_root_value(&root_alias, span)? {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            Self::append_nested_array_offset_alias(&mut array, &keys, source_value, span)?;
        self.write_alias_root_value(&root_alias, Value::Array(array), span)?;
        self.bind_direct_alias_group_to_array_offset_alias(
            source_name,
            ArrayOffsetAlias {
                root,
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn append_nested_array_offset_to_static_source(
        &mut self,
        array_name: &str,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source(array_name, source_name, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let mut array = match self.read_storage_named(array_name) {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            Self::append_nested_array_offset_alias(&mut array, &keys, source_value, span)?;
        self.write_storage_named(array_name, Value::Array(array));
        self.bind_direct_alias_group_to_array_offset_alias(
            source_name,
            ArrayOffsetAlias {
                root: ArrayOffsetAliasRoot::StaticArray {
                    name: array_name.to_string(),
                },
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn append_global_nested_array_offset_to_static_source(
        &mut self,
        keys: Vec<ArrayKey>,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        self.ensure_array_offset_reference_target_source("GLOBALS", source_name, span)?;
        let (global_name, keys) = Self::split_globals_reference_path(keys, span)?;

        let source_value = self.read_named(source_name).unwrap_or(Value::Null);
        let root = ArrayOffsetAliasRoot::GlobalArray { name: global_name };
        let root_alias = ArrayOffsetAlias {
            root: root.clone(),
            keys: Vec::new(),
        };
        let mut array = match self.read_alias_root_value(&root_alias, span)? {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            Self::append_nested_array_offset_alias(&mut array, &keys, source_value, span)?;
        self.write_alias_root_value(&root_alias, Value::Array(array), span)?;
        self.bind_direct_alias_group_to_array_offset_alias(
            source_name,
            ArrayOffsetAlias {
                root,
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn split_globals_reference_path(
        keys: Vec<ArrayKey>,
        span: Span,
    ) -> CompileResult<(String, Vec<ArrayKey>)> {
        let Some((first, rest)) = keys.split_first() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "$GLOBALS",
                    "nested reference bindings require a string-keyed root global",
                ),
            ));
        };
        let ArrayKey::String(global_name) = first else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "$GLOBALS",
                    "only string-keyed nested reference bindings are implemented",
                ),
            ));
        };
        Ok((global_name.clone(), rest.to_vec()))
    }

    fn ensure_array_offset_reference_target_source(
        &self,
        array_name: &str,
        source_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        if array_name == source_name {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "array-offset reference targets cannot use the same direct variable as source and array root in the current subset",
                ),
            ));
        }

        Ok(())
    }

    fn bind_direct_alias_group_to_array_offset_alias(
        &mut self,
        source_name: &str,
        alias: ArrayOffsetAlias,
    ) {
        if let Some(existing_aliases) = self.array_offset_aliases.get(source_name).cloned() {
            let mut aliases = existing_aliases.clone();
            if !aliases.contains(&alias) {
                aliases.push(alias);
            }

            let names: Vec<String> = self
                .array_offset_aliases
                .iter()
                .filter_map(|(candidate, candidate_aliases)| {
                    if *candidate_aliases == existing_aliases {
                        Some(candidate.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for name in names {
                self.bind_static_to_array_offset_aliases(&name, aliases.clone());
            }
            return;
        }

        let names = self.direct_names_sharing_cell(source_name);
        let names = if names.is_empty() {
            vec![source_name.to_string()]
        } else {
            names
        };

        for name in names {
            self.bind_static_to_array_offset_alias(&name, alias.clone());
        }
    }

    fn direct_names_sharing_cell(&self, name: &str) -> Vec<String> {
        let Some(source_cell) = self.read_cell(name) else {
            return Vec::new();
        };
        let mut names: Vec<_> = self
            .routed_storage(name)
            .borrow()
            .iter()
            .filter_map(|(candidate, cell)| {
                if Rc::ptr_eq(cell, &source_cell) {
                    Some(candidate.clone())
                } else {
                    None
                }
            })
            .collect();
        names.sort();
        names
    }

    fn materialize_array_offset_alias(
        &mut self,
        alias: &ArrayOffsetAlias,
        span: Span,
    ) -> CompileResult<()> {
        if alias.keys.is_empty() {
            if self.read_alias_root_value(alias, span)?.is_none() {
                self.write_alias_root_value(alias, Value::Null, span)?;
            }
            return Ok(());
        }

        let mut array = match self.read_alias_root_value(alias, span)? {
            Some(Value::Array(array)) => array,
            Some(Value::Null) | None => PhpArray::new(),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(format!(
                        "cannot read offset on {}",
                        other.type_name()
                    )),
                ));
            }
        };
        Self::materialize_nested_array_offset_alias(&mut array, &alias.keys, span)?;
        self.write_alias_root_value(alias, Value::Array(array), span)
    }

    fn read_array_offset_alias(&self, alias: &ArrayOffsetAlias) -> Option<Value> {
        if alias.keys.is_empty() {
            return self.read_alias_root_value(alias, Span::new(0, 0)).ok()?;
        }

        match self.read_alias_root_value(alias, Span::new(0, 0)).ok()? {
            Some(Value::Array(array)) => Self::read_nested_array_offset_alias(&array, &alias.keys),
            _ => None,
        }
    }

    fn write_array_offset_alias(&mut self, alias: &ArrayOffsetAlias, value: Value) -> bool {
        if alias.keys.is_empty() {
            return self
                .write_alias_root_value(alias, value, Span::new(0, 0))
                .is_ok();
        }

        let Ok(Some(Value::Array(mut array))) = self.read_alias_root_value(alias, Span::new(0, 0))
        else {
            return false;
        };

        if !Self::write_nested_array_offset_alias(&mut array, &alias.keys, value) {
            return false;
        }
        self.write_alias_root_value(alias, Value::Array(array), Span::new(0, 0))
            .is_ok()
    }

    fn write_array_offset_aliases(&mut self, aliases: &[ArrayOffsetAlias], value: Value) -> bool {
        if aliases.is_empty() {
            return false;
        }

        aliases
            .iter()
            .all(|alias| self.write_array_offset_alias(alias, value.clone()))
    }

    fn read_alias_root_value(
        &self,
        alias: &ArrayOffsetAlias,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        match &alias.root {
            ArrayOffsetAliasRoot::StaticArray { name } => Ok(self.read_storage_named(name)),
            ArrayOffsetAliasRoot::GlobalArray { name } => Ok(self.read_global_name(name)),
            ArrayOffsetAliasRoot::PublicObjectProperty { object, property } => {
                match self.read_storage_named(object) {
                    Some(Value::Object(object)) => object
                        .read_public_property(property)
                        .map(Some)
                        .map_err(|error| runtime_error(span, error)),
                    Some(other) => Err(runtime_error(
                        span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot read property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                    None => Err(runtime_error(
                        span,
                        RuntimeError::undefined_variable(object),
                    )),
                }
            }
            ArrayOffsetAliasRoot::ContextObjectProperty {
                object,
                property,
                current_class_id,
                protected_class_ids,
            } => match self.read_storage_named(object) {
                Some(Value::Object(object)) => object
                    .read_property_from_context(property, *current_class_id, protected_class_ids)
                    .map(Some)
                    .map_err(|error| runtime_error(span, error)),
                Some(other) => Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_variable(object),
                )),
            },
        }
    }

    fn write_alias_root_value(
        &mut self,
        alias: &ArrayOffsetAlias,
        value: Value,
        span: Span,
    ) -> CompileResult<()> {
        match &alias.root {
            ArrayOffsetAliasRoot::StaticArray { name } => {
                self.write_storage_named(name, value);
                Ok(())
            }
            ArrayOffsetAliasRoot::GlobalArray { name } => {
                self.write_global_name(name, value);
                Ok(())
            }
            ArrayOffsetAliasRoot::PublicObjectProperty { object, property } => {
                match self.read_storage_named(object) {
                    Some(Value::Object(object)) => object
                        .write_public_property(property, value)
                        .map_err(|error| runtime_error(span, error)),
                    Some(other) => Err(runtime_error(
                        span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot write property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                    None => Err(runtime_error(
                        span,
                        RuntimeError::undefined_variable(object),
                    )),
                }
            }
            ArrayOffsetAliasRoot::ContextObjectProperty {
                object,
                property,
                current_class_id,
                protected_class_ids,
            } => match self.read_storage_named(object) {
                Some(Value::Object(object)) => object
                    .write_property_from_context(
                        property,
                        value,
                        *current_class_id,
                        protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error)),
                Some(other) => Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot write property ${property} on {}",
                        other.type_name()
                    )),
                )),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_variable(object),
                )),
            },
        }
    }

    fn materialize_nested_array_offset_alias(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        span: Span,
    ) -> CompileResult<()> {
        let Some((key, rest)) = keys.split_first() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "array-offset reference aliases require at least one key",
                ),
            ));
        };

        if rest.is_empty() {
            if array.get_slot(key.clone()).is_none() {
                array.insert(key.clone(), Value::Null);
            }
            return Ok(());
        }

        let mut child = match array.get(key.clone()).cloned() {
            Some(Value::Array(child)) => child,
            Some(Value::Null) | None => PhpArray::new(),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(format!(
                        "cannot read offset on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        Self::materialize_nested_array_offset_alias(&mut child, rest, span)?;
        array.insert(key.clone(), Value::Array(child));
        Ok(())
    }

    fn read_nested_array_offset_alias(array: &PhpArray, keys: &[ArrayKey]) -> Option<Value> {
        let (key, rest) = keys.split_first()?;
        let value = array.get(key.clone())?.clone();
        if rest.is_empty() {
            return Some(value);
        }

        match value {
            Value::Array(child) => Self::read_nested_array_offset_alias(&child, rest),
            _ => None,
        }
    }

    fn write_nested_array_offset_alias(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        value: Value,
    ) -> bool {
        let Some((key, rest)) = keys.split_first() else {
            return false;
        };

        if rest.is_empty() {
            if array.get_slot(key.clone()).is_none() {
                return false;
            }
            array.insert(key.clone(), value);
            return true;
        }

        let Some(Value::Array(mut child)) = array.get(key.clone()).cloned() else {
            return false;
        };
        if !Self::write_nested_array_offset_alias(&mut child, rest, value) {
            return false;
        }
        array.insert(key.clone(), Value::Array(child));
        true
    }

    fn append_nested_array_offset_alias(
        array: &mut PhpArray,
        keys: &[ArrayKey],
        value: Value,
        span: Span,
    ) -> CompileResult<Vec<ArrayKey>> {
        let Some((key, rest)) = keys.split_first() else {
            let appended = array
                .append(value)
                .map_err(|error| runtime_error(span, error))?;
            return Ok(vec![appended]);
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

        let mut alias_keys = vec![key.clone()];
        alias_keys.extend(Self::append_nested_array_offset_alias(
            &mut child, rest, value, span,
        )?);
        array.insert(key.clone(), Value::Array(child));
        Ok(alias_keys)
    }

    fn read_cell(&self, name: &str) -> Option<VariableCell> {
        self.routed_storage(name).borrow().get(name).cloned()
    }

    fn routed_storage(&self, name: &str) -> &SymbolStorage {
        if self.imported_globals.contains(name)
            || (is_auto_global_name(name) && self.global_symbols.is_some())
        {
            self.global_symbols.as_ref().unwrap_or(&self.symbols)
        } else {
            &self.symbols
        }
    }

    fn name_routes_to_global_storage(&self, name: &str) -> bool {
        self.global_symbols.is_none()
            || self.imported_globals.contains(name)
            || is_auto_global_name(name)
    }

    fn global_storage(&self) -> &SymbolStorage {
        self.global_symbols.as_ref().unwrap_or(&self.symbols)
    }
}

fn value_cell(value: Value) -> VariableCell {
    Rc::new(RefCell::new(value))
}

fn bind_foreach_lingering_reference(
    scope: &mut SymbolTable,
    value: &str,
    array_name: &str,
    key: Option<ArrayKey>,
    span: Span,
) -> CompileResult<()> {
    if let Some(key) = key {
        scope.bind_static_to_existing_array_offset(value, array_name, key, span)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ReferenceBinding {
    param_name: String,
    target: ReferenceBindingTarget,
}

#[derive(Debug, Clone)]
enum ReferenceBindingTarget {
    CallerCell(VariableCell),
    PublicObjectPropertyArrayOffset(ArrayOffsetAlias),
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
        let mut method_signatures = HashMap::new();
        let mut abstract_classes = HashSet::new();
        let mut final_classes = HashSet::new();
        let mut abstract_methods = HashSet::new();
        let mut final_methods = HashMap::new();
        let mut class_constants = HashMap::new();
        let mut static_properties = HashMap::new();
        let instance_property_defaults = HashMap::new();
        let mut classes = PhpClassTable::with_core_classes();
        seed_core_class_constant_runtime_tables(&classes, &mut class_constants);
        for stmt in &program.statements {
            match stmt {
                Stmt::Function(function) if !function.is_nested => {
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
                    let class_id = register_class_name(&mut classes, class)?;
                    if class.is_final {
                        final_classes.insert(class_id);
                    }
                    register_final_method_markers(&mut final_methods, class_id, class);
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
                let class_id = register_class_members(
                    &mut classes,
                    &final_classes,
                    &abstract_methods,
                    &final_methods,
                    &method_signatures,
                    &interface_lookup,
                    &trait_lookup,
                    class,
                )?;
                register_class_member_runtime_tables(
                    &mut class_constants,
                    &mut static_properties,
                    &mut methods,
                    &mut method_signatures,
                    &mut abstract_methods,
                    &trait_lookup,
                    class_id,
                    class,
                )?;
                if class.is_abstract {
                    abstract_classes.insert(class_id);
                }
            }
        }

        let mut interpreter = Self {
            functions,
            methods,
            method_signatures,
            abstract_classes,
            final_classes,
            abstract_methods,
            final_methods,
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
            ignore_user_abort: false,
            ini_values: HashMap::new(),
            error_handler: None,
            error_handler_mask: None,
            mysqli_report_mode: PHP_MYSQLI_REPORT_ERROR | PHP_MYSQLI_REPORT_STRICT,
            mysqli_results: HashMap::new(),
            mysqli_pending_results: HashMap::new(),
            mysqli_pending_result_queues: HashMap::new(),
            mysqli_options: HashMap::new(),
            mysqli_wp_options: HashMap::new(),
            mysqli_transactions: HashMap::new(),
            mysqli_affected_rows: HashMap::new(),
            mysqli_insert_ids: HashMap::new(),
            mysqli_statements: HashMap::new(),
            source_file,
            max_execution_steps: options.max_execution_steps,
            trace_includes: options.trace_includes,
            execution_steps: 0,
            call_depth: 0,
            next_object_id: 1,
            next_closure_id: 1,
            next_foreach_temp_id: 1,
            active_foreach_references: Vec::new(),
            function_context: Vec::new(),
            class_context: Vec::new(),
            called_class_context: Vec::new(),
            response_headers: Vec::new(),
            output_buffers: Vec::new(),
            stdout: String::new(),
            exit_signal: None,
        };
        interpreter.initialize_superglobals();
        interpreter.initialize_static_property_defaults(program)?;
        interpreter.initialize_instance_property_defaults(program)?;
        Ok(interpreter)
    }

    fn next_foreach_temporary_array_name(&mut self) -> String {
        let id = self.next_foreach_temp_id;
        self.next_foreach_temp_id += 1;
        format!("\0foreach_ref_temp_{id}")
    }

    fn is_temporary_by_reference_foreach_iterable(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Array { .. }
            | Expr::Include { .. }
            | Expr::Require { .. }
            | Expr::Cast { .. }
            | Expr::Ternary { .. }
            | Expr::ShortTernary { .. } => true,
            Expr::Call { name, .. } => match self.lookup_direct_function_call(name) {
                Some(Callable::User(function)) => !function.returns_by_reference,
                _ => true,
            },
            _ => false,
        }
    }

    fn initialize_superglobals(&mut self) {
        let mut server = PhpArray::new();
        server.insert("SERVER_SOFTWARE", Value::String("phpc".to_string()));
        server.insert("REQUEST_URI", Value::String("/".to_string()));
        server.insert("HTTP_HOST", Value::String("localhost".to_string()));
        server.insert("PHP_SELF", Value::String("/index.php".to_string()));
        server.insert("SCRIPT_NAME", Value::String("/index.php".to_string()));
        server.insert("SCRIPT_FILENAME", Value::String("/index.php".to_string()));
        server.insert("QUERY_STRING", Value::String(String::new()));

        self.global_symbols
            .borrow_mut()
            .insert("_SERVER".to_string(), value_cell(Value::Array(server)));
        self.global_symbols.borrow_mut().insert(
            "_COOKIE".to_string(),
            value_cell(Value::Array(PhpArray::new())),
        );
        self.global_symbols.borrow_mut().insert(
            "_GET".to_string(),
            value_cell(Value::Array(PhpArray::new())),
        );
        self.global_symbols.borrow_mut().insert(
            "_POST".to_string(),
            value_cell(Value::Array(PhpArray::new())),
        );
        self.global_symbols.borrow_mut().insert(
            "_REQUEST".to_string(),
            value_cell(Value::Array(PhpArray::new())),
        );
        self.global_symbols.borrow_mut().insert(
            "_FILES".to_string(),
            value_cell(Value::Array(PhpArray::new())),
        );
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
                Stmt::Function(function) if !function.is_nested => {
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
                    if class.is_final {
                        self.final_classes.insert(class_id);
                    }
                    register_final_method_markers(&mut self.final_methods, class_id, class);
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
            let class_id = register_class_members(
                &mut self.classes,
                &self.final_classes,
                &self.abstract_methods,
                &self.final_methods,
                &self.method_signatures,
                &self.interface_lookup,
                &self.trait_lookup,
                class,
            )?;
            register_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                &mut self.method_signatures,
                &mut self.abstract_methods,
                &self.trait_lookup,
                class_id,
                class,
            )?;
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
        if class.is_final {
            self.final_classes.insert(class_id);
        }
        if let Err(error) = register_class_members(
            &mut self.classes,
            &self.final_classes,
            &self.abstract_methods,
            &self.final_methods,
            &self.method_signatures,
            &self.interface_lookup,
            &self.trait_lookup,
            class,
        ) {
            self.abstract_classes.remove(&class_id);
            self.final_classes.remove(&class_id);
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }
        register_final_method_markers(&mut self.final_methods, class_id, class);
        register_class_member_runtime_tables(
            &mut self.class_constants,
            &mut self.static_properties,
            &mut self.methods,
            &mut self.method_signatures,
            &mut self.abstract_methods,
            &self.trait_lookup,
            class_id,
            class,
        )?;
        if let Err(error) = self.initialize_static_property_defaults_for_class(class_id, class) {
            remove_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                &mut self.method_signatures,
                &mut self.abstract_methods,
                class_id,
            );
            self.abstract_classes.remove(&class_id);
            self.final_classes.remove(&class_id);
            self.final_methods
                .retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }
        if let Err(error) = self.initialize_instance_property_defaults_for_class(class_id, class) {
            remove_class_member_runtime_tables(
                &mut self.class_constants,
                &mut self.static_properties,
                &mut self.methods,
                &mut self.method_signatures,
                &mut self.abstract_methods,
                class_id,
            );
            self.instance_property_defaults
                .retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
            self.abstract_classes.remove(&class_id);
            self.final_classes.remove(&class_id);
            self.final_methods
                .retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
            self.classes.remove_last_declared_class(class_id);
            return Err(error);
        }

        Ok(())
    }

    fn register_nested_function_declaration(
        &mut self,
        function: &FunctionDecl,
    ) -> CompileResult<()> {
        let key = function.name.to_ascii_lowercase();
        if self.functions.contains_key(&key) {
            return Err(runtime_error(
                function.span,
                RuntimeError::duplicate_function(callable_name(&function.name)),
            ));
        }
        self.functions.insert(key, Rc::new(function.clone()));
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
            Flow::Normal | Flow::Return(_) => {
                self.flush_output_buffers();
                Ok(Execution {
                    stdout: self.stdout.clone(),
                    stderr: String::new(),
                    exit_code: 0,
                })
            }
            Flow::Exit(code) => {
                self.flush_output_buffers();
                Ok(Execution {
                    stdout: self.stdout.clone(),
                    stderr: String::new(),
                    exit_code: code,
                })
            }
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

    fn append_output(&mut self, output: &str) {
        if let Some(buffer) = self.output_buffers.last_mut() {
            buffer.push_str(output);
        } else {
            self.stdout.push_str(output);
        }
    }

    fn flush_output_buffers(&mut self) {
        while let Some(output) = self.output_buffers.pop() {
            self.append_output(&output);
        }
    }

    fn execute_statement(&mut self, stmt: &Stmt, scope: &mut SymbolTable) -> CompileResult<Flow> {
        self.tick(stmt.span())?;
        match stmt {
            Stmt::Namespace { .. } | Stmt::Use { .. } => Ok(Flow::Normal),
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.evaluate(expr, scope)?;
                    let output = self.value_to_echo_string(value, expr.span())?;
                    self.append_output(&output);
                }
                Ok(Flow::Normal)
            }
            Stmt::Print { expr, .. } => {
                let value = self.evaluate(expr, scope)?;
                let output = self.value_to_echo_string(value, expr.span())?;
                self.append_output(&output);
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
                    let array_name = match iterable {
                        Expr::Variable(name, _) => name.clone(),
                        expr if self.is_temporary_by_reference_foreach_iterable(expr) => {
                            let value = self.evaluate(expr, scope)?;
                            let Value::Array(array) = value else {
                                return Err(runtime_error(
                                    *span,
                                    RuntimeError::invalid_foreach(format!(
                                        "can only iterate arrays in the current subset, got {}",
                                        value.type_name()
                                    )),
                                ));
                            };
                            let temp_name = self.next_foreach_temporary_array_name();
                            scope.write_static(&temp_name, Value::Array(array));
                            temp_name
                        }
                        _ => {
                            return Err(runtime_error(
                                *span,
                                RuntimeError::unsupported_call(
                                    "foreach",
                                    "by-reference iteration currently requires a direct array variable or temporary array expression",
                                ),
                            ));
                        }
                    };
                    match scope.read_static(&array_name, *span)? {
                        Value::Array(_) => {}
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
                    let mut position = 0usize;
                    let mut lingering_reference_key = None;

                    loop {
                        let entry_key = match scope.read_static(&array_name, *span)? {
                            Value::Array(array) => {
                                let Some(entry) = array.entries().get(position) else {
                                    break;
                                };
                                entry.key.clone()
                            }
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
                        self.tick(*span)?;

                        if let Some(key) = key {
                            scope.write_static(key, value_from_array_key(&entry_key));
                        }
                        scope.bind_static_to_existing_array_offset(
                            value,
                            &array_name,
                            entry_key.clone(),
                            *span,
                        )?;
                        self.active_foreach_references.push(ActiveForeachReference {
                            array_name: array_name.clone(),
                            value_name: value.clone(),
                            key: entry_key.clone(),
                        });
                        let flow_result = self.execute_statements(body, scope);
                        self.active_foreach_references.pop();
                        let flow = flow_result?;

                        let value_still_bound =
                            scope.is_static_bound_to_array_offset(value, &array_name, &entry_key);
                        let next_position = match scope.read_static(&array_name, *span)? {
                            Value::Array(array) => {
                                let current_position = array
                                    .entries()
                                    .iter()
                                    .position(|entry| entry.key == entry_key);
                                lingering_reference_key =
                                    if value_still_bound && current_position.is_some() {
                                        Some(entry_key.clone())
                                    } else {
                                        None
                                    };
                                match current_position {
                                    Some(current_position) if current_position > position => {
                                        position
                                    }
                                    Some(current_position) => current_position + 1,
                                    None => {
                                        lingering_reference_key = None;
                                        position
                                    }
                                }
                            }
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
                        position = next_position;

                        match flow {
                            Flow::Normal => {}
                            Flow::Continue { depth, .. } if depth <= 1 => {}
                            Flow::Continue {
                                depth,
                                span: flow_span,
                            } => {
                                bind_foreach_lingering_reference(
                                    scope,
                                    value,
                                    &array_name,
                                    lingering_reference_key,
                                    *span,
                                )?;
                                return Ok(Flow::Continue {
                                    depth: depth - 1,
                                    span: flow_span,
                                });
                            }
                            Flow::Break { depth, .. } if depth <= 1 => break,
                            Flow::Break {
                                depth,
                                span: flow_span,
                            } => {
                                bind_foreach_lingering_reference(
                                    scope,
                                    value,
                                    &array_name,
                                    lingering_reference_key,
                                    *span,
                                )?;
                                return Ok(Flow::Break {
                                    depth: depth - 1,
                                    span: flow_span,
                                });
                            }
                            Flow::Goto {
                                label,
                                span: flow_span,
                            } => {
                                bind_foreach_lingering_reference(
                                    scope,
                                    value,
                                    &array_name,
                                    lingering_reference_key,
                                    *span,
                                )?;
                                return Ok(Flow::Goto {
                                    label,
                                    span: flow_span,
                                });
                            }
                            flow @ (Flow::Return(_) | Flow::Exit(_)) => {
                                return Ok(flow);
                            }
                        }
                    }

                    bind_foreach_lingering_reference(
                        scope,
                        value,
                        &array_name,
                        lingering_reference_key,
                        *span,
                    )?;
                    return Ok(Flow::Normal);
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
                    scope.write_static(value, entry.value_cloned());
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
                self.execute_unset_object_property(object, property, *span, scope, true)?;
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
            Stmt::Function(function) => {
                if function.is_nested {
                    self.register_nested_function_declaration(function)?;
                }
                Ok(Flow::Normal)
            }
            Stmt::Interface(_) | Stmt::Trait(_) | Stmt::Enum(_) => Ok(Flow::Normal),
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
            } => self.execute_unset_object_property(object, property, span, scope, true),
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
        call_magic_on_missing: bool,
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
            if call_magic_on_missing {
                self.call_magic_property_method(object, "__unset", property, span)?;
            }
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
        self.execute_unset_object_property(object_name, &property_name, span, scope, false)
    }

    fn execute_unset_array_index(
        &mut self,
        name: &str,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let key = self.evaluate_array_key(index, scope)?;
        let foreach_detach = self
            .active_foreach_references
            .iter()
            .rev()
            .find(|active| active.array_name == name && active.key == key)
            .and_then(|active| {
                scope
                    .is_static_bound_to_array_offset(&active.value_name, name, &key)
                    .then(|| {
                        scope
                            .read_named(&active.value_name)
                            .map(|value| (active.value_name.clone(), value))
                    })
                    .flatten()
            });

        match scope.read_named(name) {
            Some(Value::Array(mut array)) => {
                array.remove(key.clone());
                scope.write_static(name, Value::Array(array));
                if let Some((value_name, value)) = foreach_detach {
                    scope.bind_static_to_cell(&value_name, value_cell(value));
                }
                Ok(())
            }
            Some(Value::Object(object)) => {
                self.call_array_access_method(
                    object,
                    "offsetUnset",
                    vec![Self::array_key_value(Some(key))],
                    span,
                )?;
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
            Value::Object(object)
                if keys.len() == 1
                    && self
                        .classes
                        .implements_interface(object.class_id(), "ArrayAccess") =>
            {
                self.call_array_access_method(
                    object.clone(),
                    "offsetUnset",
                    vec![Self::array_key_value(Some(keys[0].clone()))],
                    span,
                )?;
                Ok(())
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
            Expr::ObjectStaticProperty {
                target,
                property,
                span,
            } => self.evaluate_object_static_property(target, property, *span, scope),
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
        let class_name = self.resolve_new_class_name(class_name, span, scope)?;
        let (class_id, declared_class_name) = {
            let class = self
                .classes
                .lookup_class(&class_name)
                .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(&class_name)))?;
            (class.id(), class.name().to_string())
        };
        if declared_class_name.eq_ignore_ascii_case("PDO") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_object_instantiation(
                    declared_class_name,
                    "PDO connections, drivers, statements, and host database state are not implemented in the current subset",
                ),
            ));
        }
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
        ensure_supported_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, scope)?;

        self.call_user_function_with_checked_values(
            function,
            values,
            Some(object.clone()),
            Some(constructor_class_id),
            Some(object.class_id()),
            reference_bindings,
            Some(scope),
        )?;
        Ok(Value::Object(object))
    }

    fn resolve_new_class_name(
        &self,
        class_name: &NewClassName,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<String> {
        match class_name {
            NewClassName::Named(name) => Ok(name.clone()),
            NewClassName::DynamicVariable(name) => match scope.read_static(name, span)? {
                Value::String(class_name) => Ok(class_name),
                other => Err(runtime_error(
                    span,
                    RuntimeError::unsupported_object_instantiation(
                        "dynamic class name",
                        format!(
                            "dynamic class variable must contain a string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                )),
            },
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
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    scope.bind_static_to_static(name, source_name, span)?;
                } else if let ReferenceSource::ArrayIndex {
                    name: array_name,
                    index,
                    ..
                } = source
                {
                    let key = self.evaluate_array_key(index, scope)?;
                    self.reject_array_access_reference_source_if_needed(array_name, span, scope)?;
                    scope.bind_static_to_existing_array_offset(name, array_name, key, span)?;
                } else if let ReferenceSource::NestedArrayIndex {
                    name: array_name,
                    indices,
                    ..
                } = source
                {
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    self.reject_array_access_reference_source_if_needed(array_name, span, scope)?;
                    scope.bind_static_to_existing_nested_array_offset(
                        name, array_name, keys, span,
                    )?;
                } else if let ReferenceSource::ArrayAppend {
                    name: array_name,
                    indices,
                    ..
                } = source
                {
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    self.reject_array_access_reference_source_if_needed(array_name, span, scope)?;
                    scope.bind_static_to_appended_array_offset(name, array_name, keys, span)?;
                } else if let ReferenceSource::ObjectPropertyArrayIndex {
                    object,
                    property,
                    index,
                    ..
                } = source
                {
                    let key = self.evaluate_array_key(index, scope)?;
                    self.reject_object_property_array_access_reference_source_if_needed(
                        object, property, span, scope,
                    )?;
                    self.bind_static_to_context_object_property_array_offset(
                        name,
                        object,
                        property,
                        vec![key],
                        span,
                        scope,
                    )?;
                } else if let ReferenceSource::ObjectPropertyNestedArrayIndex {
                    object,
                    property,
                    indices,
                    ..
                } = source
                {
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    self.reject_object_property_array_access_reference_source_if_needed(
                        object, property, span, scope,
                    )?;
                    self.bind_static_to_context_object_property_array_offset(
                        name, object, property, keys, span, scope,
                    )?;
                } else if let ReferenceSource::ObjectPropertyArrayAppend {
                    object,
                    property,
                    indices,
                    ..
                } = source
                {
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    self.reject_object_property_array_access_reference_source_if_needed(
                        object, property, span, scope,
                    )?;
                    self.bind_static_to_appended_context_object_property_array_offset(
                        name, object, property, keys, span, scope,
                    )?;
                } else if let ReferenceSource::Property {
                    expr:
                        Expr::Property {
                            target, property, ..
                        },
                    ..
                } = source
                {
                    if let Expr::Variable(object, _) = target.as_ref() {
                        self.bind_static_to_context_property_or_magic_get(
                            name, object, property, span, scope,
                        )?;
                    } else {
                        let value = self
                            .evaluate_direct_variable_reference_source_value(source, span, scope)?;
                        scope.write_static(name, value);
                    }
                } else if let ReferenceSource::Property {
                    expr:
                        Expr::DynamicProperty {
                            target, property, ..
                        },
                    ..
                } = source
                {
                    if let Expr::Variable(object, _) = target.as_ref() {
                        let property =
                            self.evaluate_dynamic_property_name(property, span, scope)?;
                        self.bind_static_to_dynamic_property_or_magic_get(
                            name, object, &property, span, scope,
                        )?;
                    } else {
                        let value = self
                            .evaluate_direct_variable_reference_source_value(source, span, scope)?;
                        scope.write_static(name, value);
                    }
                } else if let ReferenceSource::MethodCall { expr, .. } = source {
                    let cell = self.evaluate_reference_return_call_cell(expr, span, scope)?;
                    scope.bind_static_to_cell(name, cell);
                } else {
                    let value =
                        self.evaluate_direct_variable_reference_source_value(source, span, scope)?;
                    scope.write_static(name, value);
                }
                Ok(())
            }
            AssignTarget::ArrayIndex {
                name,
                index: Some(index),
                ..
            } => {
                let key = self.evaluate_array_key(index, scope)?;
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_array_access_reference_target_if_needed(name, span, scope)?;
                    if name == "GLOBALS" {
                        let Some(global_name) = globals_offset_name(&key) else {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "$GLOBALS",
                                    "only string-keyed direct offset reference bindings are implemented",
                                ),
                            ));
                        };
                        scope.bind_global_name_to_static_source(global_name, source_name, span)?;
                        return Ok(());
                    }
                    scope.bind_array_offset_to_static_source(name, key, source_name, span)?;
                    return Ok(());
                }

                let value = self.evaluate_container_reference_source_value(source, span, scope)?;
                if name == "GLOBALS" {
                    let Some(global_name) = globals_offset_name(&key) else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "$GLOBALS",
                                "only string-keyed direct offset writes are implemented",
                            ),
                        ));
                    };
                    scope.write_global_name(global_name, value);
                    return Ok(());
                }
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
            AssignTarget::ArrayIndex {
                name, index: None, ..
            } => {
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_array_access_reference_target_if_needed(name, span, scope)?;
                    if name == "GLOBALS" {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "$GLOBALS",
                                "append reference binding through $GLOBALS is not implemented",
                            ),
                        ));
                    }
                    scope.append_array_offset_to_static_source(name, source_name, span)?;
                    return Ok(());
                }

                Err(unsupported())
            }
            AssignTarget::NestedArrayIndex { name, indices, .. } => {
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_array_access_reference_target_if_needed(name, span, scope)?;
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    if name == "GLOBALS" {
                        scope.bind_global_nested_array_offset_to_static_source(
                            keys,
                            source_name,
                            span,
                        )?;
                        return Ok(());
                    }
                    scope.bind_nested_array_offset_to_static_source(
                        name,
                        keys,
                        source_name,
                        span,
                    )?;
                    return Ok(());
                }

                Err(unsupported())
            }
            AssignTarget::NestedArrayAppend { name, indices, .. } => {
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_array_access_reference_target_if_needed(name, span, scope)?;
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    if name == "GLOBALS" {
                        scope.append_global_nested_array_offset_to_static_source(
                            keys,
                            source_name,
                            span,
                        )?;
                        return Ok(());
                    }
                    scope.append_nested_array_offset_to_static_source(
                        name,
                        keys,
                        source_name,
                        span,
                    )?;
                    return Ok(());
                }

                Err(unsupported())
            }
            AssignTarget::ObjectPropertyArrayIndex {
                object,
                property,
                indices,
                ..
            } => {
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_object_property_array_access_reference_target_if_needed(
                        object, property, span, scope,
                    )?;
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    scope.bind_object_property_array_offset_to_static_source(
                        object,
                        property,
                        keys,
                        source_name,
                        span,
                    )?;
                    return Ok(());
                }

                Err(unsupported())
            }
            AssignTarget::ObjectPropertyArrayAppend {
                object,
                property,
                indices,
                ..
            } => {
                if let ReferenceSource::Variable {
                    name: source_name, ..
                } = source
                {
                    self.reject_object_property_array_access_reference_target_if_needed(
                        object, property, span, scope,
                    )?;
                    let keys = indices
                        .iter()
                        .map(|index| self.evaluate_array_key(index, scope))
                        .collect::<CompileResult<Vec<_>>>()?;
                    scope.append_object_property_array_offset_to_static_source(
                        object,
                        property,
                        keys,
                        source_name,
                        span,
                    )?;
                    return Ok(());
                }

                Err(unsupported())
            }
            _ => Err(unsupported()),
        }
    }

    fn reject_array_access_reference_target_if_needed(
        &self,
        name: &str,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<()> {
        if let Some(value) = scope.read_named(name) {
            self.reject_array_access_reference_target_value(&value, span)?;
        }
        Ok(())
    }

    fn reject_array_access_reference_source_if_needed(
        &self,
        name: &str,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<()> {
        if let Some(value) = scope.read_named(name) {
            self.reject_array_access_reference_source_value(&value, span)?;
        }
        Ok(())
    }

    fn reject_object_property_array_access_reference_target_if_needed(
        &self,
        object_name: &str,
        property: &str,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<()> {
        let Some(Value::Object(object)) = scope.read_named(object_name) else {
            return Ok(());
        };
        if let Ok(value) = object.read_public_property(property) {
            self.reject_array_access_reference_target_value(&value, span)?;
        }
        Ok(())
    }

    fn reject_object_property_array_access_reference_source_if_needed(
        &self,
        object_name: &str,
        property: &str,
        span: Span,
        scope: &SymbolTable,
    ) -> CompileResult<()> {
        let Some(Value::Object(object)) = scope.read_named(object_name) else {
            return Ok(());
        };
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        if let Ok(value) =
            object.read_property_from_context(property, current_class_id, &protected_class_ids)
        {
            self.reject_array_access_reference_source_value(&value, span)?;
        }
        Ok(())
    }

    fn bind_static_to_context_object_property_array_offset(
        &self,
        target_name: &str,
        object_name: &str,
        property: &str,
        keys: Vec<ArrayKey>,
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
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let visibility = object
            .property_visibility_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?;

        let root = if visibility == Visibility::Public {
            ArrayOffsetAliasRoot::PublicObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
            }
        } else {
            ArrayOffsetAliasRoot::ContextObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
                current_class_id,
                protected_class_ids,
            }
        };
        let alias = ArrayOffsetAlias { root, keys };
        scope.materialize_array_offset_alias(&alias, span)?;
        scope.bind_static_to_array_offset_alias(target_name, alias);
        Ok(())
    }

    fn bind_static_to_appended_context_object_property_array_offset(
        &self,
        target_name: &str,
        object_name: &str,
        property: &str,
        keys: Vec<ArrayKey>,
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
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let visibility = object
            .property_visibility_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(span, error))?;

        let root = if visibility == Visibility::Public {
            ArrayOffsetAliasRoot::PublicObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
            }
        } else {
            ArrayOffsetAliasRoot::ContextObjectProperty {
                object: object_name.to_string(),
                property: property.to_string(),
                current_class_id,
                protected_class_ids,
            }
        };
        let root_alias = ArrayOffsetAlias {
            root: root.clone(),
            keys: Vec::new(),
        };
        let mut array = match scope.read_alias_root_value(&root_alias, span)? {
            Some(Value::Array(array)) => array,
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
        let alias_keys =
            SymbolTable::append_nested_array_offset_alias(&mut array, &keys, Value::Null, span)?;
        scope.write_alias_root_value(&root_alias, Value::Array(array), span)?;
        scope.bind_static_to_array_offset_alias(
            target_name,
            ArrayOffsetAlias {
                root,
                keys: alias_keys,
            },
        );
        Ok(())
    }

    fn reject_array_access_reference_target_value(
        &self,
        value: &Value,
        span: Span,
    ) -> CompileResult<()> {
        if let Value::Object(object) = value {
            if self
                .classes
                .implements_interface(object.class_id(), "ArrayAccess")
            {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "ArrayAccess object offsets cannot be assigned by reference in the current runtime",
                    ),
                ));
            }
        }
        Ok(())
    }

    fn reject_array_access_reference_source_value(
        &self,
        value: &Value,
        span: Span,
    ) -> CompileResult<()> {
        if let Value::Object(object) = value {
            if self
                .classes
                .implements_interface(object.class_id(), "ArrayAccess")
            {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "ArrayAccess offset reference sources require by-reference offsetGet() and reference containers, which are not implemented",
                    ),
                ));
            }
        }
        Ok(())
    }

    fn bind_static_to_context_property_or_magic_get(
        &mut self,
        target_name: &str,
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
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        match object.read_property_from_context(property, current_class_id, &protected_class_ids) {
            Ok(_) => {
                scope.bind_static_to_existing_context_object_property(
                    target_name,
                    object_name,
                    property,
                    current_class_id,
                    protected_class_ids,
                    span,
                )?;
                Ok(())
            }
            Err(error) if Self::is_undefined_property_error(&error) => {
                if let Some(cell) =
                    self.call_magic_get_reference_return_cell(object, property, span)?
                {
                    scope.bind_static_to_cell(target_name, cell);
                    Ok(())
                } else {
                    Err(runtime_error(span, error))
                }
            }
            Err(error) => Err(runtime_error(span, error)),
        }
    }

    fn bind_static_to_dynamic_property_or_magic_get(
        &mut self,
        target_name: &str,
        object_name: &str,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let object = match scope.read_static(object_name, span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} on {}",
                        other.type_name()
                    )),
                ));
            }
        };

        match object.read_public_property(property) {
            Ok(_) => {
                scope.bind_static_to_dynamic_object_property(
                    target_name,
                    object_name,
                    property,
                    span,
                )?;
                Ok(())
            }
            Err(error) if Self::is_undefined_property_error(&error) => {
                if let Some(cell) =
                    self.call_magic_get_reference_return_cell(object, property, span)?
                {
                    scope.bind_static_to_cell(target_name, cell);
                    Ok(())
                } else {
                    scope.bind_static_to_dynamic_object_property(
                        target_name,
                        object_name,
                        property,
                        span,
                    )
                }
            }
            Err(error) => Err(runtime_error(span, error)),
        }
    }

    fn evaluate_reference_return_call_cell(
        &mut self,
        expr: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
        match expr {
            Expr::Call {
                name,
                args,
                span: call_span,
            } => self.call_reference_return_function(name, args, *call_span, scope),
            Expr::MethodCall {
                target,
                method,
                args,
                span: call_span,
            } => self.call_reference_return_instance_method(target, method, args, *call_span, scope),
            Expr::StaticMethodCall {
                class_name,
                method,
                args,
                span: call_span,
            } => {
                self.call_reference_return_named_static_method(class_name, method, args, *call_span, scope)
            }
            Expr::SelfMethodCall {
                method,
                args,
                span: call_span,
            } => self.call_reference_return_self_method(method, args, *call_span, scope),
            Expr::ParentMethodCall {
                method,
                args,
                span: call_span,
            } => self.call_reference_return_parent_method(method, args, *call_span, scope),
            Expr::LateStaticMethodCall {
                method,
                args,
                span: call_span,
            } => self.call_reference_return_late_static_method(method, args, *call_span, scope),
            Expr::ObjectStaticMethodCall {
                target,
                method,
                args,
                span: call_span,
            } => self.call_reference_return_dynamic_static_method(
                target, method, args, *call_span, scope,
            ),
            _ => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "reference assignment",
                    "only direct function-call, object method-call, named static method-call, self:: static method-call, parent:: static method-call, static:: late-static method-call, and dynamic static receiver method-call reference-return sources are implemented in the current subset",
                ),
            )),
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
            ReferenceSource::ArrayIndex { name, index, .. } => {
                return self.reject_array_offset_reference_source(name, index, span, scope);
            }
            ReferenceSource::NestedArrayIndex { .. }
            | ReferenceSource::ArrayAppend { .. }
            | ReferenceSource::ObjectPropertyArrayIndex { .. }
            | ReferenceSource::ObjectPropertyArrayAppend { .. }
            | ReferenceSource::ObjectPropertyNestedArrayIndex { .. } => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "nested array-offset reference sources require a direct variable target in the current subset",
                    ),
                ));
            }
            ReferenceSource::MethodCall { .. } => {
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
            ReferenceSource::ArrayIndex { name, index, .. } => {
                return self.reject_array_offset_reference_source(name, index, span, scope);
            }
            ReferenceSource::NestedArrayIndex { .. }
            | ReferenceSource::ArrayAppend { .. }
            | ReferenceSource::ObjectPropertyArrayIndex { .. }
            | ReferenceSource::ObjectPropertyArrayAppend { .. }
            | ReferenceSource::ObjectPropertyNestedArrayIndex { .. } => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "nested array-offset reference sources require a direct variable target in the current subset",
                    ),
                ));
            }
            ReferenceSource::MethodCall { .. } => {
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

    fn reject_array_offset_reference_source(
        &mut self,
        name: &str,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let key = self.evaluate_array_key(index, scope)?;
        let value = scope.read_static(name, span)?;

        match value {
            Value::Array(array) => {
                let _slot = array.get_slot(key);
                Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "reference assignment",
                        "array-offset reference sources require array slot reference cells, which are not implemented",
                    ),
                ))
            }
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot read offset on {}",
                    other.type_name()
                )),
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
                let target_is_alias = scope.is_array_offset_alias_name(name);
                if !target_is_alias {
                    let object_alias_fallbacks = scope.public_object_roots_alias_fallbacks(name);
                    scope.remove_static_root_from_array_offset_aliases(name);
                    scope.remove_public_object_roots_from_array_offset_aliases(
                        name,
                        &object_alias_fallbacks,
                    );
                }
                scope.write_static(name, value.clone());
                if !target_is_alias && matches!(value, Value::Array(_)) {
                    match expr {
                        Expr::Variable(source_name, _) => {
                            scope.mirror_static_array_offset_aliases_from_copy(name, source_name);
                        }
                        Expr::Property {
                            target, property, ..
                        } => {
                            if let Expr::Variable(object_name, _) = target.as_ref() {
                                scope.mirror_public_object_property_array_offset_aliases_from_copy(
                                    name,
                                    object_name,
                                    property,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                if !target_is_alias && matches!(value, Value::Object(_)) {
                    if let Expr::Clone { expr, .. } = expr {
                        if let Expr::Variable(source_name, _) = expr.as_ref() {
                            scope.mirror_object_property_aliases_from_clone(name, source_name);
                        }
                    }
                }
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
                if name == "GLOBALS" {
                    let Some(key) = key.as_ref() else {
                        return Err(runtime_error(
                            *span,
                            RuntimeError::unsupported_call(
                                "$GLOBALS",
                                "append-offset writes are not implemented",
                            ),
                        ));
                    };
                    let Some(global_name) = globals_offset_name(key) else {
                        return Err(runtime_error(
                            *span,
                            RuntimeError::unsupported_call(
                                "$GLOBALS",
                                "only string-keyed direct offset writes are implemented",
                            ),
                        ));
                    };
                    scope.write_global_name(global_name, value.clone());
                    scope.sync_array_offset_aliases_for_global_root(global_name);
                    return Ok(value);
                }
                let mut slot = scope
                    .read_named(name)
                    .unwrap_or_else(|| Value::Array(PhpArray::new()));

                if matches!(slot, Value::Null) {
                    slot = Value::Array(PhpArray::new());
                }

                if let Value::Object(object) = slot {
                    self.call_array_access_method(
                        object,
                        "offsetSet",
                        vec![Self::array_key_value(key), value.clone()],
                        *span,
                    )?;
                    return Ok(value);
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
                scope.sync_array_offset_aliases_for_static_root(name);

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
                if name == "GLOBALS" {
                    Self::write_global_nested_array_assignment(&keys, value.clone(), *span, scope)?;
                    return Ok(value);
                }
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
                if name == "GLOBALS" {
                    Self::write_global_nested_array_append(&keys, value.clone(), *span, scope)?;
                    return Ok(value);
                }
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
                    Value::Object(object_value) => {
                        let alias_fallbacks =
                            scope.public_object_property_root_alias_fallbacks(object, property);
                        match object_value.write_property_from_context(
                            property,
                            value.clone(),
                            current_class_id,
                            &protected_class_ids,
                        ) {
                            Ok(()) => {
                                scope.remove_public_object_property_root_from_array_offset_aliases(
                                    object,
                                    property,
                                    &alias_fallbacks,
                                );
                                scope.sync_array_offset_aliases_for_object_property_root(
                                    object, property,
                                );
                                Ok(value)
                            }
                            Err(error) if Self::is_undefined_property_error(&error) => {
                                match self.call_magic_instance_method_with_values(
                                    object_value,
                                    "__set",
                                    vec![Value::String(property.clone()), value.clone()],
                                    *span,
                                )? {
                                    Some(_) => Ok(value),
                                    None => Err(runtime_error(*span, error)),
                                }
                            }
                            Err(error) => Err(runtime_error(*span, error)),
                        }
                    }
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
                scope.sync_array_offset_aliases_for_object_property_root(object, property);
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
                scope.sync_array_offset_aliases_for_object_property_root(object, property);
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
                    Value::Object(object_value) => {
                        let alias_fallbacks =
                            scope.public_object_property_root_alias_fallbacks(object, &property);
                        object_value
                            .write_dynamic_public_property(&property, value.clone())
                            .map_err(|error| runtime_error(*span, error))?;
                        scope.remove_public_object_property_root_from_array_offset_aliases(
                            object,
                            &property,
                            &alias_fallbacks,
                        );
                        scope.sync_array_offset_aliases_for_object_property_root(object, &property);
                        Ok(value)
                    }
                    other => Err(runtime_error(
                        *span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot write property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                }
            }
            AssignTarget::ObjectStaticProperty {
                target,
                property,
                span,
            } => {
                let value = self.evaluate(expr, scope)?;
                self.write_object_static_property(target, property, value, *span, scope)
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
                scope.sync_array_offset_aliases_for_static_root(name);
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

    fn write_global_nested_array_assignment(
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (global_name, keys) = SymbolTable::split_globals_reference_path(keys.to_vec(), span)?;
        let mut slot = scope
            .read_global_name(&global_name)
            .unwrap_or_else(|| Value::Array(PhpArray::new()));

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::write_nested_array_value(array, &keys, value, span)?;
                scope.write_global_name(&global_name, slot);
                scope.sync_array_offset_aliases_for_global_root(&global_name);
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

        if let Value::Object(object) = slot {
            if keys.len() == 1
                && self
                    .classes
                    .implements_interface(object.class_id(), "ArrayAccess")
            {
                self.call_array_access_method(
                    object,
                    "offsetSet",
                    vec![Self::array_key_value(Some(keys[0].clone())), value],
                    span,
                )?;
                return Ok(());
            }

            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot write offset on object".to_string()),
            ));
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

        if let Value::Object(object) = slot {
            if keys.is_empty()
                && self
                    .classes
                    .implements_interface(object.class_id(), "ArrayAccess")
            {
                self.call_array_access_method(
                    object,
                    "offsetSet",
                    vec![Self::array_key_value(None), value],
                    span,
                )?;
                return Ok(());
            }

            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access("cannot write offset on object".to_string()),
            ));
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

    fn read_nested_array_value(
        array: &PhpArray,
        keys: &[ArrayKey],
        span: Span,
    ) -> CompileResult<Value> {
        let Some((key, rest)) = keys.split_first() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array access",
                    "nested array access requires at least one key",
                ),
            ));
        };

        let value = array.get(key.clone()).cloned().ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_array_key(key.diagnostic_key()),
            )
        })?;
        if rest.is_empty() {
            return Ok(value);
        }

        match value {
            Value::Array(child) => Self::read_nested_array_value(&child, rest, span),
            other => Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "cannot read offset from {}",
                    other.type_name()
                )),
            )),
        }
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

    fn write_global_nested_array_append(
        keys: &[ArrayKey],
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<()> {
        let (global_name, keys) = SymbolTable::split_globals_reference_path(keys.to_vec(), span)?;
        let mut slot = scope
            .read_global_name(&global_name)
            .unwrap_or_else(|| Value::Array(PhpArray::new()));

        if matches!(slot, Value::Null) {
            slot = Value::Array(PhpArray::new());
        }

        match &mut slot {
            Value::Array(array) => {
                Self::append_nested_array_value(array, &keys, value, span)?;
                scope.write_global_name(&global_name, slot);
                scope.sync_array_offset_aliases_for_global_root(&global_name);
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
        let value = self.apply_compound_assignment_op(left, op, right, span)?;
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
                    Some(Value::Object(object)) => {
                        let value = self.call_array_access_method(
                            object,
                            "offsetGet",
                            vec![Self::array_key_value(Some(key.clone()))],
                            span,
                        )?;
                        Ok((
                            CompoundAssignmentPlace::ArrayAccessOffset {
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
            | AssignTarget::ObjectPropertyArrayAppend { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "nested array targets are not implemented",
                ),
            )),
            AssignTarget::ObjectPropertyArrayIndex {
                object,
                property,
                indices,
                ..
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                match scope.read_static(object, span)? {
                    Value::Object(object_value) => {
                        let (current_class_id, protected_class_ids) =
                            self.current_property_access_context();
                        let property_value = object_value
                            .read_property_from_context(
                                property,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(span, error))?;
                        if let Value::Object(array_access_object) = property_value.clone() {
                            if keys.len() == 1
                                && self.classes.implements_interface(
                                    array_access_object.class_id(),
                                    "ArrayAccess",
                                )
                            {
                                let left = self.call_array_access_method(
                                    array_access_object,
                                    "offsetGet",
                                    vec![Self::array_key_value(Some(keys[0].clone()))],
                                    span,
                                )?;
                                return Ok((
                                    CompoundAssignmentPlace::ObjectPropertyArrayAccessOffset {
                                        object: object.clone(),
                                        property: property.clone(),
                                        key: keys[0].clone(),
                                    },
                                    left,
                                ));
                            }
                        }
                        let Value::Array(array) = property_value else {
                            return Err(runtime_error(
                                span,
                                RuntimeError::invalid_array_access(format!(
                                    "cannot read offset from {}",
                                    property_value.type_name()
                                )),
                            ));
                        };
                        let left = Self::read_nested_array_value(&array, &keys, span)?;
                        Ok((
                            CompoundAssignmentPlace::ObjectPropertyArrayIndex {
                                object: object.clone(),
                                property: property.clone(),
                                keys,
                            },
                            left,
                        ))
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
            AssignTarget::ObjectStaticProperty { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compound assignment",
                    "dynamic static property targets are not implemented",
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
            CompoundAssignmentPlace::ArrayAccessOffset { name, key } => {
                match scope.read_named(&name) {
                    Some(Value::Object(object)) => self
                        .call_array_access_method(
                            object,
                            "offsetSet",
                            vec![Self::array_key_value(Some(key)), value],
                            span,
                        )
                        .map(|_| ()),
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
            CompoundAssignmentPlace::ArrayAccessTemporary => Ok(()),
            CompoundAssignmentPlace::ObjectPropertyArrayAccessOffset {
                object,
                property,
                key,
            } => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                match scope.read_static(&object, span)? {
                    Value::Object(object) => {
                        let property_value = object
                            .read_property_from_context(
                                &property,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(span, error))?;
                        match property_value {
                            Value::Object(array_access_object) => self
                                .call_array_access_method(
                                    array_access_object,
                                    "offsetSet",
                                    vec![Self::array_key_value(Some(key)), value],
                                    span,
                                )
                                .map(|_| ()),
                            other => Err(runtime_error(
                                span,
                                RuntimeError::invalid_array_access(format!(
                                    "cannot write offset on {}",
                                    other.type_name()
                                )),
                            )),
                        }
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::invalid_property_access(format!(
                            "cannot write property ${property} on {}",
                            other.type_name()
                        )),
                    )),
                }
            }
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
            CompoundAssignmentPlace::ObjectPropertyArrayIndex {
                object,
                property,
                keys,
            } => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                match scope.read_static(&object, span)? {
                    Value::Object(object) => {
                        let mut property_value = object
                            .read_property_from_context(
                                &property,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(span, error))?;
                        let Value::Array(array) = &mut property_value else {
                            return Err(runtime_error(
                                span,
                                RuntimeError::invalid_array_access(format!(
                                    "cannot write offset on {}",
                                    property_value.type_name()
                                )),
                            ));
                        };
                        Self::write_nested_array_value(array, &keys, value, span)?;
                        object
                            .write_property_from_context(
                                &property,
                                property_value,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(span, error))
                    }
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
        &mut self,
        left: Value,
        op: CompoundAssignOp,
        right: Value,
        span: Span,
    ) -> CompileResult<Value> {
        if matches!(op, CompoundAssignOp::Concat) {
            let left = self.value_to_echo_string(left, span)?;
            let right = self.value_to_echo_string(right, span)?;
            return Ok(Value::String(format!("{left}{right}")));
        }

        let value = match op {
            CompoundAssignOp::Add => left.php_add(&right),
            CompoundAssignOp::Sub => left.php_sub(&right),
            CompoundAssignOp::Mul => left.php_mul(&right),
            CompoundAssignOp::Div => left.php_div(&right),
            CompoundAssignOp::Mod => left.php_mod(&right),
            CompoundAssignOp::Concat => {
                unreachable!("concatenation is handled before runtime helpers")
            }
            CompoundAssignOp::BitwiseAnd => left.php_bitwise_and(&right),
            CompoundAssignOp::BitwiseOr => left.php_bitwise_or(&right),
            CompoundAssignOp::BitwiseXor => left.php_bitwise_xor(&right),
            CompoundAssignOp::ShiftLeft => left.php_shift_left(&right),
            CompoundAssignOp::ShiftRight => left.php_shift_right(&right),
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
                    Some(Value::Object(object)) => {
                        let value = self.call_array_access_method(
                            object,
                            "offsetGet",
                            vec![Self::array_key_value(Some(key))],
                            span,
                        )?;
                        Ok((CompoundAssignmentPlace::ArrayAccessTemporary, value))
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
            | AssignTarget::ObjectPropertyArrayAppend { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "nested array targets are not implemented",
                ),
            )),
            AssignTarget::ObjectPropertyArrayIndex {
                object,
                property,
                indices,
                ..
            } => {
                let keys = indices
                    .iter()
                    .map(|index| self.evaluate_array_key(index, scope))
                    .collect::<CompileResult<Vec<_>>>()?;
                if keys.len() != 1 {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "increment/decrement",
                            "nested array targets are not implemented",
                        ),
                    ));
                }
                match scope.read_static(object, span)? {
                    Value::Object(object_value) => {
                        let (current_class_id, protected_class_ids) =
                            self.current_property_access_context();
                        let property_value = object_value
                            .read_property_from_context(
                                property,
                                current_class_id,
                                &protected_class_ids,
                            )
                            .map_err(|error| runtime_error(span, error))?;
                        match property_value {
                            Value::Object(array_access_object) => {
                                let value = self.call_array_access_method(
                                    array_access_object,
                                    "offsetGet",
                                    vec![Self::array_key_value(Some(keys[0].clone()))],
                                    span,
                                )?;
                                Ok((CompoundAssignmentPlace::ArrayAccessTemporary, value))
                            }
                            _ => Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "increment/decrement",
                                    "nested array targets are not implemented",
                                ),
                            )),
                        }
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
            AssignTarget::ObjectStaticProperty { .. } => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "increment/decrement",
                    "dynamic static property targets are not implemented",
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
                            "only int and float variables, array/object offsets, object properties, or static properties are implemented, got {}",
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
            AssignTarget::ObjectStaticProperty { span, .. } => Err(runtime_error(
                *span,
                RuntimeError::unsupported_call(
                    "??=",
                    "dynamic static property targets are not implemented",
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

    fn create_mysqli_result_placeholder(
        &mut self,
        span: Span,
        fields: Vec<String>,
        rows: Vec<Vec<(String, Value)>>,
    ) -> CompileResult<Value> {
        let class_id = self
            .classes
            .lookup_class_id("mysqli_result")
            .ok_or_else(|| {
                runtime_error(
                    span,
                    RuntimeError::undefined_class("mysqli_result core placeholder"),
                )
            })?;
        let object_id = self.allocate_object_id();
        let class = self
            .classes
            .get(class_id)
            .expect("core mysqli_result class id should resolve");
        self.mysqli_results.insert(
            object_id,
            MysqliResultState {
                fields,
                rows,
                row_cursor: 0,
                field_cursor: 0,
                last_lengths: None,
            },
        );
        Ok(Value::Object(PhpObject::from_class_with_id(
            class, object_id,
        )))
    }

    fn create_mysqli_stmt_placeholder(
        &mut self,
        span: Span,
        connection_handle_id: Option<i64>,
        query: Option<String>,
    ) -> CompileResult<Value> {
        let class_id = self.classes.lookup_class_id("mysqli_stmt").ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_class("mysqli_stmt core placeholder"),
            )
        })?;
        let object_id = self.allocate_object_id();
        let class = self
            .classes
            .get(class_id)
            .expect("core mysqli_stmt class id should resolve");
        self.mysqli_statements.insert(
            object_id,
            MysqliStatementState {
                connection_handle_id,
                param_count: query
                    .as_deref()
                    .map(mysqli_placeholder_param_count)
                    .unwrap_or(0),
                query,
                bound_parameter_types: None,
                bound_parameter_variables: Vec::new(),
                bound_parameter_values: Vec::new(),
                executed_result: None,
                affected_rows: 0,
                buffered_result: None,
                buffered_result_cursor: 0,
                bound_result_variables: Vec::new(),
                attributes: HashMap::new(),
                long_data: HashMap::new(),
            },
        );
        Ok(Value::Object(PhpObject::from_class_with_id(
            class, object_id,
        )))
    }

    fn create_stdclass_with_properties(
        &mut self,
        properties: Vec<(String, Value)>,
        span: Span,
    ) -> CompileResult<Value> {
        let class_id = self.classes.lookup_class_id("stdClass").ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::undefined_class("stdClass core placeholder"),
            )
        })?;
        let object_id = self.allocate_object_id();
        let class = self
            .classes
            .get(class_id)
            .expect("core stdClass class id should resolve");
        let object = PhpObject::from_class_with_id(class, object_id);
        for (name, value) in properties {
            object
                .write_dynamic_public_property(&name, value)
                .map_err(|error| runtime_error(span, error))?;
        }
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
        let handle_id = expect_mysqli_handle_id("mysqli_real_connect()", &args[0], span)?;

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
            let Value::Int(flags) = flags else {
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
            };
            let unsupported = flags & !PHP_MYSQLI_CLIENT_ALL_SUPPORTED;
            if unsupported != 0 || *flags < 0 {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_connect()",
                        format!(
                            "only MYSQLI_CLIENT_* flag combinations are supported in the current subset, unsupported bits {unsupported}"
                        ),
                    ),
                ));
            }
        }

        if let Some(init_command) = self.mysqli_init_command(handle_id) {
            if !is_mysqli_init_command_placeholder_query(init_command) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_connect()",
                        format!(
                            "MYSQLI_INIT_COMMAND execution is not implemented for arbitrary SQL; only deterministic no-result init commands are supported in the current subset, got {init_command}"
                        ),
                    ),
                ));
            }
        }

        self.clear_mysqli_pending_results(handle_id);

        handle
            .write_public_property("connect_errno", Value::Int(0))
            .map_err(|error| runtime_error(span, error))?;
        handle
            .write_public_property("connect_error", Value::Null)
            .map_err(|error| runtime_error(span, error))?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_connect(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if args.len() > 6 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_connect()",
                    ArityExpectation::Between { min: 0, max: 6 },
                    args.len(),
                ),
            ));
        }

        for (index, label) in ["hostname", "username", "password", "database"]
            .iter()
            .enumerate()
        {
            if let Some(value) = args.get(index) {
                if !matches!(value, Value::String(_) | Value::Null) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_connect()",
                            format!(
                                "{label} argument must be string or null in the current subset, got {}",
                                value.type_name()
                            ),
                        ),
                    ));
                }
            }
        }

        if let Some(port) = args.get(4) {
            if !matches!(port, Value::Int(_) | Value::Null) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_connect()",
                        format!(
                            "port argument must be int or null in the current subset, got {}",
                            port.type_name()
                        ),
                    ),
                ));
            }
        }

        if let Some(socket) = args.get(5) {
            if !matches!(socket, Value::String(_) | Value::Null) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_connect()",
                        format!(
                            "socket argument must be string or null in the current subset, got {}",
                            socket.type_name()
                        ),
                    ),
                ));
            }
        }

        self.create_mysqli_placeholder(span)
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

    fn call_mysqli_get_server_version(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_server_version", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_server_version()", &args[0], span)?;
        Ok(Value::Int(80000))
    }

    fn call_mysqli_get_host_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_host_info", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_host_info()", &args[0], span)?;
        Ok(Value::String(
            "localhost via TCP/IP (phpc-placeholder)".to_string(),
        ))
    }

    fn call_mysqli_get_client_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        if args.len() > 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_get_client_info()",
                    ArityExpectation::Between { min: 0, max: 1 },
                    args.len(),
                ),
            ));
        }
        if let Some(arg) = args.first() {
            match arg {
                Value::Null => {}
                Value::Object(_) => expect_mysqli_handle("mysqli_get_client_info()", arg, span)?,
                arg => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_get_client_info()",
                            format!(
                                "optional argument must be mysqli object or null in the current subset, got {}",
                                arg.type_name()
                            ),
                        ),
                    ));
                }
            }
        }
        Ok(Value::String("mysqlnd 8.0.0-phpc-placeholder".to_string()))
    }

    fn call_mysqli_get_client_version(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_client_version", args, 0, span)?;
        Ok(Value::Int(80000))
    }

    fn call_mysqli_get_proto_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_proto_info", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_proto_info()", &args[0], span)?;
        Ok(Value::Int(10))
    }

    fn call_mysqli_thread_id(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_thread_id", args, 1, span)?;
        expect_mysqli_handle("mysqli_thread_id()", &args[0], span)?;
        Ok(Value::Int(1))
    }

    fn call_mysqli_kill(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_kill", args, 2, span)?;
        expect_mysqli_handle("mysqli_kill()", &args[0], span)?;
        let Value::Int(process_id) = args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_kill()",
                    format!(
                        "process_id argument must be int in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        Ok(Value::Bool(process_id == 1))
    }

    fn call_mysqli_change_user(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_change_user", args, 4, span)?;
        expect_mysqli_handle("mysqli_change_user()", &args[0], span)?;
        let Value::String(_) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_change_user()",
                    format!(
                        "username argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        let Value::String(_) = &args[2] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_change_user()",
                    format!(
                        "password argument must be string in the current subset, got {}",
                        args[2].type_name()
                    ),
                ),
            ));
        };
        match &args[3] {
            Value::Null | Value::String(_) => Ok(Value::Bool(true)),
            value => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_change_user()",
                    format!(
                        "database argument must be string or null in the current subset, got {}",
                        value.type_name()
                    ),
                ),
            )),
        }
    }

    fn call_mysqli_refresh(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_refresh", args, 2, span)?;
        expect_mysqli_handle("mysqli_refresh()", &args[0], span)?;
        let Value::Int(flags) = args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_refresh()",
                    format!(
                        "flags argument must be int in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        if flags == 0 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_refresh()",
                    "flags argument must include at least one MYSQLI_REFRESH_* flag in the current subset",
                ),
            ));
        }
        let unsupported = flags & !PHP_MYSQLI_REFRESH_ALL_SUPPORTED;
        if unsupported != 0 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_refresh()",
                    format!(
                        "only MYSQLI_REFRESH_* flag combinations are supported in the current subset, unsupported bits {unsupported}"
                    ),
                ),
            ));
        }
        Ok(Value::Bool(true))
    }

    fn call_mysqli_get_charset(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_charset", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_charset()", &args[0], span)?;
        self.create_stdclass_with_properties(
            vec![
                ("charset".to_string(), Value::String("utf8mb4".to_string())),
                (
                    "collation".to_string(),
                    Value::String("utf8mb4_unicode_520_ci".to_string()),
                ),
                ("dir".to_string(), Value::String(String::new())),
                ("min_length".to_string(), Value::Int(1)),
                ("max_length".to_string(), Value::Int(4)),
                ("number".to_string(), Value::Int(246)),
                ("state".to_string(), Value::Int(0)),
            ],
            span,
        )
    }

    fn call_mysqli_character_set_name(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_character_set_name", args, 1, span)?;
        expect_mysqli_handle("mysqli_character_set_name()", &args[0], span)?;
        Ok(Value::String("utf8mb4".to_string()))
    }

    fn call_mysqli_field_count(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_field_count", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_field_count()", &args[0], span)?;
        Ok(Value::Int(
            self.mysqli_pending_results
                .get(&handle_id)
                .map(|result| result.fields.len() as i64)
                .unwrap_or(0),
        ))
    }

    fn set_mysqli_pending_result_queue(
        &mut self,
        handle_id: i64,
        results: Vec<MysqliMultiResultSlot>,
    ) {
        let mut results: VecDeque<_> = results.into();
        let Some(first_slot) = results.pop_front() else {
            self.mysqli_pending_results.remove(&handle_id);
            self.mysqli_pending_result_queues.remove(&handle_id);
            return;
        };

        match first_slot {
            MysqliMultiResultSlot::NoResult => {
                self.mysqli_pending_results.remove(&handle_id);
            }
            MysqliMultiResultSlot::Result(result) => {
                self.mysqli_pending_results.insert(handle_id, result);
            }
        }
        if results.is_empty() {
            self.mysqli_pending_result_queues.remove(&handle_id);
        } else {
            self.mysqli_pending_result_queues.insert(handle_id, results);
        }
    }

    fn call_mysqli_close(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_close", args, 1, span)?;
        expect_mysqli_handle("mysqli_close()", &args[0], span)?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_options(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_option_setter("mysqli_options", args, span)
    }

    fn call_mysqli_set_opt(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_option_setter("mysqli_set_opt", args, span)
    }

    fn call_mysqli_ssl_set(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_ssl_set", args, 6, span)?;
        expect_mysqli_handle("mysqli_ssl_set()", &args[0], span)?;
        for (index, value) in args.iter().enumerate().skip(1) {
            if !matches!(value, Value::String(_) | Value::Null) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_ssl_set()",
                        format!(
                            "{} must be string or null in the current subset, got {}",
                            positional_argument_label(index),
                            value.type_name()
                        ),
                    ),
                ));
            }
        }
        Ok(Value::Bool(true))
    }

    fn call_mysqli_option_setter(
        &mut self,
        function: &str,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        let call_name = callable_name(function);
        expect_arity(function, args, 3, span)?;
        let handle_id = expect_mysqli_handle_id(&call_name, &args[0], span)?;
        match &args[1] {
            Value::Int(
                PHP_MYSQLI_OPT_INT_AND_FLOAT_NATIVE
                | PHP_MYSQLI_OPT_LOCAL_INFILE
                | PHP_MYSQLI_OPT_SSL_VERIFY_SERVER_CERT
                | PHP_MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS,
            ) => {
                if !matches!(args[2], Value::Bool(_) | Value::Int(_)) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "value must be bool or int for the selected mysqli option in the current subset, got {}",
                                args[2].type_name()
                            ),
                        ),
                    ));
                }
                self.record_mysqli_option(handle_id, args[1].clone(), args[2].clone());
                return Ok(Value::Bool(true));
            }
            Value::Int(
                PHP_MYSQLI_OPT_CONNECT_TIMEOUT
                | PHP_MYSQLI_OPT_READ_TIMEOUT
                | PHP_MYSQLI_OPT_NET_CMD_BUFFER_SIZE
                | PHP_MYSQLI_OPT_NET_READ_BUFFER_SIZE,
            ) => {
                if !matches!(args[2], Value::Int(_)) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "value must be int for the selected mysqli option in the current subset, got {}",
                                args[2].type_name()
                            ),
                        ),
                    ));
                }
                self.record_mysqli_option(handle_id, args[1].clone(), args[2].clone());
                return Ok(Value::Bool(true));
            }
            Value::Int(PHP_MYSQLI_INIT_COMMAND | PHP_MYSQLI_OPT_LOAD_DATA_LOCAL_DIR) => {
                if !matches!(args[2], Value::String(_)) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "value must be string for the selected mysqli option in the current subset, got {}",
                                args[2].type_name()
                            ),
                        ),
                    ));
                }
                self.record_mysqli_option(handle_id, args[1].clone(), args[2].clone());
                return Ok(Value::Bool(true));
            }
            Value::Int(option) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        call_name.clone(),
                        format!("unsupported mysqli option in the current subset, got {option}"),
                    ),
                ));
            }
            value => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        call_name.clone(),
                        format!(
                            "option must be int in the current subset, got {}",
                            value.type_name()
                        ),
                    ),
                ));
            }
        }
    }

    fn call_mysqli_connect_errno(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_connect_errno", args, 0, span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_connect_error(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_connect_error", args, 0, span)?;
        Ok(Value::Null)
    }

    fn call_mysqli_error_list(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_error_list", args, 1, span)?;
        expect_mysqli_handle("mysqli_error_list()", &args[0], span)?;
        Ok(Value::Array(PhpArray::new()))
    }

    fn call_mysqli_get_connection_stats(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_connection_stats", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_connection_stats()", &args[0], span)?;
        let mut stats = PhpArray::new();
        stats.insert("bytes_sent", Value::Int(0));
        stats.insert("bytes_received", Value::Int(0));
        stats.insert("packets_sent", Value::Int(0));
        stats.insert("packets_received", Value::Int(0));
        stats.insert("result_set_queries", Value::Int(0));
        stats.insert("non_result_set_queries", Value::Int(0));
        stats.insert("connect_success", Value::Int(1));
        stats.insert("active_connections", Value::Int(1));
        Ok(Value::Array(stats))
    }

    fn call_mysqli_get_links_stats(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_links_stats", args, 0, span)?;
        let mut stats = PhpArray::new();
        stats.insert("total", Value::Int(0));
        stats.insert("active_plinks", Value::Int(0));
        stats.insert("cached_plinks", Value::Int(0));
        Ok(Value::Array(stats))
    }

    fn call_mysqli_get_client_stats(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_client_stats", args, 0, span)?;
        let mut stats = PhpArray::new();
        stats.insert("bytes_sent", Value::Int(0));
        stats.insert("bytes_received", Value::Int(0));
        stats.insert("packets_sent", Value::Int(0));
        stats.insert("packets_received", Value::Int(0));
        stats.insert("protocol_overhead_in", Value::Int(0));
        stats.insert("protocol_overhead_out", Value::Int(0));
        stats.insert("connect_success", Value::Int(0));
        stats.insert("active_connections", Value::Int(0));
        Ok(Value::Array(stats))
    }

    fn call_mysqli_thread_safe(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_thread_safe", args, 0, span)?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_init(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_init", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_stmt_init()", &args[0], span)?;
        self.create_mysqli_stmt_placeholder(span, Some(handle_id), None)
    }

    fn call_mysqli_prepare(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_prepare", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_prepare()", &args[0], span)?;
        let query = string_builtin_argument("mysqli_prepare()", "query", &args[1], span)?;
        self.create_mysqli_stmt_placeholder(span, Some(handle_id), Some(query))
    }

    fn call_mysqli_stmt_prepare(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_prepare", args, 2, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_prepare()", &args[0], span)?;
        let query = string_builtin_argument("mysqli_stmt_prepare()", "query", &args[1], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_prepare()", stmt_id, span)?;
        state.param_count = mysqli_placeholder_param_count(&query);
        state.query = Some(query);
        state.bound_parameter_types = None;
        state.bound_parameter_variables.clear();
        state.bound_parameter_values.clear();
        state.executed_result = None;
        state.affected_rows = 0;
        state.buffered_result = None;
        state.buffered_result_cursor = 0;
        state.bound_result_variables.clear();
        state.long_data.clear();
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_param_count(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_param_count", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_param_count()", &args[0], span)?;
        let state = self.mysqli_statement_state("mysqli_stmt_param_count()", stmt_id, span)?;
        Ok(Value::Int(state.param_count as i64))
    }

    fn call_mysqli_stmt_get_warnings(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_get_warnings", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_get_warnings()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_get_warnings()", stmt_id, span)?;
        Ok(Value::Bool(false))
    }

    fn call_mysqli_stmt_error_list(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_error_list", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_error_list()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_error_list()", stmt_id, span)?;
        Ok(Value::Array(PhpArray::new()))
    }

    fn call_mysqli_stmt_bind_param(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        if args.len() < 2 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_stmt_bind_param()",
                    ArityExpectation::AtLeast(2),
                    args.len(),
                ),
            ));
        }
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_bind_param()",
                "mysqli statement objects, by-reference parameter binding, type strings, and prepared statement execution are not implemented in the current subset",
            ),
        ))
    }

    fn record_mysqli_option(&mut self, handle_id: i64, option: Value, value: Value) {
        let Value::Int(option_id) = option else {
            return;
        };
        self.mysqli_options
            .entry(handle_id)
            .or_default()
            .insert(option_id, value);
    }

    fn mysqli_init_command(&self, handle_id: i64) -> Option<&str> {
        self.mysqli_options
            .get(&handle_id)
            .and_then(|options| options.get(&PHP_MYSQLI_INIT_COMMAND))
            .and_then(|value| match value {
                Value::String(command) => Some(command.as_str()),
                _ => None,
            })
    }

    fn mysqli_local_infile_enabled(&self, handle_id: i64) -> bool {
        self.mysqli_options
            .get(&handle_id)
            .and_then(|options| options.get(&PHP_MYSQLI_OPT_LOCAL_INFILE))
            .is_some_and(|value| match value {
                Value::Bool(enabled) => *enabled,
                Value::Int(enabled) => *enabled != 0,
                _ => false,
            })
    }

    fn clear_mysqli_pending_results(&mut self, handle_id: i64) {
        self.mysqli_pending_results.remove(&handle_id);
        self.mysqli_pending_result_queues.remove(&handle_id);
    }

    fn call_mysqli_stmt_bind_result(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        if args.len() < 2 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_stmt_bind_result()",
                    ArityExpectation::AtLeast(2),
                    args.len(),
                ),
            ));
        }
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_bind_result()",
                "mysqli statement objects, by-reference result binding, result buffer mutation, and fetch integration are not implemented in the current subset",
            ),
        ))
    }

    fn call_mysqli_stmt_execute(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_stmt_execute_values("mysqli_stmt_execute()", args, span)
    }

    fn call_mysqli_execute(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_stmt_execute_values("mysqli_execute()", args, span)
    }

    fn call_mysqli_stmt_execute_values(
        &mut self,
        function: &'static str,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    function,
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        let stmt_id = expect_mysqli_stmt_handle(function, &args[0], span)?;
        if args.len() == 2 {
            if !matches!(args[1], Value::Null) {
                let params = mysqli_execute_params_from_value(function, &args[1], span)?;
                self.mysqli_statement_state_mut(function, stmt_id, span)?
                    .bound_parameter_values = params;
            }
        }

        self.execute_mysqli_stmt_placeholder(function, stmt_id, span)
    }

    fn execute_mysqli_stmt_placeholder(
        &mut self,
        function: &'static str,
        stmt_id: i64,
        span: Span,
    ) -> CompileResult<Value> {
        let (connection_handle_id, query, bound_parameters) = {
            let state = self.mysqli_statement_state(function, stmt_id, span)?;
            if state.param_count != 0 && state.bound_parameter_values.len() != state.param_count {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        function,
                        "bound parameter values are not available for the current placeholder statement",
                    ),
                ));
            }
            let mut bound_parameters = state.bound_parameter_values.clone();
            if let Some(types) = state.bound_parameter_types.as_deref() {
                for (index, ty) in types.chars().enumerate() {
                    if ty == 'b' {
                        if let Some(data) = state.long_data.get(&index) {
                            if index < bound_parameters.len() {
                                bound_parameters[index] = Value::String(data.clone());
                            }
                        }
                    }
                }
            }
            (
                state.connection_handle_id,
                state.query.clone(),
                bound_parameters,
            )
        };

        let Some(query) = query else {
            let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
            state.executed_result = None;
            state.affected_rows = 0;
            state.buffered_result = None;
            state.buffered_result_cursor = 0;
            return Ok(Value::Bool(false));
        };

        if is_wordpress_option_prepared_update_query(&query) {
            if let Some(handle_id) = connection_handle_id {
                if self.mysqli_wp_options.contains_key(&handle_id) {
                    let [Value::String(option_value), Value::String(option_name)] =
                        bound_parameters.as_slice()
                    else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                function,
                                "prepared wp_options update requires string option value and option name parameters in the current subset",
                            ),
                        ));
                    };
                    let options = self
                        .mysqli_wp_options
                        .get_mut(&handle_id)
                        .expect("checked wp_options state should exist");
                    let affected_rows = if let Some(option) = options.get_mut(option_name) {
                        option.value = option_value.clone();
                        1
                    } else {
                        0
                    };
                    self.mysqli_affected_rows.insert(handle_id, affected_rows);
                    let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                    state.executed_result = None;
                    state.affected_rows = affected_rows;
                    state.buffered_result = None;
                    state.buffered_result_cursor = 0;
                    return Ok(Value::Bool(true));
                }
            }
        }

        if is_wordpress_option_prepared_insert_query(&query) {
            if let Some(handle_id) = connection_handle_id {
                let [Value::String(option_name), Value::String(option_value), Value::String(autoload)] =
                    bound_parameters.as_slice()
                else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            function,
                            "prepared wp_options insert requires string option name, option value, and autoload parameters in the current subset",
                        ),
                    ));
                };
                let options = self.mysqli_wp_options.entry(handle_id).or_default();
                if options.contains_key(option_name) {
                    self.mysqli_affected_rows.insert(handle_id, 0);
                    let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                    state.executed_result = None;
                    state.affected_rows = 0;
                    state.buffered_result = None;
                    state.buffered_result_cursor = 0;
                    return Ok(Value::Bool(false));
                }
                options.insert(
                    option_name.clone(),
                    WordPressOptionState {
                        value: option_value.clone(),
                        autoload: autoload.clone(),
                    },
                );
                self.mysqli_affected_rows.insert(handle_id, 1);
                let next_insert_id = self
                    .mysqli_insert_ids
                    .get(&handle_id)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(1);
                self.mysqli_insert_ids.insert(handle_id, next_insert_id);
                let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                state.executed_result = None;
                state.affected_rows = 1;
                state.buffered_result = None;
                state.buffered_result_cursor = 0;
                return Ok(Value::Bool(true));
            }
        }

        if is_wordpress_option_prepared_insert_on_duplicate_query(&query) {
            if let Some(handle_id) = connection_handle_id {
                let [Value::String(option_name), Value::String(option_value), Value::String(autoload)] =
                    bound_parameters.as_slice()
                else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            function,
                            "prepared wp_options insert-on-duplicate requires string option name, option value, and autoload parameters in the current subset",
                        ),
                    ));
                };
                let previous = self.mysqli_wp_options.entry(handle_id).or_default().insert(
                    option_name.clone(),
                    WordPressOptionState {
                        value: option_value.clone(),
                        autoload: autoload.clone(),
                    },
                );
                let affected_rows = if previous.is_some() { 2 } else { 1 };
                self.mysqli_affected_rows.insert(handle_id, affected_rows);
                let next_insert_id = self
                    .mysqli_insert_ids
                    .get(&handle_id)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(1);
                self.mysqli_insert_ids.insert(handle_id, next_insert_id);
                let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                state.executed_result = None;
                state.affected_rows = affected_rows;
                state.buffered_result = None;
                state.buffered_result_cursor = 0;
                return Ok(Value::Bool(true));
            }
        }

        if is_wordpress_option_prepared_replace_query(&query) {
            if let Some(handle_id) = connection_handle_id {
                let [Value::String(option_name), Value::String(option_value), Value::String(autoload)] =
                    bound_parameters.as_slice()
                else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            function,
                            "prepared wp_options replace requires string option name, option value, and autoload parameters in the current subset",
                        ),
                    ));
                };
                let previous = self.mysqli_wp_options.entry(handle_id).or_default().insert(
                    option_name.clone(),
                    WordPressOptionState {
                        value: option_value.clone(),
                        autoload: autoload.clone(),
                    },
                );
                let affected_rows = if previous.is_some() { 2 } else { 1 };
                self.mysqli_affected_rows.insert(handle_id, affected_rows);
                let next_insert_id = self
                    .mysqli_insert_ids
                    .get(&handle_id)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(1);
                self.mysqli_insert_ids.insert(handle_id, next_insert_id);
                let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                state.executed_result = None;
                state.affected_rows = affected_rows;
                state.buffered_result = None;
                state.buffered_result_cursor = 0;
                return Ok(Value::Bool(true));
            }
        }

        if is_wordpress_option_prepared_delete_query(&query) {
            if let Some(handle_id) = connection_handle_id {
                if self.mysqli_wp_options.contains_key(&handle_id) {
                    let [Value::String(option_name)] = bound_parameters.as_slice() else {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                function,
                                "prepared wp_options delete requires a string option name parameter in the current subset",
                            ),
                        ));
                    };
                    let options = self
                        .mysqli_wp_options
                        .get_mut(&handle_id)
                        .expect("checked wp_options state should exist");
                    let affected_rows = if options.remove(option_name).is_some() {
                        1
                    } else {
                        0
                    };
                    self.mysqli_affected_rows.insert(handle_id, affected_rows);
                    let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
                    state.executed_result = None;
                    state.affected_rows = affected_rows;
                    state.buffered_result = None;
                    state.buffered_result_cursor = 0;
                    return Ok(Value::Bool(true));
                }
            }
        }

        if is_mysqli_mutation_query(&query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "statement mutation execution and host database state are not implemented in the current subset",
                ),
            ));
        }

        let result = self.mysqli_statement_result_for_query_with_params(
            function,
            connection_handle_id,
            &query,
            &bound_parameters,
            span,
        )?;
        let state = self.mysqli_statement_state_mut(function, stmt_id, span)?;
        state.executed_result = result;
        state.affected_rows = 0;
        state.buffered_result = None;
        state.buffered_result_cursor = 0;
        Ok(Value::Bool(true))
    }

    fn mysqli_statement_result_for_query_with_params(
        &self,
        function: &str,
        connection_handle_id: Option<i64>,
        query: &str,
        params: &[Value],
        span: Span,
    ) -> CompileResult<Option<MysqliPendingResultState>> {
        if query == "SELECT option_value FROM wp_options WHERE option_name = ?" {
            if let Some(option_value) = match (connection_handle_id, params) {
                (Some(handle_id), [Value::String(option_name)]) => self
                    .mysqli_wp_options
                    .get(&handle_id)
                    .and_then(|options| options.get(option_name))
                    .map(|option| option.value.clone()),
                _ => None,
            } {
                return Ok(Some(MysqliPendingResultState {
                    fields: vec!["option_value".to_string()],
                    rows: vec![vec![(
                        "option_value".to_string(),
                        Value::String(option_value),
                    )]],
                }));
            }
            return Ok(Some(MysqliPendingResultState {
                fields: Vec::new(),
                rows: Vec::new(),
            }));
        }

        if query == "SELECT option_name, option_value FROM wp_options WHERE option_name = ?" {
            if let Some((option_name, option_value)) = match (connection_handle_id, params) {
                (Some(handle_id), [Value::String(option_name)]) => self
                    .mysqli_wp_options
                    .get(&handle_id)
                    .and_then(|options| options.get(option_name))
                    .map(|option| (option_name.clone(), option.value.clone())),
                _ => None,
            } {
                return Ok(Some(MysqliPendingResultState {
                    fields: vec!["option_name".to_string(), "option_value".to_string()],
                    rows: vec![vec![
                        ("option_name".to_string(), Value::String(option_name)),
                        ("option_value".to_string(), Value::String(option_value)),
                    ]],
                }));
            }
            return Ok(Some(MysqliPendingResultState {
                fields: Vec::new(),
                rows: Vec::new(),
            }));
        }

        if query == "SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1" {
            if let Some((option_value, autoload)) = match (connection_handle_id, params) {
                (Some(handle_id), [Value::String(option_name)]) => self
                    .mysqli_wp_options
                    .get(&handle_id)
                    .and_then(|options| options.get(option_name))
                    .map(|option| (option.value.clone(), option.autoload.clone())),
                _ => None,
            } {
                return Ok(Some(MysqliPendingResultState {
                    fields: vec!["option_value".to_string(), "autoload".to_string()],
                    rows: vec![vec![
                        ("option_value".to_string(), Value::String(option_value)),
                        ("autoload".to_string(), Value::String(autoload)),
                    ]],
                }));
            }
            return Ok(Some(MysqliPendingResultState {
                fields: Vec::new(),
                rows: Vec::new(),
            }));
        }

        mysqli_statement_result_for_query_with_params(function, query, params, span)
    }

    fn call_mysqli_stmt_get_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_get_result", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_get_result()", &args[0], span)?;
        let result = self
            .mysqli_statement_state("mysqli_stmt_get_result()", stmt_id, span)?
            .executed_result
            .clone();
        let Some(result) = result else {
            return Ok(Value::Bool(false));
        };
        self.create_mysqli_result_placeholder(span, result.fields, result.rows)
    }

    fn call_mysqli_stmt_close(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_close", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_close()", &args[0], span)?;
        self.mysqli_statements.remove(&stmt_id);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_errno(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_errno", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_errno()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_errno()", stmt_id, span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_stmt_error(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_error", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_error()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_error()", stmt_id, span)?;
        Ok(Value::String(String::new()))
    }

    fn call_mysqli_stmt_affected_rows(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_affected_rows", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_affected_rows()", &args[0], span)?;
        let state = self.mysqli_statement_state("mysqli_stmt_affected_rows()", stmt_id, span)?;
        Ok(Value::Int(state.affected_rows))
    }

    fn call_mysqli_stmt_store_result(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_store_result", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_store_result()", &args[0], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_store_result()", stmt_id, span)?;
        let Some(result) = state.executed_result.clone() else {
            state.buffered_result = None;
            state.buffered_result_cursor = 0;
            return Ok(Value::Bool(false));
        };
        state.buffered_result = Some(result);
        state.buffered_result_cursor = 0;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_num_rows(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_num_rows", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_num_rows()", &args[0], span)?;
        let state = self.mysqli_statement_state("mysqli_stmt_num_rows()", stmt_id, span)?;
        Ok(Value::Int(
            state
                .buffered_result
                .as_ref()
                .map(|result| result.rows.len() as i64)
                .unwrap_or(0),
        ))
    }

    fn call_mysqli_stmt_fetch(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_fetch", args, 1, span)?;
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_fetch()",
                "mysqli statement objects, bound result buffers, cursor advancement, and host database rows are not implemented in the current subset",
            ),
        ))
    }

    fn call_mysqli_stmt_result_metadata(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_result_metadata", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_result_metadata()", &args[0], span)?;
        let Some(query) = self
            .mysqli_statement_state("mysqli_stmt_result_metadata()", stmt_id, span)?
            .query
            .as_deref()
        else {
            return Ok(Value::Bool(false));
        };
        let Some(result) =
            mysqli_statement_result_for_query("mysqli_stmt_result_metadata()", query, span)?
        else {
            return Ok(Value::Bool(false));
        };
        self.create_mysqli_result_placeholder(span, result.fields, Vec::new())
    }

    fn call_mysqli_stmt_field_count(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_field_count", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_field_count()", &args[0], span)?;
        let Some(query) = self
            .mysqli_statement_state("mysqli_stmt_field_count()", stmt_id, span)?
            .query
            .as_deref()
        else {
            return Ok(Value::Int(0));
        };
        let Some(result) =
            mysqli_statement_result_for_query("mysqli_stmt_field_count()", query, span)?
        else {
            return Ok(Value::Int(0));
        };
        Ok(Value::Int(result.fields.len() as i64))
    }

    fn call_mysqli_stmt_free_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_free_result", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_free_result()", &args[0], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_free_result()", stmt_id, span)?;
        state.buffered_result = None;
        state.buffered_result_cursor = 0;
        Ok(Value::Null)
    }

    fn call_mysqli_stmt_data_seek(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_data_seek", args, 2, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_data_seek()", &args[0], span)?;
        let offset = expect_mysqli_stmt_data_seek_offset(&args[1], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_data_seek()", stmt_id, span)?;
        let Some(result) = state.buffered_result.as_ref() else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_data_seek()",
                    "buffered statement result state is not available in the current subset",
                ),
            ));
        };
        if offset >= result.rows.len() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_data_seek()",
                    format!(
                        "offset {offset} is outside the current buffered statement row range {}",
                        result.rows.len()
                    ),
                ),
            ));
        }
        state.buffered_result_cursor = offset;
        Ok(Value::Null)
    }

    fn call_mysqli_stmt_attr_get(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_attr_get", args, 2, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_attr_get()", &args[0], span)?;
        let attribute = expect_mysqli_stmt_attribute("mysqli_stmt_attr_get()", &args[1], span)?;
        let state = self.mysqli_statement_state("mysqli_stmt_attr_get()", stmt_id, span)?;
        Ok(state
            .attributes
            .get(&attribute)
            .cloned()
            .unwrap_or_else(|| Value::Int(mysqli_stmt_attribute_default(attribute))))
    }

    fn call_mysqli_stmt_attr_set(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_attr_set", args, 3, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_attr_set()", &args[0], span)?;
        let attribute = expect_mysqli_stmt_attribute("mysqli_stmt_attr_set()", &args[1], span)?;
        let value = expect_mysqli_stmt_attribute_value("mysqli_stmt_attr_set()", &args[2], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_attr_set()", stmt_id, span)?;
        state.attributes.insert(attribute, Value::Int(value));
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_send_long_data(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_send_long_data", args, 3, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_send_long_data()", &args[0], span)?;
        let Value::Int(param_num) = args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_send_long_data()",
                    format!(
                        "param_num argument must be int in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        if param_num < 0 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_send_long_data()",
                    "param_num argument must be non-negative in the current subset",
                ),
            ));
        }
        let data = string_builtin_argument("mysqli_stmt_send_long_data()", "data", &args[2], span)?;
        let state =
            self.mysqli_statement_state_mut("mysqli_stmt_send_long_data()", stmt_id, span)?;
        if param_num as usize >= state.param_count {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_send_long_data()",
                    format!(
                        "param_num argument must reference one of the current {} placeholder parameter(s), got {}",
                        state.param_count, param_num
                    ),
                ),
            ));
        }
        state
            .long_data
            .entry(param_num as usize)
            .or_default()
            .push_str(&data);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_reset(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_reset", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_reset()", &args[0], span)?;
        let state = self.mysqli_statement_state_mut("mysqli_stmt_reset()", stmt_id, span)?;
        state.query = None;
        state.param_count = 0;
        state.bound_parameter_types = None;
        state.bound_parameter_variables.clear();
        state.bound_parameter_values.clear();
        state.executed_result = None;
        state.affected_rows = 0;
        state.buffered_result = None;
        state.buffered_result_cursor = 0;
        state.bound_result_variables.clear();
        state.attributes.clear();
        state.long_data.clear();
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_more_results(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_more_results", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_more_results()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_more_results()", stmt_id, span)?;
        Ok(Value::Bool(false))
    }

    fn call_mysqli_stmt_next_result(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_next_result", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_next_result()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_next_result()", stmt_id, span)?;
        Ok(Value::Bool(false))
    }

    fn call_mysqli_stmt_sqlstate(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_sqlstate", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_sqlstate()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_sqlstate()", stmt_id, span)?;
        Ok(Value::String("00000".to_string()))
    }

    fn call_mysqli_stmt_warning_count(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_warning_count", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_warning_count()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_warning_count()", stmt_id, span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_stmt_insert_id(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stmt_insert_id", args, 1, span)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_insert_id()", &args[0], span)?;
        self.mysqli_statement_state("mysqli_stmt_insert_id()", stmt_id, span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_execute_query(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(2..=3).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_execute_query()",
                    ArityExpectation::Between { min: 2, max: 3 },
                    args.len(),
                ),
            ));
        }

        let handle_id = expect_mysqli_handle_id("mysqli_execute_query()", &args[0], span)?;
        let query = string_builtin_argument("mysqli_execute_query()", "query", &args[1], span)?;
        let params = match args.get(2) {
            None | Some(Value::Null) => Vec::new(),
            Some(value) => mysqli_execute_params_from_value("mysqli_execute_query()", value, span)?,
        };

        let parameter_count = mysqli_placeholder_param_count(&query);
        if params.len() != parameter_count {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_execute_query()",
                    format!(
                        "params array length must match query placeholder count {parameter_count}, got {}",
                        params.len()
                    ),
                ),
            ));
        }

        if is_mysqli_no_result_placeholder_query(&query) {
            return Ok(Value::Bool(true));
        }

        if is_mysqli_mutation_query(&query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_execute_query()",
                    "mutation SQL execution and host database state are not implemented in the current subset",
                ),
            ));
        }

        if let Some(result) = self.mysqli_statement_result_for_query_with_params(
            "mysqli_execute_query()",
            Some(handle_id),
            &query,
            &params,
            span,
        )? {
            return self.create_mysqli_result_placeholder(span, result.fields, result.rows);
        }

        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_execute_query()",
                format!(
                    "only deterministic WordPress no-result and exact SELECT placeholder shapes are implemented in the current subset; got {query}"
                ),
            ),
        ))
    }

    fn call_mysqli_dump_debug_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_dump_debug_info", args, 1, span)?;
        expect_mysqli_handle("mysqli_dump_debug_info()", &args[0], span)?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_debug(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_debug", args, 1, span)?;
        let _options = string_builtin_argument("mysqli_debug()", "options", &args[0], span)?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stat(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_stat", args, 1, span)?;
        expect_mysqli_handle("mysqli_stat()", &args[0], span)?;
        Ok(Value::String(
            "Uptime: 0  Threads: 0  Questions: 0  Slow queries: 0  Opens: 0  Flush tables: 0  Open tables: 0  Queries per second avg: 0.000"
                .to_string(),
        ))
    }

    fn call_mysqli_autocommit(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_autocommit", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_autocommit()", &args[0], span)?;
        let Value::Bool(mode) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_autocommit()",
                    format!(
                        "mode argument must be bool in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        if *mode {
            self.mysqli_transactions.remove(&handle_id);
        } else {
            self.begin_mysqli_transaction_snapshot(handle_id);
        }
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_begin_transaction(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        if !(1..=3).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_begin_transaction()",
                    ArityExpectation::Between { min: 1, max: 3 },
                    args.len(),
                ),
            ));
        }
        let handle_id = expect_mysqli_handle_id("mysqli_begin_transaction()", &args[0], span)?;
        if let Some(flags) = args.get(1) {
            match flags {
                Value::Int(0) => {}
                Value::Int(flags) => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_begin_transaction()",
                            format!(
                                "only flags value 0 is implemented in the current subset, got {flags}"
                            ),
                        ),
                    ));
                }
                flags => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_begin_transaction()",
                            format!(
                                "flags argument must be int in the current subset, got {}",
                                flags.type_name()
                            ),
                        ),
                    ));
                }
            }
        }
        if let Some(name) = args.get(2) {
            match name {
                Value::Null | Value::String(_) => {}
                name => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "mysqli_begin_transaction()",
                            format!(
                                "name argument must be string or null in the current subset, got {}",
                                name.type_name()
                            ),
                        ),
                    ));
                }
            }
        }
        self.begin_mysqli_transaction_snapshot(handle_id);
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn expect_mysqli_transaction_completion_args(
        &self,
        function: &str,
        args: &[Value],
        span: Span,
    ) -> CompileResult<()> {
        let call_name = format!("{function}()");
        if !(1..=3).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    call_name.clone(),
                    ArityExpectation::Between { min: 1, max: 3 },
                    args.len(),
                ),
            ));
        }
        expect_mysqli_handle(&call_name, &args[0], span)?;
        if let Some(flags) = args.get(1) {
            match flags {
                Value::Int(0) => {}
                Value::Int(flags) => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "only flags value 0 is implemented in the current subset, got {flags}"
                            ),
                        ),
                    ));
                }
                flags => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "flags argument must be int in the current subset, got {}",
                                flags.type_name()
                            ),
                        ),
                    ));
                }
            }
        }
        if let Some(name) = args.get(2) {
            match name {
                Value::Null | Value::String(_) => {}
                name => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            call_name,
                            format!(
                                "name argument must be string or null in the current subset, got {}",
                                name.type_name()
                            ),
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn begin_mysqli_transaction_snapshot(&mut self, handle_id: i64) {
        if self.mysqli_transactions.contains_key(&handle_id) {
            return;
        }
        let wp_options_snapshot = self.mysqli_wp_options.get(&handle_id).cloned();
        self.mysqli_transactions.insert(
            handle_id,
            MysqliTransactionState {
                wp_options_snapshot,
                wp_option_savepoints: HashMap::new(),
            },
        );
    }

    fn current_mysqli_wp_options_snapshot(
        &self,
        handle_id: i64,
    ) -> Option<HashMap<String, WordPressOptionState>> {
        self.mysqli_wp_options.get(&handle_id).cloned()
    }

    fn restore_mysqli_wp_options_snapshot(
        &mut self,
        handle_id: i64,
        snapshot: Option<HashMap<String, WordPressOptionState>>,
    ) {
        if let Some(options) = snapshot {
            self.mysqli_wp_options.insert(handle_id, options);
        } else {
            self.mysqli_wp_options.remove(&handle_id);
        }
    }

    fn call_mysqli_commit(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.expect_mysqli_transaction_completion_args("mysqli_commit", args, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_commit()", &args[0], span)?;
        self.mysqli_transactions.remove(&handle_id);
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_rollback(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.expect_mysqli_transaction_completion_args("mysqli_rollback", args, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_rollback()", &args[0], span)?;
        if let Some(Value::String(name)) = args.get(2) {
            if let Some(snapshot) = self
                .mysqli_transactions
                .get(&handle_id)
                .and_then(|transaction| transaction.wp_option_savepoints.get(name))
                .cloned()
            {
                self.restore_mysqli_wp_options_snapshot(handle_id, snapshot);
            }
        } else if let Some(transaction) = self.mysqli_transactions.remove(&handle_id) {
            self.restore_mysqli_wp_options_snapshot(handle_id, transaction.wp_options_snapshot);
        }
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_savepoint(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_savepoint", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_savepoint()", &args[0], span)?;
        let Value::String(name) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_savepoint()",
                    format!(
                        "name argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        self.begin_mysqli_transaction_snapshot(handle_id);
        let snapshot = self.current_mysqli_wp_options_snapshot(handle_id);
        if let Some(transaction) = self.mysqli_transactions.get_mut(&handle_id) {
            transaction
                .wp_option_savepoints
                .insert(name.clone(), snapshot);
        }
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_release_savepoint(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("mysqli_release_savepoint", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_release_savepoint()", &args[0], span)?;
        let Value::String(name) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_release_savepoint()",
                    format!(
                        "name argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        if let Some(transaction) = self.mysqli_transactions.get_mut(&handle_id) {
            transaction.wp_option_savepoints.remove(name);
        }
        self.mysqli_affected_rows.insert(handle_id, 0);
        Ok(Value::Bool(true))
    }

    fn call_mysqli_set_charset(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_set_charset", args, 2, span)?;
        expect_mysqli_handle("mysqli_set_charset()", &args[0], span)?;
        let Value::String(charset) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_set_charset()",
                    format!(
                        "charset must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };
        if charset.eq_ignore_ascii_case("utf8mb4") {
            return Ok(Value::Bool(true));
        }
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_set_charset()",
                format!(
                    "only the deterministic utf8mb4 placeholder charset is implemented in the current subset, got {charset}"
                ),
            ),
        ))
    }

    fn call_mysqli_query(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(2..=3).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_query()",
                    ArityExpectation::Between { min: 2, max: 3 },
                    args.len(),
                ),
            ));
        }
        if args.len() == 3 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_query()",
                    "result mode arguments are not implemented; pass exactly two arguments in the current subset",
                ),
            ));
        }

        let handle_id = expect_mysqli_handle_id("mysqli_query()", &args[0], span)?;

        let Value::String(query) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_query()",
                    format!(
                        "query argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };

        if is_wordpress_charset_setup_query(query) || is_wordpress_sql_mode_assignment_query(query)
        {
            self.mysqli_affected_rows.insert(handle_id, 0);
            return Ok(Value::Bool(true));
        }

        if let Some((option_name, option_value, autoload)) =
            parse_wordpress_option_insert_on_duplicate_query(query)
        {
            let previous = self.mysqli_wp_options.entry(handle_id).or_default().insert(
                option_name,
                WordPressOptionState {
                    value: option_value,
                    autoload,
                },
            );
            let affected_rows = if previous.is_some() { 2 } else { 1 };
            self.mysqli_affected_rows.insert(handle_id, affected_rows);
            let next_insert_id = self
                .mysqli_insert_ids
                .get(&handle_id)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            self.mysqli_insert_ids.insert(handle_id, next_insert_id);
            return Ok(Value::Bool(true));
        }

        if let Some((option_name, option_value, autoload)) =
            parse_wordpress_option_replace_query(query)
        {
            let previous = self.mysqli_wp_options.entry(handle_id).or_default().insert(
                option_name,
                WordPressOptionState {
                    value: option_value,
                    autoload,
                },
            );
            let affected_rows = if previous.is_some() { 2 } else { 1 };
            self.mysqli_affected_rows.insert(handle_id, affected_rows);
            let next_insert_id = self
                .mysqli_insert_ids
                .get(&handle_id)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            self.mysqli_insert_ids.insert(handle_id, next_insert_id);
            return Ok(Value::Bool(true));
        }

        if let Some((option_name, option_value, autoload)) =
            parse_wordpress_option_insert_query(query)
        {
            let options = self.mysqli_wp_options.entry(handle_id).or_default();
            if options.contains_key(&option_name) {
                self.mysqli_affected_rows.insert(handle_id, 0);
                return Ok(Value::Bool(false));
            }
            options.insert(
                option_name,
                WordPressOptionState {
                    value: option_value,
                    autoload,
                },
            );
            self.mysqli_affected_rows.insert(handle_id, 1);
            let next_insert_id = self
                .mysqli_insert_ids
                .get(&handle_id)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            self.mysqli_insert_ids.insert(handle_id, next_insert_id);
            return Ok(Value::Bool(true));
        }

        if let Some((option_name, option_value)) = parse_wordpress_option_update_query(query) {
            if let Some(options) = self.mysqli_wp_options.get_mut(&handle_id) {
                let affected_rows = if let Some(option) = options.get_mut(&option_name) {
                    option.value = option_value;
                    1
                } else {
                    0
                };
                self.mysqli_affected_rows.insert(handle_id, affected_rows);
                return Ok(Value::Bool(true));
            }
        }

        if let Some(option_name) = parse_wordpress_option_delete_query(query) {
            if let Some(options) = self.mysqli_wp_options.get_mut(&handle_id) {
                let affected_rows = if options.remove(&option_name).is_some() {
                    1
                } else {
                    0
                };
                self.mysqli_affected_rows.insert(handle_id, affected_rows);
                return Ok(Value::Bool(true));
            }
        }

        if let Some(option_name) = parse_wordpress_option_value_select_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            if let Some(option_value) = self
                .mysqli_wp_options
                .get(&handle_id)
                .and_then(|options| options.get(&option_name))
                .map(|option| option.value.clone())
            {
                return self.create_mysqli_result_placeholder(
                    span,
                    vec!["option_value".to_string()],
                    vec![vec![(
                        "option_value".to_string(),
                        Value::String(option_value),
                    )]],
                );
            }
            return self.create_mysqli_result_placeholder(span, Vec::new(), Vec::new());
        }

        if let Some(option_name) = parse_wordpress_option_autoload_select_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            if let Some(autoload) = self
                .mysqli_wp_options
                .get(&handle_id)
                .and_then(|options| options.get(&option_name))
                .map(|option| option.autoload.clone())
            {
                return self.create_mysqli_result_placeholder(
                    span,
                    vec!["autoload".to_string()],
                    vec![vec![("autoload".to_string(), Value::String(autoload))]],
                );
            }
            return self.create_mysqli_result_placeholder(span, Vec::new(), Vec::new());
        }

        if let Some(option_name) = parse_wordpress_option_value_autoload_select_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            if let Some((option_value, autoload)) = self
                .mysqli_wp_options
                .get(&handle_id)
                .and_then(|options| options.get(&option_name))
                .map(|option| (option.value.clone(), option.autoload.clone()))
            {
                return self.create_mysqli_result_placeholder(
                    span,
                    vec!["option_value".to_string(), "autoload".to_string()],
                    vec![vec![
                        ("option_value".to_string(), Value::String(option_value)),
                        ("autoload".to_string(), Value::String(autoload)),
                    ]],
                );
            }
            return self.create_mysqli_result_placeholder(span, Vec::new(), Vec::new());
        }

        if let Some(filter) = parse_wordpress_options_row_select_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            if let Some(options) = self.mysqli_wp_options.get(&handle_id) {
                let rows = wordpress_option_rows_for_filter(options, &filter);
                if !rows.is_empty() {
                    return self.create_mysqli_result_placeholder(
                        span,
                        vec!["option_name".to_string(), "option_value".to_string()],
                        rows,
                    );
                }
            }
            return self.create_mysqli_result_placeholder(span, Vec::new(), Vec::new());
        }

        if is_wordpress_empty_result_query(query) || is_wordpress_empty_options_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            return self.create_mysqli_result_placeholder(span, Vec::new(), Vec::new());
        }

        if is_wordpress_seed_post_query(query) {
            self.mysqli_affected_rows.insert(handle_id, 0);
            return self.create_mysqli_result_placeholder(
                span,
                vec!["ID".to_string(), "post_title".to_string()],
                vec![vec![
                    ("ID".to_string(), Value::Int(1)),
                    (
                        "post_title".to_string(),
                        Value::String("Hello world placeholder".to_string()),
                    ),
                ]],
            );
        }

        if query == "SELECT @@SESSION.sql_mode" {
            self.mysqli_affected_rows.insert(handle_id, 0);
            return Ok(Value::Bool(false));
        }

        if is_mysqli_load_data_local_infile_query(query) {
            if self.mysqli_local_infile_enabled(handle_id) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_query()",
                        format!(
                            "LOAD DATA LOCAL INFILE execution is not implemented in the current subset; MYSQLI_OPT_LOCAL_INFILE placeholder state is recorded but host file loading and mutation SQL remain unsupported; got {query}"
                        ),
                    ),
                ));
            }

            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_query()",
                    format!(
                        "LOAD DATA LOCAL INFILE is disabled by MYSQLI_OPT_LOCAL_INFILE in the current placeholder connection; real local infile loading is not implemented; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_select_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_query()",
                    format!(
                        "non-empty mysqli result sets are not implemented in the current subset; only deterministic WordPress SQL mode, charset setup, empty options, metadata, and exact empty-result placeholders are supported; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_mutation_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_query()",
                    format!(
                        "mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got {query}"
                    ),
                ),
            ));
        }

        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_query()",
                format!(
                    "only the WordPress SQL mode probe, SQL-mode assignment, charset setup query, empty/exact wp_options SELECT placeholders, and exact wp_options insert/replace/update/delete state-island queries are implemented in the current subset; got {query}"
                ),
            ),
        ))
    }

    fn call_mysqli_real_query(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_real_query", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_real_query()", &args[0], span)?;
        let Value::String(query) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_query()",
                    format!(
                        "query argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };

        if is_mysqli_no_result_placeholder_query(query) {
            self.set_mysqli_pending_result_queue(handle_id, vec![MysqliMultiResultSlot::NoResult]);
            return Ok(Value::Bool(true));
        }

        if let Some(result) = mysqli_pending_result_for_query(query) {
            self.set_mysqli_pending_result_queue(
                handle_id,
                vec![MysqliMultiResultSlot::Result(result)],
            );
            return Ok(Value::Bool(true));
        }

        if is_mysqli_select_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_query()",
                    format!(
                        "general result-producing mysqli_real_query() SQL is not implemented; only deterministic pending result placeholders are supported in the current subset; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_mutation_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_query()",
                    format!(
                        "mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_load_data_local_infile_query(query) {
            if self.mysqli_local_infile_enabled(handle_id) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_real_query()",
                        format!(
                            "LOAD DATA LOCAL INFILE execution is not implemented in the current subset; MYSQLI_OPT_LOCAL_INFILE placeholder state is recorded but host file loading and mutation SQL remain unsupported; got {query}"
                        ),
                    ),
                ));
            }

            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_real_query()",
                    format!(
                        "LOAD DATA LOCAL INFILE is disabled by MYSQLI_OPT_LOCAL_INFILE in the current placeholder connection; real local infile loading is not implemented; got {query}"
                    ),
                ),
            ));
        }

        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_real_query()",
                format!(
                    "only the WordPress charset setup query, SQL-mode assignment, and deterministic pending result placeholders are implemented for mysqli_real_query() in the current subset; got {query}"
                ),
            ),
        ))
    }

    fn call_mysqli_multi_query(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_multi_query", args, 2, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_multi_query()", &args[0], span)?;
        let Value::String(query) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_multi_query()",
                    format!(
                        "query argument must be string in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };

        if query.contains(';') {
            if let Some(results) = mysqli_pending_results_for_multi_statement_query(query) {
                self.set_mysqli_pending_result_queue(handle_id, results);
                return Ok(Value::Bool(true));
            }

            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_multi_query()",
                    format!(
                        "multi-statement mysqli_multi_query() SQL is not implemented; only deterministic known no-result/result queues are supported in the current subset; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_no_result_placeholder_query(query) {
            self.set_mysqli_pending_result_queue(handle_id, vec![MysqliMultiResultSlot::NoResult]);
            return Ok(Value::Bool(true));
        }

        if let Some(result) = mysqli_pending_result_for_query(query) {
            self.set_mysqli_pending_result_queue(
                handle_id,
                vec![MysqliMultiResultSlot::Result(result)],
            );
            return Ok(Value::Bool(true));
        }

        if is_mysqli_select_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_multi_query()",
                    format!(
                        "general result-producing mysqli_multi_query() SQL is not implemented; only deterministic pending result placeholders are supported in the current subset; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_mutation_query(query) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_multi_query()",
                    format!(
                        "mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got {query}"
                    ),
                ),
            ));
        }

        if is_mysqli_load_data_local_infile_query(query) {
            if self.mysqli_local_infile_enabled(handle_id) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_multi_query()",
                        format!(
                            "LOAD DATA LOCAL INFILE execution is not implemented in the current subset; MYSQLI_OPT_LOCAL_INFILE placeholder state is recorded but host file loading and mutation SQL remain unsupported; got {query}"
                        ),
                    ),
                ));
            }

            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_multi_query()",
                    format!(
                        "LOAD DATA LOCAL INFILE is disabled by MYSQLI_OPT_LOCAL_INFILE in the current placeholder connection; real local infile loading is not implemented; got {query}"
                    ),
                ),
            ));
        }

        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_multi_query()",
                format!(
                    "only the WordPress charset setup query and SQL-mode assignment are implemented for mysqli_multi_query() in the current subset; got {query}"
                ),
            ),
        ))
    }

    fn call_mysqli_errno(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_errno", args, 1, span)?;
        expect_mysqli_handle("mysqli_errno()", &args[0], span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_error(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_error", args, 1, span)?;
        expect_mysqli_handle("mysqli_error()", &args[0], span)?;
        Ok(Value::String(String::new()))
    }

    fn call_mysqli_sqlstate(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_sqlstate", args, 1, span)?;
        expect_mysqli_handle("mysqli_sqlstate()", &args[0], span)?;
        Ok(Value::String("00000".to_string()))
    }

    fn call_mysqli_warning_count(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_warning_count", args, 1, span)?;
        expect_mysqli_handle("mysqli_warning_count()", &args[0], span)?;
        Ok(Value::Int(0))
    }

    fn call_mysqli_info(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_info", args, 1, span)?;
        expect_mysqli_handle("mysqli_info()", &args[0], span)?;
        Ok(Value::Null)
    }

    fn call_mysqli_get_warnings(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_get_warnings", args, 1, span)?;
        expect_mysqli_handle("mysqli_get_warnings()", &args[0], span)?;
        Ok(Value::Bool(false))
    }

    fn call_mysqli_affected_rows(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_affected_rows", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_affected_rows()", &args[0], span)?;
        Ok(Value::Int(
            self.mysqli_affected_rows
                .get(&handle_id)
                .copied()
                .unwrap_or(0),
        ))
    }

    fn call_mysqli_insert_id(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_insert_id", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_insert_id()", &args[0], span)?;
        Ok(Value::Int(
            self.mysqli_insert_ids.get(&handle_id).copied().unwrap_or(0),
        ))
    }

    fn call_mysqli_ping(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_ping", args, 1, span)?;
        expect_mysqli_handle("mysqli_ping()", &args[0], span)?;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_select_db(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_select_db", args, 2, span)?;
        let Value::Object(handle) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_select_db()",
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
                    "mysqli_select_db()",
                    format!(
                        "first argument must be mysqli object in the current subset, got {} object",
                        handle.class_name()
                    ),
                ),
            ));
        }
        if !matches!(args[1], Value::String(_) | Value::Null) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_select_db()",
                    format!(
                        "database argument must be string or null in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        }

        Ok(Value::Bool(true))
    }

    fn call_mysqli_real_escape_string(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_escape_string_impl("mysqli_real_escape_string", args, span)
    }

    fn call_mysqli_escape_string(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        self.call_mysqli_escape_string_impl("mysqli_escape_string", args, span)
    }

    fn call_mysqli_escape_string_impl(
        &self,
        function: &str,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        let call_name = callable_name(function);
        expect_arity(function, args, 2, span)?;
        let Value::Object(handle) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    call_name.clone(),
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
                    call_name.clone(),
                    format!(
                        "first argument must be mysqli object in the current subset, got {} object",
                        handle.class_name()
                    ),
                ),
            ));
        }

        if matches!(args[1], Value::Array(_)) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    call_name,
                    "data argument arrays are not implemented in the current subset",
                ),
            ));
        }

        let value = args[1]
            .try_echo_string()
            .map_err(|error| runtime_error(span, error))?;

        Ok(Value::String(mysql_escape_string(&value)))
    }

    fn call_mysqli_fetch_object(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_object", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_object()", &args[0], span)?;
        let row = {
            let state = self.mysqli_result_state_mut("mysqli_fetch_object()", result_id, span)?;
            if state.row_cursor >= state.rows.len() {
                return Ok(Value::Bool(false));
            }
            let row = state.rows[state.row_cursor].clone();
            state.row_cursor += 1;
            state.last_lengths = Some(mysqli_row_value_lengths(&row, span)?);
            row
        };
        self.create_stdclass_with_properties(row, span)
    }

    fn call_mysqli_fetch_assoc(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_assoc", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_assoc()", &args[0], span)?;
        self.fetch_mysqli_assoc_row("mysqli_fetch_assoc()", result_id, span)
    }

    fn call_mysqli_fetch_row(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_row", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_row()", &args[0], span)?;
        self.fetch_mysqli_array_row("mysqli_fetch_row()", result_id, PHP_MYSQLI_NUM, span)
    }

    fn call_mysqli_fetch_array(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_fetch_array()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }
        let result_id = expect_mysqli_result_handle("mysqli_fetch_array()", &args[0], span)?;
        let mode = match args.get(1) {
            Some(Value::Int(mode @ (PHP_MYSQLI_ASSOC | PHP_MYSQLI_NUM | PHP_MYSQLI_BOTH))) => *mode,
            Some(mode) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_fetch_array()",
                        format!(
                            "mode must be MYSQLI_ASSOC, MYSQLI_NUM, or MYSQLI_BOTH in the current subset, got {}",
                            mode.type_name()
                        ),
                    ),
                ));
            }
            None => PHP_MYSQLI_BOTH,
        };
        self.fetch_mysqli_array_row("mysqli_fetch_array()", result_id, mode, span)
    }

    fn call_mysqli_fetch_all(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_fetch_all()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }
        let result_id = expect_mysqli_result_handle("mysqli_fetch_all()", &args[0], span)?;
        let mode = match args.get(1) {
            Some(Value::Int(mode @ (PHP_MYSQLI_ASSOC | PHP_MYSQLI_NUM | PHP_MYSQLI_BOTH))) => *mode,
            Some(mode) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_fetch_all()",
                        format!(
                            "mode must be MYSQLI_ASSOC, MYSQLI_NUM, or MYSQLI_BOTH in the current subset, got {}",
                            mode.type_name()
                        ),
                    ),
                ));
            }
            None => PHP_MYSQLI_NUM,
        };
        let rows = {
            let state = self.mysqli_result_state_mut("mysqli_fetch_all()", result_id, span)?;
            let rows = state.rows[state.row_cursor..].to_vec();
            state.row_cursor = state.rows.len();
            if let Some(row) = rows.last() {
                state.last_lengths = Some(mysqli_row_value_lengths(row, span)?);
            }
            rows
        };

        let mut outer = PhpArray::new();
        for (index, row) in rows.into_iter().enumerate() {
            outer.insert(index as i64, Self::mysqli_array_from_row(row, mode));
        }
        Ok(Value::Array(outer))
    }

    fn call_mysqli_fetch_column(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_fetch_column()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }
        let result_id = expect_mysqli_result_handle("mysqli_fetch_column()", &args[0], span)?;
        let column = match args.get(1) {
            Some(Value::Int(column)) => *column,
            Some(column) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_fetch_column()",
                        format!(
                            "column must be int in the current subset, got {}",
                            column.type_name()
                        ),
                    ),
                ));
            }
            None => 0,
        };
        let row = {
            let state = self.mysqli_result_state_mut("mysqli_fetch_column()", result_id, span)?;
            if state.row_cursor >= state.rows.len() {
                return Ok(Value::Bool(false));
            }
            let row = state.rows[state.row_cursor].clone();
            state.row_cursor += 1;
            state.last_lengths = Some(mysqli_row_value_lengths(&row, span)?);
            row
        };
        let Ok(column) = usize::try_from(column) else {
            return Ok(Value::Null);
        };
        Ok(row
            .get(column)
            .map(|(_, value)| value.clone())
            .unwrap_or(Value::Null))
    }

    fn fetch_mysqli_assoc_row(
        &mut self,
        function: &str,
        result_id: i64,
        span: Span,
    ) -> CompileResult<Value> {
        let row = {
            let state = self.mysqli_result_state_mut(function, result_id, span)?;
            if state.row_cursor >= state.rows.len() {
                return Ok(Value::Bool(false));
            }
            let row = state.rows[state.row_cursor].clone();
            state.row_cursor += 1;
            state.last_lengths = Some(mysqli_row_value_lengths(&row, span)?);
            row
        };

        let mut array = PhpArray::new();
        for (name, value) in row {
            array.insert(name, value);
        }
        Ok(Value::Array(array))
    }

    fn fetch_mysqli_array_row(
        &mut self,
        function: &str,
        result_id: i64,
        mode: i64,
        span: Span,
    ) -> CompileResult<Value> {
        let row = {
            let state = self.mysqli_result_state_mut(function, result_id, span)?;
            if state.row_cursor >= state.rows.len() {
                return Ok(Value::Bool(false));
            }
            let row = state.rows[state.row_cursor].clone();
            state.row_cursor += 1;
            state.last_lengths = Some(mysqli_row_value_lengths(&row, span)?);
            row
        };

        let mut array = PhpArray::new();
        for (index, (name, value)) in row.into_iter().enumerate() {
            if mode == PHP_MYSQLI_NUM || mode == PHP_MYSQLI_BOTH {
                array.insert(index as i64, value.clone());
            }
            if mode == PHP_MYSQLI_ASSOC || mode == PHP_MYSQLI_BOTH {
                array.insert(name, value);
            }
        }
        Ok(Value::Array(array))
    }

    fn mysqli_array_from_row(row: Vec<(String, Value)>, mode: i64) -> Value {
        let mut array = PhpArray::new();
        for (index, (name, value)) in row.into_iter().enumerate() {
            if mode == PHP_MYSQLI_NUM || mode == PHP_MYSQLI_BOTH {
                array.insert(index as i64, value.clone());
            }
            if mode == PHP_MYSQLI_ASSOC || mode == PHP_MYSQLI_BOTH {
                array.insert(name, value);
            }
        }
        Value::Array(array)
    }

    fn call_mysqli_fetch_field(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_field", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_field()", &args[0], span)?;
        let field = {
            let state = self.mysqli_result_state_mut("mysqli_fetch_field()", result_id, span)?;
            if state.field_cursor >= state.fields.len() {
                return Ok(Value::Bool(false));
            }
            let field = state.fields[state.field_cursor].clone();
            state.field_cursor += 1;
            field
        };
        self.create_stdclass_with_properties(mysqli_field_metadata_properties(&field), span)
    }

    fn call_mysqli_fetch_fields(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_fields", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_fields()", &args[0], span)?;
        let fields = self
            .mysqli_result_state("mysqli_fetch_fields()", result_id, span)?
            .fields
            .clone();
        let mut array = PhpArray::new();
        for (index, field) in fields.into_iter().enumerate() {
            let field = self
                .create_stdclass_with_properties(mysqli_field_metadata_properties(&field), span)?;
            array.insert(index as i64, field);
        }
        Ok(Value::Array(array))
    }

    fn call_mysqli_fetch_field_direct(
        &mut self,
        args: &[Value],
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_field_direct", args, 2, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_field_direct()", &args[0], span)?;
        let index = match &args[1] {
            Value::Int(index) => *index,
            value => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_fetch_field_direct()",
                        format!(
                            "field index must be int in the current subset, got {}",
                            value.type_name()
                        ),
                    ),
                ));
            }
        };
        let Ok(index) = usize::try_from(index) else {
            return Ok(Value::Bool(false));
        };
        let field = self
            .mysqli_result_state("mysqli_fetch_field_direct()", result_id, span)?
            .fields
            .get(index)
            .cloned();
        match field {
            Some(field) => {
                self.create_stdclass_with_properties(mysqli_field_metadata_properties(&field), span)
            }
            None => Ok(Value::Bool(false)),
        }
    }

    fn call_mysqli_num_fields(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_num_fields", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_num_fields()", &args[0], span)?;
        let state = self.mysqli_result_state("mysqli_num_fields()", result_id, span)?;
        Ok(Value::Int(state.fields.len() as i64))
    }

    fn call_mysqli_num_rows(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_num_rows", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_num_rows()", &args[0], span)?;
        let state = self.mysqli_result_state("mysqli_num_rows()", result_id, span)?;
        Ok(Value::Int(state.rows.len() as i64))
    }

    fn call_mysqli_fetch_lengths(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_fetch_lengths", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_fetch_lengths()", &args[0], span)?;
        let state = self.mysqli_result_state("mysqli_fetch_lengths()", result_id, span)?;
        let Some(lengths) = &state.last_lengths else {
            return Ok(Value::Bool(false));
        };
        let mut array = PhpArray::new();
        for (index, length) in lengths.iter().enumerate() {
            array.insert(index as i64, Value::Int(*length as i64));
        }
        Ok(Value::Array(array))
    }

    fn call_mysqli_data_seek(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_data_seek", args, 2, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_data_seek()", &args[0], span)?;
        let offset = match &args[1] {
            Value::Int(offset) => *offset,
            value => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_data_seek()",
                        format!(
                            "offset must be int in the current subset, got {}",
                            value.type_name()
                        ),
                    ),
                ));
            }
        };
        let state = self.mysqli_result_state_mut("mysqli_data_seek()", result_id, span)?;
        let Ok(offset) = usize::try_from(offset) else {
            return Ok(Value::Bool(false));
        };
        if offset >= state.rows.len() {
            return Ok(Value::Bool(false));
        }
        state.row_cursor = offset;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_field_seek(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_field_seek", args, 2, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_field_seek()", &args[0], span)?;
        let offset = match &args[1] {
            Value::Int(offset) => *offset,
            value => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_field_seek()",
                        format!(
                            "field offset must be int in the current subset, got {}",
                            value.type_name()
                        ),
                    ),
                ));
            }
        };
        let state = self.mysqli_result_state_mut("mysqli_field_seek()", result_id, span)?;
        let Ok(offset) = usize::try_from(offset) else {
            return Ok(Value::Bool(false));
        };
        if offset >= state.fields.len() {
            return Ok(Value::Bool(false));
        }
        state.field_cursor = offset;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_field_tell(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_field_tell", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_field_tell()", &args[0], span)?;
        let state = self.mysqli_result_state("mysqli_field_tell()", result_id, span)?;
        Ok(Value::Int(state.field_cursor as i64))
    }

    fn call_mysqli_free_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_free_result", args, 1, span)?;
        let result_id = expect_mysqli_result_handle("mysqli_free_result()", &args[0], span)?;
        self.mysqli_results.remove(&result_id);
        Ok(Value::Null)
    }

    fn call_mysqli_more_results(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_more_results", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_more_results()", &args[0], span)?;
        Ok(Value::Bool(
            self.mysqli_pending_result_queues
                .get(&handle_id)
                .is_some_and(|results| !results.is_empty()),
        ))
    }

    fn call_mysqli_next_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_next_result", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_next_result()", &args[0], span)?;
        if self.mysqli_pending_results.contains_key(&handle_id) {
            return Ok(Value::Bool(false));
        }

        let Some(results) = self.mysqli_pending_result_queues.get_mut(&handle_id) else {
            return Ok(Value::Bool(false));
        };
        let Some(slot) = results.pop_front() else {
            self.mysqli_pending_result_queues.remove(&handle_id);
            return Ok(Value::Bool(false));
        };
        let queue_is_empty = results.is_empty();
        if queue_is_empty {
            self.mysqli_pending_result_queues.remove(&handle_id);
        }
        match slot {
            MysqliMultiResultSlot::NoResult => {
                self.mysqli_pending_results.remove(&handle_id);
            }
            MysqliMultiResultSlot::Result(result) => {
                self.mysqli_pending_results.insert(handle_id, result);
            }
        }
        Ok(Value::Bool(true))
    }

    fn call_mysqli_store_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_store_result", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_store_result()", &args[0], span)?;
        let Some(result) = self.mysqli_pending_results.remove(&handle_id) else {
            return Ok(Value::Bool(false));
        };
        self.create_mysqli_result_placeholder(span, result.fields, result.rows)
    }

    fn call_mysqli_use_result(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_use_result", args, 1, span)?;
        let handle_id = expect_mysqli_handle_id("mysqli_use_result()", &args[0], span)?;
        let Some(result) = self.mysqli_pending_results.remove(&handle_id) else {
            return Ok(Value::Bool(false));
        };
        self.create_mysqli_result_placeholder(span, result.fields, result.rows)
    }

    fn call_mysqli_reap_async_query(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("mysqli_reap_async_query", args, 1, span)?;
        expect_mysqli_handle("mysqli_reap_async_query()", &args[0], span)?;
        Ok(Value::Bool(false))
    }

    fn call_mysqli_poll(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(4..=5).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_poll()",
                    ArityExpectation::Between { min: 4, max: 5 },
                    args.len(),
                ),
            ));
        }
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_poll()",
                "async socket readiness and by-reference array mutation are not implemented in the current subset",
            ),
        ))
    }

    fn mysqli_result_state(
        &self,
        function: &str,
        result_id: i64,
        span: Span,
    ) -> CompileResult<&MysqliResultState> {
        self.mysqli_results.get(&result_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "placeholder result state is not available in the current subset",
                ),
            )
        })
    }

    fn mysqli_result_state_mut(
        &mut self,
        function: &str,
        result_id: i64,
        span: Span,
    ) -> CompileResult<&mut MysqliResultState> {
        self.mysqli_results.get_mut(&result_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "placeholder result state is not available in the current subset",
                ),
            )
        })
    }

    fn mysqli_statement_state(
        &self,
        function: &str,
        stmt_id: i64,
        span: Span,
    ) -> CompileResult<&MysqliStatementState> {
        self.mysqli_statements.get(&stmt_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "placeholder statement state is not available in the current subset",
                ),
            )
        })
    }

    fn mysqli_statement_state_mut(
        &mut self,
        function: &str,
        stmt_id: i64,
        span: Span,
    ) -> CompileResult<&mut MysqliStatementState> {
        self.mysqli_statements.get_mut(&stmt_id).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "placeholder statement state is not available in the current subset",
                ),
            )
        })
    }

    fn evaluate_array_index(
        &mut self,
        target: &Expr,
        index: &Expr,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if let Expr::Variable(name, _) = target {
            if name == "GLOBALS" {
                let key = self.evaluate_array_key(index, scope)?;
                let Some(global_name) = globals_offset_name(&key) else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "$GLOBALS",
                            "only string-keyed direct offset reads are implemented",
                        ),
                    ));
                };
                return scope.read_global_name(global_name).ok_or_else(|| {
                    runtime_error(span, RuntimeError::undefined_variable(global_name))
                });
            }
        }

        let target_value = self.evaluate(target, scope)?;
        let key = self.evaluate_array_key(index, scope)?;

        match target_value {
            Value::Array(array) => array.get(key.clone()).cloned().ok_or_else(|| {
                runtime_error(
                    span,
                    RuntimeError::undefined_array_key(key.diagnostic_key()),
                )
            }),
            Value::Object(object) => self.call_array_access_method(
                object,
                "offsetGet",
                vec![Self::array_key_value(Some(key))],
                span,
            ),
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
        if let Expr::Variable(name, _) = target {
            let key = self.evaluate_array_key(index, scope)?;
            return match scope.read_named(name) {
                Some(Value::Array(array)) => Ok(array
                    .get(key)
                    .cloned()
                    .filter(|value| !matches!(value, Value::Null))),
                Some(Value::Object(object))
                    if self
                        .classes
                        .implements_interface(object.class_id(), "ArrayAccess") =>
                {
                    if !self.array_access_offset_exists(
                        object.clone(),
                        key.clone(),
                        index.span(),
                    )? {
                        return Ok(None);
                    }
                    Ok(Some(self.call_array_access_method(
                        object,
                        "offsetGet",
                        vec![Self::array_key_value(Some(key))],
                        index.span(),
                    )?)
                    .filter(|value| !matches!(value, Value::Null)))
                }
                Some(_) | None => Ok(None),
            };
        }

        if let Some((object_name, property, indices)) =
            Self::collect_direct_object_property_array_index_path(target, index)
        {
            let mut keys = Vec::with_capacity(indices.len());
            for index in indices {
                keys.push(self.evaluate_array_key(index, scope)?);
            }

            let Some(Value::Object(object)) = scope.read_named(object_name) else {
                return Ok(None);
            };

            let (current_class_id, protected_class_ids) = self.current_property_access_context();
            let Some(value) = object
                .read_property_for_isset_from_context(
                    property,
                    current_class_id,
                    &protected_class_ids,
                )
                .map_err(|error| runtime_error(target.span(), error))?
            else {
                return Ok(None);
            };

            return match value {
                Value::Object(object)
                    if keys.len() == 1
                        && self
                            .classes
                            .implements_interface(object.class_id(), "ArrayAccess") =>
                {
                    if !self.array_access_offset_exists(
                        object.clone(),
                        keys[0].clone(),
                        index.span(),
                    )? {
                        return Ok(None);
                    }
                    Ok(Some(self.call_array_access_method(
                        object,
                        "offsetGet",
                        vec![Self::array_key_value(Some(keys[0].clone()))],
                        index.span(),
                    )?)
                    .filter(|value| !matches!(value, Value::Null)))
                }
                value => Ok(Self::array_path_value(&value, &keys)
                    .filter(|value| !matches!(value, Value::Null))),
            };
        }

        Err(runtime_error(
            target.span(),
            RuntimeError::unsupported_call(
                "??",
                "left operand must be a direct variable, direct array offset, direct object property, or supported static property in the current subset",
            ),
        ))
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
                match object.read_property_from_context(
                    property,
                    current_class_id,
                    &protected_class_ids,
                ) {
                    Ok(value) => Ok(value),
                    Err(error) if Self::is_undefined_property_error(&error) => self
                        .call_magic_property_method(object, "__get", property, span)?
                        .ok_or_else(|| runtime_error(span, error)),
                    Err(error) => Err(runtime_error(span, error)),
                }
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

    fn call_magic_property_method(
        &mut self,
        object: PhpObject,
        method_name: &str,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        self.call_magic_instance_method_with_values(
            object,
            method_name,
            vec![Value::String(property.to_string())],
            span,
        )
    }

    fn call_magic_get_reference_return_cell(
        &mut self,
        object: PhpObject,
        property: &str,
        span: Span,
    ) -> CompileResult<Option<VariableCell>> {
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(object.class_id(), "__get")
        else {
            return Ok(None);
        };

        if is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::__get()"),
                    "static magic instance methods are not implemented in the current subset",
                ),
            ));
        }

        self.ensure_instance_method_visible(class_id, &class_name, "__get", visibility, span)?;

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::__get()"),
                    "magic __get reference sources require __get() to return by reference in the current subset",
                ),
            ));
        }
        ensure_user_function_arity(function, 1, span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let called_class_id = object.class_id();
        self.call_reference_return_function_with_checked_values(
            function,
            vec![Value::String(property.to_string())],
            Some(object),
            Some(class_id),
            Some(called_class_id),
            Vec::new(),
        )
        .map(Some)
    }

    fn call_magic_instance_method_with_values(
        &mut self,
        object: PhpObject,
        method_name: &str,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Option<Value>> {
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(object.class_id(), method_name)
        else {
            return Ok(None);
        };

        if is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "static magic instance methods are not implemented in the current subset",
                ),
            ));
        }

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, args.len(), span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let called_class_id = object.class_id();
        self.call_user_function_with_this(
            function,
            object,
            args,
            Some(class_id),
            Some(called_class_id),
        )
        .map(Some)
    }

    fn is_undefined_property_error(error: &RuntimeError) -> bool {
        matches!(error.kind(), RuntimeErrorKind::UndefinedProperty { .. })
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
            let receiver_class_name = self
                .classes
                .get(object.class_id())
                .expect("object class id should resolve to class metadata")
                .name()
                .to_string();
            let Some(method) = self.resolve_instance_method(object.class_id(), method_name) else {
                return match self.call_missing_instance_method_via_magic(
                    object,
                    method_name,
                    args,
                    span,
                    caller_scope,
                )? {
                    Some(value) => Ok(value),
                    None => Err(runtime_error(
                        span,
                        RuntimeError::undefined_function(format!(
                            "{receiver_class_name}::{method_name}()"
                        )),
                    )),
                };
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
        ensure_supported_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        let called_class_id = object.class_id();
        self.call_user_function_with_checked_values(
            function,
            values,
            Some(object),
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
            Some(caller_scope),
        )
    }

    fn call_missing_instance_method_via_magic(
        &mut self,
        object: PhpObject,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Option<Value>> {
        if self
            .resolve_instance_method(object.class_id(), "__call")
            .is_none()
        {
            return Ok(None);
        }

        let mut argument_array = PhpArray::new();
        for arg in args {
            let value = self.evaluate(arg, caller_scope)?;
            argument_array
                .append(value)
                .map_err(|error| runtime_error(arg.span(), error))?;
        }

        self.call_magic_instance_method_with_values(
            object,
            "__call",
            vec![
                Value::String(method_name.to_string()),
                Value::Array(argument_array),
            ],
            span,
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
        ensure_supported_function_signature(function, args.len(), span)?;
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
                Vec::new(),
                None,
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
        let receiver_class_name = receiver_class.name().to_string();
        let Some((
            declaring_class_id,
            declaring_class_name,
            _resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(class_id, method_name)
        else {
            return match self.call_missing_static_method_via_magic(
                class_id,
                method_name,
                args,
                span,
                caller_scope,
            )? {
                Some(value) => Ok(value),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{receiver_class_name}::{method_name}()"
                    )),
                )),
            };
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
            ensure_supported_function_metadata(function, span)?;
            self.ensure_user_function_call_depth(function, span)?;

            let (values, reference_bindings) =
                self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

            return self.call_user_function_with_checked_values(
                function,
                values,
                None,
                Some(declaring_class_id),
                Some(class_id),
                reference_bindings,
                Some(caller_scope),
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
        let receiver_class_name = receiver_class.name().to_string();
        let Some((
            declaring_class_id,
            declaring_class_name,
            resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(receiver_class_id, method_name)
        else {
            return match self.call_missing_static_method_via_magic(
                receiver_class_id,
                method_name,
                args,
                span,
                caller_scope,
            )? {
                Some(value) => Ok(value),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{receiver_class_name}::{method_name}()"
                    )),
                )),
            };
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
        ensure_supported_function_signature(function, args.len(), span)?;
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
            Vec::new(),
            None,
        )
    }

    fn call_missing_static_method_via_magic(
        &mut self,
        receiver_class_id: ClassId,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Option<Value>> {
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(receiver_class_id, "__callStatic")
        else {
            return Ok(None);
        };

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::__callStatic()"),
                    "magic static missing-method dispatch requires a static __callStatic method in the current subset",
                ),
            ));
        }

        self.ensure_instance_method_visible(
            class_id,
            &class_name,
            "__callStatic",
            visibility,
            span,
        )?;

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        ensure_user_function_arity(function, 2, span)?;
        ensure_supported_function_signature(function, 2, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let mut argument_array = PhpArray::new();
        for arg in args {
            let value = self.evaluate(arg, caller_scope)?;
            argument_array
                .append(value)
                .map_err(|error| runtime_error(arg.span(), error))?;
        }

        self.call_user_function_with_checked_values(
            function,
            vec![
                Value::String(method_name.to_string()),
                Value::Array(argument_array),
            ],
            None,
            Some(class_id),
            Some(receiver_class_id),
            Vec::new(),
            None,
        )
        .map(Some)
    }

    fn evaluate_interpolated_string(
        &mut self,
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
                    let text = self.value_to_echo_string(value, span)?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::ArrayOffset { variable, key } => {
                    let value =
                        self.evaluate_interpolated_array_offset(variable, key, span, scope)?;
                    let text = self.value_to_echo_string(value, span)?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::ObjectProperty { variable, property } => {
                    let value = self
                        .evaluate_interpolated_object_property(variable, property, span, scope)?;
                    let text = self.value_to_echo_string(value, span)?;
                    output.push_str(&text);
                }
                InterpolatedStringPart::AccessChain { variable, segments } => {
                    let value =
                        self.evaluate_interpolated_access_chain(variable, segments, span, scope)?;
                    let text = self.value_to_echo_string(value, span)?;
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

    fn evaluate_object_static_property(
        &mut self,
        target: &Expr,
        property: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let (class_id, class_name) =
            self.resolve_dynamic_static_receiver(target, property, span, scope)?;
        self.read_resolved_static_property(class_id, &class_name, property, span)
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

    fn write_object_static_property(
        &mut self,
        target: &Expr,
        property: &str,
        value: Value,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let (class_id, class_name) =
            self.resolve_dynamic_static_receiver(target, property, span, scope)?;
        self.write_resolved_static_property(class_id, &class_name, property, value, span)
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

    fn resolve_dynamic_static_receiver(
        &mut self,
        target: &Expr,
        member_name: &str,
        span: Span,
        scope: &mut SymbolTable,
    ) -> CompileResult<(ClassId, String)> {
        let target_value = self.evaluate(target, scope)?;
        match target_value {
            Value::Object(object) => {
                let class_id = object.class_id();
                let class_name = self
                    .classes
                    .get(class_id)
                    .expect("object class id should resolve to class metadata")
                    .name()
                    .to_string();
                Ok((class_id, class_name))
            }
            Value::String(class_name) => {
                let class_id = self.classes.lookup_class_id(&class_name).ok_or_else(|| {
                    runtime_error(span, RuntimeError::undefined_class(&class_name))
                })?;
                Ok((class_id, class_name))
            }
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("::${member_name}"),
                    format!(
                        "dynamic static property receiver must be object or class string, got {}",
                        other.type_name()
                    ),
                ),
            )),
        }
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
        let current_class_name = current_class.name().to_string();
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(current_class_id, method_name)
        else {
            return match self.call_missing_static_method_via_magic(
                current_class_id,
                method_name,
                args,
                span,
                caller_scope,
            )? {
                Some(value) => Ok(value),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{current_class_name}::{method_name}()"
                    )),
                )),
            };
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        let called_class_id = self
            .called_class_context
            .last()
            .copied()
            .unwrap_or(current_class_id);

        if is_static {
            let function =
                self.method_function(class_id, &class_name, &resolved_method_name, span)?;
            let function = function.as_ref();
            ensure_user_function_arity(function, args.len(), span)?;
            ensure_supported_function_metadata(function, span)?;
            self.ensure_user_function_call_depth(function, span)?;

            let (values, reference_bindings) =
                self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

            self.call_user_function_with_checked_values(
                function,
                values,
                None,
                Some(class_id),
                Some(called_class_id),
                reference_bindings,
                Some(caller_scope),
            )
        } else {
            let function =
                self.method_function(class_id, &class_name, &resolved_method_name, span)?;
            let function = function.as_ref();
            ensure_user_function_arity(function, args.len(), span)?;
            ensure_supported_function_signature(function, args.len(), span)?;
            self.ensure_user_function_call_depth(function, span)?;

            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(self.evaluate(arg, caller_scope)?);
            }

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
        let called_class_name = called_class.name().to_string();
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(called_class_id, method_name)
        else {
            return match self.call_missing_static_method_via_magic(
                called_class_id,
                method_name,
                args,
                span,
                caller_scope,
            )? {
                Some(value) => Ok(value),
                None => Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{called_class_name}::{method_name}()"
                    )),
                )),
            };
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
        ensure_supported_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_user_function_with_checked_values(
            function,
            values,
            None,
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
            Some(caller_scope),
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
                if key == "preg_replace_callback" {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        values.push(self.evaluate(arg, caller_scope)?);
                    }
                    return self.call_preg_replace_callback(values, span);
                }
                if key == "str_replace" {
                    return self.call_str_replace_with_optional_count(args, span, caller_scope);
                }
                if key == "call_user_func" {
                    return self.call_user_func_direct(args, span, caller_scope);
                }
                if key == "call_user_func_array" {
                    return self.call_user_func_array_direct(args, span, caller_scope);
                }
                if key == "mysqli_stmt_bind_param" {
                    return self.call_mysqli_stmt_bind_param_direct(args, span, caller_scope);
                }
                if let Some(function) = mysqli_stmt_execute_function_label(&key) {
                    return self.call_mysqli_stmt_execute_direct(
                        function,
                        args,
                        span,
                        caller_scope,
                    );
                }
                if key == "mysqli_stmt_bind_result" {
                    return self.call_mysqli_stmt_bind_result_direct(args, span, caller_scope);
                }
                if key == "mysqli_stmt_fetch" {
                    return self.call_mysqli_stmt_fetch_direct(args, span, caller_scope);
                }
                if key == "compact" {
                    return self.call_compact(args, span, caller_scope);
                }
                if key == "ksort" {
                    return self.call_ksort(args, span, caller_scope);
                }
                if key == "array_unshift" {
                    return self.call_array_unshift(args, span, caller_scope);
                }
                if key == "array_pop" {
                    return self.call_array_pop(args, span, caller_scope);
                }
                if key == "next" {
                    return self.call_next(args, span, caller_scope);
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
                if key == "preg_replace_callback" {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        values.push(self.evaluate(arg, caller_scope)?);
                    }
                    return self.call_preg_replace_callback(values, span);
                }
                if key == "str_replace" {
                    return self.call_str_replace_with_optional_count(args, span, caller_scope);
                }
                if key == "call_user_func" {
                    return self.call_user_func_direct(args, span, caller_scope);
                }
                if key == "call_user_func_array" {
                    return self.call_user_func_array_direct(args, span, caller_scope);
                }
                if key == "mysqli_stmt_bind_param" {
                    return self.call_mysqli_stmt_bind_param_direct(args, span, caller_scope);
                }
                if let Some(function) = mysqli_stmt_execute_function_label(&key) {
                    return self.call_mysqli_stmt_execute_direct(
                        function,
                        args,
                        span,
                        caller_scope,
                    );
                }
                if key == "mysqli_stmt_bind_result" {
                    return self.call_mysqli_stmt_bind_result_direct(args, span, caller_scope);
                }
                if key == "mysqli_stmt_fetch" {
                    return self.call_mysqli_stmt_fetch_direct(args, span, caller_scope);
                }
                if key == "compact" {
                    return self.call_compact(args, span, caller_scope);
                }
                if key == "ksort" {
                    return self.call_ksort(args, span, caller_scope);
                }
                if key == "array_unshift" {
                    return self.call_array_unshift(args, span, caller_scope);
                }
                if key == "array_pop" {
                    return self.call_array_pop(args, span, caller_scope);
                }
                if key == "next" {
                    return self.call_next(args, span, caller_scope);
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

    fn call_user_func_direct(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
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

        let callback = self.evaluate(&args[0], caller_scope)?;
        let callback_name = match &callback {
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
        if let Callable::Builtin(key) = &callable {
            if let Some(function) = mysqli_stmt_execute_function_label(key) {
                return self.call_mysqli_stmt_execute_direct(
                    function,
                    &args[1..],
                    span,
                    caller_scope,
                );
            }
        }

        let mut values = Vec::with_capacity(args.len().saturating_sub(1));
        for arg in &args[1..] {
            values.push(self.evaluate(arg, caller_scope)?);
        }
        self.call_callable_with_values(callable, values, span)
    }

    fn call_user_func_array_direct(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() != 2 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "call_user_func_array()",
                    ArityExpectation::Exactly(2),
                    args.len(),
                ),
            ));
        }

        let callback = self.evaluate(&args[0], caller_scope)?;

        match &callback {
            Value::String(callback_name) => {
                let callable = self.lookup_function(callback_name).ok_or_else(|| {
                    runtime_error(
                        span,
                        RuntimeError::undefined_function(callable_name(callback_name)),
                    )
                })?;
                if let Callable::Builtin(key) = &callable {
                    if let Some(function) = mysqli_stmt_execute_function_label(key) {
                        let positional_args = self.evaluate_call_user_func_array_arguments(
                            &args[1],
                            span,
                            caller_scope,
                        )?;
                        return self.call_mysqli_stmt_execute_with_refresh(
                            function,
                            &positional_args,
                            span,
                            caller_scope,
                        );
                    }
                }
                if let Callable::User(function) = &callable {
                    if function.params.iter().any(|param| param.by_reference) {
                        return self.call_user_func_array_user_function(
                            function.clone(),
                            &args[1],
                            span,
                            caller_scope,
                        );
                    }
                }
                let positional_args =
                    self.evaluate_call_user_func_array_arguments(&args[1], span, caller_scope)?;
                self.call_callable_with_values(callable, positional_args, span)
            }
            Value::Array(callback) => {
                self.call_user_func_array_array_callable(callback, &args[1], span, caller_scope)
            }
            Value::Closure(_) => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    "closure invocation is not implemented",
                ),
            )),
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    format!(
                        "callback must evaluate to string or array callable in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            )),
        }
    }

    fn evaluate_call_user_func_array_arguments(
        &mut self,
        argument_expr: &Expr,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Vec<Value>> {
        let argument_array = self.evaluate(argument_expr, caller_scope)?;
        let Value::Array(argument_array) = &argument_array else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    format!(
                        "argument array must be array in the current subset, got {}",
                        argument_array.type_name()
                    ),
                ),
            ));
        };

        Self::call_user_func_array_positional_values(argument_array, span)
    }

    fn call_user_func_array_positional_values(
        argument_array: &PhpArray,
        span: Span,
    ) -> CompileResult<Vec<Value>> {
        let mut positional_args = Vec::with_capacity(argument_array.len());
        for entry in argument_array.entries() {
            if matches!(entry.key, ArrayKey::String(_)) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "call_user_func_array()",
                        "string-keyed named arguments are not implemented in the current subset",
                    ),
                ));
            }
            positional_args.push(entry.value_cloned());
        }

        Ok(positional_args)
    }

    fn call_user_func_array_user_function(
        &mut self,
        function: Rc<FunctionDecl>,
        argument_expr: &Expr,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let function = function.as_ref();
        let (values, reference_bindings) = self.evaluate_call_user_func_array_checked_arguments(
            function,
            argument_expr,
            span,
            caller_scope,
        )?;
        self.ensure_user_function_call_depth(function, span)?;

        self.call_user_function_with_checked_values(
            function,
            values,
            None,
            None,
            None,
            reference_bindings,
            Some(caller_scope),
        )
    }

    fn evaluate_call_user_func_array_checked_arguments(
        &mut self,
        function: &FunctionDecl,
        argument_expr: &Expr,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<(Vec<Value>, Vec<ReferenceBinding>)> {
        let Expr::Array { items, .. } = argument_expr else {
            let positional_args =
                self.evaluate_call_user_func_array_arguments(argument_expr, span, caller_scope)?;
            ensure_user_function_arity(function, positional_args.len(), span)?;
            ensure_supported_function_metadata(function, span)?;
            if function
                .params
                .iter()
                .enumerate()
                .any(|(index, param)| param.by_reference && index < positional_args.len())
            {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        callable_name(&function.name),
                        "call_user_func_array() reference parameter invocation requires an array literal with by-reference direct variable elements in the current subset",
                    ),
                ));
            }
            return Ok((positional_args, Vec::new()));
        };

        ensure_user_function_arity(function, items.len(), span)?;
        ensure_supported_function_metadata(function, span)?;
        if !function
            .params
            .iter()
            .enumerate()
            .any(|(index, param)| param.by_reference && index < items.len())
        {
            let positional_args =
                self.evaluate_call_user_func_array_arguments(argument_expr, span, caller_scope)?;
            return Ok((positional_args, Vec::new()));
        }
        if function
            .params
            .iter()
            .enumerate()
            .any(|(index, param)| param.by_reference && param.is_variadic && index < items.len())
        {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "variadic reference parameter invocation is not implemented",
                ),
            ));
        }

        let mut values = Vec::with_capacity(items.len());
        let mut reference_bindings = Vec::new();
        for (index, item) in items.iter().enumerate() {
            if item.key.is_some() {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        callable_name(&function.name),
                        "call_user_func_array() reference parameter invocation is only implemented for unkeyed literal argument arrays in the current subset",
                    ),
                ));
            }

            let Some(param) = function.params.get(index) else {
                values.push(self.evaluate(&item.value, caller_scope)?);
                continue;
            };

            if param.by_reference {
                if !item.by_reference {
                    return Err(runtime_error(
                        item.value.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "call_user_func_array() reference parameter invocation requires a by-reference array element in the current subset",
                        ),
                    ));
                }
                let Expr::Variable(caller_name, _) = &item.value else {
                    return Err(runtime_error(
                        item.value.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "call_user_func_array() reference parameter invocation is only implemented for direct variable array elements in the current subset",
                        ),
                    ));
                };
                if caller_scope.is_array_offset_alias_name(caller_name) {
                    return Err(runtime_error(
                        item.value.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "call_user_func_array() reference array elements routed through array-offset alias metadata are not implemented",
                        ),
                    ));
                }
                let caller_cell = caller_scope.read_cell(caller_name).ok_or_else(|| {
                    runtime_error(
                        item.value.span(),
                        RuntimeError::undefined_variable(caller_name),
                    )
                })?;
                values.push(caller_cell.borrow().clone());
                reference_bindings.push(ReferenceBinding {
                    param_name: param.name.clone(),
                    target: ReferenceBindingTarget::CallerCell(caller_cell),
                });
            } else {
                values.push(self.evaluate(&item.value, caller_scope)?);
            }
        }

        Ok((values, reference_bindings))
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

    fn call_str_replace_with_optional_count(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
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

        let search = self.evaluate(&args[0], caller_scope)?;
        let replace = self.evaluate(&args[1], caller_scope)?;
        let subject = self.evaluate(&args[2], caller_scope)?;
        let (result, count) = str_replace_scalar_result(&search, &replace, &subject, span)?;

        if args.len() == 4 {
            let Expr::Variable(count_name, _) = &args[3] else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "str_replace()",
                        "count output must be a direct variable in the current subset",
                    ),
                ));
            };
            caller_scope.write_static(count_name, Value::Int(count));
        }

        Ok(Value::String(result))
    }

    fn call_mysqli_stmt_bind_param_direct(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() < 3 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_stmt_bind_param()",
                    ArityExpectation::AtLeast(3),
                    args.len(),
                ),
            ));
        }

        let statement = self.evaluate(&args[0], caller_scope)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_bind_param()", &statement, span)?;
        let types_value = self.evaluate(&args[1], caller_scope)?;
        let types =
            string_builtin_argument("mysqli_stmt_bind_param()", "types", &types_value, span)?;
        validate_mysqli_stmt_bind_param_types(&types, args.len() - 2, span)?;

        let mut variable_names = Vec::with_capacity(args.len() - 2);
        let mut variable_values = Vec::with_capacity(args.len() - 2);
        for arg in &args[2..] {
            let Expr::Variable(name, _) = arg else {
                return Err(runtime_error(
                    arg.span(),
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_bind_param()",
                        "parameter bindings must be direct variables in the current subset",
                    ),
                ));
            };
            let value = caller_scope.read_static(name, arg.span())?;
            validate_mysqli_stmt_bound_parameter_value(&value, arg.span())?;
            variable_names.push(name.clone());
            variable_values.push(value);
        }

        let state = self.mysqli_statement_state_mut("mysqli_stmt_bind_param()", stmt_id, span)?;
        if state.param_count != variable_values.len() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_bind_param()",
                    format!(
                        "bound parameter count must match current placeholder count {}, got {}",
                        state.param_count,
                        variable_values.len()
                    ),
                ),
            ));
        }

        state.bound_parameter_types = Some(types);
        state.bound_parameter_variables = variable_names;
        state.bound_parameter_values = variable_values;
        state.executed_result = None;
        state.affected_rows = 0;
        state.buffered_result = None;
        state.buffered_result_cursor = 0;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_execute_direct(
        &mut self,
        function: &'static str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    function,
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        let statement = self.evaluate(&args[0], caller_scope)?;
        let stmt_id = expect_mysqli_stmt_handle(function, &statement, span)?;
        if let Some(params_arg) = args.get(1) {
            let params = self.evaluate(params_arg, caller_scope)?;
            if matches!(params, Value::Null) {
                self.refresh_mysqli_stmt_bound_parameter_variables(
                    function,
                    stmt_id,
                    span,
                    caller_scope,
                )?;
            } else {
                let params = mysqli_execute_params_from_value(function, &params, span)?;
                self.mysqli_statement_state_mut(function, stmt_id, span)?
                    .bound_parameter_values = params;
            }
        } else {
            self.refresh_mysqli_stmt_bound_parameter_variables(
                function,
                stmt_id,
                span,
                caller_scope,
            )?;
        }

        self.execute_mysqli_stmt_placeholder(function, stmt_id, span)
    }

    fn call_mysqli_stmt_execute_with_refresh(
        &mut self,
        function: &'static str,
        args: &[Value],
        span: Span,
        caller_scope: &SymbolTable,
    ) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    function,
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        let stmt_id = expect_mysqli_stmt_handle(function, &args[0], span)?;
        if args.len() == 2 {
            if matches!(args[1], Value::Null) {
                self.refresh_mysqli_stmt_bound_parameter_variables(
                    function,
                    stmt_id,
                    span,
                    caller_scope,
                )?;
            } else {
                let params = mysqli_execute_params_from_value(function, &args[1], span)?;
                self.mysqli_statement_state_mut(function, stmt_id, span)?
                    .bound_parameter_values = params;
            }
        } else {
            self.refresh_mysqli_stmt_bound_parameter_variables(
                function,
                stmt_id,
                span,
                caller_scope,
            )?;
        }

        self.execute_mysqli_stmt_placeholder(function, stmt_id, span)
    }

    fn refresh_mysqli_stmt_bound_parameter_variables(
        &mut self,
        function: &'static str,
        stmt_id: i64,
        span: Span,
        caller_scope: &SymbolTable,
    ) -> CompileResult<()> {
        let variable_names = self
            .mysqli_statement_state(function, stmt_id, span)?
            .bound_parameter_variables
            .clone();
        if !variable_names.is_empty() {
            let mut variable_values = Vec::with_capacity(variable_names.len());
            for name in &variable_names {
                let value = caller_scope.read_static(name, span)?;
                validate_mysqli_stmt_bound_parameter_value(&value, span)?;
                variable_values.push(value);
            }
            self.mysqli_statement_state_mut(function, stmt_id, span)?
                .bound_parameter_values = variable_values;
        }

        Ok(())
    }

    fn call_mysqli_stmt_bind_result_direct(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() < 2 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_stmt_bind_result()",
                    ArityExpectation::AtLeast(2),
                    args.len(),
                ),
            ));
        }

        let statement = self.evaluate(&args[0], caller_scope)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_bind_result()", &statement, span)?;
        let mut variable_names = Vec::with_capacity(args.len() - 1);
        for arg in &args[1..] {
            let Expr::Variable(name, _) = arg else {
                return Err(runtime_error(
                    arg.span(),
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_bind_result()",
                        "result bindings must be direct variables in the current subset",
                    ),
                ));
            };
            variable_names.push(name.clone());
        }

        let field_count = {
            let state = self.mysqli_statement_state("mysqli_stmt_bind_result()", stmt_id, span)?;
            let Some(query) = state.query.as_deref() else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_bind_result()",
                        "statement result metadata is not available in the current subset",
                    ),
                ));
            };
            let Some(result) =
                mysqli_statement_result_for_query("mysqli_stmt_bind_result()", query, span)?
            else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_bind_result()",
                        "statements without result fields cannot bind result variables in the current subset",
                    ),
                ));
            };
            result.fields.len()
        };

        if variable_names.len() != field_count {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_bind_result()",
                    format!(
                        "bound result variable count must match current placeholder field count {field_count}, got {}",
                        variable_names.len()
                    ),
                ),
            ));
        }

        let state = self.mysqli_statement_state_mut("mysqli_stmt_bind_result()", stmt_id, span)?;
        state.bound_result_variables = variable_names;
        Ok(Value::Bool(true))
    }

    fn call_mysqli_stmt_fetch_direct(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() != 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "mysqli_stmt_fetch()",
                    ArityExpectation::Exactly(1),
                    args.len(),
                ),
            ));
        }
        let statement = self.evaluate(&args[0], caller_scope)?;
        let stmt_id = expect_mysqli_stmt_handle("mysqli_stmt_fetch()", &statement, span)?;
        let (bindings, row) = {
            let state = self.mysqli_statement_state_mut("mysqli_stmt_fetch()", stmt_id, span)?;
            if state.bound_result_variables.is_empty() {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_fetch()",
                        "bound result variables are not available in the current subset",
                    ),
                ));
            }
            let Some(result) = state.buffered_result.as_ref() else {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "mysqli_stmt_fetch()",
                        "buffered statement result state is not available in the current subset",
                    ),
                ));
            };
            if state.buffered_result_cursor >= result.rows.len() {
                return Ok(Value::Null);
            }
            let bindings = state.bound_result_variables.clone();
            let row = result.rows[state.buffered_result_cursor].clone();
            state.buffered_result_cursor += 1;
            (bindings, row)
        };

        for (name, (_, value)) in bindings.iter().zip(row.into_iter()) {
            caller_scope.write_static(name, value);
        }
        Ok(Value::Bool(true))
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

    fn call_ksort(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "ksort()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        let sort_flags = if let Some(flags) = args.get(1) {
            self.evaluate(flags, caller_scope)?
        } else {
            Value::Int(0)
        };
        if sort_flags != Value::Int(1) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ksort()",
                    "only SORT_NUMERIC is implemented in the current subset",
                ),
            ));
        }

        match &args[0] {
            Expr::Variable(name, _) => {
                let mut value = caller_scope.read_static(name, span)?;
                let Value::Array(array) = &mut value else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "ksort()",
                            format!("first argument must be array, got {}", value.type_name()),
                        ),
                    ));
                };
                array
                    .sort_keys_numeric()
                    .map_err(|error| runtime_error(span, error))?;
                caller_scope.write_static(name, value);
                Ok(Value::Bool(true))
            }
            Expr::Property {
                target,
                property,
                span: property_span,
            } => {
                let Expr::Variable(object_name, _) = target.as_ref() else {
                    return Err(runtime_error(
                        target.span(),
                        RuntimeError::unsupported_call(
                            "ksort()",
                            "only direct variable and direct object-property array arguments are implemented",
                        ),
                    ));
                };
                let object = match caller_scope.read_static(object_name, *property_span)? {
                    Value::Object(object) => object,
                    other => {
                        return Err(runtime_error(
                            *property_span,
                            RuntimeError::invalid_property_access(format!(
                                "cannot read property ${property} from {}",
                                other.type_name()
                            )),
                        ));
                    }
                };
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                let mut value = object
                    .read_property_from_context(property, current_class_id, &protected_class_ids)
                    .map_err(|error| runtime_error(*property_span, error))?;
                let Value::Array(array) = &mut value else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "ksort()",
                            format!("first argument must be array, got {}", value.type_name()),
                        ),
                    ));
                };
                array
                    .sort_keys_numeric()
                    .map_err(|error| runtime_error(span, error))?;
                object
                    .write_property_from_context(
                        property,
                        value,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(*property_span, error))?;
                Ok(Value::Bool(true))
            }
            other => Err(runtime_error(
                other.span(),
                RuntimeError::unsupported_call(
                    "ksort()",
                    "only direct variable and direct object-property array arguments are implemented",
                ),
            )),
        }
    }

    fn call_array_unshift(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "array_unshift()",
                    ArityExpectation::AtLeast(1),
                    args.len(),
                ),
            ));
        }

        let Expr::Variable(array_name, _) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_unshift()",
                    "first argument must be a direct variable array in the current subset",
                ),
            ));
        };

        let mut values = Vec::with_capacity(args.len().saturating_sub(1));
        for arg in &args[1..] {
            values.push(self.evaluate(arg, caller_scope)?);
        }

        let mut array_value = caller_scope.read_static(array_name, span)?;
        let Value::Array(array) = &mut array_value else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_unshift()",
                    format!(
                        "first argument must be array, got {}",
                        array_value.type_name()
                    ),
                ),
            ));
        };

        let len = array
            .unshift_values(&values)
            .map_err(|error| runtime_error(span, error))?;
        caller_scope.write_static(array_name, array_value);
        Ok(Value::Int(len))
    }

    fn call_array_pop(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() != 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "array_pop()",
                    ArityExpectation::Exactly(1),
                    args.len(),
                ),
            ));
        }

        let Expr::Variable(array_name, _) = &args[0] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_pop()",
                    "argument must be a direct variable array in the current subset",
                ),
            ));
        };

        let mut array_value = caller_scope.read_static(array_name, span)?;
        let Value::Array(array) = &mut array_value else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_pop()",
                    format!("argument must be array, got {}", array_value.type_name()),
                ),
            ));
        };

        let value = array.pop_value();
        caller_scope.write_static(array_name, array_value);
        Ok(value)
    }

    fn call_next(
        &mut self,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        if args.len() != 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch("next()", ArityExpectation::Exactly(1), args.len()),
            ));
        }

        match &args[0] {
            Expr::Variable(array_name, _) => {
                let mut array_value = caller_scope.read_static(array_name, span)?;
                let Value::Array(array) = &mut array_value else {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "next()",
                            format!("argument must be array, got {}", array_value.type_name()),
                        ),
                    ));
                };
                let value = array.next_value();
                caller_scope.write_static(array_name, array_value);
                Ok(value)
            }
            Expr::Index {
                target,
                index,
                span: index_span,
            } => self.call_next_object_property_array_index(
                target,
                index,
                *index_span,
                span,
                caller_scope,
            ),
            other => Err(runtime_error(
                other.span(),
                RuntimeError::unsupported_call(
                    "next()",
                    "argument must be a direct variable array or direct object-property array offset in the current subset",
                ),
            )),
        }
    }

    fn call_next_object_property_array_index(
        &mut self,
        target: &Expr,
        index: &Expr,
        index_span: Span,
        call_span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let Expr::Property {
            target: object,
            property,
            span: property_span,
        } = target
        else {
            return Err(runtime_error(
                target.span(),
                RuntimeError::unsupported_call(
                    "next()",
                    "only direct object-property array offsets are implemented in the current subset",
                ),
            ));
        };
        let Expr::Variable(object_name, _) = object.as_ref() else {
            return Err(runtime_error(
                object.span(),
                RuntimeError::unsupported_call(
                    "next()",
                    "only direct object-property array offsets are implemented in the current subset",
                ),
            ));
        };

        let key = self.evaluate_array_key(index, caller_scope)?;
        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        let object = match caller_scope.read_static(object_name, *property_span)? {
            Value::Object(object) => object,
            other => {
                return Err(runtime_error(
                    *property_span,
                    RuntimeError::invalid_property_access(format!(
                        "cannot read property ${property} from {}",
                        other.type_name()
                    )),
                ));
            }
        };

        let mut property_value = object
            .read_property_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(*property_span, error))?;
        let Value::Array(property_array) = &mut property_value else {
            return Err(runtime_error(
                call_span,
                RuntimeError::unsupported_call(
                    "next()",
                    format!("argument must be array, got {}", property_value.type_name()),
                ),
            ));
        };
        let Some(slot) = property_array.get(key.clone()).cloned() else {
            return Err(runtime_error(
                index_span,
                RuntimeError::undefined_array_key(key.diagnostic_key()),
            ));
        };
        let mut slot = slot;
        let Value::Array(array) = &mut slot else {
            return Err(runtime_error(
                call_span,
                RuntimeError::unsupported_call(
                    "next()",
                    format!("argument must be array, got {}", slot.type_name()),
                ),
            ));
        };
        let value = array.next_value();
        property_array.insert(key, slot);
        object
            .write_property_from_context(
                property,
                property_value,
                current_class_id,
                &protected_class_ids,
            )
            .map_err(|error| runtime_error(*property_span, error))?;
        Ok(value)
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
                    self.append_output(&value);
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
        ensure_supported_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_user_function_with_checked_values(
            function,
            values,
            None,
            None,
            None,
            reference_bindings,
            Some(caller_scope),
        )
    }

    fn call_reference_return_function(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
        let callable = self.lookup_direct_function_call(name).ok_or_else(|| {
            runtime_error(span, RuntimeError::undefined_function(callable_name(name)))
        })?;
        let Callable::User(function) = callable else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(name),
                    "builtin functions cannot be used as reference-return sources in the current subset",
                ),
            ));
        };

        let function = function.as_ref();
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            None,
            None,
            reference_bindings,
        )
    }

    fn call_reference_return_instance_method(
        &mut self,
        target: &Expr,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
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
            let receiver_class_name = self
                .classes
                .get(object.class_id())
                .expect("object class id should resolve to class metadata")
                .name()
                .to_string();
            let Some(method) = self.resolve_instance_method(object.class_id(), method_name) else {
                return Err(runtime_error(
                    span,
                    RuntimeError::undefined_function(format!(
                        "{receiver_class_name}::{method_name}()"
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
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        let called_class_id = object.class_id();
        self.call_reference_return_function_with_checked_values(
            function,
            values,
            Some(object),
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
        )
    }

    fn call_reference_return_named_static_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
        let class_id = self
            .classes
            .lookup_class_id(class_name)
            .ok_or_else(|| runtime_error(span, RuntimeError::undefined_class(class_name)))?;
        let receiver_class = self
            .classes
            .get(class_id)
            .expect("class id should resolve to class metadata");
        let receiver_class_name = receiver_class.name().to_string();
        let Some((
            declaring_class_id,
            declaring_class_name,
            resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(class_id, method_name)
        else {
            self.reject_magic_static_reference_return_source(class_id, &receiver_class_name, span)?;
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!("{receiver_class_name}::{method_name}()")),
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
                    "non-static method dispatch through named static receivers is not implemented",
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
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            Some(declaring_class_id),
            Some(class_id),
            reference_bindings,
        )
    }

    fn call_reference_return_self_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
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
        let current_class_name = current_class.name().to_string();
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(current_class_id, method_name)
        else {
            self.reject_magic_static_reference_return_source(
                current_class_id,
                &current_class_name,
                span,
            )?;
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!("{current_class_name}::{method_name}()")),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "non-static self:: reference-return method sources are not implemented",
                ),
            ));
        }

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        let called_class_id = self
            .called_class_context
            .last()
            .copied()
            .unwrap_or(current_class_id);
        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
        )
    }

    fn call_reference_return_parent_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
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
        let parent_class_name = parent_class.name().to_string();
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(parent_class_id, method_name)
        else {
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!("{parent_class_name}::{method_name}()")),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "non-static parent:: reference-return method sources are not implemented",
                ),
            ));
        }

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        let called_class_id = self
            .called_class_context
            .last()
            .copied()
            .unwrap_or(current_class_id);
        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
        )
    }

    fn call_reference_return_late_static_method(
        &mut self,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
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
        let called_class_name = called_class.name().to_string();
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(called_class_id, method_name)
        else {
            self.reject_magic_static_reference_return_source(
                called_class_id,
                &called_class_name,
                span,
            )?;
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!("{called_class_name}::{method_name}()")),
            ));
        };

        self.ensure_instance_method_visible(class_id, &class_name, method_name, visibility, span)?;

        if !is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{class_name}::{method_name}()"),
                    "non-static static:: reference-return method sources are not implemented",
                ),
            ));
        }

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            Some(class_id),
            Some(called_class_id),
            reference_bindings,
        )
    }

    fn call_reference_return_dynamic_static_method(
        &mut self,
        target: &Expr,
        method_name: &str,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
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
        let receiver_class_name = receiver_class.name().to_string();
        let Some((
            declaring_class_id,
            declaring_class_name,
            resolved_method_name,
            visibility,
            is_static,
        )) = self.resolve_instance_method(receiver_class_id, method_name)
        else {
            self.reject_magic_static_reference_return_source(
                receiver_class_id,
                &receiver_class_name,
                span,
            )?;
            return Err(runtime_error(
                span,
                RuntimeError::undefined_function(format!("{receiver_class_name}::{method_name}()")),
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
                    "non-static dynamic static receiver reference-return method sources are not implemented",
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
        if !function.returns_by_reference {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "function does not return by reference",
                ),
            ));
        }
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_reference_return_function_metadata(function, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let (values, reference_bindings) =
            self.evaluate_user_function_call_arguments(function, args, span, caller_scope)?;

        self.call_reference_return_function_with_checked_values(
            function,
            values,
            None,
            Some(declaring_class_id),
            Some(receiver_class_id),
            reference_bindings,
        )
    }

    fn reject_magic_static_reference_return_source(
        &self,
        receiver_class_id: ClassId,
        receiver_class_name: &str,
        span: Span,
    ) -> CompileResult<()> {
        if self
            .resolve_instance_method(receiver_class_id, "__callStatic")
            .is_some()
        {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    format!("{receiver_class_name}::__callStatic()"),
                    "magic __callStatic reference-return method sources are not implemented",
                ),
            ));
        }

        Ok(())
    }

    fn call_reference_return_function_with_checked_values(
        &mut self,
        function: &FunctionDecl,
        args: Vec<Value>,
        this_object: Option<PhpObject>,
        class_context: Option<ClassId>,
        called_class_context: Option<ClassId>,
        reference_bindings: Vec<ReferenceBinding>,
    ) -> CompileResult<VariableCell> {
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

            if let Some(binding) = reference_bindings
                .iter()
                .find(|binding| binding.param_name == param.name)
            {
                if let ReferenceBindingTarget::CallerCell(caller_cell) = &binding.target {
                    local_scope.bind_static_to_cell(&param.name, caller_cell.clone());
                }
                continue;
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
        let result = self.execute_reference_return_statements(function, &mut local_scope);
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

        result
    }

    fn execute_reference_return_statements(
        &mut self,
        function: &FunctionDecl,
        scope: &mut SymbolTable,
    ) -> CompileResult<VariableCell> {
        for stmt in &function.body {
            self.tick(stmt.span())?;
            if let Stmt::Return { value, span } = stmt {
                let Some(Expr::Variable(name, variable_span)) = value else {
                    return Err(runtime_error(
                        *span,
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "reference returns are only implemented for direct variable return expressions",
                        ),
                    ));
                };
                return scope.read_cell(name).ok_or_else(|| {
                    runtime_error(*variable_span, RuntimeError::undefined_variable(name))
                });
            }

            match self.execute_statement(stmt, scope)? {
                Flow::Normal => {}
                Flow::Return(_) => {
                    return Err(runtime_error(
                        stmt.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "reference returns through nested control flow are not implemented",
                        ),
                    ));
                }
                Flow::Break { span, .. } => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control("break cannot be used outside a loop"),
                    ));
                }
                Flow::Continue { span, .. } => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::invalid_loop_control(
                            "continue cannot be used outside a loop",
                        ),
                    ));
                }
                Flow::Exit(_) => {
                    return Err(runtime_error(
                        stmt.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "exit during reference-return evaluation is not implemented",
                        ),
                    ));
                }
                Flow::Goto { label, span } => return Err(undefined_goto_label_error(span, &label)),
            }
        }

        Err(runtime_error(
            function.span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "reference-return functions must return a direct variable in the current subset",
            ),
        ))
    }

    fn call_user_function_with_values(
        &mut self,
        function: Rc<FunctionDecl>,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        let function = function.as_ref();
        ensure_user_function_arity(function, args.len(), span)?;
        ensure_supported_function_signature(function, args.len(), span)?;
        self.ensure_user_function_call_depth(function, span)?;
        self.call_user_function_with_checked_values(
            function,
            args,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
    }

    fn evaluate_user_function_call_arguments(
        &mut self,
        function: &FunctionDecl,
        args: &[Expr],
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<(Vec<Value>, Vec<ReferenceBinding>)> {
        let mut values = Vec::with_capacity(args.len());
        let mut reference_bindings = Vec::new();

        for (index, arg) in args.iter().enumerate() {
            let Some(param) = function.params.get(index) else {
                values.push(self.evaluate(arg, caller_scope)?);
                continue;
            };

            if param.by_reference {
                if let Expr::Variable(caller_name, _) = arg {
                    let caller_cell = caller_scope.read_cell(caller_name).ok_or_else(|| {
                        runtime_error(arg.span(), RuntimeError::undefined_variable(caller_name))
                    })?;
                    values.push(caller_cell.borrow().clone());
                    reference_bindings.push(ReferenceBinding {
                        param_name: param.name.clone(),
                        target: ReferenceBindingTarget::CallerCell(caller_cell),
                    });
                } else if let Some((alias, value)) = self
                    .evaluate_public_object_property_array_reference_argument(arg, caller_scope)?
                {
                    if function.returns_by_reference {
                        return Err(runtime_error(
                            arg.span(),
                            RuntimeError::unsupported_call(
                                callable_name(&function.name),
                                "object-property array reference arguments are not implemented for reference-returning functions in the current subset",
                            ),
                        ));
                    }
                    values.push(value);
                    reference_bindings.push(ReferenceBinding {
                        param_name: param.name.clone(),
                        target: ReferenceBindingTarget::PublicObjectPropertyArrayOffset(alias),
                    });
                } else {
                    return Err(runtime_error(
                        arg.span(),
                        RuntimeError::unsupported_call(
                            callable_name(&function.name),
                            "reference parameter invocation is only implemented for direct variable and direct public object-property array-offset arguments in the current subset",
                        ),
                    ));
                }
            } else {
                values.push(self.evaluate(arg, caller_scope)?);
            }
        }

        if function
            .params
            .iter()
            .enumerate()
            .any(|(index, param)| param.by_reference && param.is_variadic && index < args.len())
        {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    callable_name(&function.name),
                    "variadic reference parameter invocation is not implemented",
                ),
            ));
        }

        Ok((values, reference_bindings))
    }

    fn evaluate_public_object_property_array_reference_argument(
        &mut self,
        arg: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Option<(ArrayOffsetAlias, Value)>> {
        let Some((object, property, indices)) =
            Self::direct_object_property_array_argument_parts(arg)
        else {
            return Ok(None);
        };
        let keys = indices
            .iter()
            .map(|index| self.evaluate_array_key(index, caller_scope))
            .collect::<CompileResult<Vec<_>>>()?;
        let alias = ArrayOffsetAlias {
            root: ArrayOffsetAliasRoot::PublicObjectProperty { object, property },
            keys,
        };
        caller_scope.materialize_array_offset_alias(&alias, arg.span())?;
        let value = caller_scope
            .read_array_offset_alias(&alias)
            .ok_or_else(|| {
                runtime_error(
                    arg.span(),
                    RuntimeError::invalid_array_access(
                        "cannot bind missing object-property array offset".to_string(),
                    ),
                )
            })?;
        Ok(Some((alias, value)))
    }

    fn direct_object_property_array_argument_parts(
        expr: &Expr,
    ) -> Option<(String, String, Vec<&Expr>)> {
        let mut indices = Vec::new();
        let mut current = expr;
        while let Expr::Index { target, index, .. } = current {
            indices.push(index.as_ref());
            current = target.as_ref();
        }
        if indices.is_empty() {
            return None;
        }
        indices.reverse();

        let Expr::Property {
            target, property, ..
        } = current
        else {
            return None;
        };
        let Expr::Variable(object, _) = target.as_ref() else {
            return None;
        };
        Some((object.clone(), property.clone(), indices))
    }

    fn write_back_reference_bindings(
        &mut self,
        reference_bindings: &[ReferenceBinding],
        local_scope: &SymbolTable,
        caller_scope: &mut SymbolTable,
        span: Span,
    ) -> CompileResult<()> {
        for binding in reference_bindings {
            let ReferenceBindingTarget::PublicObjectPropertyArrayOffset(alias) = &binding.target
            else {
                continue;
            };
            let Some(value) = local_scope.read_named(&binding.param_name) else {
                continue;
            };
            if !caller_scope.write_array_offset_alias(alias, value) {
                return Err(runtime_error(
                    span,
                    RuntimeError::invalid_array_access(
                        "cannot write object-property array reference argument".to_string(),
                    ),
                ));
            }
            if let ArrayOffsetAliasRoot::PublicObjectProperty { object, property } = &alias.root {
                caller_scope.sync_array_offset_aliases_for_object_property_root(object, property);
            }
        }
        Ok(())
    }

    fn call_user_function_with_checked_values(
        &mut self,
        function: &FunctionDecl,
        args: Vec<Value>,
        this_object: Option<PhpObject>,
        class_context: Option<ClassId>,
        called_class_context: Option<ClassId>,
        reference_bindings: Vec<ReferenceBinding>,
        reference_scope: Option<&mut SymbolTable>,
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

            if let Some(binding) = reference_bindings
                .iter()
                .find(|binding| binding.param_name == param.name)
            {
                match &binding.target {
                    ReferenceBindingTarget::CallerCell(caller_cell) => {
                        local_scope.bind_static_to_cell(&param.name, caller_cell.clone());
                    }
                    ReferenceBindingTarget::PublicObjectPropertyArrayOffset(_) => {
                        if let Some(arg) = args.get(index) {
                            local_scope.write_static(&param.name, arg.clone());
                        }
                    }
                }
                continue;
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
        let writeback_result = if matches!(
            &flow,
            Ok(Flow::Normal) | Ok(Flow::Return(_)) | Ok(Flow::Exit(_))
        ) {
            if let Some(reference_scope) = reference_scope {
                self.write_back_reference_bindings(
                    &reference_bindings,
                    &local_scope,
                    reference_scope,
                    function.span,
                )
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
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

        writeback_result?;
        let flow = flow?;
        match flow {
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
            Vec::new(),
            None,
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

        if self.class_implements_or_matches_core_interface(candidate_id, class_name) {
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
        if self.class_implements_or_matches_core_interface(object.class_id(), class_name) {
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

        if self.class_implements_or_matches_core_interface(candidate_id, class_name) {
            return true;
        }

        let Some(target_class) = self.classes.lookup_class(class_name) else {
            return false;
        };

        self.classes.is_subclass_of(candidate_id, target_class.id())
    }

    fn class_implements_or_matches_core_interface(
        &self,
        class_id: ClassId,
        interface_name: &str,
    ) -> bool {
        self.classes.implements_interface(class_id, interface_name)
            || (interface_name.eq_ignore_ascii_case("Stringable")
                && self.class_has_public_instance_to_string(class_id))
    }

    fn class_has_public_instance_to_string(&self, class_id: ClassId) -> bool {
        self.resolve_instance_method(class_id, "__toString")
            .is_some_and(|(_, _, _, visibility, is_static)| {
                visibility == Visibility::Public && !is_static
            })
    }

    fn parent_class_name(&self, class_id: ClassId) -> Option<String> {
        let class = self.classes.get(class_id)?;
        let parent_id = class.parent_id()?;
        Some(self.classes.get(parent_id)?.name().to_string())
    }

    fn call_ob_start(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("ob_start", args, 0, span)?;
        self.output_buffers.push(String::new());
        Ok(Value::Bool(true))
    }

    fn call_ob_get_level(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("ob_get_level", args, 0, span)?;
        Ok(Value::Int(self.output_buffers.len() as i64))
    }

    fn call_ob_get_clean(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("ob_get_clean", args, 0, span)?;
        Ok(self
            .output_buffers
            .pop()
            .map(Value::String)
            .unwrap_or(Value::Bool(false)))
    }

    fn call_header(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
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

        let header = match &args[0] {
            Value::String(header) => header.clone(),
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
        };

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

        self.response_headers.push(header);
        Ok(Value::Null)
    }

    fn call_headers_list(&self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("headers_list", args, 0, span)?;
        let mut headers = PhpArray::new();
        for header in &self.response_headers {
            headers
                .append(Value::String(header.clone()))
                .map_err(|error| runtime_error(span, error))?;
        }
        Ok(Value::Array(headers))
    }

    fn call_header_remove(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
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

        let Some(name) = args.first() else {
            self.response_headers.clear();
            return Ok(Value::Null);
        };

        let Value::String(name) = name else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "header_remove()",
                    format!(
                        "header name argument must be string in the current subset, got {}",
                        name.type_name()
                    ),
                ),
            ));
        };

        self.response_headers
            .retain(|header| header_name(header) != Some(name.as_str()));
        Ok(Value::Null)
    }

    fn call_setcookie(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        if !(1..=2).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "setcookie()",
                    ArityExpectation::Between { min: 1, max: 2 },
                    args.len(),
                ),
            ));
        }

        let name = match &args[0] {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "setcookie()",
                        format!(
                            "name argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };

        let value = match args.get(1) {
            Some(Value::String(value)) => value.as_str(),
            Some(other) => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "setcookie()",
                        format!(
                            "value argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
            None => "",
        };

        self.response_headers
            .push(format!("Set-Cookie: {name}={value}"));
        Ok(Value::Bool(true))
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
            "ltrim" => call_ltrim(&args, span),
            "rtrim" => call_rtrim(&args, span),
            "strcasecmp" => call_strcasecmp(&args, span),
            "str_contains" => call_str_contains(&args, span),
            "str_starts_with" => call_str_starts_with(&args, span),
            "str_ends_with" => call_str_ends_with(&args, span),
            "strpos" => call_strpos(&args, span),
            "substr" => call_substr(&args, span),
            "substr_count" => call_substr_count(&args, span),
            "str_replace" => call_str_replace(&args, span),
            "preg_match" => call_preg_match(&args, span),
            "preg_replace" => call_preg_replace(&args, span),
            "preg_split" => call_preg_split(&args, span),
            "preg_replace_callback" => self.call_preg_replace_callback(args, span),
            "compact" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "compact()",
                    "caller-scope variable lookup is only implemented for direct and dynamic compact() calls in the current subset",
                ),
            )),
            "error_reporting" => self.call_error_reporting(args, span),
            "ignore_user_abort" => self.call_ignore_user_abort(args, span),
            "php_sapi_name" => call_php_sapi_name(&args, span),
            "sprintf" => call_sprintf(&args, span),
            "vsprintf" => call_vsprintf(&args, span),
            "call_user_func" => self.call_user_func_builtin(args, span),
            "call_user_func_array" => self.call_user_func_array_builtin(args, span),
            "implode" => call_implode(&args, span),
            "basename" => {
                if !(1..=2).contains(&args.len()) {
                    return Err(runtime_error(
                        span,
                        RuntimeError::arity_mismatch(
                            "basename()",
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
                                "basename()",
                                format!(
                                    "path argument must be string in the current subset, got {}",
                                    other.type_name()
                                ),
                            ),
                        ));
                    }
                };

                let suffix = match args.get(1) {
                    Some(Value::String(suffix)) => Some(suffix.as_str()),
                    Some(other) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "basename()",
                                format!(
                                    "suffix argument must be string in the current subset, got {}",
                                    other.type_name()
                                ),
                            ),
                        ));
                    }
                    None => None,
                };

                Ok(Value::String(basename_path(path, suffix)))
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
            "abs" => call_abs(&args, span),
            "version_compare" => call_version_compare(&args, span),
            "microtime" => call_microtime(&args, span),
            "date_default_timezone_set" => call_date_default_timezone_set(&args, span),
            "ini_get" => self.call_ini_get(&args, span),
            "ini_set" => self.call_ini_set(&args, span),
            "min" => call_min(&args, span),
            "rand" => call_rand(&args, span),
            "uniqid" => call_uniqid(&args, span),
            "hash_hmac" => call_hash_hmac(&args, span),
            "count" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(value) => Ok(Value::Int(value.len() as i64)),
                    Value::Object(object)
                        if self
                            .classes
                            .implements_interface(object.class_id(), "Countable") =>
                    {
                        let value = self.call_countable_count_method(object.clone(), span)?;
                        match value {
                            Value::Int(count) => Ok(Value::Int(count)),
                            other => Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "Countable::count()",
                                    format!(
                                        "count method must return int in the current subset, got {}",
                                        other.type_name()
                                    ),
                                ),
                            )),
                        }
                    }
                    _ => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "count()",
                            "only arrays and Countable objects are supported",
                        ),
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
            "current" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::Array(array) => Ok(array.current_value()),
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "current()",
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
            "ksort" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ksort()",
                    "by-reference array arguments require a direct call target in the current subset",
                ),
            )),
            "array_unshift" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_unshift()",
                    "by-reference array arguments require a direct call target in the current subset",
                ),
            )),
            "array_pop" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "array_pop()",
                    "by-reference array arguments require a direct call target in the current subset",
                ),
            )),
            "next" => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "next()",
                    "array pointer mutation requires a direct call target in the current subset",
                ),
            )),
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
                Ok(Value::Bool(self.is_countable_value(&args[0])))
            }
            "is_iterable" => {
                expect_arity(name, &args, 1, span)?;
                Ok(Value::Bool(self.is_iterable_value(&args[0])))
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
            "mysqli_connect" => self.call_mysqli_connect(&args, span),
            "mysqli_real_connect" => self.call_mysqli_real_connect(&args, span),
            "mysqli_get_server_info" => self.call_mysqli_get_server_info(&args, span),
            "mysqli_get_server_version" => self.call_mysqli_get_server_version(&args, span),
            "mysqli_get_host_info" => self.call_mysqli_get_host_info(&args, span),
            "mysqli_get_client_info" => self.call_mysqli_get_client_info(&args, span),
            "mysqli_get_client_version" => self.call_mysqli_get_client_version(&args, span),
            "mysqli_get_proto_info" => self.call_mysqli_get_proto_info(&args, span),
            "mysqli_thread_id" => self.call_mysqli_thread_id(&args, span),
            "mysqli_kill" => self.call_mysqli_kill(&args, span),
            "mysqli_change_user" => self.call_mysqli_change_user(&args, span),
            "mysqli_refresh" => self.call_mysqli_refresh(&args, span),
            "mysqli_get_charset" => self.call_mysqli_get_charset(&args, span),
            "mysqli_character_set_name" => self.call_mysqli_character_set_name(&args, span),
            "mysqli_field_count" => self.call_mysqli_field_count(&args, span),
            "mysqli_close" => self.call_mysqli_close(&args, span),
            "mysqli_options" => self.call_mysqli_options(&args, span),
            "mysqli_set_opt" => self.call_mysqli_set_opt(&args, span),
            "mysqli_ssl_set" => self.call_mysqli_ssl_set(&args, span),
            "mysqli_connect_errno" => self.call_mysqli_connect_errno(&args, span),
            "mysqli_connect_error" => self.call_mysqli_connect_error(&args, span),
            "mysqli_error_list" => self.call_mysqli_error_list(&args, span),
            "mysqli_get_connection_stats" => self.call_mysqli_get_connection_stats(&args, span),
            "mysqli_get_links_stats" => self.call_mysqli_get_links_stats(&args, span),
            "mysqli_get_client_stats" => self.call_mysqli_get_client_stats(&args, span),
            "mysqli_thread_safe" => self.call_mysqli_thread_safe(&args, span),
            "mysqli_stmt_init" => self.call_mysqli_stmt_init(&args, span),
            "mysqli_prepare" => self.call_mysqli_prepare(&args, span),
            "mysqli_stmt_prepare" => self.call_mysqli_stmt_prepare(&args, span),
            "mysqli_stmt_param_count" => self.call_mysqli_stmt_param_count(&args, span),
            "mysqli_stmt_get_warnings" => self.call_mysqli_stmt_get_warnings(&args, span),
            "mysqli_stmt_error_list" => self.call_mysqli_stmt_error_list(&args, span),
            "mysqli_stmt_bind_param" => self.call_mysqli_stmt_bind_param(&args, span),
            "mysqli_stmt_bind_result" => self.call_mysqli_stmt_bind_result(&args, span),
            "mysqli_stmt_execute" => self.call_mysqli_stmt_execute(&args, span),
            "mysqli_execute" => self.call_mysqli_execute(&args, span),
            "mysqli_stmt_get_result" => self.call_mysqli_stmt_get_result(&args, span),
            "mysqli_stmt_close" => self.call_mysqli_stmt_close(&args, span),
            "mysqli_stmt_errno" => self.call_mysqli_stmt_errno(&args, span),
            "mysqli_stmt_error" => self.call_mysqli_stmt_error(&args, span),
            "mysqli_stmt_affected_rows" => self.call_mysqli_stmt_affected_rows(&args, span),
            "mysqli_stmt_store_result" => self.call_mysqli_stmt_store_result(&args, span),
            "mysqli_stmt_num_rows" => self.call_mysqli_stmt_num_rows(&args, span),
            "mysqli_stmt_fetch" => self.call_mysqli_stmt_fetch(&args, span),
            "mysqli_stmt_result_metadata" => self.call_mysqli_stmt_result_metadata(&args, span),
            "mysqli_stmt_field_count" => self.call_mysqli_stmt_field_count(&args, span),
            "mysqli_stmt_free_result" => self.call_mysqli_stmt_free_result(&args, span),
            "mysqli_stmt_data_seek" => self.call_mysqli_stmt_data_seek(&args, span),
            "mysqli_stmt_attr_get" => self.call_mysqli_stmt_attr_get(&args, span),
            "mysqli_stmt_attr_set" => self.call_mysqli_stmt_attr_set(&args, span),
            "mysqli_stmt_send_long_data" => self.call_mysqli_stmt_send_long_data(&args, span),
            "mysqli_stmt_reset" => self.call_mysqli_stmt_reset(&args, span),
            "mysqli_stmt_more_results" => self.call_mysqli_stmt_more_results(&args, span),
            "mysqli_stmt_next_result" => self.call_mysqli_stmt_next_result(&args, span),
            "mysqli_stmt_sqlstate" => self.call_mysqli_stmt_sqlstate(&args, span),
            "mysqli_stmt_warning_count" => self.call_mysqli_stmt_warning_count(&args, span),
            "mysqli_stmt_insert_id" => self.call_mysqli_stmt_insert_id(&args, span),
            "mysqli_execute_query" => self.call_mysqli_execute_query(&args, span),
            "mysqli_dump_debug_info" => self.call_mysqli_dump_debug_info(&args, span),
            "mysqli_debug" => self.call_mysqli_debug(&args, span),
            "mysqli_stat" => self.call_mysqli_stat(&args, span),
            "mysqli_autocommit" => self.call_mysqli_autocommit(&args, span),
            "mysqli_begin_transaction" => self.call_mysqli_begin_transaction(&args, span),
            "mysqli_commit" => self.call_mysqli_commit(&args, span),
            "mysqli_rollback" => self.call_mysqli_rollback(&args, span),
            "mysqli_savepoint" => self.call_mysqli_savepoint(&args, span),
            "mysqli_release_savepoint" => self.call_mysqli_release_savepoint(&args, span),
            "mysqli_set_charset" => self.call_mysqli_set_charset(&args, span),
            "mysqli_query" => self.call_mysqli_query(&args, span),
            "mysqli_real_query" => self.call_mysqli_real_query(&args, span),
            "mysqli_multi_query" => self.call_mysqli_multi_query(&args, span),
            "mysqli_errno" => self.call_mysqli_errno(&args, span),
            "mysqli_error" => self.call_mysqli_error(&args, span),
            "mysqli_sqlstate" => self.call_mysqli_sqlstate(&args, span),
            "mysqli_warning_count" => self.call_mysqli_warning_count(&args, span),
            "mysqli_info" => self.call_mysqli_info(&args, span),
            "mysqli_get_warnings" => self.call_mysqli_get_warnings(&args, span),
            "mysqli_affected_rows" => self.call_mysqli_affected_rows(&args, span),
            "mysqli_insert_id" => self.call_mysqli_insert_id(&args, span),
            "mysqli_ping" => self.call_mysqli_ping(&args, span),
            "mysqli_select_db" => self.call_mysqli_select_db(&args, span),
            "mysqli_real_escape_string" => self.call_mysqli_real_escape_string(&args, span),
            "mysqli_escape_string" => self.call_mysqli_escape_string(&args, span),
            "mysqli_fetch_object" => self.call_mysqli_fetch_object(&args, span),
            "mysqli_fetch_assoc" => self.call_mysqli_fetch_assoc(&args, span),
            "mysqli_fetch_row" => self.call_mysqli_fetch_row(&args, span),
            "mysqli_fetch_array" => self.call_mysqli_fetch_array(&args, span),
            "mysqli_fetch_all" => self.call_mysqli_fetch_all(&args, span),
            "mysqli_fetch_column" => self.call_mysqli_fetch_column(&args, span),
            "mysqli_fetch_field" => self.call_mysqli_fetch_field(&args, span),
            "mysqli_fetch_fields" => self.call_mysqli_fetch_fields(&args, span),
            "mysqli_fetch_field_direct" => self.call_mysqli_fetch_field_direct(&args, span),
            "mysqli_num_fields" => self.call_mysqli_num_fields(&args, span),
            "mysqli_num_rows" => self.call_mysqli_num_rows(&args, span),
            "mysqli_fetch_lengths" => self.call_mysqli_fetch_lengths(&args, span),
            "mysqli_data_seek" => self.call_mysqli_data_seek(&args, span),
            "mysqli_field_seek" => self.call_mysqli_field_seek(&args, span),
            "mysqli_field_tell" => self.call_mysqli_field_tell(&args, span),
            "mysqli_free_result" => self.call_mysqli_free_result(&args, span),
            "mysqli_more_results" => self.call_mysqli_more_results(&args, span),
            "mysqli_next_result" => self.call_mysqli_next_result(&args, span),
            "mysqli_store_result" => self.call_mysqli_store_result(&args, span),
            "mysqli_use_result" => self.call_mysqli_use_result(&args, span),
            "mysqli_reap_async_query" => self.call_mysqli_reap_async_query(&args, span),
            "mysqli_poll" => self.call_mysqli_poll(&args, span),
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
            "file_get_contents" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) if path == "php://input" => Ok(Value::String(String::new())),
                    Value::String(path) if path.contains("://") => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "file_get_contents()",
                            "only php://input is supported in the current stream-wrapper subset",
                        ),
                    )),
                    Value::String(path) => {
                        let filesystem_path = local_filesystem_metadata_path(path);
                        let contents = fs::read_to_string(&filesystem_path).map_err(|error| {
                            runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "file_get_contents()",
                                    format!("local UTF-8 file read failed: {error}"),
                                ),
                            )
                        })?;
                        Ok(Value::String(contents))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "file_get_contents()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "realpath" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "realpath()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }

                        let filesystem_path = local_filesystem_metadata_path(path);
                        let Ok(resolved) = fs::canonicalize(&filesystem_path) else {
                            return Ok(Value::Bool(false));
                        };
                        let resolved = resolved.into_os_string().into_string().map_err(|_| {
                            runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "realpath()",
                                    "resolved path must be valid UTF-8 in the current subset",
                                ),
                            )
                        })?;
                        Ok(Value::String(resolved))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "realpath()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "getcwd" => {
                expect_arity(name, &args, 0, span)?;
                let path = std::env::current_dir().map_err(|error| {
                    runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "getcwd()",
                            format!("current working directory lookup failed: {error}"),
                        ),
                    )
                })?;
                let path = path.into_os_string().into_string().map_err(|_| {
                    runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "getcwd()",
                            "current working directory path must be valid UTF-8 in the current subset",
                        ),
                    )
                })?;
                Ok(Value::String(path))
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
            "is_file" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "is_file()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        Ok(Value::Bool(
                            fs::metadata(&metadata_path)
                                .map(|metadata| metadata.is_file())
                                .unwrap_or(false),
                        ))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_file()",
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
            "is_writable" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "is_writable()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        let Ok(metadata) = fs::metadata(&metadata_path) else {
                            return Ok(Value::Bool(false));
                        };
                        Ok(Value::Bool(!metadata.permissions().readonly()))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_writable()",
                            format!(
                                "path argument must be string in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    )),
                }
            }
            "is_link" => {
                expect_arity(name, &args, 1, span)?;
                match &args[0] {
                    Value::String(path) => {
                        if path.contains("://") {
                            return Err(runtime_error(
                                span,
                                RuntimeError::unsupported_call(
                                    "is_link()",
                                    "stream wrappers are not supported in the current subset",
                                ),
                            ));
                        }
                        let metadata_path = local_filesystem_metadata_path(path);
                        Ok(Value::Bool(
                            fs::symlink_metadata(&metadata_path)
                                .map(|metadata| metadata.file_type().is_symlink())
                                .unwrap_or(false),
                        ))
                    }
                    other => Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "is_link()",
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
            "ob_start" => self.call_ob_start(&args, span),
            "ob_get_level" => self.call_ob_get_level(&args, span),
            "ob_get_clean" => self.call_ob_get_clean(&args, span),
            "header" => self.call_header(&args, span),
            "header_remove" => self.call_header_remove(&args, span),
            "headers_list" => self.call_headers_list(&args, span),
            "headers_sent" => call_headers_sent(&args, span),
            "setcookie" => self.call_setcookie(&args, span),
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
                    is_core_interface_name(interface_name)
                        || self
                            .interface_lookup
                            .contains_key(&interface_name.to_ascii_lowercase()),
                )),
                [Value::String(interface_name), autoload] => {
                    let _autoload =
                        metadata_exists_autoload_flag("interface_exists()", autoload, span)?;
                    Ok(Value::Bool(
                        is_core_interface_name(interface_name)
                            || self
                                .interface_lookup
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
                for interface in CORE_INTERFACE_NAMES {
                    interfaces
                        .append(Value::String((*interface).to_string()))
                        .expect("core interface count fits in array keys");
                }
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
                    self.append_output(&format_var_dump(value));
                }
                Ok(Value::Null)
            }
            "print_r" => match args.as_slice() {
                [value] => {
                    self.append_output(&format_print_r(value));
                    Ok(Value::Bool(true))
                }
                [value, return_output] if return_output.is_truthy() => {
                    Ok(Value::String(format_print_r(value)))
                }
                [value, _] => {
                    self.append_output(&format_print_r(value));
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

    fn call_user_func_array_builtin(
        &mut self,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        expect_arity("call_user_func_array", &args, 2, span)?;

        let Value::Array(argument_array) = &args[1] else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    format!(
                        "argument array must be array in the current subset, got {}",
                        args[1].type_name()
                    ),
                ),
            ));
        };

        let mut positional_args = Vec::with_capacity(argument_array.len());
        for entry in argument_array.entries() {
            if matches!(entry.key, ArrayKey::String(_)) {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "call_user_func_array()",
                        "string-keyed named arguments are not implemented in the current subset",
                    ),
                ));
            }
            positional_args.push(entry.value_cloned());
        }

        match &args[0] {
            Value::String(callback_name) => {
                let callable = self.lookup_function(callback_name).ok_or_else(|| {
                    runtime_error(
                        span,
                        RuntimeError::undefined_function(callable_name(callback_name)),
                    )
                })?;
                self.call_callable_with_values(callable, positional_args, span)
            }
            Value::Array(callback) => {
                self.call_array_callable_with_values(callback, positional_args, span)
            }
            Value::Closure(_) => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    "closure invocation is not implemented",
                ),
            )),
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    format!(
                        "callback must evaluate to string or array callable in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            )),
        }
    }

    fn call_user_func_array_array_callable(
        &mut self,
        callback: &PhpArray,
        argument_expr: &Expr,
        span: Span,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<Value> {
        let Some((target, method_name)) = array_callable_parts(callback) else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    "array callback must be [object-or-class, method] in the current subset",
                ),
            ));
        };

        match target {
            Value::Object(object) => {
                let receiver_class = self
                    .classes
                    .get(object.class_id())
                    .expect("object class id should resolve to class metadata");
                let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
                    self.resolve_instance_method(object.class_id(), method_name)
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
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "static method dispatch through object array callables is not implemented",
                        ),
                    ));
                }
                if visibility != Visibility::Public {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "array callable method dispatch is only implemented for public methods",
                        ),
                    ));
                }

                let function =
                    self.method_function(class_id, &class_name, &resolved_method_name, span)?;
                let function = function.as_ref();
                let (values, reference_bindings) = self
                    .evaluate_call_user_func_array_checked_arguments(
                        function,
                        argument_expr,
                        span,
                        caller_scope,
                    )?;
                self.ensure_user_function_call_depth(function, span)?;
                self.call_user_function_with_checked_values(
                    function,
                    values,
                    Some(object.clone()),
                    Some(class_id),
                    Some(object.class_id()),
                    reference_bindings,
                    Some(caller_scope),
                )
            }
            Value::String(_) => {
                let positional_args = self.evaluate_call_user_func_array_arguments(
                    argument_expr,
                    span,
                    caller_scope,
                )?;
                self.call_array_callable_with_values(callback, positional_args, span)
            }
            _ => unreachable!("array_callable_parts restricts callback targets"),
        }
    }

    fn call_array_callable_with_values(
        &mut self,
        callback: &PhpArray,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        let Some((target, method_name)) = array_callable_parts(callback) else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "call_user_func_array()",
                    "array callback must be [object-or-class, method] in the current subset",
                ),
            ));
        };

        match target {
            Value::Object(object) => {
                let receiver_class = self
                    .classes
                    .get(object.class_id())
                    .expect("object class id should resolve to class metadata");
                let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
                    self.resolve_instance_method(object.class_id(), method_name)
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
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "static method dispatch through object array callables is not implemented",
                        ),
                    ));
                }
                if visibility != Visibility::Public {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{class_name}::{method_name}()"),
                            "array callable method dispatch is only implemented for public methods",
                        ),
                    ));
                }

                let function =
                    self.method_function(class_id, &class_name, &resolved_method_name, span)?;
                let function = function.as_ref();
                ensure_user_function_arity(function, args.len(), span)?;
                ensure_supported_function_signature(function, args.len(), span)?;
                self.ensure_user_function_call_depth(function, span)?;
                self.call_user_function_with_checked_values(
                    function,
                    args,
                    Some(object.clone()),
                    Some(class_id),
                    Some(object.class_id()),
                    Vec::new(),
                    None,
                )
            }
            Value::String(class_name) => {
                let class_id = self.classes.lookup_class_id(class_name).ok_or_else(|| {
                    runtime_error(span, RuntimeError::undefined_class(class_name))
                })?;
                let receiver_class = self
                    .classes
                    .get(class_id)
                    .expect("class id should resolve to class metadata");
                let Some((
                    declaring_class_id,
                    declaring_class_name,
                    resolved_method_name,
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
                if !is_static {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{declaring_class_name}::{method_name}()"),
                            "non-static method array callables require an object receiver in the current subset",
                        ),
                    ));
                }
                if visibility != Visibility::Public {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            format!("{declaring_class_name}::{method_name}()"),
                            "array callable method dispatch is only implemented for public methods",
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
                ensure_supported_function_signature(function, args.len(), span)?;
                self.ensure_user_function_call_depth(function, span)?;
                self.call_user_function_with_checked_values(
                    function,
                    args,
                    None,
                    Some(declaring_class_id),
                    Some(class_id),
                    Vec::new(),
                    None,
                )
            }
            _ => unreachable!("array_callable_parts restricts callback targets"),
        }
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

    fn call_ignore_user_abort(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        if args.len() > 1 {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "ignore_user_abort()",
                    ArityExpectation::Between { min: 0, max: 1 },
                    args.len(),
                ),
            ));
        }

        let previous = if self.ignore_user_abort { 1 } else { 0 };
        if let Some(value) = args.first() {
            match value {
                Value::Null => {}
                Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                    self.ignore_user_abort = value.is_truthy();
                }
                other => {
                    return Err(runtime_error(
                        span,
                        RuntimeError::unsupported_call(
                            "ignore_user_abort()",
                            format!(
                                "setting argument must be null or scalar in the current subset, got {}",
                                other.type_name()
                            ),
                        ),
                    ));
                }
            }
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
                vec![accumulator, entry.value_cloned()],
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
                ArrayFilterMode::Value => vec![entry.value_cloned()],
                ArrayFilterMode::Both => {
                    vec![entry.value_cloned(), value_from_array_key(&entry.key)]
                }
                ArrayFilterMode::Key => vec![value_from_array_key(&entry.key)],
            };
            let result = self.call_callable_with_values(callable.clone(), arguments, span)?;
            if result.is_truthy() {
                filtered.insert(entry.key.clone(), entry.value_cloned());
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
                self.call_callable_with_values(callable.clone(), vec![entry.value_cloned()], span)?;
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
                        .map(|entry| entry.value_cloned())
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
                    .map(|entry| entry.value_cloned())
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
            } => self.is_supported_array_offset_path_set(target, index, caller_scope),
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
                    "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            )),
        }
    }

    fn is_supported_array_offset_path_set(
        &mut self,
        target: &Expr,
        index: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        if let Some((name, indices)) = Self::collect_direct_variable_array_index_path(target, index)
        {
            let mut keys = Vec::with_capacity(indices.len());
            for index in indices {
                keys.push(self.evaluate_array_key(index, caller_scope)?);
            }

            if name == "GLOBALS" {
                let Some(global_name) = keys.first().and_then(globals_offset_name) else {
                    return Err(runtime_error(
                        target.span(),
                        RuntimeError::unsupported_call(
                            "isset()",
                            "only string-keyed direct $GLOBALS offset operands are implemented",
                        ),
                    ));
                };
                return match caller_scope.read_global_name(global_name) {
                    Some(value) => Ok(Self::array_path_isset(&value, &keys[1..])),
                    None => Ok(false),
                };
            }

            return match caller_scope.read_named(name) {
                Some(Value::Object(object))
                    if keys.len() == 1
                        && self
                            .classes
                            .implements_interface(object.class_id(), "ArrayAccess") =>
                {
                    self.array_access_offset_exists(object, keys[0].clone(), target.span())
                }
                Some(value) => Ok(Self::array_path_isset(&value, &keys)),
                None => Ok(false),
            };
        }

        let Some((object_name, property, indices)) =
            Self::collect_direct_object_property_array_index_path(target, index)
        else {
            return Err(runtime_error(
                target.span(),
                RuntimeError::unsupported_call(
                    "isset()",
                    "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            ));
        };

        let mut keys = Vec::with_capacity(indices.len());
        for index in indices {
            keys.push(self.evaluate_array_key(index, caller_scope)?);
        }

        let Some(Value::Object(object)) = caller_scope.read_named(object_name) else {
            return Ok(false);
        };

        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        match object
            .read_property_for_isset_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(target.span(), error))?
        {
            Some(Value::Object(object))
                if keys.len() == 1
                    && self
                        .classes
                        .implements_interface(object.class_id(), "ArrayAccess") =>
            {
                self.array_access_offset_exists(object, keys[0].clone(), target.span())
            }
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
                        "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name) {
            Some(Value::Object(object)) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                match object
                    .read_property_for_isset_from_context(
                        property,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))?
                {
                    Some(value) => Ok(!matches!(value, Value::Null)),
                    None => Ok(self
                        .call_magic_property_method(object, "__isset", property, span)?
                        .is_some_and(|value| value.is_truthy())),
                }
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

    fn collect_direct_object_property_array_index_path<'a>(
        target: &'a Expr,
        index: &'a Expr,
    ) -> Option<(&'a str, &'a str, Vec<&'a Expr>)> {
        let mut indices = vec![index];
        let mut current = target;

        loop {
            match current {
                Expr::Property {
                    target, property, ..
                } => {
                    let Expr::Variable(name, _) = target.as_ref() else {
                        return None;
                    };
                    indices.reverse();
                    return Some((name, property, indices));
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

    fn array_path_value(value: &Value, keys: &[ArrayKey]) -> Option<Value> {
        let mut current = value;

        for key in keys {
            let Value::Array(array) = current else {
                return None;
            };
            current = array.get(key.clone())?;
        }

        Some(current.clone())
    }

    fn array_path_empty(value: &Value, keys: &[ArrayKey]) -> bool {
        let mut current = value;

        for key in keys {
            let Value::Array(array) = current else {
                return true;
            };
            let Some(next) = array.get(key.clone()) else {
                return true;
            };
            current = next;
        }

        !current.is_truthy()
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
                self.is_supported_array_offset_path_empty(target, index, caller_scope)
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
                    "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            )),
        }
    }

    fn is_supported_array_offset_path_empty(
        &mut self,
        target: &Expr,
        index: &Expr,
        caller_scope: &mut SymbolTable,
    ) -> CompileResult<bool> {
        if let Some((name, indices)) = Self::collect_direct_variable_array_index_path(target, index)
        {
            let mut keys = Vec::with_capacity(indices.len());
            for index in indices {
                keys.push(self.evaluate_array_key(index, caller_scope)?);
            }

            return match caller_scope.read_named(name) {
                Some(Value::Object(object))
                    if keys.len() == 1
                        && self
                            .classes
                            .implements_interface(object.class_id(), "ArrayAccess") =>
                {
                    self.is_array_access_offset_empty(object, keys[0].clone(), target.span())
                }
                Some(value) => Ok(Self::array_path_empty(&value, &keys)),
                None => Ok(true),
            };
        }

        let Some((object_name, property, indices)) =
            Self::collect_direct_object_property_array_index_path(target, index)
        else {
            return Err(runtime_error(
                target.span(),
                    RuntimeError::unsupported_call(
                        "empty()",
                        "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            ));
        };

        let mut keys = Vec::with_capacity(indices.len());
        for index in indices {
            keys.push(self.evaluate_array_key(index, caller_scope)?);
        }

        let Some(Value::Object(object)) = caller_scope.read_named(object_name) else {
            return Ok(true);
        };

        let (current_class_id, protected_class_ids) = self.current_property_access_context();
        match object
            .read_property_for_isset_from_context(property, current_class_id, &protected_class_ids)
            .map_err(|error| runtime_error(target.span(), error))?
        {
            Some(Value::Object(object))
                if keys.len() == 1
                    && self
                        .classes
                        .implements_interface(object.class_id(), "ArrayAccess") =>
            {
                self.is_array_access_offset_empty(object, keys[0].clone(), target.span())
            }
            Some(value) => Ok(Self::array_path_empty(&value, &keys)),
            None => Ok(true),
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
                    "only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported",
                ),
            ));
        };

        match caller_scope.read_named(name) {
            Some(Value::Object(object)) => {
                let (current_class_id, protected_class_ids) =
                    self.current_property_access_context();
                match object
                    .read_property_for_isset_from_context(
                        property,
                        current_class_id,
                        &protected_class_ids,
                    )
                    .map_err(|error| runtime_error(span, error))?
                {
                    Some(value) => Ok(!value.is_truthy()),
                    None => {
                        let Some(isset_value) = self.call_magic_property_method(
                            object.clone(),
                            "__isset",
                            property,
                            span,
                        )?
                        else {
                            return Ok(true);
                        };
                        if !isset_value.is_truthy() {
                            return Ok(true);
                        }

                        Ok(self
                            .call_magic_property_method(object, "__get", property, span)?
                            .map_or(true, |value| !value.is_truthy()))
                    }
                }
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

    fn array_key_value(key: Option<ArrayKey>) -> Value {
        match key {
            Some(ArrayKey::Int(value)) => Value::Int(value),
            Some(ArrayKey::String(value)) => Value::String(value),
            None => Value::Null,
        }
    }

    fn call_array_access_method(
        &mut self,
        object: PhpObject,
        method_name: &str,
        args: Vec<Value>,
        span: Span,
    ) -> CompileResult<Value> {
        if !self
            .classes
            .implements_interface(object.class_id(), "ArrayAccess")
        {
            return Err(runtime_error(
                span,
                RuntimeError::invalid_array_access(format!(
                    "object of class {} does not implement ArrayAccess",
                    object.class_name()
                )),
            ));
        }

        self.call_magic_instance_method_with_values(object, method_name, args, span)?
            .ok_or_else(|| {
                runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        format!("ArrayAccess::{method_name}()"),
                        "ArrayAccess objects must declare the required offset method in the current subset",
                    ),
                )
            })
    }

    fn is_countable_value(&self, value: &Value) -> bool {
        match value {
            Value::Array(_) => true,
            Value::Object(object) => self
                .classes
                .implements_interface(object.class_id(), "Countable"),
            _ => false,
        }
    }

    fn is_iterable_value(&self, value: &Value) -> bool {
        match value {
            Value::Array(_) => true,
            Value::Object(object) => {
                let class_id = object.class_id();
                self.classes.implements_interface(class_id, "Traversable")
                    || self.classes.implements_interface(class_id, "Iterator")
                    || self
                        .classes
                        .implements_interface(class_id, "IteratorAggregate")
            }
            _ => false,
        }
    }

    fn call_countable_count_method(
        &mut self,
        object: PhpObject,
        span: Span,
    ) -> CompileResult<Value> {
        let Some((class_id, class_name, resolved_method_name, visibility, is_static)) =
            self.resolve_instance_method(object.class_id(), "count")
        else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "Countable::count()",
                    "Countable objects must declare count() in the current subset",
                ),
            ));
        };

        if is_static {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "Countable::count()",
                    "Countable count method must be non-static in the current subset",
                ),
            ));
        }

        self.ensure_instance_method_visible(class_id, &class_name, "count", visibility, span)?;

        let function = self.method_function(class_id, &class_name, &resolved_method_name, span)?;
        let function = function.as_ref();
        ensure_user_function_arity(function, 0, span)?;
        ensure_supported_function_signature(function, 0, span)?;
        self.ensure_user_function_call_depth(function, span)?;

        let called_class_id = object.class_id();
        self.call_user_function_with_this(
            function,
            object,
            Vec::new(),
            Some(class_id),
            Some(called_class_id),
        )
    }

    fn array_access_offset_exists(
        &mut self,
        object: PhpObject,
        key: ArrayKey,
        span: Span,
    ) -> CompileResult<bool> {
        Ok(self
            .call_array_access_method(
                object,
                "offsetExists",
                vec![Self::array_key_value(Some(key))],
                span,
            )?
            .is_truthy())
    }

    fn is_array_access_offset_empty(
        &mut self,
        object: PhpObject,
        key: ArrayKey,
        span: Span,
    ) -> CompileResult<bool> {
        if !self.array_access_offset_exists(object.clone(), key.clone(), span)? {
            return Ok(true);
        }

        Ok(!self
            .call_array_access_method(
                object,
                "offsetGet",
                vec![Self::array_key_value(Some(key))],
                span,
            )?
            .is_truthy())
    }

    fn object_to_string_with_magic(
        &mut self,
        object: PhpObject,
        context: &str,
        span: Span,
    ) -> CompileResult<Option<String>> {
        let Some(value) =
            self.call_magic_instance_method_with_values(object, "__toString", Vec::new(), span)?
        else {
            return Ok(None);
        };

        match value {
            Value::String(value) => Ok(Some(value)),
            other => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    context,
                    format!(
                        "__toString() must return string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            )),
        }
    }

    fn value_to_echo_string(&mut self, value: Value, span: Span) -> CompileResult<String> {
        if let Value::Object(object) = value.clone() {
            if let Some(output) =
                self.object_to_string_with_magic(object, "object-to-string", span)?
            {
                return Ok(output);
            }
        }

        value
            .try_echo_string()
            .map_err(|error| runtime_error(span, error))
    }

    fn value_to_string_cast(&mut self, value: Value, span: Span) -> CompileResult<String> {
        match value {
            Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                Ok(value.echo_string())
            }
            Value::Array(_) => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "(string)",
                    "array-to-string cast warning behavior is not implemented",
                ),
            )),
            Value::Object(object) => {
                if let Some(output) =
                    self.object_to_string_with_magic(object.clone(), "(string)", span)?
                {
                    Ok(output)
                } else {
                    Value::Object(object)
                        .try_echo_string()
                        .map_err(|error| runtime_error(span, error))
                }
            }
            Value::Closure(_) => Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "(string)",
                    "Closure __toString() and cast error behavior are not implemented",
                ),
            )),
        }
    }

    fn apply_binary(
        &mut self,
        op: BinaryOp,
        left: Value,
        right: Value,
        span: Span,
    ) -> CompileResult<Value> {
        if matches!(op, BinaryOp::Concat) {
            let left = self.value_to_echo_string(left, span)?;
            let right = self.value_to_echo_string(right, span)?;
            return Ok(Value::String(format!("{left}{right}")));
        }

        let result: RuntimeResult<Value> = match op {
            BinaryOp::Add => left.php_add(&right),
            BinaryOp::Sub => left.php_sub(&right),
            BinaryOp::Mul => left.php_mul(&right),
            BinaryOp::Div => left.php_div(&right),
            BinaryOp::Mod => left.php_mod(&right),
            BinaryOp::Concat => unreachable!("concatenation is handled before runtime helpers"),
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

    fn apply_cast(&mut self, kind: CastKind, value: Value, span: Span) -> CompileResult<Value> {
        match kind {
            CastKind::String => self.value_to_string_cast(value, span).map(Value::String),
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
    if is_core_interface_name(&class.name) {
        return Err(runtime_error(
            class.span,
            RuntimeError::duplicate_class(&class.name),
        ));
    }

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
        || is_core_interface_name(&interface.name)
        || interface_lookup.contains_key(&key)
        || trait_lookup.contains_key(&key)
        || enum_lookup.contains_key(&key)
    {
        return Err(runtime_error(
            interface.span,
            RuntimeError::duplicate_class(&interface.name),
        ));
    }

    for parent_name in &interface.parents {
        if !interface_lookup.contains_key(&parent_name.to_ascii_lowercase()) {
            return Err(runtime_error(
                interface.span,
                RuntimeError::unsupported_class_inheritance(
                    &interface.name,
                    format!(
                        "interface {} extends missing or unsupported parent interface {}",
                        interface.name, parent_name
                    ),
                ),
            ));
        }
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
        || is_core_interface_name(&trait_decl.name)
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
        || is_core_interface_name(&enum_decl.name)
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
    method_signatures: &mut HashMap<(ClassId, String), MethodSignature>,
    abstract_methods: &mut HashSet<(ClassId, String)>,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    class_id: ClassId,
    class: &ClassDecl,
) -> CompileResult<()> {
    for method in composed_trait_methods(class, trait_lookup)? {
        let key = (class_id, method.function.name.to_ascii_lowercase());
        method_signatures.insert(key.clone(), method_signature(&method.function));
        methods.insert(key, Rc::new(method.function));
    }

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
                method_signatures.insert(key.clone(), method_signature(&method.function));
                if method.is_abstract {
                    abstract_methods.insert(key);
                } else {
                    methods.insert(key, Rc::new(method.function.clone()));
                }
            }
            ClassMember::Property(_) => {}
        }
    }
    Ok(())
}

fn seed_core_class_constant_runtime_tables(
    classes: &PhpClassTable,
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
) {
    let Some(pdo_id) = classes.lookup_class_id("PDO") else {
        return;
    };
    for (name, value) in [
        ("ATTR_ERRMODE", 3),
        ("ERRMODE_SILENT", 0),
        ("ERRMODE_WARNING", 1),
        ("ERRMODE_EXCEPTION", 2),
        ("ATTR_DEFAULT_FETCH_MODE", 19),
        ("FETCH_ASSOC", 2),
        ("FETCH_NUM", 3),
        ("FETCH_BOTH", 4),
        ("MYSQL_ATTR_INIT_COMMAND", 1002),
    ] {
        let span = Span::new(1, 1);
        class_constants.insert(
            (pdo_id, name.to_string()),
            ClassConstantDecl {
                name: name.to_string(),
                visibility: ClassVisibility::Public,
                value: Expr::Int(value, span),
                span,
            },
        );
    }
}

fn remove_class_member_runtime_tables(
    class_constants: &mut HashMap<(ClassId, String), ClassConstantDecl>,
    static_properties: &mut HashMap<(ClassId, String), Value>,
    methods: &mut HashMap<(ClassId, String), Rc<FunctionDecl>>,
    method_signatures: &mut HashMap<(ClassId, String), MethodSignature>,
    abstract_methods: &mut HashSet<(ClassId, String)>,
    class_id: ClassId,
) {
    class_constants.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    static_properties.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    methods.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    method_signatures.retain(|(declaring_class_id, _), _| *declaring_class_id != class_id);
    abstract_methods.retain(|(declaring_class_id, _)| *declaring_class_id != class_id);
}

fn register_final_method_markers(
    final_methods: &mut HashMap<(ClassId, String), String>,
    class_id: ClassId,
    class: &ClassDecl,
) {
    for member in &class.members {
        let ClassMember::Method(method) = member else {
            continue;
        };
        if method.is_final {
            final_methods.insert(
                (class_id, method.function.name.to_ascii_lowercase()),
                method.function.name.clone(),
            );
        }
    }
}

fn register_class_members(
    classes: &mut PhpClassTable,
    final_classes: &HashSet<ClassId>,
    abstract_methods: &HashSet<(ClassId, String)>,
    final_methods: &HashMap<(ClassId, String), String>,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
    class: &ClassDecl,
) -> CompileResult<ClassId> {
    let id = classes
        .lookup_class_id(&class.name)
        .expect("class name pass should declare class id");

    if let Some(parent_name) = &class.parent {
        let parent_id = classes
            .lookup_class_id(parent_name)
            .ok_or_else(|| runtime_error(class.span, RuntimeError::undefined_class(parent_name)))?;
        if final_classes.contains(&parent_id) {
            let parent = classes
                .get(parent_id)
                .expect("parent class id should resolve to class metadata");
            return Err(runtime_error(
                class.span,
                RuntimeError::unsupported_class_inheritance(
                    &class.name,
                    format!("cannot extend final class {}", parent.name()),
                ),
            ));
        }
        classes
            .set_parent(id, parent_id)
            .map_err(|error| runtime_error(class.span, error))?;
    }
    let interfaces = expanded_class_interface_names(interface_lookup, &class.interfaces);
    classes
        .set_interfaces(id, interfaces)
        .map_err(|error| runtime_error(class.span, error))?;

    for method in composed_trait_methods(class, trait_lookup)? {
        let visibility = runtime_visibility(method.visibility);
        validate_final_method_override(classes, id, &class.name, &method, final_methods)
            .map_err(|error| runtime_error(method.span, error))?;
        validate_inherited_method_static_compatibility(classes, id, &class.name, &method)
            .map_err(|error| runtime_error(method.span, error))?;
        validate_inherited_method_visibility_compatibility(classes, id, &class.name, &method)
            .map_err(|error| runtime_error(method.span, error))?;
        validate_inherited_method_signature_compatibility(
            classes,
            method_signatures,
            id,
            &class.name,
            &method,
        )
        .map_err(|error| runtime_error(method.span, error))?;
        let metadata_method = PhpMethodMetadata::instance(&method.function.name, visibility);
        classes
            .get_mut(id)
            .expect("declared class id should resolve to class metadata")
            .add_method(metadata_method)
            .map_err(|error| runtime_error(method.span, error))?;
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
                validate_final_method_override(classes, id, &class.name, method, final_methods)
                    .map_err(|error| runtime_error(method.span, error))?;
                validate_inherited_method_static_compatibility(classes, id, &class.name, method)
                    .map_err(|error| runtime_error(method.span, error))?;
                validate_inherited_method_visibility_compatibility(
                    classes,
                    id,
                    &class.name,
                    method,
                )
                .map_err(|error| runtime_error(method.span, error))?;
                validate_inherited_method_signature_compatibility(
                    classes,
                    method_signatures,
                    id,
                    &class.name,
                    method,
                )
                .map_err(|error| runtime_error(method.span, error))?;
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

    let mut effective_method_signatures = method_signatures.clone();
    for method in composed_trait_methods(class, trait_lookup)? {
        effective_method_signatures.insert(
            (id, method.function.name.to_ascii_lowercase()),
            method_signature(&method.function),
        );
    }
    for member in &class.members {
        if let ClassMember::Method(method) = member {
            effective_method_signatures.insert(
                (id, method.function.name.to_ascii_lowercase()),
                method_signature(&method.function),
            );
        }
    }

    validate_abstract_method_implementation(classes, abstract_methods, id, class)
        .map_err(|error| runtime_error(class.span, error))?;
    validate_interface_method_implementation(
        classes,
        interface_lookup,
        &effective_method_signatures,
        id,
        class,
    )
    .map_err(|error| runtime_error(class.span, error))?;
    validate_core_countable_method_implementation(classes, &effective_method_signatures, id, class)
        .map_err(|error| runtime_error(class.span, error))?;
    validate_core_iterable_method_implementation(classes, &effective_method_signatures, id, class)
        .map_err(|error| runtime_error(class.span, error))?;

    Ok(id)
}

fn composed_trait_methods(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> CompileResult<Vec<ClassMethodDecl>> {
    let mut methods = Vec::new();
    let precedence_exclusions = trait_precedence_exclusions(class, trait_lookup)?;
    for trait_use in &class.trait_uses {
        let key = trait_use.name.to_ascii_lowercase();
        let trait_decl = trait_lookup.get(&key).ok_or_else(|| {
            runtime_error(
                trait_use.span,
                RuntimeError::undefined_class(&trait_use.name),
            )
        })?;
        for method in &trait_decl.methods {
            let method_key = method.function.name.to_ascii_lowercase();
            if precedence_exclusions.contains(&(key.clone(), method_key)) {
                continue;
            }
            methods.push(method.clone());
        }
        for alias in &trait_use.aliases {
            let Some(method) = trait_decl.methods.iter().find(|method| {
                method
                    .function
                    .name
                    .eq_ignore_ascii_case(&alias.method_name)
            }) else {
                return Err(runtime_error(
                    alias.span,
                    RuntimeError::unsupported_trait_use(format!(
                        "trait alias {}::{} targets a missing method",
                        trait_decl.name, alias.method_name
                    )),
                ));
            };
            let mut aliased = method.clone();
            aliased.function.name = alias.alias.clone();
            aliased.span = alias.span;
            methods.push(aliased);
        }
    }
    Ok(methods)
}

fn trait_precedence_exclusions(
    class: &ClassDecl,
    trait_lookup: &HashMap<String, Rc<TraitDecl>>,
) -> CompileResult<HashSet<(String, String)>> {
    let mut exclusions = HashSet::new();
    for trait_use in &class.trait_uses {
        let winner_key = trait_use.name.to_ascii_lowercase();
        let winner_trait = trait_lookup.get(&winner_key).ok_or_else(|| {
            runtime_error(
                trait_use.span,
                RuntimeError::undefined_class(&trait_use.name),
            )
        })?;
        for precedence in &trait_use.precedences {
            let Some(winner_method) = winner_trait.methods.iter().find(|method| {
                method
                    .function
                    .name
                    .eq_ignore_ascii_case(&precedence.method_name)
            }) else {
                return Err(runtime_error(
                    precedence.span,
                    RuntimeError::unsupported_trait_use(format!(
                        "trait precedence {}::{} targets a missing winning method",
                        winner_trait.name, precedence.method_name
                    )),
                ));
            };

            let loser_key = precedence.loser_trait_name.to_ascii_lowercase();
            let loser_trait = trait_lookup.get(&loser_key).ok_or_else(|| {
                runtime_error(
                    precedence.span,
                    RuntimeError::undefined_class(&precedence.loser_trait_name),
                )
            })?;
            if !loser_trait.methods.iter().any(|method| {
                method
                    .function
                    .name
                    .eq_ignore_ascii_case(&winner_method.function.name)
            }) {
                return Err(runtime_error(
                    precedence.span,
                    RuntimeError::unsupported_trait_use(format!(
                        "trait precedence {}::{} excludes missing loser method {}::{}",
                        winner_trait.name,
                        precedence.method_name,
                        loser_trait.name,
                        winner_method.function.name
                    )),
                ));
            }
            exclusions.insert((loser_key, winner_method.function.name.to_ascii_lowercase()));
        }
    }
    Ok(exclusions)
}

fn expanded_class_interface_names(
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    direct_interfaces: &[String],
) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    for interface_name in direct_interfaces {
        push_interface_with_parents(interface_lookup, interface_name, &mut names, &mut seen);
    }
    names
}

fn push_interface_with_parents(
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    interface_name: &str,
    names: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    let key = interface_name.to_ascii_lowercase();
    if !seen.insert(key.clone()) {
        return;
    }
    names.push(interface_name.to_string());
    let Some(interface) = interface_lookup.get(&key) else {
        return;
    };
    for parent_name in &interface.parents {
        push_interface_with_parents(interface_lookup, parent_name, names, seen);
    }
}

fn expanded_interface_methods<'a>(
    interface_lookup: &'a HashMap<String, Rc<InterfaceDecl>>,
    interface: &'a InterfaceDecl,
) -> Vec<(String, &'a InterfaceMethodDecl)> {
    let mut methods = Vec::new();
    let mut visited = HashSet::new();
    collect_interface_methods(interface_lookup, interface, &mut methods, &mut visited);
    methods
}

fn collect_interface_methods<'a>(
    interface_lookup: &'a HashMap<String, Rc<InterfaceDecl>>,
    interface: &'a InterfaceDecl,
    methods: &mut Vec<(String, &'a InterfaceMethodDecl)>,
    visited: &mut HashSet<String>,
) {
    let key = interface.name.to_ascii_lowercase();
    if !visited.insert(key) {
        return;
    }
    for parent_name in &interface.parents {
        if let Some(parent) = interface_lookup.get(&parent_name.to_ascii_lowercase()) {
            collect_interface_methods(interface_lookup, parent, methods, visited);
        }
    }
    for method in &interface.methods {
        methods.push((interface.name.clone(), method));
    }
}

fn validate_interface_method_implementation(
    classes: &PhpClassTable,
    interface_lookup: &HashMap<String, Rc<InterfaceDecl>>,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class_id: ClassId,
    class: &ClassDecl,
) -> RuntimeResult<()> {
    if class.is_abstract {
        return Ok(());
    }

    let mut missing = Vec::new();
    let mut covered_names = HashSet::new();
    for interface_name in implemented_interface_names(classes, class_id) {
        let Some(interface) = interface_lookup.get(&interface_name.to_ascii_lowercase()) else {
            continue;
        };

        let interface_methods = expanded_interface_methods(interface_lookup, interface);
        for (method_interface_name, method) in interface_methods {
            let lookup_name = method.function.name.to_ascii_lowercase();
            if !covered_names.insert(format!(
                "{}::{lookup_name}",
                method_interface_name.to_ascii_lowercase()
            )) {
                continue;
            }
            let Some((declaring_class_id, declaring_class_name, class_method)) =
                find_public_method(classes, class_id, &lookup_name)
            else {
                missing.push(format!(
                    "{method_interface_name}::{}()",
                    method.function.name
                ));
                continue;
            };

            if class_method.is_static() {
                return Err(RuntimeError::unsupported_class_inheritance(
                    &class.name,
                    format!(
                        "concrete class {} must implement interface method {}::{}() as {} method; found {} {}::{}()",
                        class.name,
                        method_interface_name,
                        method.function.name,
                        method_static_name(false),
                        method_static_name(class_method.is_static()),
                        declaring_class_name,
                        class_method.name()
                    ),
                ));
            }

            let interface_required = required_param_count(&method.function);
            let class_required = public_method_required_param_count(
                method_signatures,
                class,
                class_id,
                declaring_class_id,
                &lookup_name,
            );
            if class_required > interface_required {
                return Err(RuntimeError::unsupported_class_inheritance(
                    &class.name,
                    format!(
                        "method {}::{}() cannot require more parameters than interface method {}::{}()",
                        declaring_class_name,
                        class_method.name(),
                        method_interface_name,
                        method.function.name
                    ),
                ));
            }

            let class_signature = public_method_signature(
                method_signatures,
                class,
                class_id,
                declaring_class_id,
                &lookup_name,
            );
            validate_interface_parameter_type_compatibility(
                &class.name,
                &method_interface_name,
                method,
                declaring_class_name,
                class_method.name(),
                class_signature.as_ref(),
            )?;
            validate_interface_return_type_compatibility(
                &class.name,
                &method_interface_name,
                method,
                declaring_class_name,
                class_method.name(),
                class_signature.as_ref(),
            )?;
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    let method_list = missing.join(", ");
    let method_word = if missing.len() == 1 {
        "method"
    } else {
        "methods"
    };
    Err(RuntimeError::unsupported_class_inheritance(
        &class.name,
        format!(
            "concrete class {} must implement interface {method_word} {method_list}",
            class.name
        ),
    ))
}

fn validate_core_countable_method_implementation(
    classes: &PhpClassTable,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class_id: ClassId,
    class: &ClassDecl,
) -> RuntimeResult<()> {
    if class.is_abstract
        || !implemented_interface_names(classes, class_id)
            .iter()
            .any(|interface| interface.eq_ignore_ascii_case("Countable"))
    {
        return Ok(());
    }

    let Some((declaring_class_id, declaring_class_name, class_method)) =
        find_public_method(classes, class_id, "count")
    else {
        return Err(RuntimeError::unsupported_class_inheritance(
            &class.name,
            format!(
                "concrete class {} must implement internal interface method Countable::count()",
                class.name
            ),
        ));
    };

    if class_method.is_static() {
        return Err(RuntimeError::unsupported_class_inheritance(
            &class.name,
            format!(
                "concrete class {} must implement internal interface method Countable::count() as non static method; found static {}::{}()",
                class.name,
                declaring_class_name,
                class_method.name()
            ),
        ));
    }

    let class_required = public_method_required_param_count(
        method_signatures,
        class,
        class_id,
        declaring_class_id,
        "count",
    );
    if class_required > 0 {
        return Err(RuntimeError::unsupported_class_inheritance(
            &class.name,
            format!(
                "method {}::{}() cannot require parameters for internal interface method Countable::count()",
                declaring_class_name,
                class_method.name()
            ),
        ));
    }

    Ok(())
}

fn validate_core_iterable_method_implementation(
    classes: &PhpClassTable,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class_id: ClassId,
    class: &ClassDecl,
) -> RuntimeResult<()> {
    if class.is_abstract {
        return Ok(());
    }

    let interface_names = implemented_interface_names(classes, class_id);
    let implements_iterator = interface_names
        .iter()
        .any(|interface| interface.eq_ignore_ascii_case("Iterator"));
    let implements_iterator_aggregate = interface_names
        .iter()
        .any(|interface| interface.eq_ignore_ascii_case("IteratorAggregate"));
    if interface_names
        .iter()
        .any(|interface| interface.eq_ignore_ascii_case("Traversable"))
        && !implements_iterator
        && !implements_iterator_aggregate
    {
        return Err(RuntimeError::unsupported_class_inheritance(
            &class.name,
            format!(
                "concrete class {} cannot directly implement internal interface Traversable; implement Iterator or IteratorAggregate in the current subset",
                class.name
            ),
        ));
    }

    if implements_iterator {
        validate_core_interface_required_methods(
            classes,
            method_signatures,
            class_id,
            class,
            "Iterator",
            &["current", "key", "next", "rewind", "valid"],
        )?;
    }

    if implements_iterator_aggregate {
        validate_core_interface_required_methods(
            classes,
            method_signatures,
            class_id,
            class,
            "IteratorAggregate",
            &["getIterator"],
        )?;
    }

    Ok(())
}

fn validate_core_interface_required_methods(
    classes: &PhpClassTable,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class_id: ClassId,
    class: &ClassDecl,
    interface_name: &str,
    method_names: &[&str],
) -> RuntimeResult<()> {
    let mut missing = Vec::new();
    for method_name in method_names {
        let Some((declaring_class_id, declaring_class_name, class_method)) =
            find_public_method(classes, class_id, method_name)
        else {
            missing.push(format!("{interface_name}::{method_name}()"));
            continue;
        };

        if class_method.is_static() {
            return Err(RuntimeError::unsupported_class_inheritance(
                &class.name,
                format!(
                    "concrete class {} must implement internal interface method {interface_name}::{method_name}() as non static method; found static {}::{}()",
                    class.name,
                    declaring_class_name,
                    class_method.name()
                ),
            ));
        }

        let class_required = public_method_required_param_count(
            method_signatures,
            class,
            class_id,
            declaring_class_id,
            method_name,
        );
        if class_required > 0 {
            return Err(RuntimeError::unsupported_class_inheritance(
                &class.name,
                format!(
                    "method {}::{}() cannot require parameters for internal interface method {interface_name}::{method_name}()",
                    declaring_class_name,
                    class_method.name()
                ),
            ));
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    let method_list = missing.join(", ");
    let method_word = if missing.len() == 1 {
        "method"
    } else {
        "methods"
    };
    Err(RuntimeError::unsupported_class_inheritance(
        &class.name,
        format!(
            "concrete class {} must implement internal interface {method_word} {method_list}",
            class.name
        ),
    ))
}

fn validate_interface_parameter_type_compatibility(
    class_name: &str,
    interface_name: &str,
    interface_method: &InterfaceMethodDecl,
    declaring_class_name: &str,
    class_method_name: &str,
    class_signature: Option<&MethodSignature>,
) -> RuntimeResult<()> {
    let Some(class_signature) = class_signature else {
        return Ok(());
    };

    for (index, interface_param) in interface_method.function.params.iter().enumerate() {
        let Some(class_param) = class_signature.params.get(index) else {
            continue;
        };

        match (
            interface_param
                .type_decl
                .as_ref()
                .map(|decl| decl.text.as_str()),
            class_param.type_decl.as_deref(),
        ) {
            (None, Some(class_type)) => {
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "method {}::{}() cannot add parameter type {} for parameter ${} when interface method {}::{}() has no parameter type",
                        declaring_class_name,
                        class_method_name,
                        class_type,
                        class_param.name,
                        interface_name,
                        interface_method.function.name
                    ),
                ));
            }
            (Some(interface_type), Some(class_type))
                if !class_type.eq_ignore_ascii_case(interface_type) =>
            {
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "method {}::{}() parameter ${} type {} is incompatible with interface method {}::{}() parameter type {}",
                        declaring_class_name,
                        class_method_name,
                        class_param.name,
                        class_type,
                        interface_name,
                        interface_method.function.name,
                        interface_type
                    ),
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

fn validate_interface_return_type_compatibility(
    class_name: &str,
    interface_name: &str,
    interface_method: &InterfaceMethodDecl,
    declaring_class_name: &str,
    class_method_name: &str,
    class_signature: Option<&MethodSignature>,
) -> RuntimeResult<()> {
    let Some(class_signature) = class_signature else {
        return Ok(());
    };

    match (
        interface_method
            .function
            .return_type
            .as_ref()
            .map(|decl| decl.text.as_str()),
        class_signature.return_type.as_deref(),
    ) {
        (Some(interface_type), None) => Err(RuntimeError::unsupported_class_inheritance(
            class_name,
            format!(
                "method {}::{}() must declare return type {} to match interface method {}::{}()",
                declaring_class_name,
                class_method_name,
                interface_type,
                interface_name,
                interface_method.function.name
            ),
        )),
        (Some(interface_type), Some(class_type))
            if !class_type.eq_ignore_ascii_case(interface_type) =>
        {
            Err(RuntimeError::unsupported_class_inheritance(
                class_name,
                format!(
                    "method {}::{}() return type {} is incompatible with interface method {}::{}() return type {}",
                    declaring_class_name,
                    class_method_name,
                    class_type,
                    interface_name,
                    interface_method.function.name,
                    interface_type
                ),
            ))
        }
        _ => Ok(()),
    }
}

fn public_method_required_param_count(
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class: &ClassDecl,
    class_id: ClassId,
    declaring_class_id: ClassId,
    method_lookup_name: &str,
) -> usize {
    if let Some(signature) =
        method_signatures.get(&(declaring_class_id, method_lookup_name.to_string()))
    {
        return signature.required_params;
    }

    if declaring_class_id == class_id {
        return class
            .members
            .iter()
            .find_map(|member| {
                let ClassMember::Method(method) = member else {
                    return None;
                };
                method
                    .function
                    .name
                    .eq_ignore_ascii_case(method_lookup_name)
                    .then(|| required_param_count(&method.function))
            })
            .unwrap_or(0);
    }

    method_signatures
        .get(&(declaring_class_id, method_lookup_name.to_string()))
        .map_or(0, |signature| signature.required_params)
}

fn public_method_signature(
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class: &ClassDecl,
    class_id: ClassId,
    declaring_class_id: ClassId,
    method_lookup_name: &str,
) -> Option<MethodSignature> {
    if let Some(signature) =
        method_signatures.get(&(declaring_class_id, method_lookup_name.to_string()))
    {
        return Some(signature.clone());
    }

    if declaring_class_id == class_id {
        return class.members.iter().find_map(|member| {
            let ClassMember::Method(method) = member else {
                return None;
            };
            method
                .function
                .name
                .eq_ignore_ascii_case(method_lookup_name)
                .then(|| method_signature(&method.function))
        });
    }

    method_signatures
        .get(&(declaring_class_id, method_lookup_name.to_string()))
        .cloned()
}

fn method_signature(function: &FunctionDecl) -> MethodSignature {
    MethodSignature {
        required_params: required_param_count(function),
        return_type: function.return_type.as_ref().map(|decl| decl.text.clone()),
        params: function
            .params
            .iter()
            .map(|param| ParameterSignature {
                name: param.name.clone(),
                type_decl: param.type_decl.as_ref().map(|decl| decl.text.clone()),
            })
            .collect(),
    }
}

fn implemented_interface_names(classes: &PhpClassTable, class_id: ClassId) -> Vec<String> {
    let mut names = Vec::new();
    let mut current = Some(class_id);
    while let Some(current_id) = current {
        let current_class = classes
            .get(current_id)
            .expect("class id should resolve to class metadata");
        names.extend(current_class.interfaces().iter().cloned());
        current = current_class.parent_id();
    }

    names
}

fn find_public_method<'a>(
    classes: &'a PhpClassTable,
    class_id: ClassId,
    method_lookup_name: &str,
) -> Option<(ClassId, &'a str, &'a PhpMethodMetadata)> {
    let mut current = Some(class_id);
    while let Some(current_id) = current {
        let current_class = classes
            .get(current_id)
            .expect("class id should resolve to class metadata");
        if let Some(method) = current_class.method(method_lookup_name) {
            if method.visibility() == Visibility::Public {
                return Some((current_id, current_class.name(), method));
            }
        }
        current = current_class.parent_id();
    }

    None
}

fn validate_abstract_method_implementation(
    classes: &PhpClassTable,
    abstract_methods: &HashSet<(ClassId, String)>,
    class_id: ClassId,
    class: &ClassDecl,
) -> RuntimeResult<()> {
    if class.is_abstract {
        return Ok(());
    }

    let own_abstract_methods = abstract_method_names_for_class_decl(class);
    let mut missing = Vec::new();
    let mut covered_names = HashSet::new();

    for member in &class.members {
        let ClassMember::Method(method) = member else {
            continue;
        };
        let lookup_name = method.function.name.to_ascii_lowercase();
        if method.is_abstract {
            covered_names.insert(lookup_name);
            missing.push(format!("{}::{}()", class.name, method.function.name));
        } else {
            covered_names.insert(lookup_name);
        }
    }

    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        for method in parent.methods() {
            let lookup_name = method.name().to_ascii_lowercase();
            if !covered_names.insert(lookup_name.clone()) {
                continue;
            }
            if !abstract_methods.contains(&(parent_id, lookup_name.clone())) {
                continue;
            }
            if has_concrete_method_implementation(
                classes,
                abstract_methods,
                class_id,
                &own_abstract_methods,
                &lookup_name,
            ) {
                continue;
            }
            missing.push(format!("{}::{}()", parent.name(), method.name()));
        }

        current = parent.parent_id();
    }

    if missing.is_empty() {
        return Ok(());
    }

    let method_list = missing.join(", ");
    let method_word = if missing.len() == 1 {
        "method"
    } else {
        "methods"
    };
    Err(RuntimeError::unsupported_class_inheritance(
        &class.name,
        format!(
            "concrete class {} must implement abstract {method_word} {method_list}",
            class.name
        ),
    ))
}

fn abstract_method_names_for_class_decl(class: &ClassDecl) -> HashSet<String> {
    class
        .members
        .iter()
        .filter_map(|member| {
            let ClassMember::Method(method) = member else {
                return None;
            };
            method
                .is_abstract
                .then(|| method.function.name.to_ascii_lowercase())
        })
        .collect()
}

fn has_concrete_method_implementation(
    classes: &PhpClassTable,
    abstract_methods: &HashSet<(ClassId, String)>,
    class_id: ClassId,
    own_abstract_methods: &HashSet<String>,
    method_lookup_name: &str,
) -> bool {
    if classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .method(method_lookup_name)
        .is_some()
    {
        return !own_abstract_methods.contains(method_lookup_name);
    }

    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();
    while let Some(current_id) = current {
        let current_class = classes
            .get(current_id)
            .expect("class id should resolve to class metadata");
        if current_class.method(method_lookup_name).is_some() {
            return !abstract_methods.contains(&(current_id, method_lookup_name.to_string()));
        }
        current = current_class.parent_id();
    }

    false
}

fn validate_final_method_override(
    classes: &PhpClassTable,
    class_id: ClassId,
    class_name: &str,
    method: &ClassMethodDecl,
    final_methods: &HashMap<(ClassId, String), String>,
) -> RuntimeResult<()> {
    let method_lookup_name = method.function.name.to_ascii_lowercase();
    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        if let Some(parent_method_name) =
            final_methods.get(&(parent_id, method_lookup_name.clone()))
        {
            return Err(RuntimeError::unsupported_class_inheritance(
                class_name,
                format!(
                    "cannot override final method {}::{}()",
                    parent.name(),
                    parent_method_name
                ),
            ));
        }

        current = parent.parent_id();
    }

    Ok(())
}

fn validate_inherited_method_visibility_compatibility(
    classes: &PhpClassTable,
    class_id: ClassId,
    class_name: &str,
    method: &ClassMethodDecl,
) -> RuntimeResult<()> {
    let visibility = runtime_visibility(method.visibility);
    let method_lookup_name = method.function.name.to_ascii_lowercase();
    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        if let Some(parent_method) = parent.method(&method_lookup_name) {
            if parent_method.visibility() == Visibility::Private {
                current = parent.parent_id();
                continue;
            }

            if visibility_is_more_restrictive(visibility, parent_method.visibility()) {
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "method {}::{}() cannot reduce visibility of inherited {} method {}::{}()",
                        class_name,
                        method.function.name,
                        visibility_name(parent_method.visibility()),
                        parent.name(),
                        parent_method.name()
                    ),
                ));
            }

            return Ok(());
        }

        current = parent.parent_id();
    }

    Ok(())
}

fn validate_inherited_method_static_compatibility(
    classes: &PhpClassTable,
    class_id: ClassId,
    class_name: &str,
    method: &ClassMethodDecl,
) -> RuntimeResult<()> {
    let method_lookup_name = method.function.name.to_ascii_lowercase();
    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        if let Some(parent_method) = parent.method(&method_lookup_name) {
            if parent_method.visibility() == Visibility::Private {
                current = parent.parent_id();
                continue;
            }

            if parent_method.is_static() != method.is_static {
                let parent_static = method_static_name(parent_method.is_static());
                let child_static = method_static_name(method.is_static);
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "cannot redeclare {parent_static} method {}::{}() as {child_static} {}::{}()",
                        parent.name(),
                        parent_method.name(),
                        class_name,
                        method.function.name
                    ),
                ));
            }

            return Ok(());
        }

        current = parent.parent_id();
    }

    Ok(())
}

fn method_static_name(is_static: bool) -> &'static str {
    if is_static {
        "static"
    } else {
        "non static"
    }
}

fn validate_inherited_method_signature_compatibility(
    classes: &PhpClassTable,
    method_signatures: &HashMap<(ClassId, String), MethodSignature>,
    class_id: ClassId,
    class_name: &str,
    method: &ClassMethodDecl,
) -> RuntimeResult<()> {
    let method_lookup_name = method.function.name.to_ascii_lowercase();
    if method_lookup_name == "__construct" {
        return Ok(());
    }

    let child_required = required_param_count(&method.function);
    let mut current = classes
        .get(class_id)
        .expect("class id should resolve to class metadata")
        .parent_id();

    while let Some(parent_id) = current {
        let parent = classes
            .get(parent_id)
            .expect("parent class id should resolve to class metadata");

        if let Some(parent_method) = parent.method(&method_lookup_name) {
            if parent_method.visibility() == Visibility::Private {
                current = parent.parent_id();
                continue;
            }

            let Some(parent_signature) =
                method_signatures.get(&(parent_id, method_lookup_name.clone()))
            else {
                return Ok(());
            };

            if child_required > parent_signature.required_params {
                return Err(RuntimeError::unsupported_class_inheritance(
                    class_name,
                    format!(
                        "method {}::{}() cannot require more parameters than inherited method {}::{}()",
                        class_name,
                        method.function.name,
                        parent.name(),
                        parent_method.name()
                    ),
                ));
            }

            return Ok(());
        }

        current = parent.parent_id();
    }

    Ok(())
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

            if visibility_is_more_restrictive(visibility, parent_property.visibility()) {
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

fn visibility_is_more_restrictive(child: Visibility, parent: Visibility) -> bool {
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

    let Value::String(method_name) = entries[1].value() else {
        return None;
    };

    match entries[0].value() {
        Value::String(_) | Value::Object(_) => Some((entries[0].value(), method_name)),
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
            | "ltrim"
            | "rtrim"
            | "strcasecmp"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "strpos"
            | "substr"
            | "substr_count"
            | "str_replace"
            | "preg_match"
            | "preg_replace"
            | "preg_split"
            | "preg_replace_callback"
            | "compact"
            | "error_reporting"
            | "ignore_user_abort"
            | "php_sapi_name"
            | "sprintf"
            | "vsprintf"
            | "call_user_func"
            | "call_user_func_array"
            | "implode"
            | "basename"
            | "dirname"
            | "abs"
            | "version_compare"
            | "microtime"
            | "date_default_timezone_set"
            | "ini_get"
            | "ini_set"
            | "min"
            | "rand"
            | "uniqid"
            | "hash_hmac"
            | "count"
            | "constant"
            | "defined"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "current"
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
            | "ksort"
            | "array_unshift"
            | "array_pop"
            | "next"
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
            | "mysqli_get_server_version"
            | "mysqli_get_host_info"
            | "mysqli_get_client_info"
            | "mysqli_get_client_version"
            | "mysqli_get_proto_info"
            | "mysqli_thread_id"
            | "mysqli_kill"
            | "mysqli_change_user"
            | "mysqli_refresh"
            | "mysqli_get_charset"
            | "mysqli_character_set_name"
            | "mysqli_field_count"
            | "mysqli_close"
            | "mysqli_options"
            | "mysqli_set_opt"
            | "mysqli_ssl_set"
            | "mysqli_connect_errno"
            | "mysqli_connect_error"
            | "mysqli_error_list"
            | "mysqli_get_connection_stats"
            | "mysqli_get_links_stats"
            | "mysqli_get_client_stats"
            | "mysqli_thread_safe"
            | "mysqli_stmt_init"
            | "mysqli_prepare"
            | "mysqli_stmt_prepare"
            | "mysqli_stmt_param_count"
            | "mysqli_stmt_get_warnings"
            | "mysqli_stmt_error_list"
            | "mysqli_stmt_bind_param"
            | "mysqli_stmt_bind_result"
            | "mysqli_stmt_execute"
            | "mysqli_execute"
            | "mysqli_stmt_get_result"
            | "mysqli_stmt_close"
            | "mysqli_stmt_errno"
            | "mysqli_stmt_error"
            | "mysqli_stmt_affected_rows"
            | "mysqli_stmt_store_result"
            | "mysqli_stmt_num_rows"
            | "mysqli_stmt_fetch"
            | "mysqli_stmt_result_metadata"
            | "mysqli_stmt_field_count"
            | "mysqli_stmt_free_result"
            | "mysqli_stmt_data_seek"
            | "mysqli_stmt_attr_get"
            | "mysqli_stmt_attr_set"
            | "mysqli_stmt_send_long_data"
            | "mysqli_stmt_reset"
            | "mysqli_stmt_more_results"
            | "mysqli_stmt_next_result"
            | "mysqli_stmt_sqlstate"
            | "mysqli_stmt_warning_count"
            | "mysqli_stmt_insert_id"
            | "mysqli_execute_query"
            | "mysqli_dump_debug_info"
            | "mysqli_debug"
            | "mysqli_stat"
            | "mysqli_autocommit"
            | "mysqli_begin_transaction"
            | "mysqli_commit"
            | "mysqli_rollback"
            | "mysqli_savepoint"
            | "mysqli_release_savepoint"
            | "mysqli_set_charset"
            | "mysqli_query"
            | "mysqli_real_query"
            | "mysqli_multi_query"
            | "mysqli_errno"
            | "mysqli_error"
            | "mysqli_sqlstate"
            | "mysqli_warning_count"
            | "mysqli_info"
            | "mysqli_get_warnings"
            | "mysqli_affected_rows"
            | "mysqli_insert_id"
            | "mysqli_ping"
            | "mysqli_select_db"
            | "mysqli_real_escape_string"
            | "mysqli_escape_string"
            | "mysqli_fetch_object"
            | "mysqli_fetch_assoc"
            | "mysqli_fetch_row"
            | "mysqli_fetch_array"
            | "mysqli_fetch_all"
            | "mysqli_fetch_column"
            | "mysqli_fetch_field"
            | "mysqli_fetch_fields"
            | "mysqli_fetch_field_direct"
            | "mysqli_num_fields"
            | "mysqli_num_rows"
            | "mysqli_fetch_lengths"
            | "mysqli_data_seek"
            | "mysqli_field_seek"
            | "mysqli_field_tell"
            | "mysqli_free_result"
            | "mysqli_more_results"
            | "mysqli_next_result"
            | "mysqli_store_result"
            | "mysqli_use_result"
            | "mysqli_reap_async_query"
            | "mysqli_poll"
            | "mysqli_report"
            | "mysqli_init"
            | "file_exists"
            | "file_get_contents"
            | "realpath"
            | "getcwd"
            | "is_dir"
            | "is_file"
            | "is_readable"
            | "is_writable"
            | "is_link"
            | "register_shutdown_function"
            | "set_error_handler"
            | "restore_error_handler"
            | "ob_start"
            | "ob_get_level"
            | "ob_get_clean"
            | "header"
            | "header_remove"
            | "headers_list"
            | "headers_sent"
            | "setcookie"
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

fn basename_path(path: &str, suffix: Option<&str>) -> String {
    if path.is_empty() {
        return String::new();
    }

    let bytes = path.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b'/' {
        end -= 1;
    }

    if end == 0 {
        return String::new();
    }

    let trimmed = &path[..end];
    let start = trimmed.rfind('/').map_or(0, |position| position + 1);
    let mut name = trimmed[start..].to_string();

    if let Some(suffix) = suffix {
        if !suffix.is_empty() && name.ends_with(suffix) {
            let new_len = name.len() - suffix.len();
            name.truncate(new_len);
        }
    }

    name
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
const PHP_MYSQLI_ASSOC: i64 = 1;
const PHP_MYSQLI_NUM: i64 = 2;
const PHP_MYSQLI_BOTH: i64 = 3;
const PHP_MYSQLI_ASYNC: i64 = 8;
const PHP_MYSQLI_CLIENT_SSL: i64 = 2048;
const PHP_MYSQLI_CLIENT_COMPRESS: i64 = 32;
const PHP_MYSQLI_CLIENT_INTERACTIVE: i64 = 1024;
const PHP_MYSQLI_CLIENT_IGNORE_SPACE: i64 = 256;
const PHP_MYSQLI_CLIENT_NO_SCHEMA: i64 = 16;
const PHP_MYSQLI_CLIENT_FOUND_ROWS: i64 = 2;
const PHP_MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT: i64 = 1_073_741_824;
const PHP_MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT: i64 = 64;
const PHP_MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS: i64 = 4_194_304;
const PHP_MYSQLI_CLIENT_ALL_SUPPORTED: i64 = PHP_MYSQLI_CLIENT_SSL
    | PHP_MYSQLI_CLIENT_COMPRESS
    | PHP_MYSQLI_CLIENT_INTERACTIVE
    | PHP_MYSQLI_CLIENT_IGNORE_SPACE
    | PHP_MYSQLI_CLIENT_NO_SCHEMA
    | PHP_MYSQLI_CLIENT_FOUND_ROWS
    | PHP_MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT
    | PHP_MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT
    | PHP_MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS;
const PHP_MYSQLI_OPT_CONNECT_TIMEOUT: i64 = 0;
const PHP_MYSQLI_OPT_LOCAL_INFILE: i64 = 8;
const PHP_MYSQLI_OPT_LOAD_DATA_LOCAL_DIR: i64 = 43;
const PHP_MYSQLI_INIT_COMMAND: i64 = 3;
const PHP_MYSQLI_OPT_READ_TIMEOUT: i64 = 11;
const PHP_MYSQLI_OPT_NET_CMD_BUFFER_SIZE: i64 = 202;
const PHP_MYSQLI_OPT_NET_READ_BUFFER_SIZE: i64 = 203;
const PHP_MYSQLI_OPT_INT_AND_FLOAT_NATIVE: i64 = 201;
const PHP_MYSQLI_OPT_SSL_VERIFY_SERVER_CERT: i64 = 21;
const PHP_MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS: i64 = 37;
const PHP_MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH: i64 = 0;
const PHP_MYSQLI_STMT_ATTR_CURSOR_TYPE: i64 = 1;
const PHP_MYSQLI_STMT_ATTR_PREFETCH_ROWS: i64 = 2;
const PHP_MYSQLI_CURSOR_TYPE_NO_CURSOR: i64 = 0;
const PHP_MYSQLI_CURSOR_TYPE_READ_ONLY: i64 = 1;
const PHP_MYSQLI_CURSOR_TYPE_FOR_UPDATE: i64 = 2;
const PHP_MYSQLI_CURSOR_TYPE_SCROLLABLE: i64 = 4;
const PHP_MYSQLI_REFRESH_GRANT: i64 = 1;
const PHP_MYSQLI_REFRESH_LOG: i64 = 2;
const PHP_MYSQLI_REFRESH_TABLES: i64 = 4;
const PHP_MYSQLI_REFRESH_HOSTS: i64 = 8;
const PHP_MYSQLI_REFRESH_STATUS: i64 = 16;
const PHP_MYSQLI_REFRESH_THREADS: i64 = 32;
const PHP_MYSQLI_REFRESH_SLAVE: i64 = 64;
const PHP_MYSQLI_REFRESH_MASTER: i64 = 128;
const PHP_MYSQLI_REFRESH_BACKUP_LOG: i64 = 2_097_152;
const PHP_MYSQLI_REFRESH_ALL_SUPPORTED: i64 = PHP_MYSQLI_REFRESH_GRANT
    | PHP_MYSQLI_REFRESH_LOG
    | PHP_MYSQLI_REFRESH_TABLES
    | PHP_MYSQLI_REFRESH_HOSTS
    | PHP_MYSQLI_REFRESH_STATUS
    | PHP_MYSQLI_REFRESH_THREADS
    | PHP_MYSQLI_REFRESH_SLAVE
    | PHP_MYSQLI_REFRESH_MASTER
    | PHP_MYSQLI_REFRESH_BACKUP_LOG;

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
        "PREG_SPLIT_DELIM_CAPTURE" => Some(Value::Int(2)),
        "SORT_REGULAR" => Some(Value::Int(0)),
        "SORT_NUMERIC" => Some(Value::Int(1)),
        "SORT_STRING" => Some(Value::Int(2)),
        "MYSQLI_REPORT_OFF" => Some(Value::Int(PHP_MYSQLI_REPORT_OFF)),
        "MYSQLI_REPORT_ERROR" => Some(Value::Int(PHP_MYSQLI_REPORT_ERROR)),
        "MYSQLI_REPORT_STRICT" => Some(Value::Int(PHP_MYSQLI_REPORT_STRICT)),
        "MYSQLI_ASSOC" => Some(Value::Int(PHP_MYSQLI_ASSOC)),
        "MYSQLI_NUM" => Some(Value::Int(PHP_MYSQLI_NUM)),
        "MYSQLI_BOTH" => Some(Value::Int(PHP_MYSQLI_BOTH)),
        "MYSQLI_ASYNC" => Some(Value::Int(PHP_MYSQLI_ASYNC)),
        "MYSQLI_CLIENT_SSL" => Some(Value::Int(PHP_MYSQLI_CLIENT_SSL)),
        "MYSQLI_CLIENT_COMPRESS" => Some(Value::Int(PHP_MYSQLI_CLIENT_COMPRESS)),
        "MYSQLI_CLIENT_INTERACTIVE" => Some(Value::Int(PHP_MYSQLI_CLIENT_INTERACTIVE)),
        "MYSQLI_CLIENT_IGNORE_SPACE" => Some(Value::Int(PHP_MYSQLI_CLIENT_IGNORE_SPACE)),
        "MYSQLI_CLIENT_NO_SCHEMA" => Some(Value::Int(PHP_MYSQLI_CLIENT_NO_SCHEMA)),
        "MYSQLI_CLIENT_FOUND_ROWS" => Some(Value::Int(PHP_MYSQLI_CLIENT_FOUND_ROWS)),
        "MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT" => {
            Some(Value::Int(PHP_MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT))
        }
        "MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT" => {
            Some(Value::Int(PHP_MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT))
        }
        "MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS" => {
            Some(Value::Int(PHP_MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS))
        }
        "MYSQLI_OPT_CONNECT_TIMEOUT" => Some(Value::Int(PHP_MYSQLI_OPT_CONNECT_TIMEOUT)),
        "MYSQLI_OPT_LOCAL_INFILE" => Some(Value::Int(PHP_MYSQLI_OPT_LOCAL_INFILE)),
        "MYSQLI_OPT_LOAD_DATA_LOCAL_DIR" => Some(Value::Int(PHP_MYSQLI_OPT_LOAD_DATA_LOCAL_DIR)),
        "MYSQLI_INIT_COMMAND" => Some(Value::Int(PHP_MYSQLI_INIT_COMMAND)),
        "MYSQLI_OPT_READ_TIMEOUT" => Some(Value::Int(PHP_MYSQLI_OPT_READ_TIMEOUT)),
        "MYSQLI_OPT_NET_CMD_BUFFER_SIZE" => Some(Value::Int(PHP_MYSQLI_OPT_NET_CMD_BUFFER_SIZE)),
        "MYSQLI_OPT_NET_READ_BUFFER_SIZE" => Some(Value::Int(PHP_MYSQLI_OPT_NET_READ_BUFFER_SIZE)),
        "MYSQLI_OPT_INT_AND_FLOAT_NATIVE" => Some(Value::Int(PHP_MYSQLI_OPT_INT_AND_FLOAT_NATIVE)),
        "MYSQLI_OPT_SSL_VERIFY_SERVER_CERT" => {
            Some(Value::Int(PHP_MYSQLI_OPT_SSL_VERIFY_SERVER_CERT))
        }
        "MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS" => {
            Some(Value::Int(PHP_MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS))
        }
        "MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH" => {
            Some(Value::Int(PHP_MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH))
        }
        "MYSQLI_STMT_ATTR_CURSOR_TYPE" => Some(Value::Int(PHP_MYSQLI_STMT_ATTR_CURSOR_TYPE)),
        "MYSQLI_STMT_ATTR_PREFETCH_ROWS" => Some(Value::Int(PHP_MYSQLI_STMT_ATTR_PREFETCH_ROWS)),
        "MYSQLI_CURSOR_TYPE_NO_CURSOR" => Some(Value::Int(PHP_MYSQLI_CURSOR_TYPE_NO_CURSOR)),
        "MYSQLI_CURSOR_TYPE_READ_ONLY" => Some(Value::Int(PHP_MYSQLI_CURSOR_TYPE_READ_ONLY)),
        "MYSQLI_CURSOR_TYPE_FOR_UPDATE" => Some(Value::Int(PHP_MYSQLI_CURSOR_TYPE_FOR_UPDATE)),
        "MYSQLI_CURSOR_TYPE_SCROLLABLE" => Some(Value::Int(PHP_MYSQLI_CURSOR_TYPE_SCROLLABLE)),
        "MYSQLI_REFRESH_GRANT" => Some(Value::Int(PHP_MYSQLI_REFRESH_GRANT)),
        "MYSQLI_REFRESH_LOG" => Some(Value::Int(PHP_MYSQLI_REFRESH_LOG)),
        "MYSQLI_REFRESH_TABLES" => Some(Value::Int(PHP_MYSQLI_REFRESH_TABLES)),
        "MYSQLI_REFRESH_HOSTS" => Some(Value::Int(PHP_MYSQLI_REFRESH_HOSTS)),
        "MYSQLI_REFRESH_STATUS" => Some(Value::Int(PHP_MYSQLI_REFRESH_STATUS)),
        "MYSQLI_REFRESH_THREADS" => Some(Value::Int(PHP_MYSQLI_REFRESH_THREADS)),
        "MYSQLI_REFRESH_SLAVE" => Some(Value::Int(PHP_MYSQLI_REFRESH_SLAVE)),
        "MYSQLI_REFRESH_REPLICA" => Some(Value::Int(PHP_MYSQLI_REFRESH_SLAVE)),
        "MYSQLI_REFRESH_MASTER" => Some(Value::Int(PHP_MYSQLI_REFRESH_MASTER)),
        "MYSQLI_REFRESH_BACKUP_LOG" => Some(Value::Int(PHP_MYSQLI_REFRESH_BACKUP_LOG)),
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
            .find_map(|entry| unsupported_runtime_constant_value_type(entry.value())),
        Value::Object(_) => Some("object"),
        Value::Closure(_) => Some("closure"),
    }
}

fn is_compat_loaded_extension_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "json" | "hash" | "pdo" | "pdo_mysql"
    )
}

fn mysql_escape_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\0' => escaped.push_str("\\0"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            '"' => escaped.push_str("\\\""),
            '\u{001A}' => escaped.push_str("\\Z"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn mysqli_row_value_lengths(row: &[(String, Value)], span: Span) -> CompileResult<Vec<usize>> {
    row.iter()
        .map(|(_, value)| {
            value
                .try_echo_string()
                .map(|value| value.len())
                .map_err(|error| runtime_error(span, error))
        })
        .collect()
}

fn mysqli_field_metadata_properties(name: &str) -> Vec<(String, Value)> {
    let (field_type, length, max_length, charsetnr) = match name {
        "ID" => (3, 20, 1, 63),
        "post_title" => (253, 1020, 23, 45),
        _ => (253, 1024, 0, 45),
    };

    vec![
        ("name".to_string(), Value::String(name.to_string())),
        ("orgname".to_string(), Value::String(name.to_string())),
        ("table".to_string(), Value::String("wp_posts".to_string())),
        (
            "orgtable".to_string(),
            Value::String("wp_posts".to_string()),
        ),
        ("def".to_string(), Value::String(String::new())),
        ("db".to_string(), Value::String("wordpress".to_string())),
        ("catalog".to_string(), Value::String("def".to_string())),
        ("max_length".to_string(), Value::Int(max_length)),
        ("length".to_string(), Value::Int(length)),
        ("charsetnr".to_string(), Value::Int(charsetnr)),
        ("flags".to_string(), Value::Int(0)),
        ("type".to_string(), Value::Int(field_type)),
        ("decimals".to_string(), Value::Int(0)),
    ]
}

fn mysqli_pending_result_for_query(query: &str) -> Option<MysqliPendingResultState> {
    if is_wordpress_empty_result_query(query) || is_wordpress_empty_options_query(query) {
        return Some(MysqliPendingResultState {
            fields: Vec::new(),
            rows: Vec::new(),
        });
    }

    if is_wordpress_seed_post_query(query) {
        return Some(MysqliPendingResultState {
            fields: vec!["ID".to_string(), "post_title".to_string()],
            rows: vec![vec![
                ("ID".to_string(), Value::Int(1)),
                (
                    "post_title".to_string(),
                    Value::String("Hello world placeholder".to_string()),
                ),
            ]],
        });
    }

    None
}

fn mysqli_pending_results_for_multi_statement_query(
    query: &str,
) -> Option<Vec<MysqliMultiResultSlot>> {
    let statements: Vec<_> = query
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .collect();
    if statements.len() < 2 {
        return None;
    }

    statements
        .into_iter()
        .map(mysqli_multi_result_slot_for_query)
        .collect()
}

fn mysqli_multi_result_slot_for_query(query: &str) -> Option<MysqliMultiResultSlot> {
    if is_mysqli_no_result_placeholder_query(query) {
        return Some(MysqliMultiResultSlot::NoResult);
    }

    mysqli_pending_result_for_query(query).map(MysqliMultiResultSlot::Result)
}

enum WordPressOptionsRowFilter {
    All,
    Autoload,
    Names(Vec<String>),
}

fn parse_wordpress_option_insert_query(query: &str) -> Option<(String, String, String)> {
    let query = query.trim();
    let values = query
        .strip_prefix("INSERT INTO wp_options (option_name, option_value, autoload) VALUES (")
        .or_else(|| {
            query.strip_prefix(
                "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (",
            )
        })?
        .strip_suffix(')')?;
    let values = parse_sql_single_quoted_list(values)?;
    if values.len() != 3 {
        return None;
    }
    Some((values[0].clone(), values[1].clone(), values[2].clone()))
}

fn parse_wordpress_option_replace_query(query: &str) -> Option<(String, String, String)> {
    let query = query.trim();
    let values = query
        .strip_prefix("REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (")
        .or_else(|| {
            query.strip_prefix(
                "REPLACE INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (",
            )
        })?
        .strip_suffix(')')?;
    let values = parse_sql_single_quoted_list(values)?;
    if values.len() != 3 {
        return None;
    }
    Some((values[0].clone(), values[1].clone(), values[2].clone()))
}

fn parse_wordpress_option_insert_on_duplicate_query(
    query: &str,
) -> Option<(String, String, String)> {
    let query = query.trim();
    let values = query
        .strip_prefix("INSERT INTO wp_options (option_name, option_value, autoload) VALUES (")
        .and_then(|rest| {
            rest.strip_suffix(
                ") ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload)",
            )
            .or_else(|| {
                rest.strip_suffix(
                    ") ON DUPLICATE KEY UPDATE option_name = VALUES(option_name), option_value = VALUES(option_value), autoload = VALUES(autoload)",
                )
            })
        })
        .or_else(|| {
            query
                .strip_prefix(
                    "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (",
                )
                .and_then(|rest| {
                    rest.strip_suffix(
                        ") ON DUPLICATE KEY UPDATE `option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)",
                    )
                    .or_else(|| {
                        rest.strip_suffix(
                            ") ON DUPLICATE KEY UPDATE `option_name` = VALUES(`option_name`), `option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)",
                        )
                    })
                })
        })?;
    let values = parse_sql_single_quoted_list(values)?;
    if values.len() != 3 {
        return None;
    }
    Some((values[0].clone(), values[1].clone(), values[2].clone()))
}

fn parse_wordpress_option_value_select_query(query: &str) -> Option<String> {
    let query = query.trim();
    let rest = query
        .strip_prefix("SELECT option_value FROM wp_options WHERE option_name = ")
        .or_else(|| {
            query.strip_prefix("SELECT option_value FROM `wp_options` WHERE option_name = ")
        })?;
    let rest = rest.strip_suffix(" LIMIT 1").unwrap_or(rest);
    let values = parse_sql_single_quoted_list(rest)?;
    if values.len() != 1 {
        return None;
    }
    Some(values[0].clone())
}

fn parse_wordpress_option_autoload_select_query(query: &str) -> Option<String> {
    let query = query.trim();
    let rest = query
        .strip_prefix("SELECT autoload FROM wp_options WHERE option_name = ")
        .or_else(|| query.strip_prefix("SELECT autoload FROM `wp_options` WHERE option_name = "))
        .or_else(|| {
            query.strip_prefix("SELECT `autoload` FROM `wp_options` WHERE `option_name` = ")
        })?;
    let rest = rest.strip_suffix(" LIMIT 1").unwrap_or(rest);
    let values = parse_sql_single_quoted_list(rest)?;
    if values.len() != 1 {
        return None;
    }
    Some(values[0].clone())
}

fn parse_wordpress_option_value_autoload_select_query(query: &str) -> Option<String> {
    let query = query.trim();
    let rest = query
        .strip_prefix("SELECT option_value, autoload FROM wp_options WHERE option_name = ")
        .or_else(|| {
            query.strip_prefix(
                "SELECT `option_value`, `autoload` FROM `wp_options` WHERE `option_name` = ",
            )
        })?;
    let rest = rest.strip_suffix(" LIMIT 1")?;
    let values = parse_sql_single_quoted_list(rest)?;
    if values.len() != 1 {
        return None;
    }
    Some(values[0].clone())
}

fn parse_wordpress_option_update_query(query: &str) -> Option<(String, String)> {
    let query = query.trim();
    let rest = query
        .strip_prefix("UPDATE wp_options SET option_value = ")
        .or_else(|| query.strip_prefix("UPDATE `wp_options` SET `option_value` = "))?;
    let (value_sql, where_sql) = rest
        .split_once(" WHERE option_name = ")
        .or_else(|| rest.split_once(" WHERE `option_name` = "))?;
    let values = parse_sql_single_quoted_list(value_sql)?;
    let names = parse_sql_single_quoted_list(where_sql)?;
    if values.len() != 1 || names.len() != 1 {
        return None;
    }
    Some((names[0].clone(), values[0].clone()))
}

fn is_wordpress_option_prepared_update_query(query: &str) -> bool {
    let query = query.trim();
    query == "UPDATE wp_options SET option_value = ? WHERE option_name = ?"
        || query == "UPDATE `wp_options` SET `option_value` = ? WHERE `option_name` = ?"
}

fn is_wordpress_option_prepared_insert_query(query: &str) -> bool {
    let query = query.trim();
    query == "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)"
        || query
            == "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?)"
}

fn is_wordpress_option_prepared_insert_on_duplicate_query(query: &str) -> bool {
    let query = query.trim();
    matches!(
        query,
        "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload)"
            | "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE option_name = VALUES(option_name), option_value = VALUES(option_value), autoload = VALUES(autoload)"
            | "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE `option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)"
            | "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE `option_name` = VALUES(`option_name`), `option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)"
    )
}

fn is_wordpress_option_prepared_replace_query(query: &str) -> bool {
    let query = query.trim();
    query == "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)"
        || query
            == "REPLACE INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?)"
}

fn is_wordpress_option_prepared_delete_query(query: &str) -> bool {
    let query = query.trim();
    query == "DELETE FROM wp_options WHERE option_name = ?"
        || query == "DELETE FROM `wp_options` WHERE `option_name` = ?"
}

fn parse_wordpress_option_delete_query(query: &str) -> Option<String> {
    let query = query.trim();
    let rest = query
        .strip_prefix("DELETE FROM wp_options WHERE option_name = ")
        .or_else(|| query.strip_prefix("DELETE FROM `wp_options` WHERE `option_name` = "))?;
    let names = parse_sql_single_quoted_list(rest)?;
    if names.len() != 1 {
        return None;
    }
    Some(names[0].clone())
}

fn parse_wordpress_options_row_select_query(query: &str) -> Option<WordPressOptionsRowFilter> {
    let query = query.trim();
    let rest = query
        .strip_prefix("SELECT option_name, option_value FROM wp_options")
        .or_else(|| query.strip_prefix("SELECT option_name, option_value FROM `wp_options`"))?;
    let rest = rest.trim_start();
    if rest.is_empty() {
        return Some(WordPressOptionsRowFilter::All);
    }
    if rest == "WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )" {
        return Some(WordPressOptionsRowFilter::Autoload);
    }
    let names = rest
        .strip_prefix("WHERE option_name IN (")
        .or_else(|| rest.strip_prefix("WHERE `option_name` IN ("))?
        .strip_suffix(')')?;
    let names = parse_sql_single_quoted_list(names)?;
    Some(WordPressOptionsRowFilter::Names(names))
}

fn wordpress_option_rows_for_filter(
    options: &HashMap<String, WordPressOptionState>,
    filter: &WordPressOptionsRowFilter,
) -> Vec<Vec<(String, Value)>> {
    let mut names = match filter {
        WordPressOptionsRowFilter::All => {
            let mut names: Vec<_> = options.keys().cloned().collect();
            names.sort();
            names
        }
        WordPressOptionsRowFilter::Autoload => {
            let mut names: Vec<_> = options
                .iter()
                .filter_map(|(name, option)| {
                    is_wordpress_autoload_option_value(&option.autoload).then(|| name.clone())
                })
                .collect();
            names.sort();
            names
        }
        WordPressOptionsRowFilter::Names(names) => names.clone(),
    };
    names.dedup();
    names
        .into_iter()
        .filter_map(|name| {
            options.get(&name).map(|option| {
                vec![
                    ("option_name".to_string(), Value::String(name)),
                    (
                        "option_value".to_string(),
                        Value::String(option.value.clone()),
                    ),
                ]
            })
        })
        .collect()
}

fn is_wordpress_autoload_option_value(value: &str) -> bool {
    matches!(value, "yes" | "on" | "auto-on" | "auto")
}

fn parse_sql_single_quoted_list(input: &str) -> Option<Vec<String>> {
    let mut rest = input.trim();
    let mut values = Vec::new();
    loop {
        let (value, next) = parse_sql_single_quoted_value(rest)?;
        values.push(value);
        rest = next.trim_start();
        if rest.is_empty() {
            return Some(values);
        }
        rest = rest.strip_prefix(',')?.trim_start();
    }
}

fn parse_sql_single_quoted_value(input: &str) -> Option<(String, &str)> {
    let rest = input.strip_prefix('\'')?;
    let mut value = String::new();
    let mut chars = rest.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if ch == '\'' {
            if let Some((_, '\'')) = chars.peek().copied() {
                chars.next();
                value.push('\'');
                continue;
            }
            return Some((value, &rest[index + ch.len_utf8()..]));
        }
        if ch == '\\' {
            let (_, escaped) = chars.next()?;
            value.push(match escaped {
                '0' => '\0',
                '\'' => '\'',
                '"' => '"',
                'b' => '\u{0008}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'Z' => '\u{001A}',
                '\\' => '\\',
                '%' => '%',
                '_' => '_',
                other => other,
            });
            continue;
        }
        value.push(ch);
    }
    None
}

fn mysqli_statement_result_for_query(
    function: &str,
    query: &str,
    span: Span,
) -> CompileResult<Option<MysqliPendingResultState>> {
    mysqli_statement_result_for_query_with_params(function, query, &[], span)
}

fn mysqli_statement_result_for_query_with_params(
    function: &str,
    query: &str,
    params: &[Value],
    span: Span,
) -> CompileResult<Option<MysqliPendingResultState>> {
    if let Some(result) = mysqli_pending_result_for_query(query) {
        return Ok(Some(result));
    }

    if query == "SELECT ID, post_title FROM wp_posts WHERE ID = ?" {
        if matches!(params, [Value::Int(1)])
            || matches!(params, [Value::String(value)] if value == "1")
        {
            return Ok(Some(MysqliPendingResultState {
                fields: vec!["ID".to_string(), "post_title".to_string()],
                rows: vec![vec![
                    ("ID".to_string(), Value::Int(1)),
                    (
                        "post_title".to_string(),
                        Value::String("Hello world placeholder".to_string()),
                    ),
                ]],
            }));
        }
        return Ok(Some(MysqliPendingResultState {
            fields: vec!["ID".to_string(), "post_title".to_string()],
            rows: Vec::new(),
        }));
    }

    if query == "SELECT option_value FROM wp_options WHERE option_name = ?" {
        return Ok(Some(MysqliPendingResultState {
            fields: vec!["option_value".to_string()],
            rows: Vec::new(),
        }));
    }

    if query == "SELECT option_name, option_value FROM wp_options WHERE option_name = ?" {
        return Ok(Some(MysqliPendingResultState {
            fields: vec!["option_name".to_string(), "option_value".to_string()],
            rows: Vec::new(),
        }));
    }

    if is_mysqli_select_query(query) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                "statement result metadata is implemented only for current WordPress placeholder SELECT shapes",
            ),
        ));
    }

    Ok(None)
}

fn validate_mysqli_stmt_bind_param_types(
    types: &str,
    parameter_count: usize,
    span: Span,
) -> CompileResult<()> {
    let type_count = types.chars().count();
    if type_count != parameter_count {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_bind_param()",
                format!(
                    "type string length must match bound parameter count {parameter_count}, got {type_count}"
                ),
            ),
        ));
    }
    for ty in types.chars() {
        if !matches!(ty, 's' | 'i' | 'd' | 'b') {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "mysqli_stmt_bind_param()",
                    format!(
                        "only s, i, d, and b parameter type markers are implemented in the current subset, got {ty}"
                    ),
                ),
            ));
        }
    }
    Ok(())
}

fn validate_mysqli_stmt_bound_parameter_value(value: &Value, span: Span) -> CompileResult<()> {
    validate_mysqli_stmt_parameter_value("mysqli_stmt_bind_param()", value, span)
}

fn validate_mysqli_stmt_parameter_value(
    function: &'static str,
    value: &Value,
    span: Span,
) -> CompileResult<()> {
    if matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
    ) {
        Ok(())
    } else {
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "bound parameter values must be scalar or null in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ))
    }
}

fn mysqli_execute_params_from_value(
    function: &'static str,
    value: &Value,
    span: Span,
) -> CompileResult<Vec<Value>> {
    let Value::Array(array) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "params argument must be null, omitted, or a list array in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };

    if !array.is_list() {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                "params array must be a list in the current subset",
            ),
        ));
    }

    let mut params = Vec::with_capacity(array.len());
    for entry in array.entries() {
        validate_mysqli_stmt_parameter_value(function, entry.value(), span)?;
        params.push(entry.value_cloned());
    }
    Ok(params)
}

fn mysqli_placeholder_param_count(query: &str) -> usize {
    query.chars().filter(|ch| *ch == '?').count()
}

fn is_wordpress_empty_options_query(query: &str) -> bool {
    let query = query.trim();
    if query.starts_with("SHOW FULL COLUMNS FROM ") || query.starts_with("DESCRIBE ") {
        return true;
    }

    if let Some(rest) = query.strip_prefix("SELECT option_name, option_value FROM ") {
        let Some((table, suffix)) = rest.split_once(' ') else {
            return rest.ends_with("options");
        };
        return table.ends_with("options")
            && (suffix == "WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )"
                || (suffix.starts_with("WHERE option_name IN (") && suffix.ends_with(')')));
    }

    for prefix in [
        "SELECT option_value FROM ",
        "SELECT autoload FROM ",
        "SELECT option_name FROM ",
    ] {
        let Some(rest) = query.strip_prefix(prefix) else {
            continue;
        };
        let Some((table, suffix)) = rest.split_once(' ') else {
            continue;
        };
        if table.ends_with("options")
            && suffix.starts_with("WHERE option_name = ")
            && (suffix.ends_with(" LIMIT 1") || !suffix.contains(" LIMIT "))
        {
            return true;
        }
    }

    false
}

fn is_wordpress_charset_setup_query(query: &str) -> bool {
    query == "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'"
}

fn is_wordpress_sql_mode_assignment_query(query: &str) -> bool {
    let Some(modes) = query.strip_prefix("SET SESSION sql_mode=") else {
        return false;
    };
    let Some(modes) = modes
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
    else {
        return false;
    };

    modes.is_empty()
        || modes.split(',').all(|mode| {
            !mode.is_empty()
                && mode
                    .chars()
                    .all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
        })
}

fn is_mysqli_no_result_placeholder_query(query: &str) -> bool {
    is_wordpress_charset_setup_query(query)
        || is_wordpress_sql_mode_assignment_query(query)
        || query == "SELECT @@SESSION.sql_mode"
}

fn is_mysqli_init_command_placeholder_query(query: &str) -> bool {
    is_mysqli_no_result_placeholder_query(query) || query.eq_ignore_ascii_case("SET NAMES utf8mb4")
}

fn is_wordpress_empty_result_query(query: &str) -> bool {
    query == "SELECT * FROM wp_posts WHERE 1 = 0"
}

fn is_wordpress_seed_post_query(query: &str) -> bool {
    query == "SELECT ID, post_title FROM wp_posts WHERE ID = 1"
}

fn is_mysqli_select_query(query: &str) -> bool {
    query
        .trim_start()
        .split_whitespace()
        .next()
        .is_some_and(|keyword| keyword.eq_ignore_ascii_case("SELECT"))
}

fn is_mysqli_mutation_query(query: &str) -> bool {
    query
        .trim_start()
        .split_whitespace()
        .next()
        .is_some_and(|keyword| {
            matches!(
                keyword.to_ascii_uppercase().as_str(),
                "INSERT" | "UPDATE" | "DELETE" | "REPLACE"
            )
        })
}

fn is_mysqli_load_data_local_infile_query(query: &str) -> bool {
    let upper = query.trim_start().to_ascii_uppercase();
    upper.starts_with("LOAD DATA LOCAL INFILE")
}

fn expect_mysqli_handle(function: &str, value: &Value, span: Span) -> CompileResult<()> {
    expect_mysqli_handle_id(function, value, span).map(|_| ())
}

fn expect_mysqli_handle_id(function: &str, value: &Value, span: Span) -> CompileResult<i64> {
    let Value::Object(handle) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli object in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };
    if !handle.class_name().eq_ignore_ascii_case("mysqli") {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli object in the current subset, got {} object",
                    handle.class_name()
                ),
            ),
        ));
    }
    Ok(handle.id())
}

fn expect_mysqli_result_handle(function: &str, value: &Value, span: Span) -> CompileResult<i64> {
    let Value::Object(handle) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli_result object in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };
    if !handle.class_name().eq_ignore_ascii_case("mysqli_result") {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli_result object in the current subset, got {} object",
                    handle.class_name()
                ),
            ),
        ));
    }

    Ok(handle.id())
}

fn expect_mysqli_stmt_handle(function: &str, value: &Value, span: Span) -> CompileResult<i64> {
    let Value::Object(handle) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli_stmt object in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };
    if !handle.class_name().eq_ignore_ascii_case("mysqli_stmt") {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "first argument must be mysqli_stmt object in the current subset, got {} object",
                    handle.class_name()
                ),
            ),
        ));
    }

    Ok(handle.id())
}

fn expect_mysqli_stmt_attribute(function: &str, value: &Value, span: Span) -> CompileResult<i64> {
    let Value::Int(attribute) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "attribute argument must be int in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };

    if matches!(
        *attribute,
        PHP_MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH
            | PHP_MYSQLI_STMT_ATTR_CURSOR_TYPE
            | PHP_MYSQLI_STMT_ATTR_PREFETCH_ROWS
    ) {
        Ok(*attribute)
    } else {
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "only MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH, MYSQLI_STMT_ATTR_CURSOR_TYPE, and MYSQLI_STMT_ATTR_PREFETCH_ROWS are implemented in the current subset, got {attribute}"
                ),
            ),
        ))
    }
}

fn expect_mysqli_stmt_attribute_value(
    function: &str,
    value: &Value,
    span: Span,
) -> CompileResult<i64> {
    match value {
        Value::Int(value) => Ok(*value),
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "value argument must be int or bool in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn mysqli_stmt_attribute_default(attribute: i64) -> i64 {
    match attribute {
        PHP_MYSQLI_STMT_ATTR_CURSOR_TYPE => PHP_MYSQLI_CURSOR_TYPE_NO_CURSOR,
        PHP_MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH | PHP_MYSQLI_STMT_ATTR_PREFETCH_ROWS => 0,
        _ => 0,
    }
}

fn expect_mysqli_stmt_data_seek_offset(value: &Value, span: Span) -> CompileResult<usize> {
    let Value::Int(offset) = value else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_data_seek()",
                format!(
                    "offset argument must be int in the current subset, got {}",
                    value.type_name()
                ),
            ),
        ));
    };
    if *offset < 0 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "mysqli_stmt_data_seek()",
                format!("offset must be non-negative in the current subset, got {offset}"),
            ),
        ));
    }
    Ok(*offset as usize)
}

fn string_builtin_argument(
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

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 0x0f) as usize] as char);
    }
    value
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

fn call_ltrim(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(1..=2).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "ltrim()",
                ArityExpectation::Between { min: 1, max: 2 },
                args.len(),
            ),
        ));
    }

    if matches!(args[0], Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call("ltrim()", "arrays are not supported"),
        ));
    }

    let value = args[0]
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))?;

    if args.len() == 2 {
        if matches!(args[1], Value::Array(_)) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ltrim()",
                    "character mask arrays are not supported",
                ),
            ));
        }

        let mask = args[1]
            .try_echo_string()
            .map_err(|error| runtime_error(span, error))?;
        if mask.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ltrim()",
                    "empty character masks are not implemented in the current subset",
                ),
            ));
        }
        if mask.contains("..") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "ltrim()",
                    "character mask ranges are not implemented in the current subset",
                ),
            ));
        }

        return Ok(Value::String(
            value.trim_start_matches(|ch| mask.contains(ch)).to_string(),
        ));
    }

    Ok(Value::String(
        value
            .trim_start_matches(|ch| matches!(ch, ' ' | '\t' | '\n' | '\r' | '\0' | '\u{000B}'))
            .to_string(),
    ))
}

fn call_rtrim(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(1..=2).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "rtrim()",
                ArityExpectation::Between { min: 1, max: 2 },
                args.len(),
            ),
        ));
    }

    if matches!(args[0], Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call("rtrim()", "arrays are not supported"),
        ));
    }

    let value = args[0]
        .try_echo_string()
        .map_err(|error| runtime_error(span, error))?;

    if args.len() == 2 {
        if matches!(args[1], Value::Array(_)) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "rtrim()",
                    "character mask arrays are not supported",
                ),
            ));
        }

        let mask = args[1]
            .try_echo_string()
            .map_err(|error| runtime_error(span, error))?;
        if mask.is_empty() {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "rtrim()",
                    "empty character masks are not implemented in the current subset",
                ),
            ));
        }
        if mask.contains("..") {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "rtrim()",
                    "character mask ranges are not implemented in the current subset",
                ),
            ));
        }

        return Ok(Value::String(
            value.trim_end_matches(|ch| mask.contains(ch)).to_string(),
        ));
    }

    Ok(Value::String(
        value
            .trim_end_matches(|ch| matches!(ch, ' ' | '\t' | '\n' | '\r' | '\0' | '\u{000B}'))
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

fn call_str_starts_with(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("str_starts_with", args, 2, span)?;

    let haystack = string_contains_argument("str_starts_with()", "haystack", &args[0], span)?;
    let needle = string_contains_argument("str_starts_with()", "needle", &args[1], span)?;

    Ok(Value::Bool(haystack.starts_with(&needle)))
}

fn call_str_ends_with(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("str_ends_with", args, 2, span)?;

    let haystack = string_contains_argument("str_ends_with()", "haystack", &args[0], span)?;
    let needle = string_contains_argument("str_ends_with()", "needle", &args[1], span)?;

    Ok(Value::Bool(haystack.ends_with(&needle)))
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

fn call_substr(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=3).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "substr()",
                ArityExpectation::Between { min: 2, max: 3 },
                args.len(),
            ),
        ));
    }

    let value = string_contains_argument("substr()", "string", &args[0], span)?;
    let offset = match args.get(1) {
        Some(Value::Int(offset)) => *offset,
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "substr()",
                    format!(
                        "offset argument must be int in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => unreachable!("arity checked above"),
    };
    let value_len = value.len() as i64;
    let start = if offset >= 0 {
        offset.min(value_len)
    } else {
        (value_len + offset).max(0)
    };

    let end = match args.get(2) {
        Some(Value::Int(length)) if *length >= 0 => (start + *length).min(value_len),
        Some(Value::Int(length)) => (value_len + *length).max(0),
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "substr()",
                    format!(
                        "length argument must be int in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => value_len,
    };
    let end = end.max(start).min(value_len);
    let bytes = value.as_bytes()[start as usize..end as usize].to_vec();
    let result = String::from_utf8(bytes).map_err(|_| {
        runtime_error(
            span,
            RuntimeError::unsupported_call(
                "substr()",
                "substring byte range must remain valid UTF-8 in the current subset",
            ),
        )
    })?;

    Ok(Value::String(result))
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

    if pattern != "/[^0-9.].*/"
        && pattern != "#/[^/]*$#i"
        && !is_wordpress_redirect_sanitizer_cleanup_pattern(&pattern)
        && !is_wordpress_mail_host_cleanup_pattern(&pattern)
        && !is_wordpress_kses_control_char_cleanup_pattern(&pattern)
        && !is_wordpress_kses_slash_zero_cleanup_pattern(&pattern)
        && !is_wordpress_wpdb_prepare_placeholder_escape_pattern(&pattern)
    {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_replace()",
                "only the WordPress database-version cleanup pattern /[^0-9.].*/, path-tail pattern #/[^/]*$#i, redirect sanitizer cleanup pattern |[^a-z0-9-~+_.?#=&;,/:%!*\\[\\]()@]|i, mail host cleanup pattern #^www\\.#, KSES null cleanup patterns /[\\x00-\\x08\\x0B\\x0C\\x0E-\\x1F]/ and /\\\\+0+/, and wpdb prepare placeholder escape pattern are implemented in the current subset",
            ),
        ));
    }
    if is_wordpress_wpdb_prepare_placeholder_escape_pattern(&pattern) {
        if !is_wordpress_wpdb_prepare_placeholder_escape_replacement(&replacement) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_replace()",
                    "only the WordPress wpdb prepare placeholder replacement %%\\1 is implemented for this pattern",
                ),
            ));
        }

        return Ok(Value::String(
            escape_wordpress_wpdb_prepare_unescaped_percents(&subject),
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

    if pattern == "/[^0-9.].*/" {
        let end = subject
            .bytes()
            .position(|byte| !(byte.is_ascii_digit() || byte == b'.'))
            .unwrap_or(subject.len());
        return Ok(Value::String(subject[..end].to_string()));
    }

    if is_wordpress_redirect_sanitizer_cleanup_pattern(&pattern) {
        let sanitized: String = subject
            .chars()
            .filter(|ch| is_wordpress_redirect_sanitizer_allowed_char(*ch))
            .collect();
        return Ok(Value::String(sanitized));
    }

    if is_wordpress_mail_host_cleanup_pattern(&pattern) {
        return Ok(Value::String(
            subject.strip_prefix("www.").unwrap_or(&subject).to_string(),
        ));
    }

    if is_wordpress_kses_control_char_cleanup_pattern(&pattern) {
        let sanitized: String = subject
            .chars()
            .filter(|ch| !is_wordpress_kses_removed_control_char(*ch))
            .collect();
        return Ok(Value::String(sanitized));
    }

    if is_wordpress_kses_slash_zero_cleanup_pattern(&pattern) {
        return Ok(Value::String(remove_wordpress_kses_slash_zero(&subject)));
    }

    let end = subject.rfind('/').unwrap_or(subject.len());
    Ok(Value::String(subject[..end].to_string()))
}

fn call_preg_split(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(2..=4).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "preg_split()",
                ArityExpectation::Between { min: 2, max: 4 },
                args.len(),
            ),
        ));
    }

    if args.len() != 4 {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_split()",
                "only the WordPress wpdb prepare placeholder extraction pattern with limit -1 and PREG_SPLIT_DELIM_CAPTURE is implemented in the current subset",
            ),
        ));
    }

    let pattern = string_contains_argument("preg_split()", "pattern", &args[0], span)?;
    let subject = string_contains_argument("preg_split()", "subject", &args[1], span)?;

    if !is_wordpress_wpdb_prepare_placeholder_split_pattern(&pattern) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_split()",
                "only the WordPress wpdb prepare placeholder extraction pattern is implemented in the current subset",
            ),
        ));
    }
    if !matches!(args[2], Value::Int(-1)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_split()",
                "only limit -1 is implemented for the WordPress wpdb prepare placeholder extraction pattern",
            ),
        ));
    }
    if !matches!(args[3], Value::Int(2)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "preg_split()",
                "only PREG_SPLIT_DELIM_CAPTURE is implemented for the WordPress wpdb prepare placeholder extraction pattern",
            ),
        ));
    }

    Ok(Value::Array(split_wordpress_wpdb_prepare_placeholders(
        &subject, span,
    )?))
}

fn is_wordpress_redirect_sanitizer_cleanup_pattern(pattern: &str) -> bool {
    pattern == "|[^a-z0-9-~+_.?#=&;,/:%!*\\[\\]()@]|i"
        || pattern == "|[^a-z0-9-~+_.?#=&;,/:%!*[]()@]|i"
}

fn is_wordpress_redirect_sanitizer_allowed_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '-' | '~'
                | '+'
                | '_'
                | '.'
                | '?'
                | '#'
                | '='
                | '&'
                | ';'
                | ','
                | '/'
                | ':'
                | '%'
                | '!'
                | '*'
                | '['
                | ']'
                | '('
                | ')'
                | '@'
        )
}

fn is_wordpress_mail_host_cleanup_pattern(pattern: &str) -> bool {
    pattern == "#^www\\.#" || pattern == "#^www.#"
}

fn is_wordpress_kses_control_char_cleanup_pattern(pattern: &str) -> bool {
    pattern == "/[\\x00-\\x08\\x0B\\x0C\\x0E-\\x1F]/" || pattern == "/[x00-x08x0Bx0Cx0E-x1F]/"
}

fn is_wordpress_kses_removed_control_char(ch: char) -> bool {
    matches!(ch as u32, 0x00..=0x08 | 0x0B | 0x0C | 0x0E..=0x1F)
}

fn is_wordpress_kses_slash_zero_cleanup_pattern(pattern: &str) -> bool {
    pattern == "/\\\\\\\\+0+/" || pattern == "/\\\\+0+/"
}

fn is_wordpress_wpdb_prepare_placeholder_escape_pattern(pattern: &str) -> bool {
    pattern.starts_with("/%(?:%|$|(?!(") && pattern.ends_with(")?[sdfFi]))/")
}

fn is_wordpress_wpdb_prepare_placeholder_split_pattern(pattern: &str) -> bool {
    pattern.starts_with("/(^|[^%]|(?:%%)+)(%(") && pattern.ends_with("[sdfFi])/")
}

fn is_wordpress_wpdb_prepare_placeholder_escape_replacement(replacement: &str) -> bool {
    replacement == "%%\\1" || replacement == "%%\\\\1"
}

fn escape_wordpress_wpdb_prepare_unescaped_percents(subject: &str) -> String {
    let bytes = subject.as_bytes();
    let mut output = String::with_capacity(subject.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            let ch = subject[index..]
                .chars()
                .next()
                .expect("index is kept on a character boundary");
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        if bytes.get(index + 1) == Some(&b'%') {
            output.push_str("%%");
            index += 2;
            continue;
        }

        if is_wordpress_wpdb_prepare_placeholder_suffix(&subject[index + 1..]) {
            output.push('%');
        } else {
            output.push_str("%%");
        }
        index += 1;
    }

    output
}

fn is_wordpress_wpdb_prepare_placeholder_suffix(suffix: &str) -> bool {
    wordpress_wpdb_prepare_placeholder_suffix_len(suffix).is_some()
}

fn wordpress_wpdb_prepare_placeholder_suffix_len(suffix: &str) -> Option<usize> {
    let bytes = suffix.as_bytes();
    let mut index = 0;

    if matches!(bytes.first(), Some(b'1'..=b'9')) {
        let mut digits_end = 1;
        while matches!(bytes.get(digits_end), Some(b'0'..=b'9')) {
            digits_end += 1;
        }
        if bytes.get(digits_end) == Some(&b'$') {
            index = digits_end + 1;
        }
    }

    while matches!(bytes.get(index), Some(b'-' | b'+' | b'0'..=b'9')) {
        index += 1;
    }

    match bytes.get(index) {
        Some(b' ' | b'0') => index += 1,
        Some(b'\'') if bytes.get(index + 1).is_some() => index += 2,
        _ => {}
    }

    while matches!(bytes.get(index), Some(b'-' | b'+' | b'0'..=b'9')) {
        index += 1;
    }

    if bytes.get(index) == Some(&b'.') {
        let precision_start = index + 1;
        let mut precision_end = precision_start;
        while matches!(bytes.get(precision_end), Some(b'0'..=b'9')) {
            precision_end += 1;
        }
        if precision_end == precision_start {
            return None;
        }
        index = precision_end;
    }

    if matches!(bytes.get(index), Some(b's' | b'd' | b'f' | b'F' | b'i')) {
        Some(index + 1)
    } else {
        None
    }
}

fn split_wordpress_wpdb_prepare_placeholders(subject: &str, span: Span) -> CompileResult<PhpArray> {
    let bytes = subject.as_bytes();
    let mut array = PhpArray::new();
    let mut cursor = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }

        let Some(suffix_len) = wordpress_wpdb_prepare_placeholder_suffix_len(&subject[index + 1..])
        else {
            index += 1;
            continue;
        };

        let Some((match_start, delimiter_start)) =
            wordpress_wpdb_prepare_placeholder_delimiter(subject, index)
        else {
            index += 1;
            continue;
        };

        if match_start < cursor {
            index += 1;
            continue;
        }

        append_string_part(&mut array, &subject[cursor..match_start], span)?;
        append_string_part(&mut array, &subject[delimiter_start..index], span)?;
        let placeholder_end = index + 1 + suffix_len;
        append_string_part(&mut array, &subject[index..placeholder_end], span)?;

        cursor = placeholder_end;
        index = placeholder_end;
    }

    append_string_part(&mut array, &subject[cursor..], span)?;
    Ok(array)
}

fn wordpress_wpdb_prepare_placeholder_delimiter(
    subject: &str,
    placeholder_start: usize,
) -> Option<(usize, usize)> {
    if placeholder_start == 0 {
        return Some((0, 0));
    }

    let bytes = subject.as_bytes();
    let mut run_start = placeholder_start;
    while run_start > 0 && bytes[run_start - 1] == b'%' {
        run_start -= 1;
    }
    let percent_run_len = placeholder_start - run_start;
    if percent_run_len >= 2 {
        let capture_len = percent_run_len - (percent_run_len % 2);
        let match_start = placeholder_start - capture_len;
        return Some((match_start, match_start));
    }

    if bytes[placeholder_start - 1] != b'%' {
        let delimiter_start = subject[..placeholder_start]
            .char_indices()
            .last()
            .map(|(index, _)| index)
            .expect("placeholder_start is greater than zero");
        return Some((delimiter_start, delimiter_start));
    }

    None
}

fn append_string_part(array: &mut PhpArray, value: &str, span: Span) -> CompileResult<()> {
    array
        .append(Value::String(value.to_string()))
        .map(|_| ())
        .map_err(|error| runtime_error(span, error))
}

fn remove_wordpress_kses_slash_zero(subject: &str) -> String {
    let mut output = String::with_capacity(subject.len());
    let mut chars = subject.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        let mut slash_count = 1;
        while chars.peek() == Some(&'\\') {
            slash_count += 1;
            chars.next();
        }

        let mut zero_count = 0;
        while chars.peek() == Some(&'0') {
            zero_count += 1;
            chars.next();
        }

        if zero_count == 0 {
            for _ in 0..slash_count {
                output.push('\\');
            }
        }
    }
    output
}

impl Interpreter {
    fn call_preg_replace_callback(&mut self, args: Vec<Value>, span: Span) -> CompileResult<Value> {
        if !(3..=6).contains(&args.len()) {
            return Err(runtime_error(
                span,
                RuntimeError::arity_mismatch(
                    "preg_replace_callback()",
                    ArityExpectation::Between { min: 3, max: 6 },
                    args.len(),
                ),
            ));
        }

        if args.len() > 3 {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_replace_callback()",
                    "limit, count output, and flags arguments are not implemented; pass exactly three arguments in the current subset",
                ),
            ));
        }

        let pattern =
            string_contains_argument("preg_replace_callback()", "pattern", &args[0], span)?;
        let callback =
            string_contains_argument("preg_replace_callback()", "callback", &args[1], span)?;
        let subject =
            string_contains_argument("preg_replace_callback()", "subject", &args[2], span)?;

        if !is_wordpress_sanitize_redirect_utf8_pattern(&pattern) {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_replace_callback()",
                    "only the WordPress wp_sanitize_redirect() UTF-8 sanitizer pattern is implemented in the current subset",
                ),
            ));
        }

        if callback != "_wp_sanitize_utf8_in_redirect" {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "preg_replace_callback()",
                    "only the WordPress _wp_sanitize_utf8_in_redirect string callback is implemented in the current subset",
                ),
            ));
        }

        Ok(Value::String(percent_encode_non_ascii_utf8(&subject)))
    }
}

fn is_wordpress_sanitize_redirect_utf8_pattern(pattern: &str) -> bool {
    let compact: String = pattern.chars().filter(|ch| !ch.is_whitespace()).collect();
    compact.starts_with("/((?:")
        && (compact.contains("[\\xC2-\\xDF][\\x80-\\xBF]")
            || compact.contains("[xC2-xDF][x80-xBF]"))
        && (compact.contains("[\\xF1-\\xF3][\\x80-\\xBF]{3}")
            || compact.contains("[xF1-xF3][x80-xBF]{3}"))
        && compact.contains("){1,40}")
        && compact.ends_with(")/x")
}

fn percent_encode_non_ascii_utf8(subject: &str) -> String {
    let mut output = String::with_capacity(subject.len());
    for ch in subject.chars() {
        if ch.is_ascii() {
            output.push(ch);
        } else {
            for byte in ch.to_string().as_bytes() {
                output.push('%');
                output.push_str(&format!("{byte:02X}"));
            }
        }
    }
    output
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
    WordPressTablePrefixInvalidChar,
    WordPressSafeCollationReadQuery,
    WordPressDdlQuery,
    WordPressDmlQuery,
    WordPressInsertReplaceQuery,
    WordPressNonAsciiByte,
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
        if pattern == "|[^a-z0-9_]|i" {
            return Ok(Self::WordPressTablePrefixInvalidChar);
        }
        if pattern == "/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\\s/i"
            || pattern == "/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)s/i"
        {
            return Ok(Self::WordPressSafeCollationReadQuery);
        }
        if pattern == "/^\\s*(create|alter|truncate|drop)\\s/i"
            || pattern == "/^s*(create|alter|truncate|drop)s/i"
        {
            return Ok(Self::WordPressDdlQuery);
        }
        if pattern == "/^\\s*(insert|delete|update|replace)\\s/i"
            || pattern == "/^s*(insert|delete|update|replace)s/i"
        {
            return Ok(Self::WordPressDmlQuery);
        }
        if pattern == "/^\\s*(insert|replace)\\s/i" || pattern == "/^s*(insert|replace)s/i" {
            return Ok(Self::WordPressInsertReplaceQuery);
        }
        if pattern == "/[^\\x00-\\x7F]/" || pattern == "/[^x00-x7F]/" {
            return Ok(Self::WordPressNonAsciiByte);
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
            Self::WordPressDbHostIpv4
            | Self::WordPressDbHostIpv6
            | Self::WordPressTablePrefixInvalidChar
            | Self::WordPressSafeCollationReadQuery
            | Self::WordPressDdlQuery
            | Self::WordPressDmlQuery
            | Self::WordPressInsertReplaceQuery
            | Self::WordPressNonAsciiByte => self.captures(subject).is_some(),
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
            Self::WordPressTablePrefixInvalidChar => wordpress_table_prefix_invalid_char(subject),
            Self::WordPressSafeCollationReadQuery => wordpress_safe_collation_read_query(subject),
            Self::WordPressDdlQuery => {
                wordpress_query_classifier(subject, &["CREATE", "ALTER", "TRUNCATE", "DROP"], true)
            }
            Self::WordPressDmlQuery => wordpress_query_classifier(
                subject,
                &["INSERT", "DELETE", "UPDATE", "REPLACE"],
                true,
            ),
            Self::WordPressInsertReplaceQuery => {
                wordpress_query_classifier(subject, &["INSERT", "REPLACE"], true)
            }
            Self::WordPressNonAsciiByte => wordpress_non_ascii_byte(subject),
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

fn wordpress_table_prefix_invalid_char(subject: &str) -> Option<PhpArray> {
    let invalid = subject
        .chars()
        .find(|ch| !matches!(ch, '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'))?;
    Some(preg_match_single_capture(&invalid.to_string()))
}

fn wordpress_safe_collation_read_query(subject: &str) -> Option<PhpArray> {
    wordpress_query_classifier(
        subject,
        &["SHOW", "DESCRIBE", "DESC", "EXPLAIN", "CREATE"],
        false,
    )
}

fn wordpress_query_classifier(
    subject: &str,
    keywords: &[&str],
    allow_leading_whitespace: bool,
) -> Option<PhpArray> {
    let leading_len = if allow_leading_whitespace {
        subject
            .char_indices()
            .find(|(_, ch)| !ch.is_ascii_whitespace())
            .map(|(index, _)| index)
            .unwrap_or(subject.len())
    } else {
        0
    };
    let candidate_subject = &subject[leading_len..];

    for keyword in keywords {
        let Some(candidate) = candidate_subject.get(..keyword.len()) else {
            continue;
        };
        if !candidate.eq_ignore_ascii_case(keyword) {
            continue;
        }
        let Some(rest) = candidate_subject.get(keyword.len()..) else {
            continue;
        };
        let Some(whitespace) = rest.chars().next() else {
            continue;
        };
        if whitespace.is_ascii_whitespace() {
            let end = leading_len + keyword.len() + whitespace.len_utf8();
            return Some(preg_match_single_capture(&subject[..end]));
        }
    }

    None
}

fn wordpress_non_ascii_byte(subject: &str) -> Option<PhpArray> {
    let non_ascii = subject.chars().find(|ch| !ch.is_ascii())?;
    Some(preg_match_single_capture(&non_ascii.to_string()))
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
                "count output requires a direct str_replace() call with a direct variable in the current subset",
            ),
        ));
    }

    let (result, _) = str_replace_scalar_result(&args[0], &args[1], &args[2], span)?;

    Ok(Value::String(result))
}

fn str_replace_scalar_result(
    search: &Value,
    replace: &Value,
    subject: &Value,
    span: Span,
) -> CompileResult<(String, i64)> {
    let search_values = string_replace_search_values(search, span)?;
    if matches!(replace, Value::Array(_)) {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "str_replace()",
                "replacement argument arrays are not implemented in the current subset",
            ),
        ));
    }
    let replace = string_replace_argument("str_replace()", "replace", replace, span)?;
    let subject = string_replace_argument("str_replace()", "subject", subject, span)?;

    let mut result = subject;
    let mut total_count = 0;
    for search in search_values {
        if search.is_empty() {
            continue;
        }

        let count = result.matches(&search).count() as i64;
        if count > 0 {
            result = result.replace(&search, &replace);
            total_count += count;
        }
    }

    Ok((result, total_count))
}

fn string_replace_search_values(search: &Value, span: Span) -> CompileResult<Vec<String>> {
    match search {
        Value::Array(array) => {
            let mut values = Vec::with_capacity(array.len());
            for entry in array.entries() {
                match entry.value() {
                    Value::Array(_) => {
                        return Err(runtime_error(
                            span,
                            RuntimeError::unsupported_call(
                                "str_replace()",
                                "search array values must be null, bool, int, float, or string in the current subset, got array",
                            ),
                        ));
                    }
                    value => {
                        values.push(string_replace_argument(
                            "str_replace()",
                            "search array value",
                            value,
                            span,
                        )?);
                    }
                }
            }
            Ok(values)
        }
        value => Ok(vec![string_replace_argument(
            "str_replace()",
            "search",
            value,
            span,
        )?]),
    }
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

    bounded_sprintf("sprintf()", format, &args[1..], span).map(Value::String)
}

fn call_vsprintf(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("vsprintf()", args, 2, span)?;

    let format = match &args[0] {
        Value::String(value) => value,
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "vsprintf()",
                    format!(
                        "format argument must be string in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    };

    let Value::Array(array) = &args[1] else {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "vsprintf()",
                format!(
                    "values argument must be array in the current subset, got {}",
                    args[1].type_name()
                ),
            ),
        ));
    };

    let values = array
        .entries()
        .iter()
        .map(|entry| entry.value_cloned())
        .collect::<Vec<_>>();
    bounded_sprintf("vsprintf()", format, &values, span).map(Value::String)
}

#[derive(Debug, Clone, Copy)]
struct SprintfPlaceholder {
    arg_index: usize,
    positional: bool,
    kind: SprintfPlaceholderKind,
    width: Option<usize>,
    precision: Option<usize>,
    left_align: bool,
    show_plus: bool,
    pad: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SprintfPlaceholderKind {
    String,
    Int,
    Float,
}

fn bounded_sprintf(
    function: &'static str,
    format: &str,
    args: &[Value],
    span: Span,
) -> CompileResult<String> {
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
        let (placeholder, next_index) = parse_sprintf_placeholder(format, index, next_arg)
            .ok_or_else(|| {
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
                runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        function,
                        format!(
                            "unsupported format placeholder {} in the current subset",
                            &format[placeholder_start..placeholder_end.min(bytes.len())]
                        ),
                    ),
                )
            })?;

        if !placeholder.positional {
            next_arg += 1;
        }
        index = next_index;

        let Some(value) = args.get(placeholder.arg_index) else {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    format!(
                        "missing argument for placeholder {}",
                        placeholder.arg_index + 1
                    ),
                ),
            ));
        };
        output.push_str(&format_sprintf_value(function, &placeholder, value, span)?);
    }

    Ok(output)
}

fn parse_sprintf_placeholder(
    format: &str,
    mut index: usize,
    next_arg: usize,
) -> Option<(SprintfPlaceholder, usize)> {
    let bytes = format.as_bytes();
    let digits_start = index;
    while matches!(bytes.get(index), Some(b'0'..=b'9')) {
        index += 1;
    }

    let positional = if index > digits_start && bytes.get(index) == Some(&b'$') {
        let position = format[digits_start..index].parse::<usize>().ok()?;
        index += 1;
        Some(position.checked_sub(1)?)
    } else {
        index = digits_start;
        None
    };

    let mut width = None;
    let mut precision = None;
    let mut left_align = false;
    let mut show_plus = false;
    let mut pad = ' ';

    while index < bytes.len() {
        match bytes[index] {
            b'-' => {
                left_align = true;
                index += 1;
            }
            b'+' => {
                show_plus = true;
                index += 1;
            }
            b' ' => {
                index += 1;
            }
            b'0' if width.is_none() => {
                pad = '0';
                let width_start = index;
                while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                    index += 1;
                }
                width = format[width_start..index].parse::<usize>().ok();
            }
            b'1'..=b'9' => {
                let width_start = index;
                while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                    index += 1;
                }
                width = format[width_start..index].parse::<usize>().ok();
            }
            b'\'' => {
                index += 1;
                let ch = format[index..].chars().next()?;
                pad = ch;
                index += ch.len_utf8();
            }
            b'.' => {
                index += 1;
                let precision_start = index;
                while matches!(bytes.get(index), Some(b'0'..=b'9')) {
                    index += 1;
                }
                if precision_start == index {
                    return None;
                }
                precision = format[precision_start..index].parse::<usize>().ok();
            }
            b's' | b'd' | b'f' | b'F' => break,
            _ => return None,
        }
    }

    let kind = match bytes.get(index)? {
        b's' => SprintfPlaceholderKind::String,
        b'd' => SprintfPlaceholderKind::Int,
        b'f' | b'F' => SprintfPlaceholderKind::Float,
        _ => return None,
    };

    Some((
        SprintfPlaceholder {
            arg_index: positional.unwrap_or(next_arg),
            positional: positional.is_some(),
            kind,
            width,
            precision,
            left_align,
            show_plus,
            pad,
        },
        index + 1,
    ))
}

fn format_sprintf_value(
    function: &'static str,
    placeholder: &SprintfPlaceholder,
    value: &Value,
    span: Span,
) -> CompileResult<String> {
    let formatted = match placeholder.kind {
        SprintfPlaceholderKind::String => {
            let value = value
                .try_echo_string()
                .map_err(|error| runtime_error(span, error))?;
            if let Some(precision) = placeholder.precision {
                value.chars().take(precision).collect()
            } else {
                value
            }
        }
        SprintfPlaceholderKind::Int => {
            let value = sprintf_int_argument(function, value, span)?;
            if placeholder.show_plus && value >= 0 {
                format!("+{value}")
            } else {
                value.to_string()
            }
        }
        SprintfPlaceholderKind::Float => {
            let value = sprintf_float_argument(function, value, span)?;
            let precision = placeholder.precision.unwrap_or(6);
            if placeholder.show_plus && value >= 0.0 {
                format!("+{value:.precision$}")
            } else {
                format!("{value:.precision$}")
            }
        }
    };

    Ok(apply_sprintf_width(
        formatted,
        placeholder.width,
        placeholder.left_align,
        placeholder.pad,
    ))
}

fn apply_sprintf_width(value: String, width: Option<usize>, left_align: bool, pad: char) -> String {
    let Some(width) = width else {
        return value;
    };
    let len = value.chars().count();
    if len >= width {
        return value;
    }

    let padding = pad.to_string().repeat(width - len);
    if left_align {
        format!("{value}{padding}")
    } else if pad == '0' && (value.starts_with('-') || value.starts_with('+')) {
        format!("{}{}{}", &value[..1], padding, &value[1..])
    } else {
        format!("{padding}{value}")
    }
}

fn sprintf_int_argument(function: &'static str, value: &Value, span: Span) -> CompileResult<i64> {
    match value {
        Value::Null => Ok(0),
        Value::Bool(value) => Ok(i64::from(*value)),
        Value::Int(value) => Ok(*value),
        Value::Float(value) if value.is_finite() => Ok(*value as i64),
        Value::String(value) => parse_sprintf_numeric_string(value)
            .map(|value| value as i64)
            .ok_or_else(|| {
                runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        function,
                        "numeric placeholders require numeric scalar arguments in the current subset",
                    ),
                )
            }),
        other => Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                format!(
                    "numeric placeholders require numeric scalar arguments in the current subset, got {}",
                    other.type_name()
                ),
            ),
        )),
    }
}

fn sprintf_float_argument(function: &'static str, value: &Value, span: Span) -> CompileResult<f64> {
    let value = match value {
        Value::Null => 0.0,
        Value::Bool(value) => {
            if *value {
                1.0
            } else {
                0.0
            }
        }
        Value::Int(value) => *value as f64,
        Value::Float(value) => *value,
        Value::String(value) => parse_sprintf_numeric_string(value).ok_or_else(|| {
            runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    "numeric placeholders require numeric scalar arguments in the current subset",
                ),
            )
        })?,
        other => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    function,
                    format!(
                        "numeric placeholders require numeric scalar arguments in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    };

    if value.is_finite() {
        Ok(value)
    } else {
        Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                function,
                "numeric placeholders require finite numeric arguments in the current subset",
            ),
        ))
    }
}

fn parse_sprintf_numeric_string(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    value.parse::<f64>().ok().filter(|value| value.is_finite())
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
        match entry.value() {
            Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                parts.push(entry.value().echo_string());
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

fn header_name(header: &str) -> Option<&str> {
    let (name, _) = header.split_once(':')?;
    Some(name)
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

fn call_php_sapi_name(args: &[Value], span: Span) -> CompileResult<Value> {
    expect_arity("php_sapi_name", args, 0, span)?;
    Ok(Value::String("cli".to_string()))
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

fn call_rand(args: &[Value], span: Span) -> CompileResult<Value> {
    if !args.is_empty() {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "rand()",
                "min/max arguments are not implemented; call rand() without arguments in the current subset",
            ),
        ));
    }

    Ok(Value::Int(123456789))
}

fn call_uniqid(args: &[Value], span: Span) -> CompileResult<Value> {
    if args.len() > 2 {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "uniqid()",
                ArityExpectation::Between { min: 0, max: 2 },
                args.len(),
            ),
        ));
    }

    let prefix = match args.first() {
        Some(value) => string_builtin_argument("uniqid()", "prefix", value, span)?,
        None => String::new(),
    };

    let more_entropy = match args.get(1) {
        Some(Value::Bool(value)) => *value,
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "uniqid()",
                    format!(
                        "more_entropy argument must be bool in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
        None => false,
    };

    let mut value = format!("{prefix}0000000000000");
    if more_entropy {
        value.push_str(".00000000");
    }
    Ok(Value::String(value))
}

fn call_hash_hmac(args: &[Value], span: Span) -> CompileResult<Value> {
    if !(3..=4).contains(&args.len()) {
        return Err(runtime_error(
            span,
            RuntimeError::arity_mismatch(
                "hash_hmac()",
                ArityExpectation::Between { min: 3, max: 4 },
                args.len(),
            ),
        ));
    }

    let algorithm = string_builtin_argument("hash_hmac()", "algorithm", &args[0], span)?;
    if !algorithm.eq_ignore_ascii_case("sha256") {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                "hash_hmac()",
                "only sha256 is implemented in the current subset",
            ),
        ));
    }

    let data = string_builtin_argument("hash_hmac()", "data", &args[1], span)?;
    let key = string_builtin_argument("hash_hmac()", "key", &args[2], span)?;

    match args.get(3) {
        Some(Value::Bool(false)) | None => {}
        Some(Value::Bool(true)) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "hash_hmac()",
                    "raw binary output is not implemented; omit raw_output or pass false in the current subset",
                ),
            ));
        }
        Some(other) => {
            return Err(runtime_error(
                span,
                RuntimeError::unsupported_call(
                    "hash_hmac()",
                    format!(
                        "raw_output argument must be bool in the current subset, got {}",
                        other.type_name()
                    ),
                ),
            ));
        }
    }

    let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes()).map_err(|_| {
        runtime_error(
            span,
            RuntimeError::unsupported_call(
                "hash_hmac()",
                "key values outside the current HMAC-SHA256 subset are not implemented",
            ),
        )
    })?;
    mac.update(data.as_bytes());
    Ok(Value::String(hex_bytes(&mac.finalize().into_bytes())))
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

impl Interpreter {
    fn call_ini_get(&self, args: &[Value], span: Span) -> CompileResult<Value> {
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

        Ok(self
            .ini_value(name)
            .map(Value::String)
            .unwrap_or(Value::Bool(false)))
    }

    fn call_ini_set(&mut self, args: &[Value], span: Span) -> CompileResult<Value> {
        expect_arity("ini_set", args, 2, span)?;

        let name = match &args[0] {
            Value::String(name) => name,
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "ini_set()",
                        format!(
                            "option argument must be string in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };

        let Some(previous) = self.ini_value(name) else {
            return Ok(Value::Bool(false));
        };

        let value = match &args[1] {
            Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_) => {
                args[1].echo_string()
            }
            other => {
                return Err(runtime_error(
                    span,
                    RuntimeError::unsupported_call(
                        "ini_set()",
                        format!(
                            "value argument must be null or scalar in the current subset, got {}",
                            other.type_name()
                        ),
                    ),
                ));
            }
        };

        self.ini_values.insert(normalize_ini_name(name), value);
        Ok(Value::String(previous))
    }

    fn ini_value(&self, name: &str) -> Option<String> {
        let normalized = normalize_ini_name(name);
        self.ini_values
            .get(&normalized)
            .cloned()
            .or_else(|| compat_ini_value(&normalized).map(str::to_string))
    }
}

fn normalize_ini_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn compat_ini_value(normalized_name: &str) -> Option<&'static str> {
    match normalized_name {
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

fn ensure_supported_function_signature(
    function: &FunctionDecl,
    actual: usize,
    span: Span,
) -> CompileResult<()> {
    ensure_supported_function_metadata(function, span)?;

    if function
        .params
        .iter()
        .enumerate()
        .any(|(index, param)| param.by_reference && (index < actual || param.is_variadic))
    {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "reference parameter invocation is not implemented",
            ),
        ));
    }

    Ok(())
}

fn ensure_supported_function_metadata(function: &FunctionDecl, span: Span) -> CompileResult<()> {
    if function.returns_by_reference {
        return Err(runtime_error(
            span,
            RuntimeError::unsupported_call(
                callable_name(&function.name),
                "reference returns are not implemented",
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

fn ensure_supported_reference_return_function_metadata(
    function: &FunctionDecl,
    span: Span,
) -> CompileResult<()> {
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

fn mysqli_stmt_execute_function_label(key: &str) -> Option<&'static str> {
    match key {
        "mysqli_stmt_execute" => Some("mysqli_stmt_execute()"),
        "mysqli_execute" => Some("mysqli_execute()"),
        _ => None,
    }
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
                output.push_str(&format_var_dump_with_indent(entry.value(), indent + 1));
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
        match entry.value() {
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
