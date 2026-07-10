use crate::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub classes: Vec<ClassDecl>,
    pub traits: Vec<TraitDecl>,
    pub functions: Vec<FunctionDecl>,
    pub statements: Vec<Statement>,
    pub compile_warnings: Vec<CompileWarning>,
    pub strict_types: bool,
    pub ticks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileWarning {
    pub message: String,
    pub span: SourceSpan,
    pub kind: CompileWarningKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileWarningKind {
    Warning,
    UncaughtError,
    Deprecation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub source_file: Option<String>,
    pub parent_name: Option<String>,
    pub interfaces: Vec<String>,
    pub trait_uses: Vec<TraitUseDecl>,
    pub declaration_fatals: Vec<ClassDeclarationFatal>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
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
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDeclarationFatal {
    pub message: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumBackingType {
    Int,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub trait_uses: Vec<TraitUseDecl>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub constants: Vec<ClassConstantDecl>,
    pub methods: Vec<MethodDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitUseDecl {
    pub name: String,
    pub adaptations: Vec<TraitAdaptation>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraitAdaptation {
    Alias(TraitAliasAdaptation),
    Precedence(TraitPrecedenceAdaptation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodReference {
    pub trait_name: Option<String>,
    pub method_name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitAliasAdaptation {
    pub method: TraitMethodReference,
    pub alias: Option<String>,
    pub visibility: Option<PropertyVisibility>,
    pub is_final: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitPrecedenceAdaptation {
    pub method: TraitMethodReference,
    pub instead_of: Vec<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub declaring_class_name: Option<String>,
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
    pub hook_get_override_span: Option<SourceSpan>,
    pub hook_set_override_span: Option<SourceSpan>,
    pub hook_get_attributes: AttributeMetadata,
    pub hook_set_attributes: AttributeMetadata,
    pub hook_get_doc_comment: Option<String>,
    pub hook_set_doc_comment: Option<String>,
    pub hook_get_span: Option<SourceSpan>,
    pub hook_set_span: Option<SourceSpan>,
    pub hook_get_value: Option<Expr>,
    pub hook_set_value: Option<Expr>,
    pub hook_get_body: Option<Vec<Statement>>,
    pub hook_set_body: Option<Vec<Statement>>,
    pub hook_set_parameter_name: Option<String>,
    pub hook_set_parameter_attributes: AttributeMetadata,
    pub hook_set_parameter_type: Option<TypeHint>,
    pub hook_set_parameter_doc_comment: Option<String>,
    pub hook_set_parameter_span: Option<SourceSpan>,
    pub hook_set_parameter_is_explicit: bool,
    pub type_hint: Option<PropertyTypeHint>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub has_override_attribute: bool,
    pub value: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyVisibility {
    Public,
    Protected,
    Private,
}

impl Default for PropertyVisibility {
    fn default() -> Self {
        Self::Public
    }
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
    pub has_override_attribute: bool,
    pub value: Option<Expr>,
    pub span: SourceSpan,
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
    pub value: Expr,
    pub is_enum_case: bool,
    pub enum_case_value: Option<Expr>,
    pub is_final: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub has_explicit_visibility: bool,
    pub trait_name: Option<String>,
    pub trait_method_name: Option<String>,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub has_override_attribute: bool,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_conditionally_declared: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub type_hint: Option<TypeHint>,
    pub by_ref: bool,
    pub is_variadic: bool,
    pub default_value: Option<Expr>,
    pub promoted_property: Option<PromotedProperty>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PromotedProperty {
    pub visibility: PropertyVisibility,
    pub set_visibility: PropertyVisibility,
    pub is_final: bool,
    pub is_readonly: bool,
    pub has_hooks: bool,
    pub is_virtual: bool,
    pub hook_has_get: bool,
    pub hook_has_set: bool,
    pub hook_get_is_final: bool,
    pub hook_get_is_abstract: bool,
    pub hook_get_returns_by_ref: bool,
    pub hook_set_is_final: bool,
    pub hook_set_is_abstract: bool,
    pub hook_get_override_span: Option<SourceSpan>,
    pub hook_set_override_span: Option<SourceSpan>,
    pub hook_get_attributes: AttributeMetadata,
    pub hook_set_attributes: AttributeMetadata,
    pub hook_get_doc_comment: Option<String>,
    pub hook_set_doc_comment: Option<String>,
    pub hook_get_span: Option<SourceSpan>,
    pub hook_set_span: Option<SourceSpan>,
    pub hook_get_value: Option<Expr>,
    pub hook_set_value: Option<Expr>,
    pub hook_get_body: Option<Vec<Statement>>,
    pub hook_set_body: Option<Vec<Statement>>,
    pub hook_set_parameter_name: Option<String>,
    pub hook_set_parameter_attributes: AttributeMetadata,
    pub hook_set_parameter_type: Option<TypeHint>,
    pub hook_set_parameter_doc_comment: Option<String>,
    pub hook_set_parameter_span: Option<SourceSpan>,
    pub hook_set_parameter_is_explicit: bool,
    pub doc_comment: Option<String>,
    pub has_override_attribute: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonymousFunction {
    pub attributes: AttributeMetadata,
    pub doc_comment: Option<String>,
    pub parameters: Vec<FunctionParameter>,
    pub captures: Vec<ClosureUseCapture>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_static: bool,
    pub is_arrow: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttributeMetadata {
    pub instances: Vec<AttributeInstance>,
    pub total_count: u16,
    pub first_class_callable_argument_count: u16,
    pub attribute_count: u16,
    pub allow_dynamic_properties_count: u16,
    pub delayed_target_validation_count: u16,
    pub deprecated_count: u16,
    pub no_discard_count: u16,
    pub override_count: u16,
    pub return_type_will_change_count: u16,
    pub has_override: bool,
    pub has_attribute: bool,
    pub has_allow_dynamic_properties: bool,
    pub deprecated_message: Option<String>,
    pub deprecated_message_constant: Option<AttributeConstantReference>,
    pub deprecated_since: Option<String>,
    pub no_discard_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInstance {
    pub name: String,
    pub arguments: Vec<AttributeArgument>,
    pub source_file: String,
    pub line: usize,
    pub strict_types: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeArgument {
    pub name: Option<String>,
    pub value: AttributeArgumentValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeArgumentValue {
    pub text: String,
    pub kind: AttributeArgumentKind,
    pub constant_reference: Option<AttributeConstantReference>,
    pub expression: Option<AttributeArgumentExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeArgumentArrayElement {
    pub key: Option<AttributeArgumentExpression>,
    pub value: AttributeArgumentExpression,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArgumentExpression {
    String(String),
    Int(String),
    Float(String),
    Bool(bool),
    Null,
    PropertyMagicConstant,
    Constant(String),
    ClassName {
        class_name: String,
        scope_relative: bool,
    },
    ClassConstant {
        class_name: String,
        name: String,
        scope_relative: bool,
    },
    PropertyFetch {
        receiver: Box<AttributeArgumentExpression>,
        name: String,
        nullsafe: bool,
        line: usize,
    },
    NewObject {
        class_name: String,
        arguments: Vec<AttributeArgumentExpression>,
        line: usize,
    },
    Closure {
        function: Option<Box<AnonymousFunction>>,
        function_index: Option<usize>,
        source_text: String,
        reflection_text: String,
        line: usize,
    },
    FirstClassCallable {
        callable: String,
        line: usize,
    },
    Array(Vec<AttributeArgumentArrayElement>),
    Unary {
        op: UnaryOp,
        expr: Box<AttributeArgumentExpression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<AttributeArgumentExpression>,
        right: Box<AttributeArgumentExpression>,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeArgumentKind {
    String,
    Int,
    Float,
    Bool,
    Null,
    Array,
    Closure,
    Constant,
    ClassConstant,
    NativeEnumCase {
        class_name: String,
        case_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeConstantReference {
    Constant(String),
    ClassConstant { class_name: String, name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureUseCapture {
    pub name: String,
    pub by_ref: bool,
    pub warn_if_missing: bool,
    pub span: SourceSpan,
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
pub enum GlobalTarget {
    Variable { name: String, span: SourceSpan },
    DynamicVariable { name: Box<Expr>, span: SourceSpan },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Empty {
        span: SourceSpan,
    },
    ClassDeclaration {
        name: String,
        source: String,
        span: SourceSpan,
    },
    TraitDeclaration {
        name: String,
        source: String,
        span: SourceSpan,
    },
    FunctionDeclaration {
        name: String,
        span: SourceSpan,
    },
    Assign {
        name: String,
        op: AssignmentOp,
        value: Expr,
        span: SourceSpan,
    },
    AssignRef {
        name: String,
        source: Expr,
        span: SourceSpan,
    },
    ArrayAssign {
        target: ArrayDimTarget,
        op: AssignmentOp,
        value: Expr,
        span: SourceSpan,
    },
    ArrayAssignRef {
        target: ArrayDimTarget,
        source: Expr,
        span: SourceSpan,
    },
    Increment {
        target: IncDecTarget,
        op: IncDecOp,
        span: SourceSpan,
    },
    Unset {
        targets: Vec<UnsetTarget>,
        span: SourceSpan,
    },
    Global {
        targets: Vec<GlobalTarget>,
        span: SourceSpan,
    },
    Static {
        declarations: Vec<StaticLocalDeclaration>,
        span: SourceSpan,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        allow_global_fallback: bool,
        span: SourceSpan,
    },
    Echo {
        expressions: Vec<Expr>,
        span: SourceSpan,
    },
    Print {
        expression: Expr,
        span: SourceSpan,
    },
    Expression {
        expression: Expr,
        span: SourceSpan,
    },
    Const {
        declarations: Vec<ConstDeclaration>,
        span: SourceSpan,
    },
    Block {
        statements: Vec<Statement>,
        ticks: Option<bool>,
        span: SourceSpan,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
        span: SourceSpan,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
        span: SourceSpan,
    },
    DoWhile {
        body: Vec<Statement>,
        condition: Expr,
        span: SourceSpan,
    },
    For {
        initializers: Vec<Statement>,
        conditions: Vec<Expr>,
        updates: Vec<Statement>,
        body: Vec<Statement>,
        span: SourceSpan,
    },
    Foreach {
        iterable: Expr,
        key: Option<AssignmentTarget>,
        value: AssignmentTarget,
        value_by_ref: bool,
        body: Vec<Statement>,
        span: SourceSpan,
    },
    Switch {
        expression: Expr,
        cases: Vec<SwitchCase>,
        span: SourceSpan,
    },
    Break {
        level: usize,
        span: SourceSpan,
    },
    Continue {
        level: usize,
        span: SourceSpan,
    },
    Return {
        value: Option<Expr>,
        span: SourceSpan,
    },
    Exit {
        value: Option<Expr>,
        span: SourceSpan,
    },
    Throw {
        value: Expr,
        span: SourceSpan,
    },
    Try {
        body: Vec<Statement>,
        catches: Vec<CatchClause>,
        finally_body: Vec<Statement>,
        span: SourceSpan,
    },
    Label {
        name: String,
        span: SourceSpan,
    },
    Goto {
        label: String,
        span: SourceSpan,
    },
    InlineHtml {
        content: String,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub type_names: Vec<String>,
    pub variable: Option<String>,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDeclaration {
    pub name: String,
    pub attributes: AttributeMetadata,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticLocalDeclaration {
    pub name: String,
    pub value: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDimTarget {
    pub array: String,
    pub dimensions: Vec<Option<Expr>>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable {
        name: String,
        span: SourceSpan,
    },
    DynamicVariable {
        name: Box<Expr>,
        span: SourceSpan,
    },
    DynamicArrayDim {
        name: Box<Expr>,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    ArrayDim(ArrayDimTarget),
    ValueArrayDim {
        array: Box<Expr>,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    PropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    DynamicPropertyArrayDim {
        receiver: Box<Expr>,
        name: Box<Expr>,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    DynamicStaticPropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    DynamicStaticPropertyName {
        class_name: String,
        name: Box<Expr>,
        span: SourceSpan,
    },
    Property {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    DynamicProperty {
        receiver: Box<Expr>,
        name: Box<Expr>,
        span: SourceSpan,
    },
    StaticProperty {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    DynamicStaticProperty {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    List(ListAssignmentTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentTarget {
    pub elements: Vec<ListAssignmentElement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentElement {
    pub key: Option<Expr>,
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
        span: SourceSpan,
    },
    DynamicVariable {
        name: Box<Expr>,
        span: SourceSpan,
    },
    ArrayDim(ArrayDimTarget),
    PropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    Property {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    DynamicProperty {
        receiver: Box<Expr>,
        name: Box<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncDecTarget {
    Variable {
        name: String,
        span: SourceSpan,
    },
    DynamicVariable {
        name: Box<Expr>,
        span: SourceSpan,
    },
    DynamicArrayDim {
        name: Box<Expr>,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    ArrayDim(ArrayDimTarget),
    ValueArrayDim {
        array: Box<Expr>,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    PropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Option<Expr>>,
        span: SourceSpan,
    },
    Property {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    DynamicProperty {
        receiver: Box<Expr>,
        name: Box<Expr>,
        span: SourceSpan,
    },
    StaticProperty {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    DynamicStaticPropertyName {
        class_name: String,
        name: Box<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnsetTarget {
    Variable {
        name: String,
        span: SourceSpan,
    },
    DynamicVariable {
        name: Box<Expr>,
        span: SourceSpan,
    },
    DynamicArrayDim {
        name: Box<Expr>,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    ValueArrayDim {
        array: Box<Expr>,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    PropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    DynamicStaticPropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    DynamicStaticPropertyName {
        class_name: String,
        name: Box<Expr>,
        span: SourceSpan,
    },
    Property {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    DynamicProperty {
        receiver: Box<Expr>,
        name: Box<Expr>,
        span: SourceSpan,
    },
    StaticProperty {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    ArrayDim(ArrayDimTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,
    CoalesceAssign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    PowerAssign,
    DivideAssign,
    ModuloAssign,
    ConcatAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludeKind {
    Include,
    IncludeOnce,
    Require,
    RequireOnce,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListExpr {
    pub elements: Vec<ListExprElement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListExprElement {
    pub key: Option<Expr>,
    pub target: Option<ListExprElementTarget>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListExprElementTarget {
    Value(Expr),
    Reference(ReferenceTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String, SourceSpan),
    InterpolatedString(Vec<StringPart>, SourceSpan),
    ShellExec {
        command: Box<Expr>,
        span: SourceSpan,
    },
    Int(i64, SourceSpan),
    Float(f64, SourceSpan),
    Bool(bool, SourceSpan),
    Null(SourceSpan),
    Variable(String, SourceSpan),
    DynamicVariable {
        name: Box<Expr>,
        span: SourceSpan,
    },
    AnonymousFunction(AnonymousFunction),
    IncDec {
        target: IncDecTarget,
        op: IncDecOp,
        result: IncDecResult,
        span: SourceSpan,
    },
    Assign {
        target: AssignmentTarget,
        op: AssignmentOp,
        value: Box<Expr>,
        span: SourceSpan,
    },
    AssignRef {
        target: AssignmentTarget,
        source: Box<Expr>,
        span: SourceSpan,
    },
    Constant(String, SourceSpan),
    MagicConstant(MagicConstantKind, SourceSpan),
    Call {
        name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        allow_global_fallback: bool,
        call_line: usize,
        span: SourceSpan,
    },
    FirstClassCallable {
        callable: Box<Expr>,
        span: SourceSpan,
    },
    DynamicCall {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        static_method_syntax: bool,
        call_line: usize,
        span: SourceSpan,
    },
    MethodCall {
        receiver: Box<Expr>,
        name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        nullsafe: bool,
        call_line: usize,
        span: SourceSpan,
    },
    DynamicMethodCall {
        receiver: Box<Expr>,
        name: Box<Expr>,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        call_line: usize,
        span: SourceSpan,
    },
    NewObject {
        class_name: String,
        source_name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        constructor_parentheses: bool,
        anonymous_class_source: Option<String>,
        call_line: usize,
        span: SourceSpan,
    },
    DynamicNewObject {
        class_name: Box<Expr>,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        constructor_parentheses: bool,
        call_line: usize,
        span: SourceSpan,
    },
    Clone {
        expr: Box<Expr>,
        with_properties: Option<Box<Expr>>,
        span: SourceSpan,
    },
    PropertyFetch {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    NullsafePropertyFetch {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    DynamicPropertyFetch {
        receiver: Box<Expr>,
        name: Box<Expr>,
        nullsafe: bool,
        span: SourceSpan,
    },
    StaticPropertyFetch {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    DynamicStaticPropertyNameFetch {
        class_name: String,
        name: Box<Expr>,
        span: SourceSpan,
    },
    ParentPropertyHookCall {
        property_name: String,
        hook_name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        span: SourceSpan,
    },
    DynamicStaticPropertyFetch {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    ClassConstantFetch {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    DynamicClassConstantFetch {
        class_name: Option<String>,
        receiver: Option<Box<Expr>>,
        name: Box<Expr>,
        span: SourceSpan,
    },
    DynamicClassNameFetch {
        receiver: Box<Expr>,
        allow_string: bool,
        span: SourceSpan,
    },
    InstanceOf {
        expr: Box<Expr>,
        target: InstanceOfTarget,
        span: SourceSpan,
    },
    Match {
        subject: Box<Expr>,
        arms: Vec<MatchArm>,
        span: SourceSpan,
    },
    Array {
        elements: Vec<ArrayElement>,
        short_syntax: bool,
        span: SourceSpan,
    },
    List(ListExpr),
    ArrayAccess {
        array: Box<Expr>,
        index: Option<Box<Expr>>,
        span: SourceSpan,
    },
    Isset {
        targets: Vec<Expr>,
        span: SourceSpan,
    },
    Empty {
        target: Box<Expr>,
        span: SourceSpan,
    },
    Print {
        expression: Box<Expr>,
        span: SourceSpan,
    },
    Include {
        kind: IncludeKind,
        path: Box<Expr>,
        span: SourceSpan,
    },
    Exit {
        value: Option<Box<Expr>>,
        span: SourceSpan,
    },
    Throw {
        value: Box<Expr>,
        span: SourceSpan,
    },
    Yield {
        key: Option<Box<Expr>>,
        value: Option<Box<Expr>>,
        span: SourceSpan,
    },
    YieldFrom {
        expr: Box<Expr>,
        span: SourceSpan,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: SourceSpan,
    },
    Cast {
        kind: CastKind,
        expr: Box<Expr>,
        span: SourceSpan,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: SourceSpan,
    },
    Ternary {
        condition: Box<Expr>,
        if_true: Option<Box<Expr>>,
        if_false: Box<Expr>,
        span: SourceSpan,
    },
    Grouped {
        expr: Box<Expr>,
        span: SourceSpan,
    },
    PipeValue {
        expr: Box<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<Expr>,
    pub value: Expr,
    pub is_default: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<Expr>,
    pub value: ArrayElementValue,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementValue {
    Hole(SourceSpan),
    Value(Expr),
    Reference(ReferenceTarget),
    Unpack(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
    Expression(Box<Expr>),
    LegacyDollarBraceVariable(String),
    LegacyDollarBraceExpression(Box<Expr>),
    DynamicVariableExpression(Box<Expr>),
    PropertyFetch {
        variable: String,
        property: String,
    },
    PropertyChain {
        variable: String,
        properties: Vec<String>,
    },
    MethodCall {
        variable: String,
        method: String,
    },
    ArrayAccess {
        array: String,
        indices: Vec<StringInterpolationIndex>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringInterpolationIndex {
    String(String),
    Int(i64),
    Variable(String),
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
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Xor,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceOfTarget {
    ClassName {
        name: String,
        source_name: String,
        span: SourceSpan,
    },
    Expr(Box<Expr>),
}

impl InstanceOfTarget {
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::ClassName { span, .. } => *span,
            Self::Expr(expr) => expr.span(),
        }
    }
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

impl Expr {
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::String(_, span)
            | Expr::InterpolatedString(_, span)
            | Expr::ShellExec { span, .. }
            | Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::Bool(_, span)
            | Expr::Null(span)
            | Expr::Variable(_, span)
            | Expr::DynamicVariable { span, .. }
            | Expr::Constant(_, span)
            | Expr::MagicConstant(_, span) => *span,
            Expr::AnonymousFunction(function) => function.span,
            Expr::IncDec { span, .. } => *span,
            Expr::Assign { span, .. } | Expr::AssignRef { span, .. } => *span,
            Expr::Call { span, .. }
            | Expr::FirstClassCallable { span, .. }
            | Expr::DynamicCall { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::DynamicMethodCall { span, .. }
            | Expr::NewObject { span, .. }
            | Expr::DynamicNewObject { span, .. }
            | Expr::Clone { span, .. }
            | Expr::PropertyFetch { span, .. }
            | Expr::NullsafePropertyFetch { span, .. }
            | Expr::DynamicPropertyFetch { span, .. }
            | Expr::StaticPropertyFetch { span, .. }
            | Expr::DynamicStaticPropertyNameFetch { span, .. }
            | Expr::ParentPropertyHookCall { span, .. }
            | Expr::DynamicStaticPropertyFetch { span, .. }
            | Expr::ClassConstantFetch { span, .. }
            | Expr::DynamicClassConstantFetch { span, .. }
            | Expr::DynamicClassNameFetch { span, .. }
            | Expr::InstanceOf { span, .. }
            | Expr::Match { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::List(list) => list.span,
            Expr::ArrayAccess { span, .. } => *span,
            Expr::Isset { span, .. } => *span,
            Expr::Empty { span, .. } => *span,
            Expr::Print { span, .. } => *span,
            Expr::Include { span, .. } => *span,
            Expr::Exit { span, .. } => *span,
            Expr::Throw { span, .. } => *span,
            Expr::Yield { span, .. } | Expr::YieldFrom { span, .. } => *span,
            Expr::Unary { span, .. } | Expr::Cast { span, .. } => *span,
            Expr::Binary { span, .. }
            | Expr::Ternary { span, .. }
            | Expr::Grouped { span, .. }
            | Expr::PipeValue { span, .. } => *span,
        }
    }
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
