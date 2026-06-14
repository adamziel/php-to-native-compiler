use crate::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub classes: Vec<ClassDecl>,
    pub functions: Vec<FunctionDecl>,
    pub statements: Vec<Statement>,
    pub strict_types: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent_name: Option<String>,
    pub interfaces: Vec<String>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_interface: bool,
    pub is_readonly: bool,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub constants: Vec<ClassConstantDecl>,
    pub methods: Vec<MethodDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub set_visibility: PropertyVisibility,
    pub is_readonly: bool,
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
    pub value: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstantDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_static: bool,
    pub is_abstract: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_hint: Option<TypeHint>,
    pub by_ref: bool,
    pub is_variadic: bool,
    pub default_value: Option<Expr>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnonymousFunction {
    pub parameters: Vec<FunctionParameter>,
    pub captures: Vec<ClosureUseCapture>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
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
    Int,
    Float,
    String,
    Bool,
    Mixed,
    Void,
    Class(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Empty {
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
        names: Vec<String>,
        span: SourceSpan,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
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
        condition: Option<Expr>,
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
    Throw {
        value: Expr,
        span: SourceSpan,
    },
    Try {
        body: Vec<Statement>,
        catches: Vec<CatchClause>,
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
    pub type_name: String,
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
    pub value: Expr,
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
    StaticProperty {
        class_name: String,
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
    ArrayDim(ArrayDimTarget),
    Property {
        receiver: Box<Expr>,
        name: String,
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
    Property {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    StaticProperty {
        class_name: String,
        name: String,
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
    PropertyArrayDim {
        receiver: Box<Expr>,
        name: String,
        dimensions: Vec<Expr>,
        span: SourceSpan,
    },
    Property {
        receiver: Box<Expr>,
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
        span: SourceSpan,
    },
    MethodCall {
        receiver: Box<Expr>,
        name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        span: SourceSpan,
    },
    NewObject {
        class_name: String,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        span: SourceSpan,
    },
    DynamicNewObject {
        class_name: Box<Expr>,
        arguments: Vec<Expr>,
        argument_names: Vec<Option<String>>,
        span: SourceSpan,
    },
    Clone {
        expr: Box<Expr>,
        span: SourceSpan,
    },
    PropertyFetch {
        receiver: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    StaticPropertyFetch {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    ClassConstantFetch {
        class_name: String,
        name: String,
        span: SourceSpan,
    },
    DynamicClassNameFetch {
        receiver: Box<Expr>,
        span: SourceSpan,
    },
    Array {
        elements: Vec<ArrayElement>,
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
    Throw {
        value: Box<Expr>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<Expr>,
    pub value: ArrayElementValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementValue {
    Value(Expr),
    Reference(ReferenceTarget),
    Unpack(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
    LegacyDollarBraceVariable(String),
    PropertyFetch {
        variable: String,
        property: String,
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
}

impl Expr {
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::String(_, span)
            | Expr::InterpolatedString(_, span)
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
            | Expr::NewObject { span, .. }
            | Expr::DynamicNewObject { span, .. }
            | Expr::Clone { span, .. }
            | Expr::PropertyFetch { span, .. }
            | Expr::StaticPropertyFetch { span, .. }
            | Expr::ClassConstantFetch { span, .. }
            | Expr::DynamicClassNameFetch { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::List(list) => list.span,
            Expr::ArrayAccess { span, .. } => *span,
            Expr::Isset { span, .. } => *span,
            Expr::Empty { span, .. } => *span,
            Expr::Print { span, .. } => *span,
            Expr::Include { span, .. } => *span,
            Expr::Throw { span, .. } => *span,
            Expr::Unary { span, .. } | Expr::Cast { span, .. } => *span,
            Expr::Binary { span, .. } | Expr::Ternary { span, .. } | Expr::Grouped { span, .. } => {
                *span
            }
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
}
