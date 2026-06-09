use crate::ast::{
    ArrayElement as AstArrayElement, AssignmentOp, BinaryOp as AstBinaryOp,
    CastKind as AstCastKind, Expr, FunctionDecl as AstFunctionDecl,
    FunctionParameter as AstFunctionParameter, IncDecOp as AstIncDecOp,
    MagicConstantKind as AstMagicConstantKind, Program, Statement, StringPart as AstStringPart,
    TypeHint as AstTypeHint, UnaryOp as AstUnaryOp,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub functions: Vec<FunctionDecl>,
    pub instructions: Vec<Instruction>,
    pub source_file: String,
    pub source_dir: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_hint: Option<TypeHint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeHint {
    Null,
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
    DefineConstant {
        name: String,
        value: ValueExpr,
        line: usize,
    },
    Echo(ValueExpr),
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        line: usize,
    },
    Return {
        value: Option<ValueExpr>,
        line: usize,
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
    DoWhile {
        body: Vec<Instruction>,
        condition: ValueExpr,
    },
    For {
        initializers: Vec<Instruction>,
        condition: Option<ValueExpr>,
        updates: Vec<Instruction>,
        body: Vec<Instruction>,
    },
    Switch {
        expression: ValueExpr,
        cases: Vec<SwitchCase>,
    },
    Break {
        level: usize,
        line: usize,
    },
    Label {
        name: String,
    },
    Goto {
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub condition: Option<ValueExpr>,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Load(String),
    Constant(String),
    MagicConstant {
        kind: MagicConstantKind,
        line: usize,
    },
    Array(Vec<ArrayElement>),
    ArrayAccess {
        array: Box<ValueExpr>,
        index: Box<ValueExpr>,
        line: usize,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        line: usize,
    },
    Unary {
        op: UnaryOp,
        expr: Box<ValueExpr>,
    },
    Cast {
        kind: CastKind,
        expr: Box<ValueExpr>,
        line: usize,
    },
    Binary {
        op: BinaryOp,
        left: Box<ValueExpr>,
        right: Box<ValueExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<ValueExpr>,
    pub value: ValueExpr,
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
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    Spaceship,
    Identical,
    NotIdentical,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Positive,
    Negate,
    Not,
    BitwiseNot,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecOp {
    Increment,
    Decrement,
}

pub fn lower(program: &Program) -> Module {
    lower_with_source(program, String::new(), String::new())
}

pub fn lower_with_source(program: &Program, source_file: String, source_dir: String) -> Module {
    Module {
        functions: program.functions.iter().map(lower_function).collect(),
        instructions: lower_statements(&program.statements),
        source_file,
        source_dir,
    }
}

fn lower_function(function: &AstFunctionDecl) -> FunctionDecl {
    FunctionDecl {
        name: function.name.clone(),
        parameters: function.parameters.iter().map(lower_parameter).collect(),
        return_type: function.return_type.map(lower_type_hint),
        body: lower_statements(&function.body),
    }
}

fn lower_parameter(parameter: &AstFunctionParameter) -> FunctionParameter {
    FunctionParameter {
        name: parameter.name.clone(),
        type_hint: parameter.type_hint.map(lower_type_hint),
    }
}

fn lower_type_hint(type_hint: AstTypeHint) -> TypeHint {
    match type_hint {
        AstTypeHint::Null => TypeHint::Null,
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
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    instructions.push(Instruction::DefineConstant {
                        name: declaration.name.clone(),
                        value: lower_expr(&declaration.value),
                        line: declaration.span.line,
                    });
                }
            }
            Statement::Call {
                name,
                arguments,
                span,
            } => {
                instructions.push(Instruction::InternalCall {
                    name: name.clone(),
                    arguments: arguments.iter().map(lower_expr).collect(),
                    line: span.line,
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
            Statement::Block { statements, .. } => {
                instructions.extend(lower_statements(statements));
            }
            Statement::While {
                condition, body, ..
            } => {
                instructions.push(Instruction::While {
                    condition: lower_expr(condition),
                    body: lower_statements(body),
                });
            }
            Statement::DoWhile {
                body, condition, ..
            } => {
                instructions.push(Instruction::DoWhile {
                    body: lower_statements(body),
                    condition: lower_expr(condition),
                });
            }
            Statement::For {
                initializers,
                condition,
                updates,
                body,
                ..
            } => {
                instructions.push(Instruction::For {
                    initializers: lower_statements(initializers),
                    condition: condition.as_ref().map(lower_expr),
                    updates: lower_statements(updates),
                    body: lower_statements(body),
                });
            }
            Statement::Switch {
                expression, cases, ..
            } => {
                instructions.push(Instruction::Switch {
                    expression: lower_expr(expression),
                    cases: cases
                        .iter()
                        .map(|case| SwitchCase {
                            condition: case.condition.as_ref().map(lower_expr),
                            body: lower_statements(&case.body),
                        })
                        .collect(),
                });
            }
            Statement::Break { level, span } => {
                instructions.push(Instruction::Break {
                    level: *level,
                    line: span.line,
                });
            }
            Statement::Return { value, span } => {
                instructions.push(Instruction::Return {
                    value: value.as_ref().map(lower_expr),
                    line: span.line,
                });
            }
            Statement::Label { name, .. } => {
                instructions.push(Instruction::Label { name: name.clone() });
            }
            Statement::Goto { label, .. } => {
                instructions.push(Instruction::Goto {
                    label: label.clone(),
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
        AssignmentOp::PowerAssign => lower_compound_assignment(name, BinaryOp::Power, right),
        AssignmentOp::DivideAssign => lower_compound_assignment(name, BinaryOp::Divide, right),
        AssignmentOp::ModuloAssign => lower_compound_assignment(name, BinaryOp::Modulo, right),
        AssignmentOp::ConcatAssign => lower_compound_assignment(name, BinaryOp::Concat, right),
        AssignmentOp::BitwiseAndAssign => {
            lower_compound_assignment(name, BinaryOp::BitwiseAnd, right)
        }
        AssignmentOp::BitwiseOrAssign => {
            lower_compound_assignment(name, BinaryOp::BitwiseOr, right)
        }
        AssignmentOp::BitwiseXorAssign => {
            lower_compound_assignment(name, BinaryOp::BitwiseXor, right)
        }
        AssignmentOp::ShiftLeftAssign => {
            lower_compound_assignment(name, BinaryOp::ShiftLeft, right)
        }
        AssignmentOp::ShiftRightAssign => {
            lower_compound_assignment(name, BinaryOp::ShiftRight, right)
        }
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
        Expr::InterpolatedString(parts, _) => lower_interpolated_string(parts),
        Expr::Int(value, _) => ValueExpr::Int(*value),
        Expr::Float(value, _) => ValueExpr::Float(*value),
        Expr::Bool(value, _) => ValueExpr::Bool(*value),
        Expr::Null(_) => ValueExpr::Null,
        Expr::Variable(name, _) => ValueExpr::Load(name.clone()),
        Expr::Constant(name, _) => ValueExpr::Constant(name.clone()),
        Expr::MagicConstant(kind, span) => ValueExpr::MagicConstant {
            kind: lower_magic_constant_kind(*kind),
            line: span.line,
        },
        Expr::Array { elements, .. } => {
            ValueExpr::Array(elements.iter().map(lower_array_element).collect())
        }
        Expr::ArrayAccess { array, index, span } => ValueExpr::ArrayAccess {
            array: Box::new(lower_expr(array)),
            index: Box::new(lower_expr(index)),
            line: span.line,
        },
        Expr::Call {
            name,
            arguments,
            span,
        } => ValueExpr::InternalCall {
            name: name.clone(),
            arguments: arguments.iter().map(lower_expr).collect(),
            line: span.line,
        },
        Expr::Unary { op, expr, .. } => ValueExpr::Unary {
            op: lower_unary_op(*op),
            expr: Box::new(lower_expr(expr)),
        },
        Expr::Cast { kind, expr, span } => ValueExpr::Cast {
            kind: lower_cast_kind(*kind),
            expr: Box::new(lower_expr(expr)),
            line: span.line,
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

fn lower_array_element(element: &AstArrayElement) -> ArrayElement {
    ArrayElement {
        key: element.key.as_ref().map(lower_expr),
        value: lower_expr(&element.value),
    }
}

fn lower_interpolated_string(parts: &[AstStringPart]) -> ValueExpr {
    let mut values = parts.iter().filter_map(|part| match part {
        AstStringPart::Literal(value) if value.is_empty() => None,
        AstStringPart::Literal(value) => Some(ValueExpr::String(value.clone())),
        AstStringPart::Variable(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::Load(name.clone())),
            line: 0,
        }),
    });

    let Some(mut expr) = values.next() else {
        return ValueExpr::String(String::new());
    };

    for next in values {
        expr = ValueExpr::Binary {
            op: BinaryOp::Concat,
            left: Box::new(expr),
            right: Box::new(next),
        };
    }
    expr
}

fn lower_unary_op(op: AstUnaryOp) -> UnaryOp {
    match op {
        AstUnaryOp::Positive => UnaryOp::Positive,
        AstUnaryOp::Negate => UnaryOp::Negate,
        AstUnaryOp::Not => UnaryOp::Not,
        AstUnaryOp::BitwiseNot => UnaryOp::BitwiseNot,
    }
}

fn lower_cast_kind(kind: AstCastKind) -> CastKind {
    match kind {
        AstCastKind::Int => CastKind::Int,
        AstCastKind::Integer => CastKind::Integer,
        AstCastKind::Float => CastKind::Float,
        AstCastKind::Double => CastKind::Double,
        AstCastKind::String => CastKind::String,
        AstCastKind::Binary => CastKind::Binary,
        AstCastKind::Bool => CastKind::Bool,
        AstCastKind::Boolean => CastKind::Boolean,
    }
}

fn lower_magic_constant_kind(kind: AstMagicConstantKind) -> MagicConstantKind {
    match kind {
        AstMagicConstantKind::Line => MagicConstantKind::Line,
        AstMagicConstantKind::File => MagicConstantKind::File,
        AstMagicConstantKind::Dir => MagicConstantKind::Dir,
        AstMagicConstantKind::Function => MagicConstantKind::Function,
        AstMagicConstantKind::Method => MagicConstantKind::Method,
        AstMagicConstantKind::Class => MagicConstantKind::Class,
        AstMagicConstantKind::Trait => MagicConstantKind::Trait,
        AstMagicConstantKind::Namespace => MagicConstantKind::Namespace,
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
        AstBinaryOp::Power => BinaryOp::Power,
        AstBinaryOp::Divide => BinaryOp::Divide,
        AstBinaryOp::Modulo => BinaryOp::Modulo,
        AstBinaryOp::Concat => BinaryOp::Concat,
        AstBinaryOp::ShiftLeft => BinaryOp::ShiftLeft,
        AstBinaryOp::ShiftRight => BinaryOp::ShiftRight,
        AstBinaryOp::Equal => BinaryOp::Equal,
        AstBinaryOp::NotEqual => BinaryOp::NotEqual,
        AstBinaryOp::Spaceship => BinaryOp::Spaceship,
        AstBinaryOp::Identical => BinaryOp::Identical,
        AstBinaryOp::NotIdentical => BinaryOp::NotIdentical,
        AstBinaryOp::Less => BinaryOp::Less,
        AstBinaryOp::LessEqual => BinaryOp::LessEqual,
        AstBinaryOp::Greater => BinaryOp::Greater,
        AstBinaryOp::GreaterEqual => BinaryOp::GreaterEqual,
        AstBinaryOp::BitwiseAnd => BinaryOp::BitwiseAnd,
        AstBinaryOp::BitwiseXor => BinaryOp::BitwiseXor,
        AstBinaryOp::BitwiseOr => BinaryOp::BitwiseOr,
        AstBinaryOp::And => BinaryOp::And,
        AstBinaryOp::Or => BinaryOp::Or,
    }
}
