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
        initializer: Option<ForAction>,
        condition: Option<Expr>,
        increment: Option<ForAction>,
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
    UnsetMany {
        targets: Vec<UnsetTarget>,
        span: Span,
    },
    ConstDeclaration {
        declarations: Vec<ConstDeclarator>,
        span: Span,
    },
    Function(FunctionDecl),
    Class(ClassDecl),
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Global {
        names: Vec<String>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Variable {
        name: String,
        span: Span,
    },
    ArrayIndex {
        name: String,
        index: Option<Expr>,
        span: Span,
    },
    Property {
        object: String,
        property: String,
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

impl AssignTarget {
    pub fn span(&self) -> Span {
        match self {
            AssignTarget::Variable { span, .. }
            | AssignTarget::ArrayIndex { span, .. }
            | AssignTarget::Property { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Property(ClassPropertyDecl),
    Method(ClassMethodDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassPropertyDecl {
    pub name: String,
    pub visibility: ClassVisibility,
    pub is_static: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethodDecl {
    pub function: FunctionDecl,
    pub visibility: ClassVisibility,
    pub is_static: bool,
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
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayItem {
    pub key: Option<Expr>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null(Span),
    Bool(bool, Span),
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
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
    GlobalConstant {
        name: String,
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
    New {
        class_name: String,
        args: Vec<Expr>,
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

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Null(span)
            | Expr::Bool(_, span)
            | Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::String(_, span)
            | Expr::Variable(_, span)
            | Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span }
            | Expr::GlobalConstant { span, .. }
            | Expr::Array { span, .. }
            | Expr::Index { span, .. }
            | Expr::AppendIndex { span, .. }
            | Expr::Property { span, .. }
            | Expr::Call { span, .. }
            | Expr::DynamicCall { span, .. }
            | Expr::New { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
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
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
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
    Concat,
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
