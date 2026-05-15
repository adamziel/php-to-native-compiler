#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Namespace {
        name: String,
        span: Span,
    },
    Use {
        imports: Vec<UseImport>,
        span: Span,
    },
    Echo {
        exprs: Vec<Expr>,
        span: Span,
    },
    Print {
        expr: Expr,
        span: Span,
    },
    Assign {
        target: AssignTarget,
        expr: Expr,
        span: Span,
    },
    ReferenceAssign {
        target: AssignTarget,
        source: ReferenceSource,
        span: Span,
    },
    CompoundAssign {
        target: AssignTarget,
        op: CompoundAssignOp,
        expr: Expr,
        span: Span,
    },
    IncrementDecrement {
        target: AssignTarget,
        op: IncrementDecrementOp,
        span: Span,
    },
    NullCoalesceAssign {
        target: AssignTarget,
        expr: Expr,
        span: Span,
    },
    Expr {
        expr: Expr,
        span: Span,
    },
    Goto {
        label: String,
        span: Span,
    },
    Label {
        name: String,
        span: Span,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
        span: Span,
    },
    For {
        initializers: Vec<ForAction>,
        conditions: Vec<Expr>,
        increments: Vec<ForAction>,
        body: Vec<Stmt>,
        span: Span,
    },
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
        span: Span,
    },
    Foreach {
        iterable: Expr,
        key: Option<String>,
        value: String,
        by_reference: bool,
        body: Vec<Stmt>,
        span: Span,
    },
    UnsetVariable {
        name: String,
        span: Span,
    },
    UnsetArrayIndex {
        name: String,
        index: Expr,
        span: Span,
    },
    UnsetNestedArrayIndex {
        name: String,
        indices: Vec<Expr>,
        span: Span,
    },
    UnsetObjectProperty {
        object: String,
        property: String,
        span: Span,
    },
    UnsetDynamicObjectProperty {
        object: String,
        property: Expr,
        span: Span,
    },
    UnsetStaticProperty {
        class_name: String,
        property: String,
        span: Span,
    },
    UnsetSelfStaticProperty {
        property: String,
        span: Span,
    },
    UnsetParentStaticProperty {
        property: String,
        span: Span,
    },
    UnsetLateStaticProperty {
        property: String,
        span: Span,
    },
    UnsetMany {
        targets: Vec<UnsetTarget>,
        span: Span,
    },
    ConstDeclaration {
        declarations: Vec<ConstDeclarator>,
        span: Span,
    },
    Require {
        path: Expr,
        once: bool,
        span: Span,
    },
    Include {
        path: Expr,
        once: bool,
        span: Span,
    },
    Function(FunctionDecl),
    Interface(InterfaceDecl),
    Trait(TraitDecl),
    Enum(EnumDecl),
    Class(ClassDecl),
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Throw {
        expr: Expr,
        span: Span,
    },
    Try {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
        span: Span,
    },
    Break {
        depth: usize,
        span: Span,
    },
    Continue {
        depth: usize,
        span: Span,
    },
    Global {
        names: Vec<String>,
        span: Span,
    },
    StaticLocal {
        declarations: Vec<StaticLocalDeclarator>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Namespace { span, .. }
            | Stmt::Use { span, .. }
            | Stmt::Echo { span, .. }
            | Stmt::Print { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::ReferenceAssign { span, .. }
            | Stmt::CompoundAssign { span, .. }
            | Stmt::IncrementDecrement { span, .. }
            | Stmt::NullCoalesceAssign { span, .. }
            | Stmt::Expr { span, .. }
            | Stmt::Goto { span, .. }
            | Stmt::Label { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Foreach { span, .. }
            | Stmt::UnsetVariable { span, .. }
            | Stmt::UnsetArrayIndex { span, .. }
            | Stmt::UnsetNestedArrayIndex { span, .. }
            | Stmt::UnsetObjectProperty { span, .. }
            | Stmt::UnsetDynamicObjectProperty { span, .. }
            | Stmt::UnsetStaticProperty { span, .. }
            | Stmt::UnsetSelfStaticProperty { span, .. }
            | Stmt::UnsetParentStaticProperty { span, .. }
            | Stmt::UnsetLateStaticProperty { span, .. }
            | Stmt::UnsetMany { span, .. }
            | Stmt::ConstDeclaration { span, .. }
            | Stmt::Require { span, .. }
            | Stmt::Include { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Throw { span, .. }
            | Stmt::Try { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Continue { span, .. }
            | Stmt::Global { span, .. }
            | Stmt::StaticLocal { span, .. } => *span,
            Stmt::Function(function) => function.span,
            Stmt::Interface(interface) => interface.span,
            Stmt::Trait(trait_decl) => trait_decl.span,
            Stmt::Enum(enum_decl) => enum_decl.span,
            Stmt::Class(class) => class.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseImport {
    pub name: String,
    pub alias: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Variable {
        name: String,
        span: Span,
    },
    List {
        names: Vec<Option<String>>,
        span: Span,
    },
    ArrayIndex {
        name: String,
        index: Option<Expr>,
        span: Span,
    },
    NestedArrayIndex {
        name: String,
        indices: Vec<Expr>,
        span: Span,
    },
    NestedArrayAppend {
        name: String,
        indices: Vec<Expr>,
        span: Span,
    },
    Property {
        object: String,
        property: String,
        span: Span,
    },
    ObjectPropertyArrayIndex {
        object: String,
        property: String,
        indices: Vec<Expr>,
        span: Span,
    },
    ObjectPropertyArrayAppend {
        object: String,
        property: String,
        indices: Vec<Expr>,
        span: Span,
    },
    DynamicProperty {
        object: String,
        property: Expr,
        span: Span,
    },
    StaticProperty {
        class_name: String,
        property: String,
        span: Span,
    },
    SelfStaticProperty {
        property: String,
        span: Span,
    },
    ParentStaticProperty {
        property: String,
        span: Span,
    },
    LateStaticProperty {
        property: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceSource {
    Variable {
        name: String,
        span: Span,
    },
    ArrayIndex {
        name: String,
        index: Expr,
        span: Span,
    },
    Property {
        expr: Expr,
        span: Span,
    },
    MethodCall {
        expr: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDeclarator {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticLocalDeclarator {
    pub name: String,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnsetTarget {
    Variable {
        name: String,
        span: Span,
    },
    ArrayIndex {
        name: String,
        index: Expr,
        span: Span,
    },
    NestedArrayIndex {
        name: String,
        indices: Vec<Expr>,
        span: Span,
    },
    ObjectPropertyArrayIndex {
        object: String,
        property: String,
        indices: Vec<Expr>,
        span: Span,
    },
    ObjectProperty {
        object: String,
        property: String,
        span: Span,
    },
    DynamicObjectProperty {
        object: String,
        property: Expr,
        span: Span,
    },
    StaticProperty {
        class_name: String,
        property: String,
        span: Span,
    },
    SelfStaticProperty {
        property: String,
        span: Span,
    },
    ParentStaticProperty {
        property: String,
        span: Span,
    },
    LateStaticProperty {
        property: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForAction {
    Assign {
        target: AssignTarget,
        expr: Expr,
    },
    CompoundAssign {
        target: AssignTarget,
        op: CompoundAssignOp,
        expr: Expr,
        span: Span,
    },
    IncrementDecrement {
        target: AssignTarget,
        op: IncrementDecrementOp,
        span: Span,
    },
    Expr {
        expr: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub types: Vec<CatchType>,
    pub variable: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchType {
    pub name: String,
    pub span: Span,
}

impl AssignTarget {
    pub fn span(&self) -> Span {
        match self {
            AssignTarget::Variable { span, .. }
            | AssignTarget::List { span, .. }
            | AssignTarget::ArrayIndex { span, .. }
            | AssignTarget::NestedArrayIndex { span, .. }
            | AssignTarget::NestedArrayAppend { span, .. }
            | AssignTarget::Property { span, .. }
            | AssignTarget::ObjectPropertyArrayIndex { span, .. }
            | AssignTarget::ObjectPropertyArrayAppend { span, .. }
            | AssignTarget::DynamicProperty { span, .. }
            | AssignTarget::StaticProperty { span, .. }
            | AssignTarget::SelfStaticProperty { span, .. }
            | AssignTarget::ParentStaticProperty { span, .. }
            | AssignTarget::LateStaticProperty { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub members: Vec<ClassMember>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_readonly: bool,
    pub is_nested: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub name: String,
    pub methods: Vec<InterfaceMethodDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethodDecl {
    pub function: FunctionDecl,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub cases: Vec<EnumCaseDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCaseDecl {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Property(ClassPropertyDecl),
    Constant(ClassConstantDecl),
    Method(ClassMethodDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstantDecl {
    pub name: String,
    pub visibility: ClassVisibility,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassPropertyDecl {
    pub name: String,
    pub visibility: ClassVisibility,
    pub is_static: bool,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethodDecl {
    pub function: FunctionDecl,
    pub visibility: ClassVisibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassVisibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeDecl>,
    pub returns_by_reference: bool,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub type_decl: Option<TypeDecl>,
    pub by_reference: bool,
    pub is_variadic: bool,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureCapture {
    pub name: String,
    pub by_reference: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayItem {
    pub key: Option<Expr>,
    pub value: Expr,
    pub by_reference: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null(Span),
    Bool(bool, Span),
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    InterpolatedString {
        parts: Vec<InterpolatedStringPart>,
        span: Span,
    },
    Variable(String, Span),
    MagicLine {
        span: Span,
    },
    MagicFile {
        span: Span,
    },
    MagicDir {
        span: Span,
    },
    MagicFunction {
        span: Span,
    },
    MagicMethod {
        span: Span,
    },
    GlobalConstant {
        name: String,
        span: Span,
    },
    ClassNameConstant {
        class_name: String,
        span: Span,
    },
    SelfClassNameConstant {
        span: Span,
    },
    ParentClassNameConstant {
        span: Span,
    },
    StaticClassNameConstant {
        span: Span,
    },
    ClassConstant {
        class_name: String,
        constant: String,
        span: Span,
    },
    SelfClassConstant {
        constant: String,
        span: Span,
    },
    ParentClassConstant {
        constant: String,
        span: Span,
    },
    LateStaticClassConstant {
        constant: String,
        span: Span,
    },
    Array {
        items: Vec<ArrayItem>,
        span: Span,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    AppendIndex {
        target: Box<Expr>,
        span: Span,
    },
    Property {
        target: Box<Expr>,
        property: String,
        span: Span,
    },
    DynamicProperty {
        target: Box<Expr>,
        property: Box<Expr>,
        span: Span,
    },
    StaticProperty {
        class_name: String,
        property: String,
        span: Span,
    },
    SelfStaticProperty {
        property: String,
        span: Span,
    },
    ParentStaticProperty {
        property: String,
        span: Span,
    },
    LateStaticProperty {
        property: String,
        span: Span,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    ParentMethodCall {
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    StaticMethodCall {
        class_name: String,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    ObjectStaticMethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    SelfMethodCall {
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    LateStaticMethodCall {
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    DynamicCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    InstanceOf {
        expr: Box<Expr>,
        class_name: String,
        span: Span,
    },
    Closure {
        params: Vec<FunctionParam>,
        captures: Vec<ClosureCapture>,
        return_type: Option<TypeDecl>,
        body: Vec<Stmt>,
        is_arrow: bool,
        span: Span,
    },
    New {
        class_name: NewClassName,
        args: Vec<Expr>,
        span: Span,
    },
    Clone {
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    ErrorControl {
        expr: Box<Expr>,
        span: Span,
    },
    Include {
        path: Box<Expr>,
        once: bool,
        span: Span,
    },
    Require {
        path: Box<Expr>,
        once: bool,
        span: Span,
    },
    Cast {
        kind: CastKind,
        expr: Box<Expr>,
        span: Span,
    },
    Ternary {
        condition: Box<Expr>,
        if_true: Box<Expr>,
        if_false: Box<Expr>,
        span: Span,
    },
    ShortTernary {
        condition: Box<Expr>,
        if_false: Box<Expr>,
        span: Span,
    },
    Assign {
        target: Box<AssignTarget>,
        expr: Box<Expr>,
        span: Span,
    },
    CompoundAssign {
        target: Box<AssignTarget>,
        op: CompoundAssignOp,
        expr: Box<Expr>,
        span: Span,
    },
    NullCoalesceAssign {
        target: Box<AssignTarget>,
        expr: Box<Expr>,
        span: Span,
    },
    IncrementDecrement {
        target: Box<AssignTarget>,
        op: IncrementDecrementOp,
        position: IncrementDecrementPosition,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewClassName {
    Named(String),
    SelfClass,
    ParentClass,
    StaticClass,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedStringPart {
    Literal(String),
    Variable(String),
    ArrayOffset {
        variable: String,
        key: InterpolatedArrayKey,
    },
    ObjectProperty {
        variable: String,
        property: String,
    },
    AccessChain {
        variable: String,
        segments: Vec<InterpolatedAccessSegment>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedAccessSegment {
    ArrayOffset(InterpolatedArrayKey),
    ObjectProperty(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolatedArrayKey {
    Int(i64),
    String(String),
    Variable(String),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Null(span)
            | Expr::Bool(_, span)
            | Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::String(_, span)
            | Expr::InterpolatedString { span, .. }
            | Expr::Variable(_, span)
            | Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span }
            | Expr::MagicMethod { span }
            | Expr::GlobalConstant { span, .. }
            | Expr::ClassNameConstant { span, .. }
            | Expr::SelfClassNameConstant { span }
            | Expr::ParentClassNameConstant { span }
            | Expr::StaticClassNameConstant { span }
            | Expr::ClassConstant { span, .. }
            | Expr::SelfClassConstant { span, .. }
            | Expr::ParentClassConstant { span, .. }
            | Expr::LateStaticClassConstant { span, .. }
            | Expr::Array { span, .. }
            | Expr::Index { span, .. }
            | Expr::AppendIndex { span, .. }
            | Expr::Property { span, .. }
            | Expr::DynamicProperty { span, .. }
            | Expr::StaticProperty { span, .. }
            | Expr::SelfStaticProperty { span, .. }
            | Expr::ParentStaticProperty { span, .. }
            | Expr::LateStaticProperty { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::ParentMethodCall { span, .. }
            | Expr::StaticMethodCall { span, .. }
            | Expr::ObjectStaticMethodCall { span, .. }
            | Expr::SelfMethodCall { span, .. }
            | Expr::LateStaticMethodCall { span, .. }
            | Expr::Call { span, .. }
            | Expr::DynamicCall { span, .. }
            | Expr::InstanceOf { span, .. }
            | Expr::Closure { span, .. }
            | Expr::New { span, .. }
            | Expr::Clone { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::ErrorControl { span, .. }
            | Expr::Include { span, .. }
            | Expr::Require { span, .. }
            | Expr::Cast { span, .. }
            | Expr::Ternary { span, .. }
            | Expr::ShortTernary { span, .. }
            | Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::NullCoalesceAssign { span, .. }
            | Expr::IncrementDecrement { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    String,
    Int,
    Bool,
    Float,
    Array,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Eq,
    Ne,
    StrictEq,
    StrictNe,
    Lt,
    Le,
    Gt,
    Ge,
    NullCoalesce,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundAssignOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementDecrementOp {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementDecrementPosition {
    Pre,
    Post,
}
