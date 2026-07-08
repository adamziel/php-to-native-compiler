use std::collections::{HashMap, HashSet};

use crate::ast::{
    AnonymousFunction as AstAnonymousFunction, ArrayDimTarget as AstArrayDimTarget,
    ArrayElement as AstArrayElement, ArrayElementValue as AstArrayElementValue, AssignmentOp,
    AssignmentTarget as AstAssignmentTarget, AttributeArgumentExpression, AttributeArgumentKind,
    AttributeConstantReference, AttributeMetadata, BinaryOp as AstBinaryOp,
    CastKind as AstCastKind, CatchClause as AstCatchClause, ClassDecl as AstClassDecl,
    ClosureUseCapture as AstClosureUseCapture, CompileWarning as AstCompileWarning,
    CompileWarningKind as AstCompileWarningKind, EnumBackingType as AstEnumBackingType, Expr,
    FunctionDecl as AstFunctionDecl, FunctionParameter as AstFunctionParameter, GlobalTarget,
    IncDecOp as AstIncDecOp, IncDecResult as AstIncDecResult, IncDecTarget as AstIncDecTarget,
    IncludeKind as AstIncludeKind, InstanceOfTarget as AstInstanceOfTarget,
    ListAssignmentElement as AstListAssignmentElement,
    ListAssignmentElementTarget as AstListAssignmentElementTarget,
    ListAssignmentTarget as AstListAssignmentTarget,
    ListExprElementTarget as AstListExprElementTarget, MagicConstantKind as AstMagicConstantKind,
    MatchArm as AstMatchArm, Program, PropertyTypeKind as AstPropertyTypeKind,
    PropertyVisibility as AstPropertyVisibility, ReferenceTarget as AstReferenceTarget, Statement,
    StringInterpolationIndex as AstStringInterpolationIndex, StringPart as AstStringPart,
    TypeHint as AstTypeHint, UnaryOp as AstUnaryOp, UnsetTarget as AstUnsetTarget,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub classes: Vec<ClassDecl>,
    pub traits: Vec<TraitDecl>,
    pub functions: Vec<FunctionDecl>,
    pub includes: Vec<IncludeFile>,
    pub preload_include_indices: Vec<usize>,
    pub instructions: Vec<Instruction>,
    pub compile_warnings: Vec<CompileWarning>,
    pub source_file: String,
    pub source_dir: String,
    pub source_bytes: Vec<u8>,
    pub strict_types: bool,
    pub ticks: bool,
    pub runtime_requirements: ModuleRuntimeRequirements,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModuleRuntimeRequirements {
    pub internal_function_dispatch: bool,
    pub zend_test_observer_execute_internal: bool,
    pub zend_test_observer_show_return_value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileWarning {
    pub message: String,
    pub line: usize,
    pub kind: CompileWarningKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileWarningKind {
    Warning,
    UncaughtError,
    Deprecation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeFile {
    pub source_file: String,
    pub source_dir: String,
    pub source_bytes: Vec<u8>,
    pub path_aliases: Vec<String>,
    pub parse_error: Option<IncludeParseError>,
    pub strict_types: bool,
    pub ticks: bool,
    pub instructions: Vec<Instruction>,
    pub compile_warnings: Vec<CompileWarning>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeSource {
    pub source_file: String,
    pub source_dir: String,
    pub source_bytes: Vec<u8>,
    pub path_aliases: Vec<String>,
    pub parse_error: Option<IncludeParseError>,
    pub program: Program,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeParseError {
    pub message: String,
    pub line: usize,
}

pub type IncludeResolutionMap = HashMap<(String, usize, usize), Vec<usize>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub source_file: String,
    pub parent_name: Option<String>,
    pub interfaces: Vec<String>,
    pub trait_uses: Vec<TraitUseDecl>,
    pub declaration_fatals: Vec<ClassDeclarationFatal>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub line: usize,
    pub end_line: usize,
    pub initially_declared: bool,
    pub is_conditionally_declared: bool,
    pub is_syntactically_conditionally_declared: bool,
    pub is_anonymous: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_interface: bool,
    pub is_readonly: bool,
    pub is_enum: bool,
    pub enum_backing_type: Option<EnumBackingType>,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub constants: Vec<ClassConstantDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDeclarationFatal {
    pub message: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassNameEntry {
    source_file: String,
    name: String,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumBackingType {
    Int,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub source_file: String,
    pub trait_uses: Vec<TraitUseDecl>,
    pub attributes: AttributeMetadata,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub doc_comment: Option<String>,
    pub line: usize,
    pub end_line: usize,
    pub initially_declared: bool,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub constants: Vec<ClassConstantDecl>,
    pub methods: Vec<TraitMethodDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitUseDecl {
    pub name: String,
    pub aliases: Vec<TraitAliasDecl>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitAliasDecl {
    pub trait_name: Option<String>,
    pub method_name: String,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDecl {
    pub name: String,
    pub function_index: usize,
    pub visibility: PropertyVisibility,
    pub attributes: AttributeMetadata,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub parameters: Vec<FunctionParameter>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub declaring_class_name: String,
    pub trait_name: Option<String>,
    pub visibility: PropertyVisibility,
    pub set_visibility: PropertyVisibility,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_readonly: bool,
    pub is_promoted: bool,
    pub has_hooks: bool,
    pub is_virtual: bool,
    pub hook_has_get: bool,
    pub hook_has_set: bool,
    pub hook_get_is_final: bool,
    pub hook_get_is_abstract: bool,
    pub hook_get_returns_by_ref: bool,
    pub hook_set_is_final: bool,
    pub hook_set_is_abstract: bool,
    pub hook_get_attributes: AttributeMetadata,
    pub hook_set_attributes: AttributeMetadata,
    pub hook_get_doc_comment: Option<String>,
    pub hook_set_doc_comment: Option<String>,
    pub hook_get_line: usize,
    pub hook_set_line: usize,
    pub hook_get_value: Option<ValueExpr>,
    pub hook_set_value: Option<ValueExpr>,
    pub hook_get_body: Option<Vec<Instruction>>,
    pub hook_set_body: Option<Vec<Instruction>>,
    pub hook_set_parameter_name: Option<String>,
    pub hook_set_parameter_attributes: AttributeMetadata,
    pub hook_set_parameter_type: Option<TypeHint>,
    pub hook_set_parameter_doc_comment: Option<String>,
    pub type_hint: Option<PropertyTypeHint>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub value: Option<ValueExpr>,
    pub line: usize,
    pub source_order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyVisibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticPropertyDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub set_visibility: PropertyVisibility,
    pub is_final: bool,
    pub type_hint: Option<PropertyTypeHint>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub value: Option<ValueExpr>,
    pub source_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyTypeHint {
    pub text: String,
    pub kind: PropertyTypeKind,
    pub allows_null: bool,
    pub semantic_type: Option<TypeHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyTypeKind {
    Null,
    Array,
    Int,
    Float,
    String,
    Bool,
    Mixed,
    Object,
    Class(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstantDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub type_hint: Option<TypeHint>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub deprecated_message_dependency: Option<DeprecatedMessageDependency>,
    pub deprecated_message_runtime_reference: Option<AttributeConstantReference>,
    pub is_enum_case: bool,
    pub enum_case_value: Option<ValueExpr>,
    pub is_final: bool,
    pub value: ValueExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedMessageDependency {
    pub subject: String,
    pub message: Option<String>,
    pub since: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub source_file: String,
    pub function_index: usize,
    pub visibility: PropertyVisibility,
    pub has_explicit_visibility: bool,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub display_name: String,
    pub source_file: String,
    pub strict_types: bool,
    pub doc_comment: Option<String>,
    pub class_name: Option<String>,
    pub trait_name: Option<String>,
    pub trait_method_name: Option<String>,
    pub method_name: Option<String>,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub deprecated_message_runtime_reference: Option<AttributeConstantReference>,
    pub no_discard_message: Option<String>,
    pub attributes: AttributeMetadata,
    pub is_static: bool,
    pub line: usize,
    pub end_line: usize,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_generator: bool,
    pub is_anonymous: bool,
    pub initially_declared: bool,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub type_hint: Option<TypeHint>,
    pub by_ref: bool,
    pub is_variadic: bool,
    pub default_value: Option<ValueExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeHint {
    Null,
    Array,
    Callable,
    Int,
    Float,
    String,
    Bool,
    True,
    False,
    Object,
    Iterable,
    Mixed,
    Void,
    Never,
    Static,
    Nullable(Box<TypeHint>),
    Union(Vec<TypeHint>),
    Intersection(Vec<TypeHint>),
    Class(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Store {
        name: String,
        value: ValueExpr,
        line: usize,
    },
    StoreRef {
        name: String,
        source: ValueExpr,
        line: usize,
    },
    StoreArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        value: ValueExpr,
        compound_op: Option<BinaryOp>,
        line: usize,
    },
    StoreArrayDimRef {
        target: ArrayDimTarget,
        source: ValueExpr,
    },
    Increment {
        target: IncDecTarget,
        op: IncDecOp,
        line: usize,
    },
    Tick {
        line: usize,
    },
    TickScope {
        enabled: bool,
        body: Vec<Instruction>,
    },
    UnsetVariable {
        name: String,
    },
    UnsetDynamicVariable {
        name: ValueExpr,
        line: usize,
    },
    BindGlobal {
        name: String,
    },
    BindDynamicGlobal {
        name: ValueExpr,
        line: usize,
    },
    DeclareFunction {
        function_index: usize,
    },
    DeclareTrait {
        trait_index: usize,
    },
    EarlyDeclareClass {
        class_index: usize,
        line: usize,
    },
    DeclareClass {
        class_index: usize,
        line: usize,
        allow_predeclared: bool,
    },
    ValidateClass {
        class_index: usize,
        line: usize,
    },
    BindStatic {
        name: String,
        value: Option<ValueExpr>,
        line: usize,
    },
    UnsetArrayDim {
        array: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetDynamicArrayDim {
        name: ValueExpr,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetValueArrayDim {
        array: ValueExpr,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetPropertyArrayDim {
        receiver: ValueExpr,
        name: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetStaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetDynamicStaticPropertyArrayDim {
        receiver: ValueExpr,
        name: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetDynamicStaticPropertyName {
        class_name: String,
        name: ValueExpr,
        line: usize,
    },
    UnsetProperty {
        receiver: ValueExpr,
        name: String,
        line: usize,
    },
    UnsetDynamicProperty {
        receiver: ValueExpr,
        name: ValueExpr,
        line: usize,
    },
    UnsetStaticProperty {
        class_name: String,
        name: String,
        line: usize,
    },
    DefineConstant {
        name: String,
        attributes: AttributeMetadata,
        value: ValueExpr,
        deprecated_message: Option<String>,
        deprecated_since: Option<String>,
        deprecated_message_dependency: Option<DeprecatedMessageDependency>,
        line: usize,
    },
    Expression(ValueExpr),
    Echo {
        value: ValueExpr,
        line: usize,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        allow_global_fallback: bool,
        line: usize,
    },
    Return {
        value: Option<ValueExpr>,
        line: usize,
    },
    Exit {
        value: Option<ValueExpr>,
        line: usize,
    },
    Throw {
        value: ValueExpr,
        line: usize,
    },
    Try {
        body: Vec<Instruction>,
        catches: Vec<CatchClause>,
        finally_body: Vec<Instruction>,
    },
    Branch {
        condition: ValueExpr,
        then_body: Vec<Instruction>,
        else_body: Vec<Instruction>,
    },
    While {
        condition: ValueExpr,
        body: Vec<Instruction>,
    },
    DoWhile {
        body: Vec<Instruction>,
        condition: ValueExpr,
    },
    For {
        initializers: Vec<Instruction>,
        conditions: Vec<ValueExpr>,
        updates: Vec<Instruction>,
        body: Vec<Instruction>,
    },
    Foreach {
        iterable: ValueExpr,
        key: Option<AssignmentTarget>,
        value: AssignmentTarget,
        value_by_ref: bool,
        body: Vec<Instruction>,
        line: usize,
    },
    Switch {
        expression: ValueExpr,
        cases: Vec<SwitchCase>,
    },
    Break {
        level: usize,
        line: usize,
    },
    Continue {
        level: usize,
        line: usize,
    },
    Label {
        name: String,
    },
    Goto {
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub type_names: Vec<String>,
    pub variable: Option<String>,
    pub body: Vec<Instruction>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<ValueExpr>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Closure {
        function_index: usize,
        captures: Vec<ClosureCapture>,
        line: usize,
    },
    Load {
        name: String,
        line: usize,
    },
    LegacyDollarBraceStringVariable {
        name: String,
        line: usize,
    },
    LegacyDollarBraceExpressionVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    IncDec {
        target: IncDecTarget,
        op: IncDecOp,
        result: IncDecResult,
        line: usize,
    },
    Assign {
        target: AssignmentTarget,
        op: AssignmentOp,
        value: Box<ValueExpr>,
    },
    AssignRef {
        target: AssignmentTarget,
        source: Box<ValueExpr>,
    },
    Constant {
        name: String,
        deprecated_message: Option<String>,
        deprecated_since: Option<String>,
        deprecated_message_dependency: Option<DeprecatedMessageDependency>,
        line: usize,
    },
    MagicConstant {
        kind: MagicConstantKind,
        line: usize,
    },
    Array(Vec<ArrayElement>),
    ArrayAccess {
        array: Box<ValueExpr>,
        index: Box<ValueExpr>,
        line: usize,
    },
    ArrayAppendAccess {
        array: Box<ValueExpr>,
        line: usize,
    },
    Isset {
        targets: Vec<ValueExpr>,
    },
    Empty {
        target: Box<ValueExpr>,
    },
    Print {
        expression: Box<ValueExpr>,
    },
    Include {
        kind: AstIncludeKind,
        path: Box<ValueExpr>,
        candidates: Vec<usize>,
        line: usize,
    },
    OpcacheCompileFile {
        path: Box<ValueExpr>,
        candidates: Vec<usize>,
        line: usize,
    },
    Exit {
        value: Option<Box<ValueExpr>>,
        line: usize,
    },
    Throw {
        value: Box<ValueExpr>,
        line: usize,
    },
    Yield {
        key: Option<Box<ValueExpr>>,
        value: Option<Box<ValueExpr>>,
        line: usize,
    },
    YieldFrom {
        expr: Box<ValueExpr>,
        line: usize,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        allow_global_fallback: bool,
        line: usize,
    },
    FirstClassCallable {
        callable: Box<ValueExpr>,
        line: usize,
    },
    DynamicCall {
        callee: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        static_method_syntax: bool,
        line: usize,
    },
    MethodCall {
        receiver: Box<ValueExpr>,
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        nullsafe: bool,
        line: usize,
    },
    DynamicMethodCall {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    NewObject {
        class_name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    DynamicNewObject {
        class_name: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    Clone {
        expr: Box<ValueExpr>,
        with_properties: Option<Box<ValueExpr>>,
        line: usize,
    },
    PropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    NullsafePropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicPropertyFetch {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        nullsafe: bool,
        line: usize,
    },
    StaticPropertyFetch {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicStaticPropertyNameFetch {
        class_name: String,
        name: Box<ValueExpr>,
        line: usize,
    },
    ParentPropertyHookCall {
        property_name: String,
        hook_name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    DynamicStaticPropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    ClassConstantFetch {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicClassConstantFetch {
        class_name: Option<String>,
        receiver: Option<Box<ValueExpr>>,
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicClassNameFetch {
        receiver: Box<ValueExpr>,
        allow_string: bool,
        line: usize,
    },
    InstanceOf {
        expr: Box<ValueExpr>,
        target: InstanceOfTarget,
        line: usize,
    },
    Unary {
        op: UnaryOp,
        expr: Box<ValueExpr>,
        line: usize,
    },
    Cast {
        kind: CastKind,
        expr: Box<ValueExpr>,
        line: usize,
    },
    Binary {
        op: BinaryOp,
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
        line: usize,
    },
    Ternary {
        condition: Box<ValueExpr>,
        if_true: Option<Box<ValueExpr>>,
        if_false: Box<ValueExpr>,
        line: usize,
    },
    PipeValue {
        expr: Box<ValueExpr>,
        line: usize,
    },
    Match {
        subject: Box<ValueExpr>,
        arms: Vec<MatchArm>,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceOfTarget {
    ClassName(String),
    Expr(Box<ValueExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<ValueExpr>,
    pub value: ValueExpr,
    pub is_default: bool,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureCapture {
    pub name: String,
    pub by_ref: bool,
    pub warn_if_missing: bool,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<ValueExpr>,
    pub value: ArrayElementValue,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementValue {
    Value(ValueExpr),
    Reference(ReferenceTarget),
    Unpack { value: ValueExpr, line: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDimTarget {
    pub array: String,
    pub dimensions: Vec<Option<ValueExpr>>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicArrayDim {
        name: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ValueArrayDim {
        array: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    DynamicPropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    DynamicStaticPropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    DynamicStaticPropertyName {
        class_name: String,
        name: Box<ValueExpr>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicProperty {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
    StaticProperty {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicStaticProperty {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    List(ListAssignmentTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentTarget {
    pub elements: Vec<ListAssignmentElement>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentElement {
    pub key: Option<ValueExpr>,
    pub target: ListAssignmentElementTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListAssignmentElementTarget {
    Value(Box<AssignmentTarget>),
    Reference(ReferenceTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceTarget {
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    ArrayDim(ArrayDimTarget),
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicProperty {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Power,
    Divide,
    Modulo,
    Concat,
    Coalesce,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    Spaceship,
    Identical,
    NotIdentical,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    And,
    Xor,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Positive,
    Negate,
    Not,
    BitwiseNot,
    ErrorSuppress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Int,
    Integer,
    Float,
    Double,
    String,
    Binary,
    Bool,
    Boolean,
    Array,
    Object,
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicConstantKind {
    Line,
    File,
    Dir,
    Function,
    Method,
    Class,
    Trait,
    Namespace,
    Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecOp {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecResult {
    Pre,
    Post,
}

pub fn lower(program: &Program) -> Module {
    lower_with_source(program, String::new(), String::new(), Vec::new())
}

pub fn lower_with_source(
    program: &Program,
    source_file: String,
    source_dir: String,
    source_bytes: Vec<u8>,
) -> Module {
    lower_with_source_and_includes(
        program,
        source_file,
        source_dir,
        source_bytes,
        Vec::new(),
        Vec::new(),
        &IncludeResolutionMap::new(),
    )
}

pub fn lower_with_source_and_includes(
    program: &Program,
    source_file: String,
    source_dir: String,
    source_bytes: Vec<u8>,
    include_sources: Vec<IncludeSource>,
    preload_include_indices: Vec<usize>,
    include_resolutions: &IncludeResolutionMap,
) -> Module {
    let mut context = LoweringContext::new(
        program,
        source_file.clone(),
        source_dir.clone(),
        collect_include_trait_method_sources(&include_sources),
        include_resolutions,
    );
    let include_function_indices = context.declare_include_functions(&include_sources);
    context.declare_include_class_names(&include_sources);
    for function in &program.functions {
        let previous_function_display_name = std::mem::replace(
            &mut context.current_function_display_name,
            Some(function.name.clone()),
        );
        let body = context.lower_statements(&function.body);
        context.current_function_display_name = previous_function_display_name;
        let function_index = context
            .function_index_by_name(&function.name)
            .expect("declared function should have an IR function entry");
        context.functions[function_index].body = body;
    }
    for (include, function_indices) in include_sources.iter().zip(include_function_indices.iter()) {
        context.lower_include_functions(include, function_indices);
    }
    let mut classes: Vec<_> = program
        .classes
        .iter()
        .map(|class| context.lower_class(class))
        .collect();
    let mut traits: Vec<_> = program
        .traits
        .iter()
        .map(|trait_decl| context.lower_trait(trait_decl))
        .collect();
    for include in &include_sources {
        classes.extend(context.lower_include_classes(include));
        traits.extend(context.lower_include_traits(include));
    }
    let includes = include_sources
        .iter()
        .zip(include_function_indices.iter())
        .map(|(include, function_indices)| context.lower_include_source(include, function_indices))
        .collect();
    let instructions = context.lower_statements(&program.statements);
    Module {
        classes,
        traits,
        functions: context.functions,
        includes,
        preload_include_indices,
        instructions,
        compile_warnings: lower_compile_warnings(&program.compile_warnings),
        source_file,
        source_dir,
        source_bytes,
        strict_types: program.strict_types,
        ticks: program.ticks,
        runtime_requirements: ModuleRuntimeRequirements::default(),
    }
}

fn collect_include_trait_method_sources(
    include_sources: &[IncludeSource],
) -> HashMap<(String, String), String> {
    let mut sources = HashMap::new();
    for include in include_sources {
        for trait_decl in &include.program.traits {
            let trait_name = trait_decl.name.to_ascii_lowercase();
            for method in &trait_decl.methods {
                sources
                    .entry((trait_name.clone(), method.name.to_ascii_lowercase()))
                    .or_insert_with(|| include.source_file.clone());
            }
        }
    }
    sources
}

struct LoweringContext<'a> {
    functions: Vec<FunctionDecl>,
    constant_deprecations: HashMap<String, DeprecatedMetadata>,
    constant_values: HashMap<String, String>,
    include_trait_method_sources: HashMap<(String, String), String>,
    source_file: String,
    source_dir: String,
    strict_types: bool,
    ticks_enabled: bool,
    include_resolutions: &'a IncludeResolutionMap,
    class_names: Vec<ClassNameEntry>,
    trait_names: Vec<ClassNameEntry>,
    runtime_class_names: HashSet<(String, String)>,
    early_bound_class_names: HashSet<(String, String)>,
    current_class_name: Option<String>,
    current_class_parent_name: Option<String>,
    current_trait_name: Option<String>,
    current_function_display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeprecatedMetadata {
    message: Option<String>,
    since: Option<String>,
    message_dependency: Option<DeprecatedMessageDependency>,
    message_runtime_reference: Option<AttributeConstantReference>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncDecTarget {
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicArrayDim {
        name: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ValueArrayDim {
        array: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicProperty {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
    StaticProperty {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicStaticPropertyName {
        class_name: String,
        name: Box<ValueExpr>,
        line: usize,
    },
}

fn class_runtime_declaration_key(source_file: &str, name: &str) -> (String, String) {
    (source_file.to_string(), name.to_ascii_lowercase())
}

fn ast_class_source_file(default_source_file: &str, source_file: Option<&str>) -> String {
    match source_file {
        Some("__PTN_EVAL_CODE__") => format!("{default_source_file} : eval()'d code"),
        Some(source_file) if source_file.starts_with("__PTN_EVAL_CODE__:") => {
            let line = source_file
                .strip_prefix("__PTN_EVAL_CODE__:")
                .expect("checked eval source prefix");
            format!("{default_source_file}({line}) : eval()'d code")
        }
        Some(source_file) => source_file.to_string(),
        None => default_source_file.to_string(),
    }
}

impl<'a> LoweringContext<'a> {
    fn new(
        program: &Program,
        source_file: String,
        source_dir: String,
        include_trait_method_sources: HashMap<(String, String), String>,
        include_resolutions: &'a IncludeResolutionMap,
    ) -> Self {
        let constant_values = collect_constant_values(program);
        let constant_deprecations = collect_constant_deprecations(program, &constant_values);
        let class_names = program
            .classes
            .iter()
            .map(|class| ClassNameEntry {
                source_file: source_file.clone(),
                name: class.name.clone(),
                line: class.span.line,
            })
            .collect();
        let trait_names = program
            .traits
            .iter()
            .map(|trait_decl| ClassNameEntry {
                source_file: source_file.clone(),
                name: trait_decl.name.clone(),
                line: trait_decl.span.line,
            })
            .collect();
        let runtime_class_names = program
            .classes
            .iter()
            .filter(|class| class.is_conditionally_declared)
            .map(|class| class_runtime_declaration_key(&source_file, &class.name))
            .collect();
        let mut context = Self {
            functions: Vec::new(),
            constant_deprecations,
            constant_values,
            include_trait_method_sources,
            source_file,
            source_dir,
            strict_types: program.strict_types,
            ticks_enabled: program.ticks,
            include_resolutions,
            class_names,
            trait_names,
            runtime_class_names,
            early_bound_class_names: HashSet::new(),
            current_class_name: None,
            current_class_parent_name: None,
            current_trait_name: None,
            current_function_display_name: None,
        };
        for function in &program.functions {
            context.declare_function(function);
        }
        context
    }

    fn source_file_for_method(
        &self,
        method: &crate::ast::MethodDecl,
        default_source_file: &str,
    ) -> String {
        let Some(trait_name) = method.trait_name.as_ref() else {
            return default_source_file.to_string();
        };
        let trait_method_name = method.trait_method_name.as_ref().unwrap_or(&method.name);
        self.include_trait_method_sources
            .get(&(
                trait_name.to_ascii_lowercase(),
                trait_method_name.to_ascii_lowercase(),
            ))
            .cloned()
            .unwrap_or_else(|| default_source_file.to_string())
    }

    fn declare_include_class_names(&mut self, include_sources: &[IncludeSource]) {
        for include in include_sources {
            self.class_names
                .extend(include.program.classes.iter().map(|class| ClassNameEntry {
                    source_file: include.source_file.clone(),
                    name: class.name.clone(),
                    line: class.span.line,
                }));
            self.trait_names
                .extend(
                    include
                        .program
                        .traits
                        .iter()
                        .map(|trait_decl| ClassNameEntry {
                            source_file: include.source_file.clone(),
                            name: trait_decl.name.clone(),
                            line: trait_decl.span.line,
                        }),
                );
            self.runtime_class_names.extend(
                include
                    .program
                    .classes
                    .iter()
                    .map(|class| class_runtime_declaration_key(&include.source_file, &class.name)),
            );
            self.early_bound_class_names.extend(
                include
                    .program
                    .classes
                    .iter()
                    .filter(|class| !class.is_conditionally_declared)
                    .map(|class| class_runtime_declaration_key(&include.source_file, &class.name)),
            );
        }
    }

    fn lower_expr_in_class_scope(
        &mut self,
        expr: &Expr,
        class_name: &str,
        parent_name: Option<&str>,
    ) -> ValueExpr {
        let previous_class_name =
            std::mem::replace(&mut self.current_class_name, Some(class_name.to_string()));
        let previous_parent_name = std::mem::replace(
            &mut self.current_class_parent_name,
            parent_name.map(ToString::to_string),
        );
        let lowered = self.lower_expr(expr);
        self.current_class_parent_name = previous_parent_name;
        self.current_class_name = previous_class_name;
        lowered
    }

    fn declare_include_functions(&mut self, include_sources: &[IncludeSource]) -> Vec<Vec<usize>> {
        let mut indices = Vec::with_capacity(include_sources.len());
        for include in include_sources {
            let previous_source_file =
                std::mem::replace(&mut self.source_file, include.source_file.clone());
            let previous_source_dir =
                std::mem::replace(&mut self.source_dir, include.source_dir.clone());
            let previous_strict_types =
                std::mem::replace(&mut self.strict_types, include.program.strict_types);
            let previous_ticks_enabled =
                std::mem::replace(&mut self.ticks_enabled, include.program.ticks);
            let mut include_indices = Vec::with_capacity(include.program.functions.len());
            for function in &include.program.functions {
                let function_index = self.declare_function(function);
                self.functions[function_index].initially_declared = false;
                include_indices.push(function_index);
            }
            indices.push(include_indices);
            self.source_file = previous_source_file;
            self.source_dir = previous_source_dir;
            self.strict_types = previous_strict_types;
            self.ticks_enabled = previous_ticks_enabled;
        }
        indices
    }

    fn lower_include_functions(&mut self, include: &IncludeSource, function_indices: &[usize]) {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let previous_strict_types =
            std::mem::replace(&mut self.strict_types, include.program.strict_types);
        let previous_ticks_enabled =
            std::mem::replace(&mut self.ticks_enabled, include.program.ticks);
        for (offset, function) in include.program.functions.iter().enumerate() {
            let previous_function_display_name = std::mem::replace(
                &mut self.current_function_display_name,
                Some(function.name.clone()),
            );
            let body = self.lower_statements(&function.body);
            self.current_function_display_name = previous_function_display_name;
            let function_index = function_indices[offset];
            self.functions[function_index].body = body;
        }
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        self.strict_types = previous_strict_types;
        self.ticks_enabled = previous_ticks_enabled;
    }

    fn lower_include_classes(&mut self, include: &IncludeSource) -> Vec<ClassDecl> {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let previous_strict_types =
            std::mem::replace(&mut self.strict_types, include.program.strict_types);
        let previous_ticks_enabled =
            std::mem::replace(&mut self.ticks_enabled, include.program.ticks);
        let mut classes: Vec<_> = include
            .program
            .classes
            .iter()
            .map(|class| self.lower_class(class))
            .collect();
        let include_source_file = include.source_file.clone();
        for class in &mut classes {
            class.source_file = include_source_file.clone();
            class.initially_declared = false;
            for method in &mut class.methods {
                if let Some(function) = self.functions.get_mut(method.function_index) {
                    if function.trait_name.is_none() {
                        function.source_file = include_source_file.clone();
                    }
                    method.source_file = function.source_file.clone();
                }
            }
        }
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        self.strict_types = previous_strict_types;
        self.ticks_enabled = previous_ticks_enabled;
        classes
    }

    fn lower_include_traits(&mut self, include: &IncludeSource) -> Vec<TraitDecl> {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let previous_strict_types =
            std::mem::replace(&mut self.strict_types, include.program.strict_types);
        let mut traits: Vec<_> = include
            .program
            .traits
            .iter()
            .map(|trait_decl| self.lower_trait(trait_decl))
            .collect();
        for trait_decl in &mut traits {
            trait_decl.initially_declared = false;
        }
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        self.strict_types = previous_strict_types;
        traits
    }

    fn declare_function(&mut self, function: &AstFunctionDecl) -> usize {
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| self.lower_parameter(parameter))
            .collect();
        let previous_function_display_name = std::mem::replace(
            &mut self.current_function_display_name,
            Some(function.name.clone()),
        );
        let attributes = self.annotate_attribute_metadata(&function.attributes);
        self.current_function_display_name = previous_function_display_name;
        let deprecated_metadata = self.function_deprecated_metadata(&attributes, None, None, None);
        let function_index = self.functions.len();
        self.functions.push(FunctionDecl {
            name: function.name.clone(),
            display_name: function.name.clone(),
            source_file: self.source_file.clone(),
            strict_types: self.strict_types,
            doc_comment: function.doc_comment.clone(),
            class_name: None,
            trait_name: None,
            trait_method_name: None,
            method_name: None,
            deprecated_message: deprecated_metadata.message,
            deprecated_since: deprecated_metadata.since,
            deprecated_message_runtime_reference: deprecated_metadata.message_runtime_reference,
            no_discard_message: attributes.no_discard_message.clone(),
            attributes,
            is_static: false,
            line: function.span.line,
            end_line: function.span.end_line,
            parameters,
            return_type: function.return_type.clone().map(lower_type_hint),
            return_by_ref: function.return_by_ref,
            is_generator: statements_contain_yield(&function.body),
            is_anonymous: false,
            initially_declared: !function.is_conditionally_declared,
            body: Vec::new(),
        });
        function_index
    }

    fn function_index_by_name(&self, name: &str) -> Option<usize> {
        self.functions
            .iter()
            .position(|function| {
                function.class_name.is_none()
                    && !function.is_anonymous
                    && function.source_file == self.source_file
                    && function.name.eq_ignore_ascii_case(name)
            })
            .or_else(|| {
                self.functions.iter().position(|function| {
                    function.class_name.is_none()
                        && !function.is_anonymous
                        && function.name.eq_ignore_ascii_case(name)
                })
            })
    }

    fn class_index_by_name(&self, name: &str) -> Option<usize> {
        self.class_names
            .iter()
            .position(|class_name| {
                class_name.source_file == self.source_file
                    && class_name.name.eq_ignore_ascii_case(name)
            })
            .or_else(|| {
                self.class_names
                    .iter()
                    .position(|class_name| class_name.name.eq_ignore_ascii_case(name))
            })
    }

    fn class_index_by_declaration(&self, name: &str, line: usize) -> Option<usize> {
        self.class_names
            .iter()
            .position(|class_name| {
                class_name.source_file == self.source_file
                    && class_name.line == line
                    && class_name.name.eq_ignore_ascii_case(name)
            })
            .or_else(|| self.class_index_by_name(name))
    }

    fn trait_index_by_declaration(&self, name: &str, line: usize) -> Option<usize> {
        self.trait_names
            .iter()
            .position(|trait_name| {
                trait_name.source_file == self.source_file
                    && trait_name.line == line
                    && trait_name.name.eq_ignore_ascii_case(name)
            })
            .or_else(|| {
                self.trait_names.iter().position(|trait_name| {
                    trait_name.source_file == self.source_file
                        && trait_name.name.eq_ignore_ascii_case(name)
                })
            })
            .or_else(|| {
                self.trait_names
                    .iter()
                    .position(|trait_name| trait_name.name.eq_ignore_ascii_case(name))
            })
    }

    fn lower_include_source(
        &mut self,
        include: &IncludeSource,
        function_indices: &[usize],
    ) -> IncludeFile {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let previous_strict_types =
            std::mem::replace(&mut self.strict_types, include.program.strict_types);
        let previous_ticks_enabled =
            std::mem::replace(&mut self.ticks_enabled, include.program.ticks);
        let include_constant_values = collect_constant_values(&include.program);
        let include_constant_deprecations =
            collect_constant_deprecations(&include.program, &include_constant_values);
        let previous_constant_deprecations = std::mem::replace(
            &mut self.constant_deprecations,
            include_constant_deprecations,
        );
        let previous_constant_values =
            std::mem::replace(&mut self.constant_values, include_constant_values);
        let mut instructions = Vec::new();
        for (offset, _function) in include
            .program
            .functions
            .iter()
            .enumerate()
            .filter(|(_, function)| !function.is_conditionally_declared)
        {
            if let Some(function_index) = function_indices.get(offset).copied() {
                instructions.push(Instruction::DeclareFunction { function_index });
            }
        }
        for trait_decl in &include.program.traits {
            if let Some(trait_index) =
                self.trait_index_by_declaration(&trait_decl.name, trait_decl.span.line)
            {
                instructions.push(Instruction::DeclareTrait { trait_index });
            }
        }
        for class in include
            .program
            .classes
            .iter()
            .filter(|class| !class.is_conditionally_declared)
        {
            if let Some(class_index) = self.class_index_by_declaration(&class.name, class.span.line)
            {
                instructions.push(Instruction::EarlyDeclareClass {
                    class_index,
                    line: class.span.line,
                });
            }
        }
        instructions.extend(self.lower_statements(&include.program.statements));
        self.constant_deprecations = previous_constant_deprecations;
        self.constant_values = previous_constant_values;
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        self.strict_types = previous_strict_types;
        self.ticks_enabled = previous_ticks_enabled;
        IncludeFile {
            source_file: include.source_file.clone(),
            source_dir: include.source_dir.clone(),
            source_bytes: include.source_bytes.clone(),
            path_aliases: include.path_aliases.clone(),
            parse_error: include.parse_error.clone(),
            strict_types: include.program.strict_types,
            ticks: include.program.ticks,
            instructions,
            compile_warnings: lower_compile_warnings(&include.program.compile_warnings),
        }
    }

    fn global_deprecated_metadata(
        &self,
        attributes: &AttributeMetadata,
        current_name: &str,
    ) -> DeprecatedMetadata {
        let current_subject = format!("Constant {}", current_name.trim_start_matches('\\'));
        let message = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => self
                    .constant_values
                    .get(&name.to_ascii_lowercase())
                    .cloned(),
                AttributeConstantReference::ClassConstant { .. } => None,
            })
            .or_else(|| attributes.deprecated_message.clone());
        let message_dependency = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    let subject = format!("Constant {}", name.trim_start_matches('\\'));
                    (subject != current_subject)
                        .then(|| {
                            self.constant_deprecations
                                .get(&name.to_ascii_lowercase())
                                .map(|metadata| (subject, metadata))
                        })
                        .flatten()
                }
                AttributeConstantReference::ClassConstant { .. } => None,
            })
            .map(|(subject, metadata)| DeprecatedMessageDependency {
                subject,
                message: metadata.message.clone(),
                since: metadata.since.clone(),
            });
        DeprecatedMetadata {
            message,
            since: attributes.deprecated_since.clone(),
            message_dependency,
            message_runtime_reference: None,
        }
    }

    fn class_deprecated_metadata(
        &self,
        attributes: &AttributeMetadata,
        class_name: &str,
        current_name: &str,
        class_constant_values: &HashMap<String, String>,
        class_constant_deprecations: &HashMap<String, DeprecatedMetadata>,
    ) -> DeprecatedMetadata {
        let current_subject = format!("Constant {class_name}::{current_name}");
        let referenced_message =
            attributes
                .deprecated_message_constant
                .as_ref()
                .and_then(|reference| match reference {
                    AttributeConstantReference::Constant(name) => self
                        .constant_values
                        .get(&name.to_ascii_lowercase())
                        .cloned(),
                    AttributeConstantReference::ClassConstant {
                        class_name: referenced_class,
                        name,
                    } => self
                        .resolve_attribute_class_name(referenced_class, class_name)
                        .filter(|resolved| resolved.eq_ignore_ascii_case(class_name))
                        .and_then(|_| {
                            class_constant_values
                                .get(&name.to_ascii_lowercase())
                                .cloned()
                        }),
                });
        let message = referenced_message.clone().or_else(|| {
            attributes
                .deprecated_message_constant
                .is_none()
                .then(|| attributes.deprecated_message.clone())
                .flatten()
        });
        let message_runtime_reference = (referenced_message.is_none())
            .then(|| attributes.deprecated_message_constant.clone())
            .flatten();
        let message_dependency = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    let subject = format!("Constant {}", name.trim_start_matches('\\'));
                    (subject != current_subject)
                        .then(|| {
                            self.constant_deprecations
                                .get(&name.to_ascii_lowercase())
                                .map(|metadata| (subject, metadata))
                        })
                        .flatten()
                }
                AttributeConstantReference::ClassConstant {
                    class_name: referenced_class,
                    name,
                } => self
                    .resolve_attribute_class_name(referenced_class, class_name)
                    .and_then(|resolved_class| {
                        let subject = format!("Constant {resolved_class}::{name}");
                        if subject == current_subject
                            || !resolved_class.eq_ignore_ascii_case(class_name)
                        {
                            return None;
                        }
                        class_constant_deprecations
                            .get(&name.to_ascii_lowercase())
                            .map(|metadata| (subject, metadata))
                    }),
            })
            .map(|(subject, metadata)| DeprecatedMessageDependency {
                subject,
                message: metadata.message.clone(),
                since: metadata.since.clone(),
            });
        DeprecatedMetadata {
            message,
            since: attributes.deprecated_since.clone(),
            message_dependency,
            message_runtime_reference,
        }
    }

    fn function_deprecated_metadata(
        &self,
        attributes: &AttributeMetadata,
        current_class: Option<&str>,
        current_parent: Option<&str>,
        class_constant_values: Option<&HashMap<String, String>>,
    ) -> DeprecatedMetadata {
        let referenced_message =
            attributes
                .deprecated_message_constant
                .as_ref()
                .and_then(|reference| match reference {
                    AttributeConstantReference::Constant(name) => self
                        .constant_values
                        .get(&name.to_ascii_lowercase())
                        .cloned(),
                    AttributeConstantReference::ClassConstant {
                        class_name: referenced_class,
                        name,
                    } => {
                        let current_class = current_class?;
                        let resolved = self.resolve_attribute_class_name_for_function(
                            referenced_class,
                            current_class,
                            current_parent,
                        )?;
                        if resolved.eq_ignore_ascii_case(current_class) {
                            class_constant_values
                                .and_then(|values| values.get(&name.to_ascii_lowercase()).cloned())
                        } else {
                            None
                        }
                    }
                });
        let message = referenced_message.clone().or_else(|| {
            attributes
                .deprecated_message_constant
                .is_none()
                .then(|| attributes.deprecated_message.clone())
                .flatten()
        });
        let message_runtime_reference = (referenced_message.is_none())
            .then(|| {
                attributes
                    .deprecated_message_constant
                    .as_ref()
                    .map(|reference| match reference {
                        AttributeConstantReference::Constant(name) => {
                            AttributeConstantReference::Constant(name.clone())
                        }
                        AttributeConstantReference::ClassConstant { class_name, name } => {
                            let resolved = current_class
                                .and_then(|current_class| {
                                    self.resolve_attribute_class_name_for_function(
                                        class_name,
                                        current_class,
                                        current_parent,
                                    )
                                })
                                .unwrap_or_else(|| class_name.trim_start_matches('\\').to_string());
                            AttributeConstantReference::ClassConstant {
                                class_name: resolved,
                                name: name.clone(),
                            }
                        }
                    })
            })
            .flatten();
        DeprecatedMetadata {
            message,
            since: attributes.deprecated_since.clone(),
            message_dependency: None,
            message_runtime_reference,
        }
    }

    fn resolve_attribute_class_name_for_function(
        &self,
        referenced_class: &str,
        current_class: &str,
        current_parent: Option<&str>,
    ) -> Option<String> {
        if referenced_class.eq_ignore_ascii_case("parent") {
            current_parent.map(ToString::to_string)
        } else {
            self.resolve_attribute_class_name(referenced_class, current_class)
        }
    }

    fn resolve_attribute_class_name(
        &self,
        referenced_class: &str,
        current_class: &str,
    ) -> Option<String> {
        if referenced_class.eq_ignore_ascii_case("self")
            || referenced_class.eq_ignore_ascii_case("static")
        {
            Some(current_class.to_string())
        } else if referenced_class.eq_ignore_ascii_case(current_class) {
            Some(current_class.to_string())
        } else {
            Some(referenced_class.trim_start_matches('\\').to_string())
        }
    }

    fn lower_anonymous_function(&mut self, function: &AstAnonymousFunction) -> ValueExpr {
        let current_class_name = self.current_class_name.clone();
        let current_class_parent_name = self.current_class_parent_name.clone();
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| {
                if let Some(current_class) = current_class_name.as_deref() {
                    self.lower_parameter_for_class_scope(
                        parameter,
                        current_class,
                        current_class_parent_name.as_deref(),
                    )
                } else {
                    self.lower_parameter(parameter)
                }
            })
            .collect();
        let attributes = if let Some(current_class) = current_class_name.as_deref() {
            self.lower_class_scoped_attribute_metadata(
                &function.attributes,
                current_class,
                current_class_parent_name.as_deref(),
            )
        } else {
            self.annotate_attribute_metadata(&function.attributes)
        };
        let deprecated_metadata = self.function_deprecated_metadata(
            &attributes,
            current_class_name.as_deref(),
            current_class_parent_name.as_deref(),
            None,
        );
        let display_name = if let Some(scope_name) = &self.current_function_display_name {
            if scope_name.starts_with("{closure:") {
                format!("{{closure:{}:{}}}", scope_name, function.span.line)
            } else {
                format!("{{closure:{}():{}}}", scope_name, function.span.line)
            }
        } else {
            format!("{{closure:{}:{}}}", self.source_file, function.span.line)
        };
        let nested_display_name = display_name.clone();
        let function_index = self.functions.len();
        self.functions.push(FunctionDecl {
            name: "{closure}".to_string(),
            display_name,
            source_file: self.source_file.clone(),
            strict_types: self.strict_types,
            doc_comment: function.doc_comment.clone(),
            class_name: self.current_class_name.clone(),
            trait_name: self.current_trait_name.clone(),
            trait_method_name: None,
            method_name: None,
            deprecated_message: deprecated_metadata.message,
            deprecated_since: deprecated_metadata.since,
            deprecated_message_runtime_reference: deprecated_metadata.message_runtime_reference,
            no_discard_message: attributes.no_discard_message.clone(),
            attributes,
            is_static: function.is_static,
            line: function.span.line,
            end_line: function.span.end_line,
            parameters,
            return_type: function.return_type.clone().map(lower_type_hint),
            return_by_ref: function.return_by_ref,
            is_generator: statements_contain_yield(&function.body),
            is_anonymous: true,
            initially_declared: true,
            body: Vec::new(),
        });
        let previous_function_display_name = std::mem::replace(
            &mut self.current_function_display_name,
            Some(nested_display_name),
        );
        let body = self.lower_statements(&function.body);
        self.current_function_display_name = previous_function_display_name;
        self.functions[function_index].body = body;
        ValueExpr::Closure {
            function_index,
            captures: function
                .captures
                .iter()
                .map(lower_closure_capture)
                .collect(),
            line: function.span.line,
        }
    }

    fn lower_class(&mut self, class: &AstClassDecl) -> ClassDecl {
        let class_source_file =
            ast_class_source_file(&self.source_file, class.source_file.as_deref());
        let parent_name = class.parent_name.as_deref();
        let class_attributes =
            self.lower_class_scoped_attribute_metadata(&class.attributes, &class.name, parent_name);
        let properties =
            class
                .properties
                .iter()
                .map(|property| PropertyDecl {
                    name: property.name.clone(),
                    declaring_class_name: property
                        .declaring_class_name
                        .clone()
                        .unwrap_or_else(|| class.name.clone()),
                    trait_name: property.trait_name.clone(),
                    visibility: lower_property_visibility(property.visibility),
                    set_visibility: lower_property_visibility(property.set_visibility),
                    is_final: property.is_final,
                    is_abstract: property.is_abstract,
                    is_readonly: property.is_readonly,
                    is_promoted: property.is_promoted,
                    has_hooks: property.has_hooks,
                    is_virtual: property.is_virtual,
                    hook_has_get: property.hook_has_get,
                    hook_has_set: property.hook_has_set,
                    hook_get_is_final: property.hook_get_is_final,
                    hook_get_is_abstract: property.hook_get_is_abstract,
                    hook_get_returns_by_ref: property.hook_get_returns_by_ref,
                    hook_set_is_final: property.hook_set_is_final,
                    hook_set_is_abstract: property.hook_set_is_abstract,
                    hook_get_attributes: self.lower_class_property_scoped_attribute_metadata(
                        &property.hook_get_attributes,
                        &class.name,
                        parent_name,
                        &property.name,
                    ),
                    hook_set_attributes: self.lower_class_property_scoped_attribute_metadata(
                        &property.hook_set_attributes,
                        &class.name,
                        parent_name,
                        &property.name,
                    ),
                    hook_get_doc_comment: property.hook_get_doc_comment.clone(),
                    hook_set_doc_comment: property.hook_set_doc_comment.clone(),
                    hook_get_line: property
                        .hook_get_span
                        .as_ref()
                        .map(|span| span.line)
                        .unwrap_or(property.span.line),
                    hook_set_line: property
                        .hook_set_span
                        .as_ref()
                        .map(|span| span.line)
                        .unwrap_or(property.span.line),
                    hook_get_value: property.hook_get_value.as_ref().map(|value| {
                        self.lower_expr_in_class_scope(value, &class.name, parent_name)
                    }),
                    hook_set_value: property.hook_set_value.as_ref().map(|value| {
                        self.lower_expr_in_class_scope(value, &class.name, parent_name)
                    }),
                    hook_get_body: property
                        .hook_get_body
                        .as_ref()
                        .map(|body| self.lower_statements(body)),
                    hook_set_body: property
                        .hook_set_body
                        .as_ref()
                        .map(|body| self.lower_statements(body)),
                    hook_set_parameter_name: property.hook_set_parameter_name.clone(),
                    hook_set_parameter_attributes: self
                        .lower_class_property_scoped_attribute_metadata(
                            &property.hook_set_parameter_attributes,
                            &class.name,
                            parent_name,
                            &property.name,
                        ),
                    hook_set_parameter_type: property
                        .hook_set_parameter_type
                        .clone()
                        .map(lower_type_hint),
                    hook_set_parameter_doc_comment: property.hook_set_parameter_doc_comment.clone(),
                    type_hint: property
                        .type_hint
                        .as_ref()
                        .map(|type_hint| lower_class_property_type_hint(class, type_hint)),
                    attributes: self.lower_class_property_scoped_attribute_metadata(
                        &property.attributes,
                        &class.name,
                        parent_name,
                        &property.name,
                    ),
                    doc_comment: property.doc_comment.clone(),
                    value: property.value.as_ref().map(|value| {
                        self.lower_expr_in_class_scope(value, &class.name, parent_name)
                    }),
                    line: property.span.line,
                    source_order: property.span.byte_start,
                })
                .collect();
        let static_properties = class
            .static_properties
            .iter()
            .map(|property| StaticPropertyDecl {
                name: property.name.clone(),
                visibility: lower_property_visibility(property.visibility),
                set_visibility: lower_property_visibility(property.set_visibility),
                is_final: property.is_final,
                type_hint: property
                    .type_hint
                    .as_ref()
                    .map(|type_hint| lower_class_property_type_hint(class, type_hint)),
                attributes: self.lower_class_property_scoped_attribute_metadata(
                    &property.attributes,
                    &class.name,
                    parent_name,
                    &property.name,
                ),
                doc_comment: property.doc_comment.clone(),
                value: property
                    .value
                    .as_ref()
                    .map(|value| self.lower_expr_in_class_scope(value, &class.name, parent_name)),
                source_order: property.span.byte_start,
            })
            .collect();
        let class_constant_values =
            collect_class_constant_values(&class.constants, &self.constant_values);
        let class_constant_deprecations = collect_class_constant_deprecations(
            &class.constants,
            &self.constant_values,
            &class_constant_values,
        );
        let constants = class
            .constants
            .iter()
            .map(|constant| {
                let attributes = self.lower_class_scoped_attribute_metadata(
                    &constant.attributes,
                    &class.name,
                    parent_name,
                );
                let metadata = self.class_deprecated_metadata(
                    &attributes,
                    &class.name,
                    &constant.name,
                    &class_constant_values,
                    &class_constant_deprecations,
                );
                ClassConstantDecl {
                    name: constant.name.clone(),
                    visibility: lower_property_visibility(constant.visibility),
                    type_hint: constant.type_hint.clone().map(lower_type_hint),
                    attributes,
                    doc_comment: constant.doc_comment.clone(),
                    deprecated_message: metadata.message,
                    deprecated_since: metadata.since,
                    deprecated_message_dependency: metadata.message_dependency,
                    deprecated_message_runtime_reference: metadata.message_runtime_reference,
                    is_enum_case: constant.is_enum_case,
                    enum_case_value: constant.enum_case_value.as_ref().map(|value| {
                        self.lower_expr_in_class_scope(value, &class.name, parent_name)
                    }),
                    is_final: constant.is_final,
                    value: self.lower_expr_in_class_scope(
                        &constant.value,
                        &class.name,
                        parent_name,
                    ),
                }
            })
            .collect();
        let class_display_name = class_display_name(class);
        let methods = class
            .methods
            .iter()
            .map(|method| {
                let method_display_name = format!("{class_display_name}::{}", method.name);
                let previous_function_display_name = std::mem::replace(
                    &mut self.current_function_display_name,
                    Some(method_display_name.clone()),
                );
                let method_attributes = self.lower_class_scoped_attribute_metadata(
                    &method.attributes,
                    &class.name,
                    parent_name,
                );
                self.current_function_display_name = previous_function_display_name;
                let parameters = method
                    .parameters
                    .iter()
                    .map(|parameter| {
                        let mut lowered = self.lower_parameter_for_class_scope(
                            parameter,
                            &class.name,
                            parent_name,
                        );
                        if method.trait_name.is_some() {
                            lowered.default_value = parameter
                                .default_value
                                .as_ref()
                                .map(|value| self.lower_expr(value));
                        }
                        lowered
                    })
                    .collect();
                let deprecated_metadata = self.function_deprecated_metadata(
                    &method_attributes,
                    Some(&class.name),
                    class.parent_name.as_deref(),
                    Some(&class_constant_values),
                );
                let method_source_file = self.source_file_for_method(method, &class_source_file);
                let function_index = self.functions.len();
                self.functions.push(FunctionDecl {
                    name: format!("{}::{}", class.name, method.name),
                    display_name: method_display_name.clone(),
                    source_file: method_source_file.clone(),
                    strict_types: self.strict_types,
                    doc_comment: method.doc_comment.clone(),
                    class_name: Some(class.name.clone()),
                    trait_name: method.trait_name.clone(),
                    trait_method_name: method.trait_method_name.clone(),
                    method_name: Some(method.name.clone()),
                    deprecated_message: deprecated_metadata.message,
                    deprecated_since: deprecated_metadata.since,
                    deprecated_message_runtime_reference: deprecated_metadata
                        .message_runtime_reference,
                    no_discard_message: method_attributes.no_discard_message.clone(),
                    attributes: method_attributes.clone(),
                    is_static: method.is_static,
                    line: method.span.line,
                    end_line: method.span.end_line,
                    parameters,
                    return_type: method.return_type.clone().map(lower_type_hint),
                    return_by_ref: method.return_by_ref,
                    is_generator: statements_contain_yield(&method.body),
                    is_anonymous: false,
                    initially_declared: true,
                    body: Vec::new(),
                });
                let previous_class_name =
                    std::mem::replace(&mut self.current_class_name, Some(class.name.clone()));
                let previous_class_parent_name = std::mem::replace(
                    &mut self.current_class_parent_name,
                    class.parent_name.clone(),
                );
                let previous_trait_name =
                    std::mem::replace(&mut self.current_trait_name, method.trait_name.clone());
                let previous_function_display_name = std::mem::replace(
                    &mut self.current_function_display_name,
                    Some(method_display_name),
                );
                let body = self.lower_statements(&method.body);
                self.current_class_name = previous_class_name;
                self.current_class_parent_name = previous_class_parent_name;
                self.current_trait_name = previous_trait_name;
                self.current_function_display_name = previous_function_display_name;
                self.functions[function_index].body = body;
                MethodDecl {
                    name: method.name.clone(),
                    source_file: method_source_file,
                    function_index,
                    visibility: lower_property_visibility(method.visibility),
                    has_explicit_visibility: method.has_explicit_visibility,
                    attributes: method_attributes,
                    doc_comment: method.doc_comment.clone(),
                    is_static: method.is_static,
                    is_final: method.is_final,
                    is_abstract: method.is_abstract,
                    line: method.span.line,
                    end_line: method.span.end_line,
                }
            })
            .collect();
        ClassDecl {
            name: class.name.clone(),
            source_file: class_source_file,
            parent_name: class.parent_name.clone(),
            interfaces: class.interfaces.clone(),
            trait_uses: lower_trait_uses(&class.trait_uses),
            declaration_fatals: class
                .declaration_fatals
                .iter()
                .map(|fatal| ClassDeclarationFatal {
                    message: fatal.message.clone(),
                    line: fatal.span.line,
                })
                .collect(),
            attributes: class_attributes,
            doc_comment: class.doc_comment.clone(),
            line: class.span.line,
            end_line: class.span.end_line,
            initially_declared: !class.is_conditionally_declared && !class.is_anonymous,
            is_conditionally_declared: class.is_conditionally_declared,
            is_syntactically_conditionally_declared: class.is_syntactically_conditionally_declared,
            is_anonymous: class.is_anonymous,
            is_abstract: class.is_abstract,
            is_final: class.is_final,
            is_interface: class.is_interface,
            is_readonly: class.is_readonly,
            is_enum: class.is_enum,
            enum_backing_type: class.enum_backing_type.map(lower_enum_backing_type),
            properties,
            static_properties,
            constants,
            methods,
        }
    }

    fn lower_statements(&mut self, statements: &[Statement]) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        for statement in statements {
            match statement {
                Statement::Empty { .. } => {}
                Statement::TraitDeclaration { name, span, .. } => {
                    if let Some(trait_index) = self.trait_index_by_declaration(name, span.line) {
                        instructions.push(Instruction::DeclareTrait { trait_index });
                    }
                }
                Statement::ClassDeclaration { name, span, .. } => {
                    if let Some(class_index) = self.class_index_by_declaration(name, span.line) {
                        let declaration_key =
                            class_runtime_declaration_key(&self.source_file, name);
                        if self.runtime_class_names.contains(&declaration_key) {
                            let allow_predeclared =
                                self.early_bound_class_names.contains(&declaration_key);
                            instructions.push(Instruction::DeclareClass {
                                class_index,
                                line: span.line,
                                allow_predeclared,
                            });
                        } else {
                            instructions.push(Instruction::ValidateClass {
                                class_index,
                                line: span.line,
                            });
                        }
                    }
                }
                Statement::FunctionDeclaration { name, .. } => {
                    if let Some(function_index) = self.function_index_by_name(name) {
                        instructions.push(Instruction::DeclareFunction { function_index });
                    }
                }
                Statement::Assign {
                    name,
                    op,
                    value,
                    span,
                } => {
                    if matches!(op, AssignmentOp::Assign) {
                        instructions.push(Instruction::Store {
                            name: name.clone(),
                            value: self.lower_expr(value),
                            line: span.line,
                        });
                    } else {
                        instructions.push(Instruction::Expression(ValueExpr::Assign {
                            target: AssignmentTarget::Variable {
                                name: name.clone(),
                                line: span.line,
                            },
                            op: *op,
                            value: Box::new(self.lower_expr(value)),
                        }));
                    }
                }
                Statement::AssignRef { name, source, span } => {
                    instructions.push(Instruction::StoreRef {
                        name: name.clone(),
                        source: self.lower_expr(source),
                        line: span.line,
                    });
                }
                Statement::ArrayAssign {
                    target, op, value, ..
                } => {
                    if matches!(op, AssignmentOp::CoalesceAssign) {
                        instructions.push(Instruction::Expression(ValueExpr::Assign {
                            target: self.lower_assignment_target(&AstAssignmentTarget::ArrayDim(
                                target.clone(),
                            )),
                            op: *op,
                            value: Box::new(self.lower_expr(value)),
                        }));
                    } else {
                        instructions.push(self.lower_array_dim_store(target, *op, value));
                    }
                }
                Statement::ArrayAssignRef { target, source, .. } => {
                    instructions.push(Instruction::StoreArrayDimRef {
                        target: self.lower_array_dim_target(target),
                        source: self.lower_expr(source),
                    });
                }
                Statement::Increment { target, op, span } => {
                    instructions.push(Instruction::Increment {
                        target: self.lower_inc_dec_target(target),
                        op: lower_inc_dec_op(*op),
                        line: span.line,
                    });
                }
                Statement::Unset { targets, .. } => {
                    for target in targets {
                        instructions.push(self.lower_unset_target(target));
                    }
                }
                Statement::Global { targets, .. } => {
                    for target in targets {
                        match target {
                            GlobalTarget::Variable { name, .. } => {
                                instructions.push(Instruction::BindGlobal { name: name.clone() });
                            }
                            GlobalTarget::DynamicVariable { name, span } => {
                                instructions.push(Instruction::BindDynamicGlobal {
                                    name: self.lower_expr(name),
                                    line: span.line,
                                });
                            }
                        }
                    }
                }
                Statement::Static { declarations, .. } => {
                    for declaration in declarations {
                        instructions.push(Instruction::BindStatic {
                            name: declaration.name.clone(),
                            value: declaration
                                .value
                                .as_ref()
                                .map(|value| self.lower_expr(value)),
                            line: declaration.span.line,
                        });
                    }
                }
                Statement::Const { declarations, .. } => {
                    for declaration in declarations {
                        let attributes = self.annotate_attribute_metadata(&declaration.attributes);
                        let metadata =
                            self.global_deprecated_metadata(&attributes, &declaration.name);
                        instructions.push(Instruction::DefineConstant {
                            name: declaration.name.clone(),
                            attributes,
                            value: self.lower_expr(&declaration.value),
                            deprecated_message: metadata.message,
                            deprecated_since: metadata.since,
                            deprecated_message_dependency: metadata.message_dependency,
                            line: declaration.span.line,
                        });
                    }
                }
                Statement::Call {
                    name,
                    arguments,
                    argument_names,
                    argument_unpacks,
                    allow_global_fallback,
                    span,
                } => {
                    if name.eq_ignore_ascii_case("opcache_compile_file")
                        && arguments.len() == 1
                        && argument_names.iter().all(Option::is_none)
                        && argument_unpacks.iter().all(|unpack| !*unpack)
                    {
                        instructions.push(Instruction::Expression(ValueExpr::OpcacheCompileFile {
                            path: Box::new(self.lower_expr(&arguments[0])),
                            candidates: self.include_candidates(*span),
                            line: span.line,
                        }));
                    } else {
                        let (arguments, argument_names) =
                            self.lower_internal_call_arguments(name, arguments, argument_names);
                        instructions.push(Instruction::InternalCall {
                            name: name.clone(),
                            arguments,
                            argument_names,
                            argument_unpacks: argument_unpacks.clone(),
                            allow_global_fallback: *allow_global_fallback,
                            line: span.line,
                        });
                    }
                }
                Statement::Echo { expressions, span } => {
                    for expression in expressions {
                        instructions.push(Instruction::Echo {
                            value: self.lower_expr(expression),
                            line: span.line,
                        });
                    }
                }
                Statement::Print { expression, .. } => {
                    instructions.push(Instruction::Expression(ValueExpr::Print {
                        expression: Box::new(self.lower_expr(expression)),
                    }));
                }
                Statement::Expression { expression, .. } => {
                    if expression_statement_is_noop(expression) {
                        if self.ticks_enabled {
                            instructions.push(Instruction::Tick {
                                line: expression.span().line,
                            });
                        }
                        continue;
                    }
                    instructions.push(Instruction::Expression(self.lower_expr(expression)));
                }
                Statement::InlineHtml { content, span } => {
                    instructions.push(Instruction::Echo {
                        value: ValueExpr::String(content.clone()),
                        line: span.line,
                    });
                }
                Statement::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    if !statements_contain_label(then_body) && !statements_contain_label(else_body)
                    {
                        if let Some(value) = ast_compile_time_condition_truth(condition) {
                            if value {
                                instructions.extend(self.lower_statements(then_body));
                            } else {
                                instructions.extend(self.lower_statements(else_body));
                            }
                            continue;
                        }
                    }
                    instructions.push(Instruction::Branch {
                        condition: self.lower_expr(condition),
                        then_body: self.lower_statements(then_body),
                        else_body: self.lower_statements(else_body),
                    });
                }
                Statement::Block {
                    statements, ticks, ..
                } => {
                    if let Some(enabled) = ticks {
                        let previous_ticks_enabled =
                            std::mem::replace(&mut self.ticks_enabled, *enabled);
                        let body = self.lower_statements(statements);
                        self.ticks_enabled = previous_ticks_enabled;
                        instructions.push(Instruction::TickScope {
                            enabled: *enabled,
                            body,
                        });
                    } else {
                        instructions.extend(self.lower_statements(statements));
                    }
                }
                Statement::While {
                    condition, body, ..
                } => {
                    instructions.push(Instruction::While {
                        condition: self.lower_expr(condition),
                        body: self.lower_statements(body),
                    });
                }
                Statement::DoWhile {
                    body, condition, ..
                } => {
                    instructions.push(Instruction::DoWhile {
                        body: self.lower_statements(body),
                        condition: self.lower_expr(condition),
                    });
                }
                Statement::For {
                    initializers,
                    conditions,
                    updates,
                    body,
                    ..
                } => {
                    instructions.push(Instruction::For {
                        initializers: self.lower_statements(initializers),
                        conditions: conditions
                            .iter()
                            .map(|condition| self.lower_expr(condition))
                            .collect(),
                        updates: self.lower_statements(updates),
                        body: self.lower_statements(body),
                    });
                }
                Statement::Foreach {
                    iterable,
                    key,
                    value,
                    value_by_ref,
                    body,
                    span,
                } => {
                    instructions.push(Instruction::Foreach {
                        iterable: self.lower_expr(iterable),
                        key: key
                            .as_ref()
                            .map(|target| self.lower_assignment_target(target)),
                        value: self.lower_assignment_target(value),
                        value_by_ref: *value_by_ref,
                        body: self.lower_statements(body),
                        line: span.line,
                    });
                }
                Statement::Switch {
                    expression, cases, ..
                } => {
                    instructions.push(Instruction::Switch {
                        expression: self.lower_expr(expression),
                        cases: cases
                            .iter()
                            .map(|case| SwitchCase {
                                condition: case
                                    .condition
                                    .as_ref()
                                    .map(|condition| self.lower_expr(condition)),
                                body: self.lower_statements(&case.body),
                            })
                            .collect(),
                    });
                }
                Statement::Break { level, span } => {
                    instructions.push(Instruction::Break {
                        level: *level,
                        line: span.line,
                    });
                }
                Statement::Continue { level, span } => {
                    instructions.push(Instruction::Continue {
                        level: *level,
                        line: span.line,
                    });
                }
                Statement::Return { value, span } => {
                    instructions.push(Instruction::Return {
                        value: value.as_ref().map(|value| self.lower_expr(value)),
                        line: span.line,
                    });
                }
                Statement::Exit { value, span } => {
                    instructions.push(Instruction::Exit {
                        value: value.as_ref().map(|value| self.lower_expr(value)),
                        line: span.line,
                    });
                }
                Statement::Throw { value, span } => {
                    instructions.push(Instruction::Throw {
                        value: self.lower_expr(value),
                        line: span.line,
                    });
                }
                Statement::Try {
                    body,
                    catches,
                    finally_body,
                    ..
                } => {
                    instructions.push(Instruction::Try {
                        body: self.lower_statements(body),
                        catches: catches
                            .iter()
                            .map(|catch| self.lower_catch_clause(catch))
                            .collect(),
                        finally_body: self.lower_statements(finally_body),
                    });
                }
                Statement::Label { name, .. } => {
                    instructions.push(Instruction::Label { name: name.clone() });
                }
                Statement::Goto { label, .. } => {
                    instructions.push(Instruction::Goto {
                        label: label.clone(),
                    });
                }
            }
        }
        instructions
    }

    fn lower_array_dim_store(
        &mut self,
        target: &AstArrayDimTarget,
        op: AssignmentOp,
        value: &Expr,
    ) -> Instruction {
        Instruction::StoreArrayDim {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| {
                    dimension
                        .as_ref()
                        .map(|dimension| self.lower_expr(dimension))
                })
                .collect(),
            value: self.lower_expr(value),
            compound_op: assignment_op_binary_op(op),
            line: target.span.line,
        }
    }

    fn lower_array_dim_target(&mut self, target: &AstArrayDimTarget) -> ArrayDimTarget {
        ArrayDimTarget {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| {
                    dimension
                        .as_ref()
                        .map(|dimension| self.lower_expr(dimension))
                })
                .collect(),
            line: target.span.line,
        }
    }

    fn lower_parameter(&mut self, parameter: &AstFunctionParameter) -> FunctionParameter {
        FunctionParameter {
            name: parameter.name.clone(),
            attributes: self.annotate_attribute_metadata(&parameter.attributes),
            doc_comment: parameter.doc_comment.clone(),
            type_hint: parameter.type_hint.clone().map(lower_type_hint),
            by_ref: parameter.by_ref,
            is_variadic: parameter.is_variadic,
            default_value: parameter
                .default_value
                .as_ref()
                .map(|value| self.lower_expr(value)),
        }
    }

    fn lower_parameter_for_class_scope(
        &mut self,
        parameter: &AstFunctionParameter,
        current_class: &str,
        current_parent: Option<&str>,
    ) -> FunctionParameter {
        let mut lowered = self.lower_parameter(parameter);
        lowered.attributes = self.lower_class_scoped_attribute_metadata(
            &parameter.attributes,
            current_class,
            current_parent,
        );
        lowered.default_value = lowered.default_value.map(|value| {
            resolve_class_scoped_parameter_default(value, current_class, current_parent)
        });
        lowered
    }

    fn lower_parameter_for_trait_scope(
        &mut self,
        parameter: &AstFunctionParameter,
        current_trait: &str,
    ) -> FunctionParameter {
        let mut lowered = self.lower_parameter(parameter);
        lowered.attributes =
            self.lower_trait_scoped_attribute_metadata(&parameter.attributes, current_trait);
        lowered
    }

    fn lower_class_scoped_attribute_metadata(
        &mut self,
        metadata: &AttributeMetadata,
        current_class: &str,
        current_parent: Option<&str>,
    ) -> AttributeMetadata {
        self.lower_class_scoped_attribute_metadata_with_property(
            metadata,
            current_class,
            current_parent,
            None,
        )
    }

    fn lower_class_property_scoped_attribute_metadata(
        &mut self,
        metadata: &AttributeMetadata,
        current_class: &str,
        current_parent: Option<&str>,
        current_property: &str,
    ) -> AttributeMetadata {
        self.lower_class_scoped_attribute_metadata_with_property(
            metadata,
            current_class,
            current_parent,
            Some(current_property),
        )
    }

    fn lower_class_scoped_attribute_metadata_with_property(
        &mut self,
        metadata: &AttributeMetadata,
        current_class: &str,
        current_parent: Option<&str>,
        current_property: Option<&str>,
    ) -> AttributeMetadata {
        let resolved = resolve_attribute_metadata_for_class_scope(
            metadata,
            current_class,
            current_parent,
            current_property,
        );
        let previous_class_name = std::mem::replace(
            &mut self.current_class_name,
            Some(current_class.to_string()),
        );
        let previous_class_parent_name = std::mem::replace(
            &mut self.current_class_parent_name,
            current_parent.map(ToString::to_string),
        );
        let annotated = self.annotate_attribute_metadata(&resolved);
        self.current_class_name = previous_class_name;
        self.current_class_parent_name = previous_class_parent_name;
        annotated
    }

    fn lower_trait_scoped_attribute_metadata(
        &mut self,
        metadata: &AttributeMetadata,
        current_trait: &str,
    ) -> AttributeMetadata {
        self.lower_trait_scoped_attribute_metadata_with_property(metadata, current_trait, None)
    }

    fn lower_trait_property_scoped_attribute_metadata(
        &mut self,
        metadata: &AttributeMetadata,
        current_trait: &str,
        current_property: &str,
    ) -> AttributeMetadata {
        self.lower_trait_scoped_attribute_metadata_with_property(
            metadata,
            current_trait,
            Some(current_property),
        )
    }

    fn lower_trait_scoped_attribute_metadata_with_property(
        &mut self,
        metadata: &AttributeMetadata,
        current_trait: &str,
        current_property: Option<&str>,
    ) -> AttributeMetadata {
        let resolved =
            resolve_attribute_metadata_for_trait_scope(metadata, current_trait, current_property);
        let previous_class_name = std::mem::replace(
            &mut self.current_class_name,
            Some(current_trait.to_string()),
        );
        let previous_class_parent_name =
            std::mem::replace(&mut self.current_class_parent_name, None);
        let annotated = self.annotate_attribute_metadata(&resolved);
        self.current_class_name = previous_class_name;
        self.current_class_parent_name = previous_class_parent_name;
        annotated
    }

    fn annotate_attribute_metadata(&mut self, metadata: &AttributeMetadata) -> AttributeMetadata {
        let mut annotated = metadata.clone();
        self.lower_attribute_metadata_closures(&mut annotated);
        for instance in &mut annotated.instances {
            instance.source_file = self.source_file.clone();
            instance.strict_types = self.strict_types;
        }
        annotated
    }

    fn lower_attribute_metadata_closures(&mut self, metadata: &mut AttributeMetadata) {
        for instance in &mut metadata.instances {
            for argument in &mut instance.arguments {
                if let Some(expression) = &mut argument.value.expression {
                    self.lower_attribute_argument_expression_closures(expression);
                }
            }
        }
    }

    fn lower_attribute_argument_expression_closures(
        &mut self,
        expression: &mut AttributeArgumentExpression,
    ) {
        match expression {
            AttributeArgumentExpression::Closure {
                function,
                function_index,
                source_text,
                reflection_text,
                line,
            } => {
                if function_index.is_some() {
                    return;
                }
                let Some(function) = function.take() else {
                    return;
                };
                *source_text = assertion_anonymous_function_text(&function);
                *line = function.span.line;
                let lowered = self.lower_anonymous_function(&function);
                let ValueExpr::Closure {
                    function_index: lowered_index,
                    ..
                } = lowered
                else {
                    return;
                };
                *reflection_text =
                    format!("Closure({})", self.functions[lowered_index].display_name);
                *function_index = Some(lowered_index);
            }
            AttributeArgumentExpression::FirstClassCallable { .. } => {}
            AttributeArgumentExpression::PropertyFetch { receiver, .. } => {
                self.lower_attribute_argument_expression_closures(receiver);
            }
            AttributeArgumentExpression::NewObject { arguments, .. } => {
                for argument in arguments {
                    self.lower_attribute_argument_expression_closures(argument);
                }
            }
            AttributeArgumentExpression::Array(elements) => {
                for element in elements {
                    if let Some(key) = &mut element.key {
                        self.lower_attribute_argument_expression_closures(key);
                    }
                    self.lower_attribute_argument_expression_closures(&mut element.value);
                }
            }
            AttributeArgumentExpression::Unary { expr, .. } => {
                self.lower_attribute_argument_expression_closures(expr);
            }
            AttributeArgumentExpression::Binary { left, right, .. } => {
                self.lower_attribute_argument_expression_closures(left);
                self.lower_attribute_argument_expression_closures(right);
            }
            _ => {}
        }
    }

    fn lower_trait(&mut self, trait_decl: &crate::ast::TraitDecl) -> TraitDecl {
        let trait_attributes =
            self.lower_trait_scoped_attribute_metadata(&trait_decl.attributes, &trait_decl.name);
        let properties = trait_decl
            .properties
            .iter()
            .map(|property| PropertyDecl {
                name: property.name.clone(),
                declaring_class_name: property
                    .declaring_class_name
                    .clone()
                    .unwrap_or_else(|| trait_decl.name.clone()),
                trait_name: property
                    .trait_name
                    .clone()
                    .or_else(|| Some(trait_decl.name.clone())),
                visibility: lower_property_visibility(property.visibility),
                set_visibility: lower_property_visibility(property.set_visibility),
                is_final: property.is_final,
                is_abstract: property.is_abstract,
                is_readonly: property.is_readonly,
                is_promoted: property.is_promoted,
                has_hooks: property.has_hooks,
                is_virtual: property.is_virtual,
                hook_has_get: property.hook_has_get,
                hook_has_set: property.hook_has_set,
                hook_get_is_final: property.hook_get_is_final,
                hook_get_is_abstract: property.hook_get_is_abstract,
                hook_get_returns_by_ref: property.hook_get_returns_by_ref,
                hook_set_is_final: property.hook_set_is_final,
                hook_set_is_abstract: property.hook_set_is_abstract,
                hook_get_attributes: self.lower_trait_property_scoped_attribute_metadata(
                    &property.hook_get_attributes,
                    &trait_decl.name,
                    &property.name,
                ),
                hook_set_attributes: self.lower_trait_property_scoped_attribute_metadata(
                    &property.hook_set_attributes,
                    &trait_decl.name,
                    &property.name,
                ),
                hook_get_doc_comment: property.hook_get_doc_comment.clone(),
                hook_set_doc_comment: property.hook_set_doc_comment.clone(),
                hook_get_line: property
                    .hook_get_span
                    .as_ref()
                    .map(|span| span.line)
                    .unwrap_or(property.span.line),
                hook_set_line: property
                    .hook_set_span
                    .as_ref()
                    .map(|span| span.line)
                    .unwrap_or(property.span.line),
                hook_get_value: property
                    .hook_get_value
                    .as_ref()
                    .map(|value| self.lower_expr(value)),
                hook_set_value: property
                    .hook_set_value
                    .as_ref()
                    .map(|value| self.lower_expr(value)),
                hook_get_body: property
                    .hook_get_body
                    .as_ref()
                    .map(|body| self.lower_statements(body)),
                hook_set_body: property
                    .hook_set_body
                    .as_ref()
                    .map(|body| self.lower_statements(body)),
                hook_set_parameter_name: property.hook_set_parameter_name.clone(),
                hook_set_parameter_attributes: self.lower_trait_property_scoped_attribute_metadata(
                    &property.hook_set_parameter_attributes,
                    &trait_decl.name,
                    &property.name,
                ),
                hook_set_parameter_type: property
                    .hook_set_parameter_type
                    .clone()
                    .map(lower_type_hint),
                hook_set_parameter_doc_comment: property.hook_set_parameter_doc_comment.clone(),
                type_hint: property.type_hint.as_ref().map(lower_property_type_hint),
                attributes: self.lower_trait_property_scoped_attribute_metadata(
                    &property.attributes,
                    &trait_decl.name,
                    &property.name,
                ),
                doc_comment: property.doc_comment.clone(),
                value: property.value.as_ref().map(|value| self.lower_expr(value)),
                line: property.span.line,
                source_order: property.span.byte_start,
            })
            .collect();
        let static_properties = trait_decl
            .static_properties
            .iter()
            .map(|property| StaticPropertyDecl {
                name: property.name.clone(),
                visibility: lower_property_visibility(property.visibility),
                set_visibility: lower_property_visibility(property.set_visibility),
                is_final: property.is_final,
                type_hint: property.type_hint.as_ref().map(lower_property_type_hint),
                attributes: self.lower_trait_property_scoped_attribute_metadata(
                    &property.attributes,
                    &trait_decl.name,
                    &property.name,
                ),
                doc_comment: property.doc_comment.clone(),
                value: property.value.as_ref().map(|value| self.lower_expr(value)),
                source_order: property.span.byte_start,
            })
            .collect();
        let trait_constant_values =
            collect_class_constant_values(&trait_decl.constants, &self.constant_values);
        let trait_constant_deprecations = collect_class_constant_deprecations(
            &trait_decl.constants,
            &self.constant_values,
            &trait_constant_values,
        );
        let constants = trait_decl
            .constants
            .iter()
            .map(|constant| {
                let attributes = self
                    .lower_trait_scoped_attribute_metadata(&constant.attributes, &trait_decl.name);
                let metadata = self.class_deprecated_metadata(
                    &attributes,
                    &trait_decl.name,
                    &constant.name,
                    &trait_constant_values,
                    &trait_constant_deprecations,
                );
                ClassConstantDecl {
                    name: constant.name.clone(),
                    visibility: lower_property_visibility(constant.visibility),
                    type_hint: constant.type_hint.clone().map(lower_type_hint),
                    attributes,
                    doc_comment: constant.doc_comment.clone(),
                    deprecated_message: metadata.message,
                    deprecated_since: metadata.since,
                    deprecated_message_dependency: metadata.message_dependency,
                    deprecated_message_runtime_reference: metadata.message_runtime_reference,
                    is_enum_case: constant.is_enum_case,
                    enum_case_value: constant
                        .enum_case_value
                        .as_ref()
                        .map(|value| self.lower_expr(value)),
                    is_final: constant.is_final,
                    value: self.lower_expr(&constant.value),
                }
            })
            .collect();
        TraitDecl {
            name: trait_decl.name.clone(),
            source_file: self.source_file.clone(),
            trait_uses: lower_trait_uses(&trait_decl.trait_uses),
            attributes: trait_attributes,
            deprecated_message: trait_decl.attributes.deprecated_message.clone(),
            deprecated_since: trait_decl.attributes.deprecated_since.clone(),
            doc_comment: trait_decl.doc_comment.clone(),
            line: trait_decl.span.line,
            end_line: trait_decl.span.end_line,
            initially_declared: true,
            properties,
            static_properties,
            constants,
            methods: trait_decl
                .methods
                .iter()
                .map(|method| {
                    let method_display_name = format!("{}::{}", trait_decl.name, method.name);
                    let previous_function_display_name = std::mem::replace(
                        &mut self.current_function_display_name,
                        Some(method_display_name.clone()),
                    );
                    let method_attributes = self.lower_trait_scoped_attribute_metadata(
                        &method.attributes,
                        &trait_decl.name,
                    );
                    self.current_function_display_name = previous_function_display_name;
                    let parameters = method
                        .parameters
                        .iter()
                        .map(|parameter| {
                            let mut lowered =
                                self.lower_parameter_for_trait_scope(parameter, &trait_decl.name);
                            lowered.default_value = parameter
                                .default_value
                                .as_ref()
                                .map(|value| self.lower_expr(value));
                            lowered
                        })
                        .collect::<Vec<_>>();
                    let deprecated_metadata = self.function_deprecated_metadata(
                        &method_attributes,
                        Some(&trait_decl.name),
                        None,
                        Some(&trait_constant_values),
                    );
                    let function_index = self.functions.len();
                    self.functions.push(FunctionDecl {
                        name: format!("{}::{}", trait_decl.name, method.name),
                        display_name: format!("{}::{}", trait_decl.name, method.name),
                        source_file: self.source_file.clone(),
                        strict_types: self.strict_types,
                        doc_comment: method.doc_comment.clone(),
                        class_name: Some(trait_decl.name.clone()),
                        trait_name: Some(trait_decl.name.clone()),
                        trait_method_name: Some(method.name.clone()),
                        method_name: Some(method.name.clone()),
                        deprecated_message: deprecated_metadata.message,
                        deprecated_since: deprecated_metadata.since,
                        deprecated_message_runtime_reference: deprecated_metadata
                            .message_runtime_reference,
                        no_discard_message: method_attributes.no_discard_message.clone(),
                        attributes: method_attributes.clone(),
                        is_static: method.is_static,
                        line: method.span.line,
                        end_line: method.span.end_line,
                        parameters: parameters.clone(),
                        return_type: method.return_type.clone().map(lower_type_hint),
                        return_by_ref: method.return_by_ref,
                        is_generator: statements_contain_yield(&method.body),
                        is_anonymous: false,
                        initially_declared: false,
                        body: Vec::new(),
                    });
                    let method_display_name = format!("{}::{}", trait_decl.name, method.name);
                    let previous_class_name = std::mem::replace(
                        &mut self.current_class_name,
                        Some(trait_decl.name.clone()),
                    );
                    let previous_class_parent_name =
                        std::mem::replace(&mut self.current_class_parent_name, None);
                    let previous_trait_name = std::mem::replace(
                        &mut self.current_trait_name,
                        Some(trait_decl.name.clone()),
                    );
                    let previous_function_display_name = std::mem::replace(
                        &mut self.current_function_display_name,
                        Some(method_display_name),
                    );
                    let body = self.lower_statements(&method.body);
                    self.current_class_name = previous_class_name;
                    self.current_class_parent_name = previous_class_parent_name;
                    self.current_trait_name = previous_trait_name;
                    self.current_function_display_name = previous_function_display_name;
                    self.functions[function_index].body = body;
                    TraitMethodDecl {
                        name: method.name.clone(),
                        function_index,
                        visibility: lower_property_visibility(method.visibility),
                        attributes: method_attributes,
                        is_static: method.is_static,
                        is_final: method.is_final,
                        is_abstract: method.is_abstract,
                        parameters,
                        line: method.span.line,
                    }
                })
                .collect(),
        }
    }
}

fn resolve_class_scoped_parameter_default(
    value: ValueExpr,
    current_class: &str,
    current_parent: Option<&str>,
) -> ValueExpr {
    match value {
        ValueExpr::ClassConstantFetch {
            class_name,
            name,
            line,
        } if name.eq_ignore_ascii_case("class") && class_name.eq_ignore_ascii_case("self") => {
            let _ = line;
            ValueExpr::String(current_class.to_string())
        }
        ValueExpr::ClassConstantFetch {
            class_name,
            name,
            line,
        } if name.eq_ignore_ascii_case("class") && class_name.eq_ignore_ascii_case("parent") => {
            let _ = line;
            current_parent
                .map(|parent| ValueExpr::String(parent.to_string()))
                .unwrap_or(ValueExpr::ClassConstantFetch {
                    class_name,
                    name,
                    line,
                })
        }
        ValueExpr::Array(elements) => ValueExpr::Array(
            elements
                .into_iter()
                .map(|element| ArrayElement {
                    key: element.key.map(|key| {
                        resolve_class_scoped_parameter_default(key, current_class, current_parent)
                    }),
                    value: match element.value {
                        ArrayElementValue::Value(value) => {
                            ArrayElementValue::Value(resolve_class_scoped_parameter_default(
                                value,
                                current_class,
                                current_parent,
                            ))
                        }
                        ArrayElementValue::Reference(target) => {
                            ArrayElementValue::Reference(target)
                        }
                        ArrayElementValue::Unpack { value, line } => ArrayElementValue::Unpack {
                            value: resolve_class_scoped_parameter_default(
                                value,
                                current_class,
                                current_parent,
                            ),
                            line,
                        },
                    },
                    line: element.line,
                })
                .collect(),
        ),
        ValueExpr::NewObject {
            class_name,
            arguments,
            argument_names,
            argument_unpacks,
            line,
        } => ValueExpr::NewObject {
            class_name,
            arguments: arguments
                .into_iter()
                .map(|argument| {
                    resolve_class_scoped_parameter_default(argument, current_class, current_parent)
                })
                .collect(),
            argument_names,
            argument_unpacks,
            line,
        },
        other => other,
    }
}

fn resolve_attribute_metadata_for_class_scope(
    metadata: &AttributeMetadata,
    current_class: &str,
    current_parent: Option<&str>,
    current_property: Option<&str>,
) -> AttributeMetadata {
    let mut resolved = metadata.clone();
    if let Some(reference) = &mut resolved.deprecated_message_constant {
        resolve_attribute_constant_reference_for_class_scope(
            reference,
            current_class,
            current_parent,
        );
    }
    for instance in &mut resolved.instances {
        for argument in &mut instance.arguments {
            if let Some(expression) = &mut argument.value.expression {
                resolve_attribute_argument_expression_for_class_scope(
                    expression,
                    current_class,
                    current_parent,
                    current_property,
                );
            }
            let Some(AttributeConstantReference::ClassConstant { class_name, name }) =
                &mut argument.value.constant_reference
            else {
                continue;
            };
            let Some(resolved_class) =
                resolve_attribute_class_scope_name(class_name, current_class, current_parent)
            else {
                continue;
            };
            *class_name = resolved_class.clone();
            match &mut argument.value.kind {
                AttributeArgumentKind::String if name.eq_ignore_ascii_case("class") => {
                    argument.value.text = resolved_class;
                }
                AttributeArgumentKind::NativeEnumCase { class_name, .. } => {
                    *class_name = resolved_class.trim_start_matches('\\').to_string();
                }
                _ => {}
            }
        }
    }
    resolved
}

fn resolve_attribute_metadata_for_trait_scope(
    metadata: &AttributeMetadata,
    current_trait: &str,
    current_property: Option<&str>,
) -> AttributeMetadata {
    let mut resolved = metadata.clone();
    for instance in &mut resolved.instances {
        for argument in &mut instance.arguments {
            if let Some(expression) = &mut argument.value.expression {
                resolve_attribute_argument_expression_for_trait_scope(
                    expression,
                    current_trait,
                    current_property,
                );
            }
            let Some(AttributeConstantReference::ClassConstant { class_name, name }) =
                &mut argument.value.constant_reference
            else {
                continue;
            };
            if !name.eq_ignore_ascii_case("class") {
                continue;
            }
            let Some(resolved_class) =
                resolve_attribute_class_scope_name(class_name, current_trait, None)
            else {
                continue;
            };
            *class_name = resolved_class.clone();
            if matches!(argument.value.kind, AttributeArgumentKind::String) {
                argument.value.text = resolved_class;
            }
        }
    }
    resolved
}

fn resolve_attribute_constant_reference_for_class_scope(
    reference: &mut AttributeConstantReference,
    current_class: &str,
    current_parent: Option<&str>,
) {
    let AttributeConstantReference::ClassConstant { class_name, .. } = reference else {
        return;
    };
    if let Some(resolved_class) =
        resolve_attribute_class_scope_name(class_name, current_class, current_parent)
    {
        *class_name = resolved_class;
    }
}

fn resolve_attribute_argument_expression_for_class_scope(
    expression: &mut AttributeArgumentExpression,
    current_class: &str,
    current_parent: Option<&str>,
    current_property: Option<&str>,
) {
    match expression {
        AttributeArgumentExpression::PropertyMagicConstant => {
            *expression =
                AttributeArgumentExpression::String(current_property.unwrap_or("").to_string());
        }
        AttributeArgumentExpression::ClassName { class_name, .. }
        | AttributeArgumentExpression::ClassConstant { class_name, .. } => {
            if let Some(resolved_class) =
                resolve_attribute_class_scope_name(class_name, current_class, current_parent)
            {
                *class_name = resolved_class;
            }
        }
        AttributeArgumentExpression::FirstClassCallable { callable, .. } => {
            resolve_first_class_callable_scope_name(callable, current_class, current_parent);
        }
        AttributeArgumentExpression::PropertyFetch { receiver, .. } => {
            resolve_attribute_argument_expression_for_class_scope(
                receiver,
                current_class,
                current_parent,
                current_property,
            );
        }
        AttributeArgumentExpression::NewObject { arguments, .. } => {
            for argument in arguments {
                resolve_attribute_argument_expression_for_class_scope(
                    argument,
                    current_class,
                    current_parent,
                    current_property,
                );
            }
        }
        AttributeArgumentExpression::Array(elements) => {
            for element in elements {
                if let Some(key) = &mut element.key {
                    resolve_attribute_argument_expression_for_class_scope(
                        key,
                        current_class,
                        current_parent,
                        current_property,
                    );
                }
                resolve_attribute_argument_expression_for_class_scope(
                    &mut element.value,
                    current_class,
                    current_parent,
                    current_property,
                );
            }
        }
        AttributeArgumentExpression::Unary { expr, .. } => {
            resolve_attribute_argument_expression_for_class_scope(
                expr,
                current_class,
                current_parent,
                current_property,
            );
        }
        AttributeArgumentExpression::Binary { left, right, .. } => {
            resolve_attribute_argument_expression_for_class_scope(
                left,
                current_class,
                current_parent,
                current_property,
            );
            resolve_attribute_argument_expression_for_class_scope(
                right,
                current_class,
                current_parent,
                current_property,
            );
        }
        _ => {}
    }
}

fn resolve_attribute_argument_expression_for_trait_scope(
    expression: &mut AttributeArgumentExpression,
    current_trait: &str,
    current_property: Option<&str>,
) {
    match expression {
        AttributeArgumentExpression::PropertyMagicConstant => {
            *expression =
                AttributeArgumentExpression::String(current_property.unwrap_or("").to_string());
        }
        AttributeArgumentExpression::ClassName { class_name, .. } => {
            if let Some(resolved_class) =
                resolve_attribute_class_scope_name(class_name, current_trait, None)
            {
                *class_name = resolved_class;
            }
        }
        AttributeArgumentExpression::ClassConstant {
            class_name, name, ..
        } if name.eq_ignore_ascii_case("class") => {
            if let Some(resolved_class) =
                resolve_attribute_class_scope_name(class_name, current_trait, None)
            {
                *class_name = resolved_class;
            }
        }
        AttributeArgumentExpression::FirstClassCallable { callable, .. } => {
            resolve_first_class_callable_scope_name(callable, current_trait, None);
        }
        AttributeArgumentExpression::PropertyFetch { receiver, .. } => {
            resolve_attribute_argument_expression_for_trait_scope(
                receiver,
                current_trait,
                current_property,
            );
        }
        AttributeArgumentExpression::NewObject { arguments, .. } => {
            for argument in arguments {
                resolve_attribute_argument_expression_for_trait_scope(
                    argument,
                    current_trait,
                    current_property,
                );
            }
        }
        AttributeArgumentExpression::Array(elements) => {
            for element in elements {
                if let Some(key) = &mut element.key {
                    resolve_attribute_argument_expression_for_trait_scope(
                        key,
                        current_trait,
                        current_property,
                    );
                }
                resolve_attribute_argument_expression_for_trait_scope(
                    &mut element.value,
                    current_trait,
                    current_property,
                );
            }
        }
        AttributeArgumentExpression::Unary { expr, .. } => {
            resolve_attribute_argument_expression_for_trait_scope(
                expr,
                current_trait,
                current_property,
            );
        }
        AttributeArgumentExpression::Binary { left, right, .. } => {
            resolve_attribute_argument_expression_for_trait_scope(
                left,
                current_trait,
                current_property,
            );
            resolve_attribute_argument_expression_for_trait_scope(
                right,
                current_trait,
                current_property,
            );
        }
        _ => {}
    }
}

fn resolve_attribute_class_scope_name(
    class_name: &str,
    current_class: &str,
    current_parent: Option<&str>,
) -> Option<String> {
    if class_name.eq_ignore_ascii_case("self") || class_name.eq_ignore_ascii_case("static") {
        Some(current_class.to_string())
    } else if class_name.eq_ignore_ascii_case("parent") {
        current_parent.map(ToString::to_string)
    } else {
        None
    }
}

fn resolve_first_class_callable_scope_name(
    callable: &mut String,
    current_class: &str,
    current_parent: Option<&str>,
) {
    let Some((class_name, method_name)) = callable.split_once("::") else {
        return;
    };
    let Some(resolved_class) =
        resolve_attribute_class_scope_name(class_name, current_class, current_parent)
    else {
        return;
    };
    *callable = format!("{resolved_class}::{method_name}");
}

fn collect_constant_deprecations(
    program: &Program,
    constant_values: &HashMap<String, String>,
) -> HashMap<String, DeprecatedMetadata> {
    let mut constants = HashMap::new();
    collect_constant_deprecations_in(&program.statements, constant_values, &mut constants);
    constants
}

fn collect_constant_deprecations_in(
    statements: &[Statement],
    constant_values: &HashMap<String, String>,
    constants: &mut HashMap<String, DeprecatedMetadata>,
) {
    for statement in statements {
        match statement {
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    if declaration.attributes.deprecated_message.is_some()
                        || declaration.attributes.deprecated_since.is_some()
                    {
                        let metadata = DeprecatedMetadata {
                            message: declaration
                                .attributes
                                .deprecated_message_constant
                                .as_ref()
                                .and_then(|reference| match reference {
                                    AttributeConstantReference::Constant(name) => {
                                        constant_values.get(&name.to_ascii_lowercase()).cloned()
                                    }
                                    AttributeConstantReference::ClassConstant { .. } => None,
                                })
                                .or_else(|| declaration.attributes.deprecated_message.clone()),
                            since: declaration.attributes.deprecated_since.clone(),
                            message_dependency: declaration
                                .attributes
                                .deprecated_message_constant
                                .as_ref()
                                .and_then(|reference| match reference {
                                    AttributeConstantReference::Constant(name) => {
                                        let subject =
                                            format!("Constant {}", name.trim_start_matches('\\'));
                                        let current_subject = format!(
                                            "Constant {}",
                                            declaration.name.trim_start_matches('\\')
                                        );
                                        (subject != current_subject)
                                            .then(|| {
                                                constants
                                                    .get(&name.to_ascii_lowercase())
                                                    .map(|metadata| (subject, metadata))
                                            })
                                            .flatten()
                                    }
                                    AttributeConstantReference::ClassConstant { .. } => None,
                                })
                                .map(|(subject, metadata)| DeprecatedMessageDependency {
                                    subject,
                                    message: metadata.message.clone(),
                                    since: metadata.since.clone(),
                                }),
                            message_runtime_reference: None,
                        };
                        constants.insert(declaration.name.to_ascii_lowercase(), metadata);
                    }
                }
            }
            Statement::Block { statements, .. } => {
                collect_constant_deprecations_in(statements, constant_values, constants);
            }
            _ => {}
        }
    }
}

fn collect_constant_values(program: &Program) -> HashMap<String, String> {
    let mut constants = HashMap::new();
    collect_constant_values_in(&program.statements, &mut constants);
    constants
}

fn collect_constant_values_in(statements: &[Statement], constants: &mut HashMap<String, String>) {
    for statement in statements {
        match statement {
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    if let Some(value) = string_constant_value(&declaration.value, constants, None)
                    {
                        constants.insert(declaration.name.to_ascii_lowercase(), value);
                    }
                }
            }
            Statement::Block { statements, .. } => {
                collect_constant_values_in(statements, constants);
            }
            _ => {}
        }
    }
}

fn expression_statement_is_noop(expression: &Expr) -> bool {
    match expression {
        Expr::Variable(_, _) => true,
        Expr::Grouped { expr, .. } => expression_statement_is_noop(expr),
        _ => false,
    }
}

fn collect_class_constant_values(
    constants: &[crate::ast::ClassConstantDecl],
    global_constants: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for constant in constants {
        if let Some(value) = string_constant_value(&constant.value, global_constants, Some(&values))
        {
            values.insert(constant.name.to_ascii_lowercase(), value);
        }
    }
    values
}

fn collect_class_constant_deprecations(
    constants: &[crate::ast::ClassConstantDecl],
    global_constants: &HashMap<String, String>,
    class_constant_values: &HashMap<String, String>,
) -> HashMap<String, DeprecatedMetadata> {
    let mut deprecations = HashMap::new();
    for constant in constants {
        if constant.attributes.deprecated_message.is_none()
            && constant.attributes.deprecated_since.is_none()
        {
            continue;
        }
        let message = constant
            .attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    global_constants.get(&name.to_ascii_lowercase()).cloned()
                }
                AttributeConstantReference::ClassConstant { name, .. } => class_constant_values
                    .get(&name.to_ascii_lowercase())
                    .cloned(),
            })
            .or_else(|| constant.attributes.deprecated_message.clone());
        deprecations.insert(
            constant.name.to_ascii_lowercase(),
            DeprecatedMetadata {
                message,
                since: constant.attributes.deprecated_since.clone(),
                message_dependency: None,
                message_runtime_reference: None,
            },
        );
    }
    deprecations
}

fn string_constant_value(
    expr: &Expr,
    global_constants: &HashMap<String, String>,
    class_constants: Option<&HashMap<String, String>>,
) -> Option<String> {
    match expr {
        Expr::String(value, _) => Some(value.clone()),
        Expr::Constant(name, _) => global_constants.get(&name.to_ascii_lowercase()).cloned(),
        Expr::ClassConstantFetch { name, .. } => {
            class_constants.and_then(|constants| constants.get(&name.to_ascii_lowercase()).cloned())
        }
        Expr::Grouped { expr, .. } => {
            string_constant_value(expr, global_constants, class_constants)
        }
        _ => None,
    }
}

fn ast_compile_time_condition_truth(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Bool(value, _) => Some(*value),
        Expr::Null(_) => Some(false),
        Expr::Int(value, _) => Some(*value != 0),
        Expr::Float(value, _) => Some(*value != 0.0),
        Expr::String(value, _) => Some(!value.is_empty() && value != "0"),
        Expr::Grouped { expr, .. } => ast_compile_time_condition_truth(expr),
        Expr::Unary {
            op: AstUnaryOp::Not,
            expr,
            ..
        } => ast_compile_time_condition_truth(expr).map(|value| !value),
        Expr::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            ..
        } if name.eq_ignore_ascii_case("class_exists")
            && arguments.len() <= 2
            && argument_names.iter().all(Option::is_none)
            && argument_unpacks.iter().all(|unpack| !unpack) =>
        {
            let class_name = ast_compile_time_string_literal(arguments.first()?)?;
            if ast_known_internal_class(class_name) {
                Some(true)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn ast_compile_time_string_literal(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::String(value, _) => Some(value),
        Expr::Grouped { expr, .. } => ast_compile_time_string_literal(expr),
        _ => None,
    }
}

fn ast_known_internal_class(class_name: &str) -> bool {
    let name = class_name.trim_start_matches('\\');
    [
        "AppendIterator",
        "ArrayIterator",
        "ArrayObject",
        "Attribute",
        "BcMath\\Number",
        "CallbackFilterIterator",
        "Closure",
        "DateInterval",
        "DatePeriod",
        "DateTime",
        "DateTimeImmutable",
        "DateTimeZone",
        "DelayedTargetValidation",
        "Deprecated",
        "DirectoryIterator",
        "EmptyIterator",
        "FilesystemIterator",
        "FilterIterator",
        "Generator",
        "GlobIterator",
        "InfiniteIterator",
        "InternalIterator",
        "IteratorIterator",
        "LimitIterator",
        "MultipleIterator",
        "NoDiscard",
        "NoRewindIterator",
        "ParentIterator",
        "RegexIterator",
        "RecursiveArrayIterator",
        "RecursiveCachingIterator",
        "RecursiveCallbackFilterIterator",
        "RecursiveIteratorIterator",
        "RecursiveRegexIterator",
        "RecursiveTreeIterator",
        "ReflectionClass",
        "ReflectionConstant",
        "ReflectionEnum",
        "ReflectionEnumBackedCase",
        "ReflectionEnumUnitCase",
        "ReflectionExtension",
        "ReflectionFiber",
        "ReflectionFunction",
        "ReflectionMethod",
        "ReflectionObject",
        "ReflectionParameter",
        "ReflectionProperty",
        "ReturnTypeWillChange",
        "SensitiveParameter",
        "SensitiveParameterValue",
        "SplDoublyLinkedList",
        "SplFileInfo",
        "SplFileObject",
        "SplTempFileObject",
        "SplFixedArray",
        "SplHeap",
        "SplMaxHeap",
        "SplMinHeap",
        "SplObjectStorage",
        "SplPriorityQueue",
        "SplQueue",
        "SplStack",
        "stdClass",
    ]
    .iter()
    .any(|known| known.eq_ignore_ascii_case(name))
}

fn lower_compile_warnings(warnings: &[AstCompileWarning]) -> Vec<CompileWarning> {
    warnings
        .iter()
        .map(|warning| CompileWarning {
            message: warning.message.clone(),
            line: warning.span.line,
            kind: lower_compile_warning_kind(warning.kind),
        })
        .collect()
}

fn lower_compile_warning_kind(kind: AstCompileWarningKind) -> CompileWarningKind {
    match kind {
        AstCompileWarningKind::Warning => CompileWarningKind::Warning,
        AstCompileWarningKind::UncaughtError => CompileWarningKind::UncaughtError,
        AstCompileWarningKind::Deprecation => CompileWarningKind::Deprecation,
    }
}

fn lower_trait_uses(trait_uses: &[crate::ast::TraitUseDecl]) -> Vec<TraitUseDecl> {
    trait_uses
        .iter()
        .map(|trait_use| TraitUseDecl {
            name: trait_use.name.clone(),
            aliases: trait_use
                .adaptations
                .iter()
                .filter_map(|adaptation| match adaptation {
                    crate::ast::TraitAdaptation::Alias(alias) => {
                        alias.alias.as_ref().map(|alias_name| TraitAliasDecl {
                            trait_name: alias.method.trait_name.clone(),
                            method_name: alias.method.method_name.clone(),
                            alias: alias_name.clone(),
                        })
                    }
                    crate::ast::TraitAdaptation::Precedence(_) => None,
                })
                .collect(),
            line: trait_use.span.line,
        })
        .collect()
}

fn lower_closure_capture(capture: &AstClosureUseCapture) -> ClosureCapture {
    ClosureCapture {
        name: capture.name.clone(),
        by_ref: capture.by_ref,
        warn_if_missing: capture.warn_if_missing,
        line: capture.span.line,
    }
}

fn lower_property_visibility(visibility: AstPropertyVisibility) -> PropertyVisibility {
    match visibility {
        AstPropertyVisibility::Public => PropertyVisibility::Public,
        AstPropertyVisibility::Protected => PropertyVisibility::Protected,
        AstPropertyVisibility::Private => PropertyVisibility::Private,
    }
}

fn lower_enum_backing_type(backing_type: AstEnumBackingType) -> EnumBackingType {
    match backing_type {
        AstEnumBackingType::Int => EnumBackingType::Int,
        AstEnumBackingType::String => EnumBackingType::String,
    }
}

fn lower_property_type_hint(type_hint: &crate::ast::PropertyTypeHint) -> PropertyTypeHint {
    PropertyTypeHint {
        text: type_hint.text.clone(),
        kind: match &type_hint.kind {
            AstPropertyTypeKind::Null => PropertyTypeKind::Null,
            AstPropertyTypeKind::Array => PropertyTypeKind::Array,
            AstPropertyTypeKind::Int => PropertyTypeKind::Int,
            AstPropertyTypeKind::Float => PropertyTypeKind::Float,
            AstPropertyTypeKind::String => PropertyTypeKind::String,
            AstPropertyTypeKind::Bool => PropertyTypeKind::Bool,
            AstPropertyTypeKind::Mixed => PropertyTypeKind::Mixed,
            AstPropertyTypeKind::Object => PropertyTypeKind::Object,
            AstPropertyTypeKind::Class(name) => PropertyTypeKind::Class(name.clone()),
            AstPropertyTypeKind::Unsupported => PropertyTypeKind::Unsupported,
        },
        allows_null: type_hint.allows_null,
        semantic_type: type_hint.semantic_type.clone().map(lower_type_hint),
    }
}

fn lower_class_property_type_hint(
    class: &AstClassDecl,
    type_hint: &crate::ast::PropertyTypeHint,
) -> PropertyTypeHint {
    let mut lowered = lower_property_type_hint(type_hint);
    if class.is_anonymous
        && matches!(
            &lowered.kind,
            PropertyTypeKind::Class(class_name)
                if class_name.eq_ignore_ascii_case(&class.name)
                    && lowered.text.eq_ignore_ascii_case(&class.name)
        )
    {
        lowered.text = "self".to_string();
    }
    lowered
}

fn class_display_name(class: &AstClassDecl) -> String {
    if class.is_anonymous && !class.name.contains("@anonymous#") {
        format!("{}#ptn", class.name)
    } else {
        class.name.clone()
    }
}

fn lower_type_hint(type_hint: AstTypeHint) -> TypeHint {
    match type_hint {
        AstTypeHint::Null => TypeHint::Null,
        AstTypeHint::Array => TypeHint::Array,
        AstTypeHint::Callable => TypeHint::Callable,
        AstTypeHint::Int => TypeHint::Int,
        AstTypeHint::Float => TypeHint::Float,
        AstTypeHint::String => TypeHint::String,
        AstTypeHint::Bool => TypeHint::Bool,
        AstTypeHint::True => TypeHint::True,
        AstTypeHint::False => TypeHint::False,
        AstTypeHint::Object => TypeHint::Object,
        AstTypeHint::Iterable => TypeHint::Iterable,
        AstTypeHint::Mixed => TypeHint::Mixed,
        AstTypeHint::Void => TypeHint::Void,
        AstTypeHint::Never => TypeHint::Never,
        AstTypeHint::Static => TypeHint::Static,
        AstTypeHint::Nullable(inner) => TypeHint::Nullable(Box::new(lower_type_hint(*inner))),
        AstTypeHint::Union(types) => {
            TypeHint::Union(types.into_iter().map(lower_type_hint).collect())
        }
        AstTypeHint::Intersection(types) => {
            TypeHint::Intersection(types.into_iter().map(lower_type_hint).collect())
        }
        AstTypeHint::Class(name) => TypeHint::Class(name),
    }
}

fn statements_contain_yield(statements: &[Statement]) -> bool {
    statements.iter().any(statement_contains_yield)
}

fn statements_contain_label(statements: &[Statement]) -> bool {
    statements.iter().any(statement_contains_label)
}

fn statement_contains_label(statement: &Statement) -> bool {
    match statement {
        Statement::Label { .. } => true,
        Statement::If {
            then_body,
            else_body,
            ..
        } => statements_contain_label(then_body) || statements_contain_label(else_body),
        Statement::Block { statements, .. }
        | Statement::While {
            body: statements, ..
        }
        | Statement::DoWhile {
            body: statements, ..
        }
        | Statement::Foreach {
            body: statements, ..
        } => statements_contain_label(statements),
        Statement::For {
            initializers,
            updates,
            body,
            ..
        } => {
            statements_contain_label(initializers)
                || statements_contain_label(updates)
                || statements_contain_label(body)
        }
        Statement::Switch { cases, .. } => cases
            .iter()
            .any(|case| statements_contain_label(&case.body)),
        Statement::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            statements_contain_label(body)
                || catches
                    .iter()
                    .any(|catch| statements_contain_label(&catch.body))
                || statements_contain_label(finally_body)
        }
        Statement::Empty { .. }
        | Statement::ClassDeclaration { .. }
        | Statement::TraitDeclaration { .. }
        | Statement::FunctionDeclaration { .. }
        | Statement::Assign { .. }
        | Statement::AssignRef { .. }
        | Statement::ArrayAssign { .. }
        | Statement::ArrayAssignRef { .. }
        | Statement::Call { .. }
        | Statement::Echo { .. }
        | Statement::Print { .. }
        | Statement::Expression { .. }
        | Statement::Return { .. }
        | Statement::Exit { .. }
        | Statement::Throw { .. }
        | Statement::Const { .. }
        | Statement::Static { .. }
        | Statement::Increment { .. }
        | Statement::Unset { .. }
        | Statement::Global { .. }
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::Goto { .. }
        | Statement::InlineHtml { .. } => false,
    }
}

fn statement_contains_yield(statement: &Statement) -> bool {
    match statement {
        Statement::Assign { value, .. }
        | Statement::AssignRef { source: value, .. }
        | Statement::ArrayAssign { value, .. }
        | Statement::ArrayAssignRef { source: value, .. }
        | Statement::Print {
            expression: value, ..
        }
        | Statement::Expression {
            expression: value, ..
        }
        | Statement::Return {
            value: Some(value), ..
        }
        | Statement::Exit {
            value: Some(value), ..
        }
        | Statement::Throw { value, .. } => expr_contains_yield(value),
        Statement::Call { arguments, .. }
        | Statement::Echo {
            expressions: arguments,
            ..
        } => arguments.iter().any(expr_contains_yield),
        Statement::Const { declarations, .. } => declarations
            .iter()
            .any(|declaration| expr_contains_yield(&declaration.value)),
        Statement::Static { declarations, .. } => declarations
            .iter()
            .filter_map(|declaration| declaration.value.as_ref())
            .any(expr_contains_yield),
        Statement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_yield(condition)
                || statements_contain_yield(then_body)
                || statements_contain_yield(else_body)
        }
        Statement::Block { statements, .. }
        | Statement::While {
            body: statements, ..
        }
        | Statement::DoWhile {
            body: statements, ..
        } => statements_contain_yield(statements),
        Statement::For {
            initializers,
            conditions,
            updates,
            body,
            ..
        } => {
            statements_contain_yield(initializers)
                || conditions.iter().any(expr_contains_yield)
                || statements_contain_yield(updates)
                || statements_contain_yield(body)
        }
        Statement::Foreach { iterable, body, .. } => {
            expr_contains_yield(iterable) || statements_contain_yield(body)
        }
        Statement::Switch {
            expression, cases, ..
        } => {
            expr_contains_yield(expression)
                || cases.iter().any(|case| {
                    case.condition.as_ref().is_some_and(expr_contains_yield)
                        || statements_contain_yield(&case.body)
                })
        }
        Statement::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            statements_contain_yield(body)
                || catches
                    .iter()
                    .any(|catch| statements_contain_yield(&catch.body))
                || statements_contain_yield(finally_body)
        }
        Statement::Empty { .. }
        | Statement::ClassDeclaration { .. }
        | Statement::TraitDeclaration { .. }
        | Statement::FunctionDeclaration { .. }
        | Statement::Return { value: None, .. }
        | Statement::Exit { value: None, .. }
        | Statement::Increment { .. }
        | Statement::Unset { .. }
        | Statement::Global { .. }
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::Label { .. }
        | Statement::Goto { .. }
        | Statement::InlineHtml { .. } => false,
    }
}

fn expr_contains_yield(expr: &Expr) -> bool {
    match expr {
        Expr::Yield { .. } | Expr::YieldFrom { .. } => true,
        Expr::AnonymousFunction(_) => false,
        Expr::DynamicVariable { name, .. }
        | Expr::Print {
            expression: name, ..
        }
        | Expr::Include { path: name, .. }
        | Expr::Throw { value: name, .. }
        | Expr::Unary { expr: name, .. }
        | Expr::Cast { expr: name, .. }
        | Expr::Clone { expr: name, .. }
        | Expr::FirstClassCallable { callable: name, .. }
        | Expr::Grouped { expr: name, .. }
        | Expr::PipeValue { expr: name, .. } => expr_contains_yield(name),
        Expr::Exit { value, .. } => value
            .as_ref()
            .is_some_and(|value| expr_contains_yield(value)),
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::IncDec { .. }
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::StaticPropertyFetch { .. }
        | Expr::DynamicStaticPropertyNameFetch { .. }
        | Expr::ClassConstantFetch { .. } => false,
        Expr::ParentPropertyHookCall { arguments, .. } => arguments.iter().any(expr_contains_yield),
        Expr::DynamicStaticPropertyFetch { receiver, .. } => expr_contains_yield(receiver),
        Expr::DynamicClassConstantFetch { receiver, name, .. } => {
            receiver
                .as_ref()
                .is_some_and(|receiver| expr_contains_yield(receiver))
                || expr_contains_yield(name)
        }
        Expr::Assign { target, value, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(value)
        }
        Expr::AssignRef { target, source, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(source)
        }
        Expr::Call { arguments, .. }
        | Expr::DynamicCall { arguments, .. }
        | Expr::MethodCall { arguments, .. }
        | Expr::DynamicMethodCall { arguments, .. }
        | Expr::NewObject { arguments, .. }
        | Expr::DynamicNewObject { arguments, .. } => arguments.iter().any(expr_contains_yield),
        Expr::PropertyFetch { receiver, .. }
        | Expr::NullsafePropertyFetch { receiver, .. }
        | Expr::DynamicClassNameFetch { receiver, .. } => expr_contains_yield(receiver),
        Expr::InstanceOf { expr, target, .. } => {
            expr_contains_yield(expr)
                || matches!(target, AstInstanceOfTarget::Expr(target) if expr_contains_yield(target))
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        Expr::Array { elements, .. } => elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || match &element.value {
                    AstArrayElementValue::Hole(_) => false,
                    AstArrayElementValue::Value(value) | AstArrayElementValue::Unpack(value) => {
                        expr_contains_yield(value)
                    }
                    AstArrayElementValue::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                }
        }),
        Expr::List(list) => list.elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || element.target.as_ref().is_some_and(|target| match target {
                    AstListExprElementTarget::Value(value) => expr_contains_yield(value),
                    AstListExprElementTarget::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                })
        }),
        Expr::ArrayAccess { array, index, .. } => {
            expr_contains_yield(array)
                || index
                    .as_ref()
                    .is_some_and(|index| expr_contains_yield(index))
        }
        Expr::Isset { targets, .. } => targets.iter().any(expr_contains_yield),
        Expr::Empty { target, .. } => expr_contains_yield(target),
        Expr::Binary { left, right, .. } => expr_contains_yield(left) || expr_contains_yield(right),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            expr_contains_yield(condition)
                || if_true
                    .as_ref()
                    .is_some_and(|if_true| expr_contains_yield(if_true))
                || expr_contains_yield(if_false)
        }
        Expr::Match { subject, arms, .. } => {
            expr_contains_yield(subject)
                || arms.iter().any(|arm| {
                    arm.conditions.iter().any(expr_contains_yield)
                        || expr_contains_yield(&arm.value)
                })
        }
    }
}

fn assignment_target_contains_yield(target: &AstAssignmentTarget) -> bool {
    match target {
        AstAssignmentTarget::Variable { .. }
        | AstAssignmentTarget::ArrayDim(_)
        | AstAssignmentTarget::StaticProperty { .. } => false,
        AstAssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
            dimensions.iter().flatten().any(expr_contains_yield)
        }
        AstAssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => expr_contains_yield(array) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        AstAssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => expr_contains_yield(name) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::DynamicPropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            expr_contains_yield(receiver)
                || expr_contains_yield(name)
                || dimensions.iter().flatten().any(expr_contains_yield)
        }
        AstAssignmentTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        AstAssignmentTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        AstAssignmentTarget::DynamicStaticProperty { receiver, .. } => {
            expr_contains_yield(receiver)
        }
        AstAssignmentTarget::DynamicStaticPropertyName { name, .. } => expr_contains_yield(name),
        AstAssignmentTarget::List(list) => list.elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || match &element.target {
                    AstListAssignmentElementTarget::Value(target) => {
                        assignment_target_contains_yield(target)
                    }
                    AstListAssignmentElementTarget::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                }
        }),
    }
}

fn reference_target_contains_yield(target: &AstReferenceTarget) -> bool {
    match target {
        AstReferenceTarget::Variable { .. } | AstReferenceTarget::ArrayDim(_) => false,
        AstReferenceTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        AstReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstReferenceTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        AstReferenceTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
    }
}

impl<'a> LoweringContext<'a> {
    fn lower_assignment_target(&mut self, target: &AstAssignmentTarget) -> AssignmentTarget {
        match target {
            AstAssignmentTarget::Variable { name, span } => AssignmentTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicVariable { name, span } => {
                AssignmentTarget::DynamicVariable {
                    name: Box::new(self.lower_expr(name)),
                    line: span.line,
                }
            }
            AstAssignmentTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => AssignmentTarget::DynamicArrayDim {
                name: Box::new(self.lower_expr(name)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::ArrayDim(target) => AssignmentTarget::ArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: target.span.line,
            },
            AstAssignmentTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => AssignmentTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicPropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => AssignmentTarget::DynamicPropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::StaticPropertyArrayDim {
                class_name,
                name,
                dimensions,
                span,
            } => AssignmentTarget::StaticPropertyArrayDim {
                class_name: class_name.clone(),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicStaticPropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => AssignmentTarget::DynamicStaticPropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::ValueArrayDim {
                array,
                dimensions,
                span,
            } => AssignmentTarget::ValueArrayDim {
                array: Box::new(self.lower_expr(array)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::Property {
                receiver,
                name,
                span,
            } => AssignmentTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => AssignmentTarget::DynamicProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            AstAssignmentTarget::StaticProperty {
                class_name,
                name,
                span,
            } => AssignmentTarget::StaticProperty {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicStaticProperty {
                receiver,
                name,
                span,
            } => AssignmentTarget::DynamicStaticProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicStaticPropertyName {
                class_name,
                name,
                span,
            } => AssignmentTarget::DynamicStaticPropertyName {
                class_name: class_name.clone(),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            AstAssignmentTarget::List(target) => {
                AssignmentTarget::List(self.lower_list_assignment_target(target))
            }
        }
    }

    fn lower_inc_dec_target(&mut self, target: &AstIncDecTarget) -> IncDecTarget {
        match target {
            AstIncDecTarget::Variable { name, span } => IncDecTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstIncDecTarget::DynamicVariable { name, span } => IncDecTarget::DynamicVariable {
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            AstIncDecTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => IncDecTarget::DynamicArrayDim {
                name: Box::new(self.lower_expr(name)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstIncDecTarget::ArrayDim(target) => IncDecTarget::ArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: target.span.line,
            },
            AstIncDecTarget::ValueArrayDim {
                array,
                dimensions,
                span,
            } => IncDecTarget::ValueArrayDim {
                array: Box::new(self.lower_expr(array)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstIncDecTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => IncDecTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstIncDecTarget::StaticPropertyArrayDim {
                class_name,
                name,
                dimensions,
                span,
            } => IncDecTarget::StaticPropertyArrayDim {
                class_name: class_name.clone(),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstIncDecTarget::Property {
                receiver,
                name,
                span,
            } => IncDecTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstIncDecTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => IncDecTarget::DynamicProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            AstIncDecTarget::StaticProperty {
                class_name,
                name,
                span,
            } => IncDecTarget::StaticProperty {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            AstIncDecTarget::DynamicStaticPropertyName {
                class_name,
                name,
                span,
            } => IncDecTarget::DynamicStaticPropertyName {
                class_name: class_name.clone(),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
        }
    }

    fn lower_list_assignment_target(
        &mut self,
        target: &AstListAssignmentTarget,
    ) -> ListAssignmentTarget {
        ListAssignmentTarget {
            elements: target
                .elements
                .iter()
                .map(|element| self.lower_list_assignment_element(element))
                .collect(),
            line: target.span.line,
        }
    }

    fn lower_list_assignment_element(
        &mut self,
        element: &AstListAssignmentElement,
    ) -> ListAssignmentElement {
        ListAssignmentElement {
            key: element.key.as_ref().map(|key| self.lower_expr(key)),
            target: match &element.target {
                AstListAssignmentElementTarget::Value(target) => {
                    ListAssignmentElementTarget::Value(Box::new(
                        self.lower_assignment_target(target),
                    ))
                }
                AstListAssignmentElementTarget::Reference(target) => {
                    ListAssignmentElementTarget::Reference(self.lower_reference_target(target))
                }
            },
        }
    }

    fn lower_reference_target(&mut self, target: &AstReferenceTarget) -> ReferenceTarget {
        match target {
            AstReferenceTarget::Variable { name, span } => ReferenceTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstReferenceTarget::DynamicVariable { name, span } => {
                ReferenceTarget::DynamicVariable {
                    name: Box::new(self.lower_expr(name)),
                    line: span.line,
                }
            }
            AstReferenceTarget::ArrayDim(target) => {
                ReferenceTarget::ArrayDim(self.lower_array_dim_target(target))
            }
            AstReferenceTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => ReferenceTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstReferenceTarget::Property {
                receiver,
                name,
                span,
            } => ReferenceTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstReferenceTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => ReferenceTarget::DynamicProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
        }
    }

    fn lower_unset_target(&mut self, target: &AstUnsetTarget) -> Instruction {
        match target {
            AstUnsetTarget::Variable { name, .. } => {
                Instruction::UnsetVariable { name: name.clone() }
            }
            AstUnsetTarget::DynamicVariable { name, span } => Instruction::UnsetDynamicVariable {
                name: self.lower_expr(name),
                line: span.line,
            },
            AstUnsetTarget::ArrayDim(target) => Instruction::UnsetArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        self.lower_expr(
                            dimension
                                .as_ref()
                                .expect("parser rejects append syntax in unset targets"),
                        )
                    })
                    .collect(),
                line: target.span.line,
            },
            AstUnsetTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => Instruction::UnsetDynamicArrayDim {
                name: self.lower_expr(name),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::ValueArrayDim {
                array,
                dimensions,
                span,
            } => Instruction::UnsetValueArrayDim {
                array: self.lower_expr(array),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => Instruction::UnsetPropertyArrayDim {
                receiver: self.lower_expr(receiver),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::StaticPropertyArrayDim {
                class_name,
                name,
                dimensions,
                span,
            } => Instruction::UnsetStaticPropertyArrayDim {
                class_name: class_name.clone(),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::DynamicStaticPropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => Instruction::UnsetDynamicStaticPropertyArrayDim {
                receiver: self.lower_expr(receiver),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::Property {
                receiver,
                name,
                span,
            } => Instruction::UnsetProperty {
                receiver: self.lower_expr(receiver),
                name: name.clone(),
                line: span.line,
            },
            AstUnsetTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => Instruction::UnsetDynamicProperty {
                receiver: self.lower_expr(receiver),
                name: self.lower_expr(name),
                line: span.line,
            },
            AstUnsetTarget::StaticProperty {
                class_name,
                name,
                span,
            } => Instruction::UnsetStaticProperty {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            AstUnsetTarget::DynamicStaticPropertyName {
                class_name,
                name,
                span,
            } => Instruction::UnsetDynamicStaticPropertyName {
                class_name: class_name.clone(),
                name: self.lower_expr(name),
                line: span.line,
            },
        }
    }
}

fn assignment_op_binary_op(op: AssignmentOp) -> Option<BinaryOp> {
    match op {
        AssignmentOp::Assign => None,
        AssignmentOp::CoalesceAssign => {
            unreachable!("null coalescing assignment lowers through ValueExpr::Assign")
        }
        AssignmentOp::AddAssign => Some(BinaryOp::Add),
        AssignmentOp::SubtractAssign => Some(BinaryOp::Subtract),
        AssignmentOp::MultiplyAssign => Some(BinaryOp::Multiply),
        AssignmentOp::PowerAssign => Some(BinaryOp::Power),
        AssignmentOp::DivideAssign => Some(BinaryOp::Divide),
        AssignmentOp::ModuloAssign => Some(BinaryOp::Modulo),
        AssignmentOp::ConcatAssign => Some(BinaryOp::Concat),
        AssignmentOp::BitwiseAndAssign => Some(BinaryOp::BitwiseAnd),
        AssignmentOp::BitwiseOrAssign => Some(BinaryOp::BitwiseOr),
        AssignmentOp::BitwiseXorAssign => Some(BinaryOp::BitwiseXor),
        AssignmentOp::ShiftLeftAssign => Some(BinaryOp::ShiftLeft),
        AssignmentOp::ShiftRightAssign => Some(BinaryOp::ShiftRight),
    }
}

impl<'a> LoweringContext<'a> {
    fn lower_catch_clause(&mut self, catch: &AstCatchClause) -> CatchClause {
        CatchClause {
            type_names: catch.type_names.clone(),
            variable: catch.variable.clone(),
            body: self.lower_statements(&catch.body),
            line: catch.span.line,
        }
    }

    fn lower_assignment_expr_value(
        &mut self,
        target: AssignmentTarget,
        op: AssignmentOp,
        value: &Expr,
    ) -> (AssignmentOp, ValueExpr) {
        match op {
            AssignmentOp::Assign | AssignmentOp::CoalesceAssign => (op, self.lower_expr(value)),
            AssignmentOp::AddAssign
            | AssignmentOp::SubtractAssign
            | AssignmentOp::MultiplyAssign
            | AssignmentOp::PowerAssign
            | AssignmentOp::DivideAssign
            | AssignmentOp::ModuloAssign
            | AssignmentOp::ConcatAssign
            | AssignmentOp::BitwiseAndAssign
            | AssignmentOp::BitwiseOrAssign
            | AssignmentOp::BitwiseXorAssign
            | AssignmentOp::ShiftLeftAssign
            | AssignmentOp::ShiftRightAssign => match target {
                AssignmentTarget::Variable { .. } => (op, self.lower_expr(value)),
                AssignmentTarget::ArrayDim { .. }
                | AssignmentTarget::DynamicVariable { .. }
                | AssignmentTarget::DynamicArrayDim { .. }
                | AssignmentTarget::PropertyArrayDim { .. }
                | AssignmentTarget::DynamicPropertyArrayDim { .. }
                | AssignmentTarget::StaticPropertyArrayDim { .. }
                | AssignmentTarget::DynamicStaticPropertyArrayDim { .. }
                | AssignmentTarget::ValueArrayDim { .. }
                | AssignmentTarget::Property { .. }
                | AssignmentTarget::DynamicProperty { .. }
                | AssignmentTarget::StaticProperty { .. }
                | AssignmentTarget::DynamicStaticProperty { .. } => (op, self.lower_expr(value)),
                AssignmentTarget::DynamicStaticPropertyName { .. } => {
                    unreachable!("parser rejects compound assignment expression targets")
                }
                AssignmentTarget::List(_) => {
                    unreachable!("parser rejects compound assignment expression targets")
                }
            },
        }
    }
}

impl<'a> LoweringContext<'a> {
    fn lower_instanceof_target(&mut self, target: &AstInstanceOfTarget) -> InstanceOfTarget {
        match target {
            AstInstanceOfTarget::ClassName { name, .. } => {
                InstanceOfTarget::ClassName(name.clone())
            }
            AstInstanceOfTarget::Expr(expr) => {
                InstanceOfTarget::Expr(Box::new(self.lower_expr(expr)))
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> ValueExpr {
        match expr {
            Expr::String(value, _) => ValueExpr::String(value.clone()),
            Expr::InterpolatedString(parts, span) => {
                lower_interpolated_string(parts, span.line, |expr| self.lower_expr(expr))
            }
            Expr::ShellExec { command, span } => ValueExpr::InternalCall {
                name: "__ptn_backtick_exec".to_string(),
                arguments: vec![ValueExpr::String(command.clone())],
                argument_names: vec![None],
                argument_unpacks: vec![false],
                allow_global_fallback: true,
                line: span.line,
            },
            Expr::Int(value, _) => ValueExpr::Int(*value),
            Expr::Float(value, _) => ValueExpr::Float(*value),
            Expr::Bool(value, _) => ValueExpr::Bool(*value),
            Expr::Null(_) => ValueExpr::Null,
            Expr::Variable(name, span) => ValueExpr::Load {
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicVariable { name, span } => ValueExpr::DynamicVariable {
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            Expr::AnonymousFunction(function) => self.lower_anonymous_function(function),
            Expr::IncDec {
                target,
                op,
                result,
                span,
            } => ValueExpr::IncDec {
                target: self.lower_inc_dec_target(target),
                op: lower_inc_dec_op(*op),
                result: lower_inc_dec_result(*result),
                line: span.line,
            },
            Expr::Assign {
                target, op, value, ..
            } => {
                let target = self.lower_assignment_target(target);
                let (op, value) = self.lower_assignment_expr_value(target.clone(), *op, value);
                ValueExpr::Assign {
                    target,
                    op,
                    value: Box::new(value),
                }
            }
            Expr::AssignRef { target, source, .. } => ValueExpr::AssignRef {
                target: self.lower_assignment_target(target),
                source: Box::new(self.lower_expr(source)),
            },
            Expr::Constant(name, span) => {
                let metadata = self.constant_deprecations.get(&name.to_ascii_lowercase());
                ValueExpr::Constant {
                    name: name.clone(),
                    deprecated_message: metadata.and_then(|metadata| metadata.message.clone()),
                    deprecated_since: metadata.and_then(|metadata| metadata.since.clone()),
                    deprecated_message_dependency: metadata
                        .and_then(|metadata| metadata.message_dependency.clone()),
                    line: span.line,
                }
            }
            Expr::MagicConstant(kind, span) => ValueExpr::MagicConstant {
                kind: lower_magic_constant_kind(*kind),
                line: span.line,
            },
            Expr::Array { elements, .. } => ValueExpr::Array(
                elements
                    .iter()
                    .map(|element| self.lower_array_element(element))
                    .collect(),
            ),
            Expr::List(_) => {
                unreachable!("list destructuring syntax must lower through assignment targets")
            }
            Expr::ArrayAccess { array, index, span } => match index {
                Some(index) => ValueExpr::ArrayAccess {
                    array: Box::new(self.lower_expr(array)),
                    index: Box::new(self.lower_expr(index)),
                    line: span.line,
                },
                None => ValueExpr::ArrayAppendAccess {
                    array: Box::new(self.lower_expr(array)),
                    line: span.line,
                },
            },
            Expr::Isset { targets, .. } => ValueExpr::Isset {
                targets: targets
                    .iter()
                    .map(|target| self.lower_expr(target))
                    .collect(),
            },
            Expr::Empty { target, .. } => ValueExpr::Empty {
                target: Box::new(self.lower_expr(target)),
            },
            Expr::Print { expression, .. } => ValueExpr::Print {
                expression: Box::new(self.lower_expr(expression)),
            },
            Expr::Include { kind, path, span } => ValueExpr::Include {
                kind: *kind,
                path: Box::new(self.lower_expr(path)),
                candidates: self.include_candidates(*span),
                line: span.line,
            },
            Expr::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                call_line: _,
                span,
                ..
            } if name.eq_ignore_ascii_case("opcache_compile_file")
                && arguments.len() == 1
                && argument_names.iter().all(Option::is_none)
                && argument_unpacks.iter().all(|unpack| !*unpack) =>
            {
                ValueExpr::OpcacheCompileFile {
                    path: Box::new(self.lower_expr(&arguments[0])),
                    candidates: self.include_candidates(*span),
                    line: span.line,
                }
            }
            Expr::Exit { value, span } => ValueExpr::Exit {
                value: value.as_ref().map(|value| Box::new(self.lower_expr(value))),
                line: span.line,
            },
            Expr::Throw { value, span } => ValueExpr::Throw {
                value: Box::new(self.lower_expr(value)),
                line: span.line,
            },
            Expr::Yield { key, value, span } => ValueExpr::Yield {
                key: key.as_ref().map(|key| Box::new(self.lower_expr(key))),
                value: value.as_ref().map(|value| Box::new(self.lower_expr(value))),
                line: span.line,
            },
            Expr::YieldFrom { expr, span } => ValueExpr::YieldFrom {
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
            Expr::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                allow_global_fallback,
                call_line,
                span: _,
            } => {
                let (arguments, argument_names) =
                    self.lower_internal_call_arguments(name, arguments, argument_names);
                ValueExpr::InternalCall {
                    name: name.clone(),
                    arguments,
                    argument_names,
                    argument_unpacks: argument_unpacks.clone(),
                    allow_global_fallback: *allow_global_fallback,
                    line: *call_line,
                }
            }
            Expr::FirstClassCallable { callable, span } => ValueExpr::FirstClassCallable {
                callable: Box::new(self.lower_expr(callable)),
                line: span.line,
            },
            Expr::DynamicCall {
                callee,
                arguments,
                argument_names,
                argument_unpacks,
                static_method_syntax,
                call_line,
                span: _,
            } => ValueExpr::DynamicCall {
                callee: Box::new(self.lower_expr(callee)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                static_method_syntax: *static_method_syntax,
                line: *call_line,
            },
            Expr::MethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                nullsafe,
                call_line,
                span: _,
            } => ValueExpr::MethodCall {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                nullsafe: *nullsafe,
                line: *call_line,
            },
            Expr::DynamicMethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                call_line,
                span: _,
            } => ValueExpr::DynamicMethodCall {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: *call_line,
            },
            Expr::NewObject {
                class_name,
                source_name: _,
                arguments,
                argument_names,
                argument_unpacks,
                anonymous_class_source: _,
                call_line,
                span: _,
                ..
            } => ValueExpr::NewObject {
                class_name: class_name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: *call_line,
            },
            Expr::DynamicNewObject {
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                call_line,
                span: _,
                ..
            } => ValueExpr::DynamicNewObject {
                class_name: Box::new(self.lower_expr(class_name)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: *call_line,
            },
            Expr::Clone {
                expr,
                with_properties,
                span,
            } => ValueExpr::Clone {
                expr: Box::new(self.lower_expr(expr)),
                with_properties: with_properties
                    .as_ref()
                    .map(|properties| Box::new(self.lower_expr(properties))),
                line: span.line,
            },
            Expr::PropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::PropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::NullsafePropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::NullsafePropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicPropertyFetch {
                receiver,
                name,
                nullsafe,
                span,
            } => ValueExpr::DynamicPropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                nullsafe: *nullsafe,
                line: span.line,
            },
            Expr::StaticPropertyFetch {
                class_name,
                name,
                span,
            } => ValueExpr::StaticPropertyFetch {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicStaticPropertyNameFetch {
                class_name,
                name,
                span,
            } => ValueExpr::DynamicStaticPropertyNameFetch {
                class_name: class_name.clone(),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            Expr::ParentPropertyHookCall {
                property_name,
                hook_name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => ValueExpr::ParentPropertyHookCall {
                property_name: property_name.clone(),
                hook_name: hook_name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::DynamicStaticPropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::DynamicStaticPropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::ClassConstantFetch {
                class_name,
                name,
                span,
            } => ValueExpr::ClassConstantFetch {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicClassConstantFetch {
                class_name,
                receiver,
                name,
                span,
            } => ValueExpr::DynamicClassConstantFetch {
                class_name: class_name.clone(),
                receiver: receiver
                    .as_ref()
                    .map(|receiver| Box::new(self.lower_expr(receiver))),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            Expr::DynamicClassNameFetch {
                receiver,
                allow_string,
                span,
            } => ValueExpr::DynamicClassNameFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                allow_string: *allow_string,
                line: span.line,
            },
            Expr::InstanceOf { expr, target, span } => ValueExpr::InstanceOf {
                expr: Box::new(self.lower_expr(expr)),
                target: self.lower_instanceof_target(target),
                line: span.line,
            },
            Expr::Unary { op, expr, span } => ValueExpr::Unary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
            Expr::Cast { kind, expr, span } => ValueExpr::Cast {
                kind: lower_cast_kind(*kind),
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => ValueExpr::Binary {
                op: lower_binary_op(*op),
                left: Box::new(self.lower_expr(left)),
                right: Box::new(self.lower_expr(right)),
                line: span.line,
            },
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => ValueExpr::Ternary {
                condition: Box::new(self.lower_expr(condition)),
                if_true: if_true
                    .as_ref()
                    .map(|if_true| Box::new(self.lower_expr(if_true))),
                if_false: Box::new(self.lower_expr(if_false)),
                line: span.line,
            },
            Expr::Match {
                subject,
                arms,
                span,
            } => ValueExpr::Match {
                subject: Box::new(self.lower_expr(subject)),
                arms: arms.iter().map(|arm| self.lower_match_arm(arm)).collect(),
                line: span.line,
            },
            Expr::Grouped { expr, .. } => self.lower_expr(expr),
            Expr::PipeValue { expr, span } => ValueExpr::PipeValue {
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
        }
    }

    fn lower_match_arm(&mut self, arm: &AstMatchArm) -> MatchArm {
        MatchArm {
            conditions: arm
                .conditions
                .iter()
                .map(|condition| self.lower_expr(condition))
                .collect(),
            value: self.lower_expr(&arm.value),
            is_default: arm.is_default,
            line: arm.span.line,
        }
    }

    fn include_candidates(&self, span: crate::diagnostic::SourceSpan) -> Vec<usize> {
        self.include_resolutions
            .get(&(self.source_file.clone(), span.byte_start, span.byte_end))
            .cloned()
            .expect("include expressions require include-aware lowering")
    }

    fn lower_array_element(&mut self, element: &AstArrayElement) -> ArrayElement {
        ArrayElement {
            key: element.key.as_ref().map(|key| self.lower_expr(key)),
            value: self.lower_array_element_value(&element.value),
            line: element.line,
        }
    }

    fn lower_array_element_value(&mut self, value: &AstArrayElementValue) -> ArrayElementValue {
        match value {
            AstArrayElementValue::Hole(_) => {
                unreachable!("array holes are rejected outside list destructuring")
            }
            AstArrayElementValue::Value(value) => ArrayElementValue::Value(self.lower_expr(value)),
            AstArrayElementValue::Reference(target) => {
                ArrayElementValue::Reference(self.lower_reference_target(target))
            }
            AstArrayElementValue::Unpack(value) => ArrayElementValue::Unpack {
                value: self.lower_expr(value),
                line: value.span().line,
            },
        }
    }

    fn lower_internal_call_arguments(
        &mut self,
        name: &str,
        arguments: &[Expr],
        argument_names: &[Option<String>],
    ) -> (Vec<ValueExpr>, Vec<Option<String>>) {
        let mut lowered_arguments: Vec<_> = arguments
            .iter()
            .map(|argument| self.lower_expr(argument))
            .collect();
        let mut lowered_names = argument_names.to_vec();

        if name.eq_ignore_ascii_case("assert") {
            let description_supplied = argument_names
                .iter()
                .any(|argument_name| argument_name.as_deref() == Some("description"))
                || (arguments.len() >= 2 && argument_names.get(1).is_none_or(Option::is_none));
            let assertion_index = arguments.iter().enumerate().find_map(|(index, _)| {
                match argument_names.get(index).and_then(|name| name.as_deref()) {
                    Some("assertion") => Some(index),
                    Some(_) => None,
                    None if index == 0 => Some(index),
                    None => None,
                }
            });

            if let Some(assertion_index) = assertion_index {
                if !description_supplied {
                    let description = if argument_names
                        .get(assertion_index)
                        .and_then(|name| name.as_deref())
                        == Some("assertion")
                    {
                        format!(
                            "assert(assertion: {})",
                            assertion_expr_text(&arguments[assertion_index])
                        )
                    } else {
                        format!(
                            "assert({})",
                            assertion_expr_text(&arguments[assertion_index])
                        )
                    };
                    lowered_arguments.push(ValueExpr::String(description));
                    lowered_names.push(
                        argument_names
                            .iter()
                            .any(Option::is_some)
                            .then(|| "description".to_string()),
                    );
                }
            } else if argument_names
                .iter()
                .any(|argument_name| argument_name.as_deref() == Some("description"))
            {
                lowered_arguments.push(ValueExpr::Null);
                lowered_names.push(Some("description".to_string()));
            }
        }

        (lowered_arguments, lowered_names)
    }
}

fn lower_interpolated_string(
    parts: &[AstStringPart],
    line: usize,
    mut lower_expr: impl FnMut(&Expr) -> ValueExpr,
) -> ValueExpr {
    let mut values = parts.iter().filter_map(|part| match part {
        AstStringPart::Literal(value) if value.is_empty() => None,
        AstStringPart::Literal(value) => Some(ValueExpr::String(value.clone())),
        AstStringPart::Variable(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::Load {
                name: name.clone(),
                line,
            }),
            line,
        }),
        AstStringPart::Expression(expr) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(lower_expr(expr)),
            line,
        }),
        AstStringPart::LegacyDollarBraceVariable(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::LegacyDollarBraceStringVariable {
                name: name.clone(),
                line,
            }),
            line,
        }),
        AstStringPart::LegacyDollarBraceExpression(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::LegacyDollarBraceExpressionVariable {
                name: Box::new(lower_expr(name)),
                line,
            }),
            line,
        }),
        AstStringPart::DynamicVariableExpression(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::DynamicVariable {
                name: Box::new(lower_expr(name)),
                line,
            }),
            line,
        }),
        AstStringPart::PropertyFetch { variable, property } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::PropertyFetch {
                receiver: Box::new(ValueExpr::Load {
                    name: variable.clone(),
                    line,
                }),
                name: property.clone(),
                line,
            }),
            line,
        }),
        AstStringPart::PropertyChain {
            variable,
            properties,
        } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(lower_interpolated_property_chain(
                variable, properties, line,
            )),
            line,
        }),
        AstStringPart::MethodCall { variable, method } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::MethodCall {
                receiver: Box::new(ValueExpr::Load {
                    name: variable.clone(),
                    line,
                }),
                name: method.clone(),
                arguments: Vec::new(),
                argument_names: Vec::new(),
                argument_unpacks: Vec::new(),
                nullsafe: false,
                line,
            }),
            line,
        }),
        AstStringPart::ArrayAccess { array, indices } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(lower_interpolated_array_access(array, indices, line)),
            line,
        }),
    });

    let Some(mut expr) = values.next() else {
        return ValueExpr::String(String::new());
    };

    for next in values {
        expr = ValueExpr::Binary {
            op: BinaryOp::Concat,
            left: Box::new(expr),
            right: Box::new(next),
            line,
        };
    }
    expr
}

const ASSERTION_PIPE_PRECEDENCE: u8 = 11;

fn assertion_binary_precedence(op: AstBinaryOp) -> u8 {
    match op {
        AstBinaryOp::Or => 4,
        AstBinaryOp::And => 5,
        AstBinaryOp::BitwiseOr => 6,
        AstBinaryOp::BitwiseXor => 7,
        AstBinaryOp::BitwiseAnd => 8,
        AstBinaryOp::Equal
        | AstBinaryOp::NotEqual
        | AstBinaryOp::Spaceship
        | AstBinaryOp::Identical
        | AstBinaryOp::NotIdentical => 9,
        AstBinaryOp::Less
        | AstBinaryOp::LessEqual
        | AstBinaryOp::Greater
        | AstBinaryOp::GreaterEqual => 10,
        AstBinaryOp::Concat => 13,
        AstBinaryOp::ShiftLeft | AstBinaryOp::ShiftRight => 18,
        AstBinaryOp::Add | AstBinaryOp::Subtract => 23,
        AstBinaryOp::Multiply | AstBinaryOp::Divide | AstBinaryOp::Modulo => 33,
        AstBinaryOp::Power => 40,
        AstBinaryOp::Coalesce | AstBinaryOp::Xor => 4,
    }
}

fn assertion_expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Grouped { expr, .. } => assertion_expr_precedence(expr),
        Expr::DynamicCall { arguments, .. }
            if matches!(arguments.as_slice(), [Expr::PipeValue { .. }]) =>
        {
            ASSERTION_PIPE_PRECEDENCE
        }
        Expr::InstanceOf { .. } => 10,
        Expr::Binary { op, .. } => assertion_binary_precedence(*op),
        Expr::Ternary { .. } => 3,
        Expr::Assign { .. } | Expr::AssignRef { .. } => 2,
        Expr::Print { .. } => 1,
        _ => u8::MAX,
    }
}

fn assertion_operand_text(expr: &Expr, parent_precedence: u8) -> String {
    if let Expr::Grouped { expr: grouped, .. } = expr {
        if !matches!(grouped.as_ref(), Expr::AnonymousFunction(_))
            && assertion_expr_precedence(grouped) > parent_precedence
        {
            return assertion_expr_text(grouped);
        }
        return assertion_expr_text(expr);
    }
    let text = assertion_expr_text(expr);
    if assertion_expr_precedence(expr) < parent_precedence {
        format!("({text})")
    } else {
        text
    }
}

fn assertion_first_class_callable_text(callable: &Expr) -> String {
    match callable {
        Expr::String(name, _) if name.eq_ignore_ascii_case("clone") => "\\clone".to_string(),
        Expr::String(name, _) if assertion_bare_callable_name(name) => name.clone(),
        _ => assertion_expr_text(callable),
    }
}

fn assertion_bare_callable_name(name: &str) -> bool {
    name.split('\\').all(|segment| {
        let mut chars = segment.chars();
        matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
            && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    })
}

fn assertion_expr_text(expr: &Expr) -> String {
    match expr {
        Expr::String(value, _) => assertion_string_text(value),
        Expr::InterpolatedString(_, _) => "\"\"".to_string(),
        Expr::ShellExec { command, .. } => format!("`{}`", command.replace('`', "\\`")),
        Expr::Int(value, _) => value.to_string(),
        Expr::Float(value, _) => assertion_float_text(*value),
        Expr::Bool(value, _) => value.to_string(),
        Expr::Null(_) => "null".to_string(),
        Expr::Variable(name, _) => format!("${name}"),
        Expr::DynamicVariable { name, .. } => format!("$${}", assertion_expr_text(name)),
        Expr::Constant(name, _) if assertion_is_exit_construct_name(name) => {
            assertion_exit_construct_text(&[])
        }
        Expr::Exit { value, .. } => match value.as_deref() {
            Some(value) => format!("\\exit({})", assertion_expr_text(value)),
            None => assertion_exit_construct_text(&[]),
        },
        Expr::Constant(name, _) => name.clone(),
        Expr::MagicConstant(kind, _) => assertion_magic_constant_text(*kind).to_string(),
        Expr::IncDec {
            target, op, result, ..
        } => assertion_inc_dec_text(target, *op, *result),
        Expr::Assign {
            target, op, value, ..
        } => format!(
            "{} {} {}",
            assertion_assignment_target_text(target),
            assertion_assignment_op_text(*op),
            assertion_expr_text(value)
        ),
        Expr::AssignRef { target, source, .. } => format!(
            "{} =& {}",
            assertion_assignment_target_text(target),
            assertion_expr_text(source)
        ),
        Expr::Call {
            name,
            arguments,
            argument_names: _,
            argument_unpacks: _,
            ..
        } if assertion_is_exit_construct_name(name) => assertion_exit_construct_text(arguments),
        Expr::Call {
            name,
            arguments,
            argument_names,
            argument_unpacks,
            ..
        } => {
            let display_name = if name.eq_ignore_ascii_case("clone") {
                "\\clone"
            } else {
                name
            };
            format!(
                "{}({})",
                display_name,
                assertion_call_argument_list_text(arguments, argument_names, argument_unpacks)
            )
        }
        Expr::FirstClassCallable { callable, .. } => {
            format!("{}(...)", assertion_first_class_callable_text(callable))
        }
        Expr::DynamicCall {
            callee, arguments, ..
        } => {
            if let [Expr::PipeValue { expr, .. }] = arguments.as_slice() {
                format!(
                    "{} |> {}",
                    assertion_operand_text(expr, ASSERTION_PIPE_PRECEDENCE),
                    assertion_operand_text(callee, ASSERTION_PIPE_PRECEDENCE)
                )
            } else {
                format!(
                    "{}({})",
                    assertion_expr_text(callee),
                    assertion_argument_list_text(arguments)
                )
            }
        }
        Expr::MethodCall {
            receiver,
            name,
            arguments,
            nullsafe,
            ..
        } => format!(
            "{}{}{}({})",
            assertion_expr_text(receiver),
            if *nullsafe { "?->" } else { "->" },
            name,
            assertion_argument_list_text(arguments)
        ),
        Expr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => format!(
            "{}->{{{}}}({})",
            assertion_expr_text(receiver),
            assertion_expr_text(name),
            assertion_argument_list_text(arguments)
        ),
        Expr::NewObject {
            source_name,
            arguments,
            anonymous_class_source,
            ..
        } => {
            if let Some(source) = anonymous_class_source {
                assertion_anonymous_class_source_text(source)
            } else {
                format!(
                    "new {source_name}({})",
                    assertion_argument_list_text(arguments)
                )
            }
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => format!(
            "new {}({})",
            assertion_expr_text(class_name),
            assertion_argument_list_text(arguments)
        ),
        Expr::Clone {
            expr,
            with_properties,
            ..
        } => match with_properties.as_deref() {
            Some(with_properties) => format!(
                "\\clone({}, {})",
                assertion_expr_text(expr),
                assertion_expr_text(with_properties)
            ),
            None => format!("\\clone({})", assertion_expr_text(expr)),
        },
        Expr::PropertyFetch { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        Expr::NullsafePropertyFetch { receiver, name, .. } => {
            format!("{}?->{name}", assertion_expr_text(receiver))
        }
        Expr::DynamicPropertyFetch {
            receiver,
            name,
            nullsafe,
            ..
        } => {
            format!(
                "{}{}{{{}}}",
                assertion_expr_text(receiver),
                if *nullsafe { "?->" } else { "->" },
                assertion_expr_text(name)
            )
        }
        Expr::StaticPropertyFetch {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        Expr::DynamicStaticPropertyNameFetch {
            class_name, name, ..
        } => format!("{class_name}::${{{}}}", assertion_expr_text(name)),
        Expr::ParentPropertyHookCall {
            property_name,
            hook_name,
            arguments,
            ..
        } => format!(
            "parent::${property_name}::{hook_name}({})",
            assertion_argument_list_text(arguments)
        ),
        Expr::DynamicStaticPropertyFetch { receiver, name, .. } => {
            format!("{}::${name}", assertion_expr_text(receiver))
        }
        Expr::ClassConstantFetch {
            class_name, name, ..
        } => format!("{class_name}::{name}"),
        Expr::DynamicClassConstantFetch {
            class_name,
            receiver,
            name,
            ..
        } => {
            let name = assertion_expr_text(name);
            match (class_name, receiver) {
                (Some(class_name), _) => format!("{class_name}::{{{name}}}"),
                (None, Some(receiver)) => {
                    format!("{}::{{{name}}}", assertion_expr_text(receiver))
                }
                (None, None) => format!("::{{{name}}}"),
            }
        }
        Expr::DynamicClassNameFetch { receiver, .. } => {
            format!("{}::class", assertion_expr_text(receiver))
        }
        Expr::InstanceOf { expr, target, .. } => {
            let target_text = match target {
                AstInstanceOfTarget::ClassName { source_name, .. } => source_name.clone(),
                AstInstanceOfTarget::Expr(target) => assertion_expr_text(target),
            };
            format!("{} instanceof {target_text}", assertion_expr_text(expr))
        }
        Expr::Array { elements, .. } => format!(
            "[{}]",
            elements
                .iter()
                .map(assertion_array_element_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::List(list) => format!(
            "list({})",
            list.elements
                .iter()
                .map(assertion_list_expr_element_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::ArrayAccess { array, index, .. } => {
            let index = index
                .as_ref()
                .map(|index| assertion_expr_text(index))
                .unwrap_or_default();
            format!("{}[{index}]", assertion_expr_text(array))
        }
        Expr::Isset { targets, .. } => {
            format!("isset({})", assertion_argument_list_text(targets))
        }
        Expr::Empty { target, .. } => format!("empty({})", assertion_expr_text(target)),
        Expr::Print { expression, .. } => format!("print {}", assertion_expr_text(expression)),
        Expr::Include { kind, path, .. } => {
            format!(
                "{} {}",
                assertion_include_kind_text(*kind),
                assertion_expr_text(path)
            )
        }
        Expr::Throw { value, .. } => format!("throw {}", assertion_expr_text(value)),
        Expr::Yield { key, value, .. } => match (key, value) {
            (Some(key), Some(value)) => {
                format!(
                    "yield {} => {}",
                    assertion_expr_text(key),
                    assertion_expr_text(value)
                )
            }
            (None, Some(value)) => format!("yield {}", assertion_expr_text(value)),
            (None, None) => "yield".to_string(),
            (Some(key), None) => format!("yield {} => null", assertion_expr_text(key)),
        },
        Expr::YieldFrom { expr, .. } => format!("yield from {}", assertion_expr_text(expr)),
        Expr::Unary { op, expr, .. } => {
            format!(
                "{}{}",
                assertion_unary_op_text(*op),
                assertion_expr_text(expr)
            )
        }
        Expr::Cast { kind, expr, .. } => {
            if matches!(kind, AstCastKind::Void) {
                format!("(void){}", assertion_expr_text(expr))
            } else {
                format!(
                    "({}) {}",
                    assertion_cast_kind_text(*kind),
                    assertion_expr_text(expr)
                )
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let precedence = assertion_binary_precedence(*op);
            format!(
                "{} {} {}",
                assertion_operand_text(left, precedence),
                assertion_binary_op_text(*op),
                assertion_operand_text(right, precedence)
            )
        }
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            if let Some(if_true) = if_true {
                format!(
                    "{} ? {} : {}",
                    assertion_expr_text(condition),
                    assertion_expr_text(if_true),
                    assertion_expr_text(if_false)
                )
            } else {
                format!(
                    "{} ?: {}",
                    assertion_expr_text(condition),
                    assertion_expr_text(if_false)
                )
            }
        }
        Expr::Match { subject, arms, .. } => format!(
            "match ({}) {{ {} }}",
            assertion_expr_text(subject),
            arms.iter()
                .map(assertion_match_arm_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Grouped { expr, .. } => format!("({})", assertion_expr_text(expr)),
        Expr::PipeValue { expr, .. } => assertion_expr_text(expr),
        Expr::AnonymousFunction(function) => assertion_anonymous_function_text(function),
    }
}

fn assertion_is_exit_construct_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("exit") || name.eq_ignore_ascii_case("die")
}

fn assertion_exit_construct_text(arguments: &[Expr]) -> String {
    format!("\\exit({})", assertion_argument_list_text(arguments))
}

fn assertion_match_arm_text(arm: &AstMatchArm) -> String {
    let conditions = if arm.is_default {
        "default".to_string()
    } else {
        arm.conditions
            .iter()
            .map(assertion_expr_text)
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!("{} => {}", conditions, assertion_expr_text(&arm.value))
}

fn assertion_string_text(value: &str) -> String {
    let mut text = String::with_capacity(value.len() + 2);
    text.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => text.push_str("\\\\"),
            '\'' => text.push_str("\\'"),
            _ => text.push(ch),
        }
    }
    text.push('\'');
    text
}

fn assertion_anonymous_function_text(function: &AstAnonymousFunction) -> String {
    let attributes = assertion_attribute_prefix_text(&function.attributes);
    if function.is_arrow {
        let return_by_ref = if function.return_by_ref { "&" } else { "" };
        let parameters = function
            .parameters
            .iter()
            .map(assertion_function_parameter_text)
            .collect::<Vec<_>>()
            .join(", ");
        let return_type = function
            .return_type
            .as_ref()
            .map(|hint| format!(": {}", assertion_type_hint_text(hint)))
            .unwrap_or_default();
        let body = match function.body.as_slice() {
            [Statement::Return {
                value: Some(value), ..
            }] => assertion_expr_text(value),
            _ => "null".to_string(),
        };

        let static_prefix = if function.is_static { "static " } else { "" };
        return format!(
            "{attributes}{static_prefix}fn{return_by_ref}({parameters}){return_type} => {body}"
        );
    }

    let body = function
        .body
        .iter()
        .filter_map(|statement| assertion_statement_text(statement, "    "))
        .collect::<Vec<_>>();
    let static_prefix = if function.is_static { "static " } else { "" };
    let return_by_ref = if function.return_by_ref { "&" } else { "" };
    let parameters = function
        .parameters
        .iter()
        .map(assertion_function_parameter_text)
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = function
        .return_type
        .as_ref()
        .map(|hint| format!(": {}", assertion_type_hint_text(hint)))
        .unwrap_or_default();
    if body.is_empty() {
        return format!(
            "{attributes}{static_prefix}function {return_by_ref}({parameters}){return_type} {{\n}}"
        );
    }
    format!(
        "{attributes}{static_prefix}function {return_by_ref}({parameters}){return_type} {{\n{}\n}}",
        body.join("\n")
    )
}

fn assertion_attribute_prefix_text(attributes: &AttributeMetadata) -> String {
    if attributes.instances.is_empty() {
        return String::new();
    }
    let mut text = attributes
        .instances
        .iter()
        .map(assertion_attribute_instance_text)
        .collect::<Vec<_>>()
        .join("\n");
    text.push(' ');
    text
}

fn assertion_attribute_instance_text(instance: &crate::ast::AttributeInstance) -> String {
    let arguments = instance
        .arguments
        .iter()
        .map(|argument| {
            let value = assertion_attribute_argument_value_text(&argument.value);
            match &argument.name {
                Some(name) => format!("{name}: {value}"),
                None => value,
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    if arguments.is_empty() {
        format!("#[{}]", instance.name)
    } else {
        format!("#[{}({})]", instance.name, arguments)
    }
}

fn assertion_attribute_argument_value_text(value: &crate::ast::AttributeArgumentValue) -> String {
    value
        .expression
        .as_ref()
        .map(assertion_attribute_argument_expression_text)
        .unwrap_or_else(|| value.text.clone())
}

fn assertion_attribute_argument_expression_text(
    expression: &AttributeArgumentExpression,
) -> String {
    match expression {
        AttributeArgumentExpression::String(value) => assertion_string_text(value),
        AttributeArgumentExpression::Int(value) | AttributeArgumentExpression::Float(value) => {
            value.clone()
        }
        AttributeArgumentExpression::Bool(true) => "true".to_string(),
        AttributeArgumentExpression::Bool(false) => "false".to_string(),
        AttributeArgumentExpression::Null => "NULL".to_string(),
        AttributeArgumentExpression::PropertyMagicConstant => "__PROPERTY__".to_string(),
        AttributeArgumentExpression::Constant(name) => name.clone(),
        AttributeArgumentExpression::ClassName { class_name, .. } => class_name.clone(),
        AttributeArgumentExpression::ClassConstant {
            class_name, name, ..
        } => format!("{class_name}::{name}"),
        AttributeArgumentExpression::PropertyFetch {
            receiver,
            name,
            nullsafe,
            ..
        } => format!(
            "{}{}>{name}",
            assertion_attribute_argument_expression_text(receiver),
            if *nullsafe { "?" } else { "-" }
        ),
        AttributeArgumentExpression::NewObject {
            class_name,
            arguments,
            ..
        } => {
            let arguments = arguments
                .iter()
                .map(assertion_attribute_argument_expression_text)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "new \\{}({})",
                class_name.trim_start_matches('\\'),
                arguments
            )
        }
        AttributeArgumentExpression::Closure {
            function,
            source_text,
            ..
        } => function
            .as_ref()
            .map(|function| assertion_anonymous_function_text(function))
            .unwrap_or_else(|| source_text.clone()),
        AttributeArgumentExpression::FirstClassCallable { callable, .. } => {
            format!("{callable}(...)")
        }
        AttributeArgumentExpression::Array(_) => "array".to_string(),
        AttributeArgumentExpression::Unary { op, expr } => {
            let prefix = match op {
                AstUnaryOp::Positive => "+",
                AstUnaryOp::Negate => "-",
                AstUnaryOp::Not => "!",
                AstUnaryOp::BitwiseNot => "~",
                AstUnaryOp::ErrorSuppress => "@",
            };
            format!(
                "{prefix}{}",
                assertion_attribute_argument_expression_text(expr)
            )
        }
        AttributeArgumentExpression::Binary {
            op, left, right, ..
        } => format!(
            "{} {} {}",
            assertion_attribute_argument_expression_text(left),
            assertion_binary_op_text(*op),
            assertion_attribute_argument_expression_text(right)
        ),
    }
}

fn assertion_statement_text(statement: &Statement, indent: &str) -> Option<String> {
    match statement {
        Statement::ClassDeclaration { source, span, .. }
        | Statement::TraitDeclaration { source, span, .. } => Some(format!(
            "{}\n",
            assertion_source_block_text(source, span.column, indent)
        )),
        Statement::Echo { expressions, .. } => Some(format!(
            "{indent}echo {};",
            expressions
                .iter()
                .map(assertion_expr_text)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        Statement::Assign {
            name, op, value, ..
        } => {
            let value = assertion_statement_value_text(value, indent);
            Some(format!(
                "{indent}${name} {} {value};",
                assertion_assignment_op_text(*op)
            ))
        }
        Statement::Expression { expression, .. } => {
            Some(assertion_expression_statement_text(expression, indent))
        }
        Statement::Return { value, .. } => Some(match value {
            Some(value) => format!("{indent}return {};", assertion_expr_text(value)),
            None => format!("{indent}return;"),
        }),
        Statement::Empty { .. } => None,
        _ => None,
    }
}

fn assertion_statement_value_text(expr: &Expr, indent: &str) -> String {
    let text = assertion_expr_text(expr);
    assertion_indent_continuation_lines(&text, indent)
}

fn assertion_indent_continuation_lines(text: &str, indent: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut formatted = first.to_string();
    for line in lines {
        formatted.push('\n');
        formatted.push_str(indent);
        formatted.push_str(line);
    }
    formatted
}

fn assertion_expression_statement_text(expr: &Expr, indent: &str) -> String {
    match expr {
        Expr::Match { subject, arms, .. } => {
            assertion_match_expression_statement_text(subject, arms, indent)
        }
        _ => format!("{indent}{};", assertion_expr_text(expr)),
    }
}

fn assertion_match_expression_statement_text(
    subject: &Expr,
    arms: &[AstMatchArm],
    indent: &str,
) -> String {
    let arm_indent = format!("{indent}    ");
    let mut text = format!("{indent}match ({}) {{", assertion_expr_text(subject));
    for arm in arms {
        text.push('\n');
        text.push_str(&arm_indent);
        text.push_str(&assertion_match_arm_text(arm));
        text.push(',');
    }
    text.push('\n');
    text.push_str(indent);
    text.push_str("};");
    text
}

fn assertion_source_block_text(source: &str, source_column: usize, indent: &str) -> String {
    let source_indent = source_column.saturating_sub(1);
    let lines = source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line = if index == 0 {
                line
            } else {
                line.get(source_indent..).unwrap_or(line)
            };
            line.to_string()
        })
        .collect::<Vec<_>>();
    assertion_normalize_class_like_source_lines(lines)
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assertion_normalize_class_like_source_lines(lines: Vec<String>) -> Vec<String> {
    if !lines
        .iter()
        .any(|line| assertion_line_starts_method_declaration(line.trim_start()))
    {
        return lines;
    }

    let mut normalized = Vec::with_capacity(lines.len() + 1);
    for (index, line) in lines.iter().enumerate() {
        if line.trim().is_empty()
            && lines
                .get(index + 1)
                .is_some_and(|next| assertion_line_starts_method_declaration(next.trim_start()))
        {
            continue;
        }
        if line == "}"
            && normalized
                .last()
                .is_some_and(|previous: &String| previous.trim() == "}")
            && !normalized
                .last()
                .is_some_and(|previous: &String| previous.is_empty())
        {
            normalized.push(String::new());
        }
        normalized.push(line.clone());
    }
    normalized
}

fn assertion_line_starts_method_declaration(line: &str) -> bool {
    let mut words = line.split_whitespace();
    let mut saw_function = false;
    for word in &mut words {
        let word = word.trim_start_matches('&');
        if word == "function" {
            saw_function = true;
            break;
        }
        if !matches!(
            word,
            "public" | "protected" | "private" | "static" | "final" | "abstract" | "readonly"
        ) {
            break;
        }
    }
    saw_function
}

fn assertion_anonymous_class_source_text(source: &str) -> String {
    let mut lines = source.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        let text = first.to_string();
        return assertion_anonymous_class_constructor_text(&text).unwrap_or(text);
    }
    let strip = rest
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut normalized = Vec::with_capacity(rest.len() + 1);
    normalized.push(first.to_string());
    normalized.extend(rest.into_iter().map(|line| {
        if line.len() >= strip {
            line[strip..].to_string()
        } else {
            line.to_string()
        }
    }));
    let text = assertion_expand_inline_property_hooks(&assertion_normalized_anonymous_class_text(
        normalized.join("\n"),
    ));
    assertion_anonymous_class_constructor_text(&text).unwrap_or(text)
}

fn assertion_normalized_anonymous_class_text(source: String) -> String {
    let mut text = source.replace("]\nclass", "] class");
    let Some(class_open) = text.rfind('{') else {
        return text;
    };
    let Some(class_close) = find_matching_ascii_delimiter(&text, class_open, b'{', b'}') else {
        return text;
    };
    if !text[class_close + 1..].trim().is_empty()
        || !text[class_open + 1..class_close].trim().is_empty()
    {
        return text;
    }
    text.truncate(class_open);
    text.push_str("{\n}");
    text
}

fn assertion_anonymous_class_constructor_text(source: &str) -> Option<String> {
    let class_open = source.find('{')?;
    let class_close = find_matching_ascii_delimiter(source, class_open, b'{', b'}')?;
    if !source[class_close + 1..].trim().is_empty() {
        return None;
    }
    let class_head = source[..class_open].trim_end();
    let class_body = source[class_open + 1..class_close].trim();
    let construct = class_body.find("function __construct")?;
    if !class_body[..construct].trim().is_empty()
        && !class_body[..construct].trim().ends_with("public")
    {
        return None;
    }
    let parameters_open = class_body[construct..].find('(')? + construct;
    let method_prefix = class_body[..parameters_open].trim();
    let parameters_close = find_matching_ascii_delimiter(class_body, parameters_open, b'(', b')')?;
    let method_body_open = class_body[parameters_close + 1..].find('{')? + parameters_close + 1;
    if !class_body[parameters_close + 1..method_body_open]
        .trim()
        .is_empty()
    {
        return None;
    }
    let method_body_close =
        find_matching_ascii_delimiter(class_body, method_body_open, b'{', b'}')?;
    if !class_body[method_body_open + 1..method_body_close]
        .trim()
        .is_empty()
    {
        return None;
    }
    if !class_body[method_body_close + 1..].trim().is_empty() {
        return None;
    }

    let parameters = class_body[parameters_open + 1..parameters_close].trim();
    let parameters = assertion_constructor_parameter_text(parameters);
    Some(format!(
        "{class_head} {{\n    {method_prefix}({parameters}) {{\n    }}\n\n}}"
    ))
}

fn assertion_constructor_parameter_text(parameters: &str) -> String {
    let mut formatted = String::new();
    let mut cursor = 0usize;
    while let Some(relative_open) = parameters[cursor..].find('{') {
        let open = cursor + relative_open;
        let Some(close) = find_matching_ascii_delimiter(parameters, open, b'{', b'}') else {
            break;
        };
        formatted.push_str(&parameters[cursor..open]);
        let hook_body = parameters[open + 1..close].trim();
        formatted.push_str("{\n");
        formatted.push_str(&assertion_property_hook_block_text(hook_body));
        formatted.push('\n');
        formatted.push_str("    }");
        cursor = close + 1;
    }
    formatted.push_str(&parameters[cursor..]);
    formatted.trim().to_string()
}

fn assertion_expand_inline_property_hooks(source: &str) -> String {
    source
        .lines()
        .flat_map(|line| {
            assertion_expand_inline_property_hook_line(line)
                .unwrap_or_else(|| vec![line.to_string()])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assertion_expand_inline_property_hook_line(line: &str) -> Option<Vec<String>> {
    let open = line.find('{')?;
    let close = find_matching_ascii_delimiter(line, open, b'{', b'}')?;
    if !line[close + 1..].trim().is_empty() {
        return None;
    }
    let prefix = &line[..open];
    let prefix_trimmed = prefix.trim_end();
    if !prefix_trimmed.contains('$') || prefix_trimmed.contains("function") {
        return None;
    }
    let body = line[open + 1..close].trim();
    if body.is_empty() || body.contains('{') || body.contains('}') {
        return None;
    }
    let hooks = body
        .split(';')
        .map(str::trim)
        .filter(|hook| !hook.is_empty())
        .collect::<Vec<_>>();
    if hooks.is_empty() {
        return None;
    }
    let indent = &line[..line.len() - line.trim_start().len()];
    let mut expanded = Vec::with_capacity(hooks.len() + 2);
    expanded.push(format!("{prefix_trimmed} {{"));
    expanded.extend(hooks.into_iter().map(|hook| format!("{indent}    {hook};")));
    expanded.push(format!("{indent}}}"));
    Some(expanded)
}

fn assertion_property_hook_block_text(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| format!("        {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn find_matching_ascii_delimiter(
    text: &str,
    open_index: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in text.as_bytes().iter().enumerate().skip(open_index) {
        if *byte == open {
            depth += 1;
        } else if *byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn assertion_float_text(value: f64) -> String {
    let text = value.to_string();
    if value.is_finite()
        && value.fract() == 0.0
        && !text.contains('.')
        && !text.contains('e')
        && !text.contains('E')
    {
        format!("{text}.0")
    } else {
        text
    }
}

fn assertion_argument_list_text(arguments: &[Expr]) -> String {
    arguments
        .iter()
        .map(assertion_expr_text)
        .collect::<Vec<_>>()
        .join(", ")
}

fn assertion_call_argument_list_text(
    arguments: &[Expr],
    argument_names: &[Option<String>],
    argument_unpacks: &[bool],
) -> String {
    arguments
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let mut text = String::new();
            if argument_unpacks.get(index).copied().unwrap_or(false) {
                text.push_str("...");
            }
            if let Some(Some(name)) = argument_names.get(index) {
                text.push_str(name);
                text.push_str(": ");
            }
            text.push_str(&assertion_expr_text(argument));
            text
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn assertion_function_parameter_text(parameter: &AstFunctionParameter) -> String {
    let mut variable = String::new();
    if parameter.by_ref {
        variable.push('&');
    }
    if parameter.is_variadic {
        variable.push_str("...");
    }
    variable.push('$');
    variable.push_str(&parameter.name);

    let mut text = if let Some(type_hint) = &parameter.type_hint {
        format!("{} {variable}", assertion_type_hint_text(type_hint))
    } else {
        variable
    };
    if let Some(default_value) = &parameter.default_value {
        text.push_str(" = ");
        text.push_str(&assertion_expr_text(default_value));
    }
    text
}

fn assertion_type_hint_text(type_hint: &AstTypeHint) -> String {
    match type_hint {
        AstTypeHint::Null => "null".to_string(),
        AstTypeHint::Array => "array".to_string(),
        AstTypeHint::Callable => "callable".to_string(),
        AstTypeHint::Int => "int".to_string(),
        AstTypeHint::Float => "float".to_string(),
        AstTypeHint::String => "string".to_string(),
        AstTypeHint::Bool => "bool".to_string(),
        AstTypeHint::True => "true".to_string(),
        AstTypeHint::False => "false".to_string(),
        AstTypeHint::Object => "object".to_string(),
        AstTypeHint::Iterable => "Traversable|array".to_string(),
        AstTypeHint::Mixed => "mixed".to_string(),
        AstTypeHint::Void => "void".to_string(),
        AstTypeHint::Never => "never".to_string(),
        AstTypeHint::Static => "static".to_string(),
        AstTypeHint::Nullable(inner) => format!("?{}", assertion_type_hint_text(inner)),
        AstTypeHint::Union(types) => types
            .iter()
            .map(assertion_type_hint_text)
            .collect::<Vec<_>>()
            .join("|"),
        AstTypeHint::Intersection(types) => types
            .iter()
            .map(assertion_type_hint_text)
            .collect::<Vec<_>>()
            .join("&"),
        AstTypeHint::Class(name) => name.clone(),
    }
}

fn assertion_array_element_text(element: &AstArrayElement) -> String {
    let value = match &element.value {
        AstArrayElementValue::Hole(_) => String::new(),
        AstArrayElementValue::Value(value) => assertion_expr_text(value),
        AstArrayElementValue::Reference(target) => {
            format!("&{}", assertion_reference_target_text(target))
        }
        AstArrayElementValue::Unpack(value) => format!("...{}", assertion_expr_text(value)),
    };
    if let Some(key) = &element.key {
        format!("{} => {value}", assertion_expr_text(key))
    } else {
        value
    }
}

fn assertion_list_expr_element_text(element: &crate::ast::ListExprElement) -> String {
    let Some(target) = &element.target else {
        return String::new();
    };
    let value = match target {
        AstListExprElementTarget::Value(value) => assertion_expr_text(value),
        AstListExprElementTarget::Reference(target) => {
            format!("&{}", assertion_reference_target_text(target))
        }
    };
    if let Some(key) = &element.key {
        format!("{} => {value}", assertion_expr_text(key))
    } else {
        value
    }
}

fn assertion_assignment_target_text(target: &AstAssignmentTarget) -> String {
    match target {
        AstAssignmentTarget::Variable { name, .. } => format!("${name}"),
        AstAssignmentTarget::DynamicVariable { name, .. } => {
            format!("$${}", assertion_expr_text(name))
        }
        AstAssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            let mut text = format!("$${}", assertion_expr_text(name));
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstAssignmentTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::DynamicPropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            );
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::StaticPropertyArrayDim {
            class_name,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{class_name}::${name}");
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}::${name}", assertion_expr_text(receiver));
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            let mut text = assertion_expr_text(array);
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstAssignmentTarget::DynamicProperty { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
        AstAssignmentTarget::StaticProperty {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        AstAssignmentTarget::DynamicStaticProperty { receiver, name, .. } => {
            format!("{}::${name}", assertion_expr_text(receiver))
        }
        AstAssignmentTarget::DynamicStaticPropertyName {
            class_name, name, ..
        } => {
            format!("{class_name}::${{{}}}", assertion_expr_text(name))
        }
        AstAssignmentTarget::List(_) => "list(...)".to_string(),
    }
}

fn assertion_array_dim_target_text(target: &AstArrayDimTarget) -> String {
    let mut text = format!("${}", target.array);
    for dimension in &target.dimensions {
        let index = dimension
            .as_ref()
            .map(assertion_expr_text)
            .unwrap_or_default();
        text.push('[');
        text.push_str(&index);
        text.push(']');
    }
    text
}

fn assertion_reference_target_text(target: &AstReferenceTarget) -> String {
    match target {
        AstReferenceTarget::Variable { name, .. } => format!("${name}"),
        AstReferenceTarget::DynamicVariable { name, .. } => {
            format!("${{{}}}", assertion_expr_text(name))
        }
        AstReferenceTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstReferenceTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstReferenceTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstReferenceTarget::DynamicProperty { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
    }
}

fn assertion_inc_dec_target_text(target: &AstIncDecTarget) -> String {
    match target {
        AstIncDecTarget::Variable { name, .. } => format!("${name}"),
        AstIncDecTarget::DynamicVariable { .. } => "${...}".to_string(),
        AstIncDecTarget::DynamicArrayDim { .. } => "${...}[...]".to_string(),
        AstIncDecTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstIncDecTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            let mut text = assertion_expr_text(array);
            for dimension in dimensions {
                text.push('[');
                if let Some(dimension) = dimension {
                    text.push_str(&assertion_expr_text(dimension));
                }
                text.push(']');
            }
            text
        }
        AstIncDecTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
            for dimension in dimensions {
                text.push('[');
                if let Some(dimension) = dimension {
                    text.push_str(&assertion_expr_text(dimension));
                }
                text.push(']');
            }
            text
        }
        AstIncDecTarget::StaticPropertyArrayDim {
            class_name,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{class_name}::${name}");
            for dimension in dimensions {
                text.push('[');
                if let Some(dimension) = dimension {
                    text.push_str(&assertion_expr_text(dimension));
                }
                text.push(']');
            }
            text
        }
        AstIncDecTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstIncDecTarget::DynamicProperty { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
        AstIncDecTarget::StaticProperty {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        AstIncDecTarget::DynamicStaticPropertyName {
            class_name, name, ..
        } => {
            format!("{class_name}::${{{}}}", assertion_expr_text(name))
        }
    }
}

fn assertion_assignment_op_text(op: AssignmentOp) -> &'static str {
    match op {
        AssignmentOp::Assign => "=",
        AssignmentOp::AddAssign => "+=",
        AssignmentOp::SubtractAssign => "-=",
        AssignmentOp::MultiplyAssign => "*=",
        AssignmentOp::PowerAssign => "**=",
        AssignmentOp::DivideAssign => "/=",
        AssignmentOp::ModuloAssign => "%=",
        AssignmentOp::ConcatAssign => ".=",
        AssignmentOp::BitwiseAndAssign => "&=",
        AssignmentOp::BitwiseOrAssign => "|=",
        AssignmentOp::BitwiseXorAssign => "^=",
        AssignmentOp::ShiftLeftAssign => "<<=",
        AssignmentOp::ShiftRightAssign => ">>=",
        AssignmentOp::CoalesceAssign => "??=",
    }
}

fn assertion_binary_op_text(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::Add => "+",
        AstBinaryOp::Subtract => "-",
        AstBinaryOp::Multiply => "*",
        AstBinaryOp::Power => "**",
        AstBinaryOp::Divide => "/",
        AstBinaryOp::Modulo => "%",
        AstBinaryOp::Concat => ".",
        AstBinaryOp::Coalesce => "??",
        AstBinaryOp::ShiftLeft => "<<",
        AstBinaryOp::ShiftRight => ">>",
        AstBinaryOp::Equal => "==",
        AstBinaryOp::NotEqual => "!=",
        AstBinaryOp::Spaceship => "<=>",
        AstBinaryOp::Identical => "===",
        AstBinaryOp::NotIdentical => "!==",
        AstBinaryOp::Less => "<",
        AstBinaryOp::LessEqual => "<=",
        AstBinaryOp::Greater => ">",
        AstBinaryOp::GreaterEqual => ">=",
        AstBinaryOp::BitwiseAnd => "&",
        AstBinaryOp::BitwiseXor => "^",
        AstBinaryOp::BitwiseOr => "|",
        AstBinaryOp::And => "&&",
        AstBinaryOp::Xor => "xor",
        AstBinaryOp::Or => "||",
    }
}

fn assertion_unary_op_text(op: AstUnaryOp) -> &'static str {
    match op {
        AstUnaryOp::Positive => "+",
        AstUnaryOp::Negate => "-",
        AstUnaryOp::Not => "!",
        AstUnaryOp::BitwiseNot => "~",
        AstUnaryOp::ErrorSuppress => "@",
    }
}

fn assertion_inc_dec_text(
    target: &AstIncDecTarget,
    op: AstIncDecOp,
    result: AstIncDecResult,
) -> String {
    let op = match op {
        AstIncDecOp::Increment => "++",
        AstIncDecOp::Decrement => "--",
    };
    let target = assertion_inc_dec_target_text(target);
    match result {
        AstIncDecResult::Pre => format!("{op}{target}"),
        AstIncDecResult::Post => format!("{target}{op}"),
    }
}

fn assertion_cast_kind_text(kind: AstCastKind) -> &'static str {
    match kind {
        AstCastKind::Int => "int",
        AstCastKind::Integer => "integer",
        AstCastKind::Float => "float",
        AstCastKind::Double => "double",
        AstCastKind::String => "string",
        AstCastKind::Binary => "binary",
        AstCastKind::Bool => "bool",
        AstCastKind::Boolean => "boolean",
        AstCastKind::Array => "array",
        AstCastKind::Object => "object",
        AstCastKind::Void => "void",
    }
}

fn assertion_magic_constant_text(kind: AstMagicConstantKind) -> &'static str {
    match kind {
        AstMagicConstantKind::File => "__FILE__",
        AstMagicConstantKind::Dir => "__DIR__",
        AstMagicConstantKind::Line => "__LINE__",
        AstMagicConstantKind::Function => "__FUNCTION__",
        AstMagicConstantKind::Class => "__CLASS__",
        AstMagicConstantKind::Method => "__METHOD__",
        AstMagicConstantKind::Trait => "__TRAIT__",
        AstMagicConstantKind::Namespace => "__NAMESPACE__",
        AstMagicConstantKind::Property => "__PROPERTY__",
    }
}

fn assertion_include_kind_text(kind: crate::ast::IncludeKind) -> &'static str {
    match kind {
        crate::ast::IncludeKind::Include => "include",
        crate::ast::IncludeKind::IncludeOnce => "include_once",
        crate::ast::IncludeKind::Require => "require",
        crate::ast::IncludeKind::RequireOnce => "require_once",
    }
}

fn lower_interpolated_array_access(
    array: &str,
    indices: &[AstStringInterpolationIndex],
    line: usize,
) -> ValueExpr {
    let mut expr = ValueExpr::Load {
        name: array.to_string(),
        line,
    };
    for index in indices {
        expr = ValueExpr::ArrayAccess {
            array: Box::new(expr),
            index: Box::new(lower_interpolated_array_index(index, line)),
            line,
        };
    }
    expr
}

fn lower_interpolated_property_chain(
    variable: &str,
    properties: &[String],
    line: usize,
) -> ValueExpr {
    let mut expr = ValueExpr::Load {
        name: variable.to_string(),
        line,
    };
    for property in properties {
        expr = ValueExpr::PropertyFetch {
            receiver: Box::new(expr),
            name: property.clone(),
            line,
        };
    }
    expr
}

fn lower_interpolated_array_index(index: &AstStringInterpolationIndex, line: usize) -> ValueExpr {
    match index {
        AstStringInterpolationIndex::String(value) => ValueExpr::String(value.clone()),
        AstStringInterpolationIndex::Int(value) => ValueExpr::Int(*value),
        AstStringInterpolationIndex::Variable(name) => ValueExpr::Load {
            name: name.clone(),
            line,
        },
    }
}

fn lower_unary_op(op: AstUnaryOp) -> UnaryOp {
    match op {
        AstUnaryOp::Positive => UnaryOp::Positive,
        AstUnaryOp::Negate => UnaryOp::Negate,
        AstUnaryOp::Not => UnaryOp::Not,
        AstUnaryOp::BitwiseNot => UnaryOp::BitwiseNot,
        AstUnaryOp::ErrorSuppress => UnaryOp::ErrorSuppress,
    }
}

fn lower_cast_kind(kind: AstCastKind) -> CastKind {
    match kind {
        AstCastKind::Int => CastKind::Int,
        AstCastKind::Integer => CastKind::Integer,
        AstCastKind::Float => CastKind::Float,
        AstCastKind::Double => CastKind::Double,
        AstCastKind::String => CastKind::String,
        AstCastKind::Binary => CastKind::Binary,
        AstCastKind::Bool => CastKind::Bool,
        AstCastKind::Boolean => CastKind::Boolean,
        AstCastKind::Array => CastKind::Array,
        AstCastKind::Object => CastKind::Object,
        AstCastKind::Void => CastKind::Void,
    }
}

fn lower_magic_constant_kind(kind: AstMagicConstantKind) -> MagicConstantKind {
    match kind {
        AstMagicConstantKind::Line => MagicConstantKind::Line,
        AstMagicConstantKind::File => MagicConstantKind::File,
        AstMagicConstantKind::Dir => MagicConstantKind::Dir,
        AstMagicConstantKind::Function => MagicConstantKind::Function,
        AstMagicConstantKind::Method => MagicConstantKind::Method,
        AstMagicConstantKind::Class => MagicConstantKind::Class,
        AstMagicConstantKind::Trait => MagicConstantKind::Trait,
        AstMagicConstantKind::Namespace => MagicConstantKind::Namespace,
        AstMagicConstantKind::Property => MagicConstantKind::Property,
    }
}

fn lower_inc_dec_op(op: AstIncDecOp) -> IncDecOp {
    match op {
        AstIncDecOp::Increment => IncDecOp::Increment,
        AstIncDecOp::Decrement => IncDecOp::Decrement,
    }
}

fn lower_inc_dec_result(result: AstIncDecResult) -> IncDecResult {
    match result {
        AstIncDecResult::Pre => IncDecResult::Pre,
        AstIncDecResult::Post => IncDecResult::Post,
    }
}

fn lower_binary_op(op: AstBinaryOp) -> BinaryOp {
    match op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Subtract => BinaryOp::Subtract,
        AstBinaryOp::Multiply => BinaryOp::Multiply,
        AstBinaryOp::Power => BinaryOp::Power,
        AstBinaryOp::Divide => BinaryOp::Divide,
        AstBinaryOp::Modulo => BinaryOp::Modulo,
        AstBinaryOp::Concat => BinaryOp::Concat,
        AstBinaryOp::Coalesce => BinaryOp::Coalesce,
        AstBinaryOp::ShiftLeft => BinaryOp::ShiftLeft,
        AstBinaryOp::ShiftRight => BinaryOp::ShiftRight,
        AstBinaryOp::Equal => BinaryOp::Equal,
        AstBinaryOp::NotEqual => BinaryOp::NotEqual,
        AstBinaryOp::Spaceship => BinaryOp::Spaceship,
        AstBinaryOp::Identical => BinaryOp::Identical,
        AstBinaryOp::NotIdentical => BinaryOp::NotIdentical,
        AstBinaryOp::Less => BinaryOp::Less,
        AstBinaryOp::LessEqual => BinaryOp::LessEqual,
        AstBinaryOp::Greater => BinaryOp::Greater,
        AstBinaryOp::GreaterEqual => BinaryOp::GreaterEqual,
        AstBinaryOp::BitwiseAnd => BinaryOp::BitwiseAnd,
        AstBinaryOp::BitwiseXor => BinaryOp::BitwiseXor,
        AstBinaryOp::BitwiseOr => BinaryOp::BitwiseOr,
        AstBinaryOp::And => BinaryOp::And,
        AstBinaryOp::Xor => BinaryOp::Xor,
        AstBinaryOp::Or => BinaryOp::Or,
    }
}
