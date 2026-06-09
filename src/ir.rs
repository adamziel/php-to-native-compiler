use crate::ast::{
    AssignmentOp, BinaryOp as AstBinaryOp, CastKind as AstCastKind, Expr, Program, Statement,
    UnaryOp as AstUnaryOp,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Store { name: String, value: ValueExpr },
    Echo(ValueExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Load(String),
    Unary {
        op: UnaryOp,
        expr: Box<ValueExpr>,
    },
    Cast {
        kind: CastKind,
        expr: Box<ValueExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Concat,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
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

pub fn lower(program: &Program) -> Module {
    let mut instructions = Vec::new();
    for statement in &program.statements {
        match statement {
            Statement::Assign {
                name, op, value, ..
            } => {
                instructions.push(Instruction::Store {
                    name: name.clone(),
                    value: lower_assignment_value(name, *op, value),
                });
            }
            Statement::Echo { expressions, .. } => {
                for expression in expressions {
                    instructions.push(Instruction::Echo(lower_expr(expression)));
                }
            }
            Statement::Print { expression, .. } => {
                instructions.push(Instruction::Echo(lower_expr(expression)));
            }
        }
    }
    Module { instructions }
}

fn lower_assignment_value(name: &str, op: AssignmentOp, value: &Expr) -> ValueExpr {
    let right = lower_expr(value);
    match op {
        AssignmentOp::Assign => right,
        AssignmentOp::AddAssign => lower_compound_assignment(name, BinaryOp::Add, right),
        AssignmentOp::ConcatAssign => lower_compound_assignment(name, BinaryOp::Concat, right),
    }
}

fn lower_compound_assignment(name: &str, op: BinaryOp, right: ValueExpr) -> ValueExpr {
    ValueExpr::Binary {
        op,
        left: Box::new(ValueExpr::Load(name.to_string())),
        right: Box::new(right),
    }
}

fn lower_expr(expr: &Expr) -> ValueExpr {
    match expr {
        Expr::String(value, _) => ValueExpr::String(value.clone()),
        Expr::Int(value, _) => ValueExpr::Int(*value),
        Expr::Float(value, _) => ValueExpr::Float(*value),
        Expr::Bool(value, _) => ValueExpr::Bool(*value),
        Expr::Null(_) => ValueExpr::Null,
        Expr::Variable(name, _) => ValueExpr::Load(name.clone()),
        Expr::Unary { op, expr, .. } => ValueExpr::Unary {
            op: lower_unary_op(*op),
            expr: Box::new(lower_expr(expr)),
        },
        Expr::Cast { kind, expr, .. } => ValueExpr::Cast {
            kind: lower_cast_kind(*kind),
            expr: Box::new(lower_expr(expr)),
        },
        Expr::Binary {
            op, left, right, ..
        } => ValueExpr::Binary {
            op: lower_binary_op(*op),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },
        Expr::Grouped { expr, .. } => lower_expr(expr),
    }
}

fn lower_unary_op(op: AstUnaryOp) -> UnaryOp {
    match op {
        AstUnaryOp::Negate => UnaryOp::Negate,
        AstUnaryOp::Not => UnaryOp::Not,
    }
}

fn lower_cast_kind(kind: AstCastKind) -> CastKind {
    match kind {
        AstCastKind::Int => CastKind::Int,
        AstCastKind::Float => CastKind::Float,
        AstCastKind::String => CastKind::String,
        AstCastKind::Bool => CastKind::Bool,
    }
}

fn lower_binary_op(op: AstBinaryOp) -> BinaryOp {
    match op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Concat => BinaryOp::Concat,
        AstBinaryOp::Equal => BinaryOp::Equal,
        AstBinaryOp::NotEqual => BinaryOp::NotEqual,
        AstBinaryOp::Less => BinaryOp::Less,
        AstBinaryOp::LessEqual => BinaryOp::LessEqual,
        AstBinaryOp::Greater => BinaryOp::Greater,
        AstBinaryOp::GreaterEqual => BinaryOp::GreaterEqual,
        AstBinaryOp::And => BinaryOp::And,
        AstBinaryOp::Or => BinaryOp::Or,
    }
}
