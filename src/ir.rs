use crate::ast::{
    AssignmentOp, BinaryOp as AstBinaryOp, CastKind as AstCastKind, Expr, IncDecOp as AstIncDecOp,
    Program, Statement, UnaryOp as AstUnaryOp,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Store {
        name: String,
        value: ValueExpr,
    },
    Increment {
        name: String,
        op: IncDecOp,
    },
    Echo(ValueExpr),
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
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
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Concat,
    Equal,
    NotEqual,
    Identical,
    NotIdentical,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecOp {
    Increment,
    Decrement,
}

pub fn lower(program: &Program) -> Module {
    Module {
        instructions: lower_statements(&program.statements),
    }
}

fn lower_statements(statements: &[Statement]) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for statement in statements {
        match statement {
            Statement::Assign {
                name, op, value, ..
            } => {
                instructions.push(Instruction::Store {
                    name: name.clone(),
                    value: lower_assignment_value(name, *op, value),
                });
            }
            Statement::Increment { name, op, .. } => {
                instructions.push(Instruction::Increment {
                    name: name.clone(),
                    op: lower_inc_dec_op(*op),
                });
            }
            Statement::Call {
                name, arguments, ..
            } => {
                instructions.push(Instruction::InternalCall {
                    name: name.clone(),
                    arguments: arguments.iter().map(lower_expr).collect(),
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
            Statement::InlineHtml { content, .. } => {
                instructions.push(Instruction::Echo(ValueExpr::String(content.clone())));
            }
            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                instructions.push(Instruction::Branch {
                    condition: lower_expr(condition),
                    then_body: lower_statements(then_body),
                    else_body: lower_statements(else_body),
                });
            }
            Statement::While {
                condition, body, ..
            } => {
                instructions.push(Instruction::While {
                    condition: lower_expr(condition),
                    body: lower_statements(body),
                });
            }
        }
    }
    instructions
}

fn lower_assignment_value(name: &str, op: AssignmentOp, value: &Expr) -> ValueExpr {
    let right = lower_expr(value);
    match op {
        AssignmentOp::Assign => right,
        AssignmentOp::AddAssign => lower_compound_assignment(name, BinaryOp::Add, right),
        AssignmentOp::SubtractAssign => lower_compound_assignment(name, BinaryOp::Subtract, right),
        AssignmentOp::MultiplyAssign => lower_compound_assignment(name, BinaryOp::Multiply, right),
        AssignmentOp::DivideAssign => lower_compound_assignment(name, BinaryOp::Divide, right),
        AssignmentOp::ModuloAssign => lower_compound_assignment(name, BinaryOp::Modulo, right),
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
        AstUnaryOp::Positive => UnaryOp::Positive,
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

fn lower_inc_dec_op(op: AstIncDecOp) -> IncDecOp {
    match op {
        AstIncDecOp::Increment => IncDecOp::Increment,
        AstIncDecOp::Decrement => IncDecOp::Decrement,
    }
}

fn lower_binary_op(op: AstBinaryOp) -> BinaryOp {
    match op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Subtract => BinaryOp::Subtract,
        AstBinaryOp::Multiply => BinaryOp::Multiply,
        AstBinaryOp::Divide => BinaryOp::Divide,
        AstBinaryOp::Modulo => BinaryOp::Modulo,
        AstBinaryOp::Concat => BinaryOp::Concat,
        AstBinaryOp::Equal => BinaryOp::Equal,
        AstBinaryOp::NotEqual => BinaryOp::NotEqual,
        AstBinaryOp::Identical => BinaryOp::Identical,
        AstBinaryOp::NotIdentical => BinaryOp::NotIdentical,
        AstBinaryOp::Less => BinaryOp::Less,
        AstBinaryOp::LessEqual => BinaryOp::LessEqual,
        AstBinaryOp::Greater => BinaryOp::Greater,
        AstBinaryOp::GreaterEqual => BinaryOp::GreaterEqual,
        AstBinaryOp::And => BinaryOp::And,
        AstBinaryOp::Or => BinaryOp::Or,
    }
}
