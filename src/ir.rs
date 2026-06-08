use crate::ast::{Expr, Program, Statement};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Echo(ValueExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub fn lower(program: &Program) -> Module {
    let mut instructions = Vec::new();
    for statement in &program.statements {
        match statement {
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
    }
}
