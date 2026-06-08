use crate::diagnostic::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Echo {
        expressions: Vec<Expr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    String(String, SourceSpan),
    Int(i64, SourceSpan),
    Float(f64, SourceSpan),
    Bool(bool, SourceSpan),
    Null(SourceSpan),
}

impl Expr {
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::String(_, span)
            | Expr::Int(_, span)
            | Expr::Float(_, span)
            | Expr::Bool(_, span)
            | Expr::Null(span) => *span,
        }
    }
}
