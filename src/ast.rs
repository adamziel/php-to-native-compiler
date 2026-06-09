use crate::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assign {
        name: String,
        op: AssignmentOp,
        value: Expr,
        span: SourceSpan,
    },
    Increment {
        name: String,
        op: IncDecOp,
        span: SourceSpan,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
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
    Switch {
        expression: Expr,
        cases: Vec<SwitchCase>,
        span: SourceSpan,
    },
    Break {
        span: SourceSpan,
    },
    InlineHtml {
        content: String,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<Expr>,
    pub body: Vec<Statement>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOp {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    ConcatAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecOp {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String, SourceSpan),
    Int(i64, SourceSpan),
    Float(f64, SourceSpan),
    Bool(bool, SourceSpan),
    Null(SourceSpan),
    Variable(String, SourceSpan),
    Call {
        name: String,
        arguments: Vec<Expr>,
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
    Grouped {
        expr: Box<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Concat,
    Equal,
    NotEqual,
    Identical,
    NotIdentical,
    BitwiseAnd,
    BitwiseOr,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Positive,
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Int,
    Float,
    String,
    Bool,
}

impl Expr {
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::String(_, span)
            | Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::Bool(_, span)
            | Expr::Null(span)
            | Expr::Variable(_, span) => *span,
            Expr::Call { span, .. } => *span,
            Expr::Unary { span, .. } | Expr::Cast { span, .. } => *span,
            Expr::Binary { span, .. } | Expr::Grouped { span, .. } => *span,
        }
    }
}
