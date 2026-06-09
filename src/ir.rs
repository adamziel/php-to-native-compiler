use crate::ast::{BinaryOp as AstBinaryOp, Expr, Program, Statement};

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
}

pub fn lower(program: &Program) -> Module {
    let mut instructions = Vec::new();
    for statement in &program.statements {
        match statement {
            Statement::Assign { name, value, .. } => {
                instructions.push(Instruction::Store {
                    name: name.clone(),
                    value: lower_expr(value),
                });
            }
            Statement::Echo { expressions, .. } => {
                for expression in expressions {
                    instructions.push(Instruction::Echo(lower_expr(expression)));
                }
            }
        }
    }
    Module { instructions }
}

fn lower_expr(expr: &Expr) -> ValueExpr {
    match expr {
        Expr::String(value, _) => ValueExpr::String(value.clone()),
        Expr::Int(value, _) => ValueExpr::Int(*value),
        Expr::Float(value, _) => ValueExpr::Float(*value),
        Expr::Bool(value, _) => ValueExpr::Bool(*value),
        Expr::Null(_) => ValueExpr::Null,
        Expr::Variable(name, _) => ValueExpr::Load(name.clone()),
        Expr::Binary {
            op, left, right, ..
        } => ValueExpr::Binary {
            op: lower_binary_op(*op),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },
    }
}

fn lower_binary_op(op: AstBinaryOp) -> BinaryOp {
    match op {
        AstBinaryOp::Add => BinaryOp::Add,
        AstBinaryOp::Concat => BinaryOp::Concat,
    }
}
