use crate::ast::{
    ArrayDimTarget as AstArrayDimTarget, ArrayElement as AstArrayElement,
    ArrayElementValue as AstArrayElementValue, AssignmentOp,
    AssignmentTarget as AstAssignmentTarget, BinaryOp as AstBinaryOp, CastKind as AstCastKind,
    CatchClause as AstCatchClause, Expr, FunctionDecl as AstFunctionDecl,
    FunctionParameter as AstFunctionParameter, IncDecOp as AstIncDecOp,
    ListAssignmentElement as AstListAssignmentElement,
    ListAssignmentElementTarget as AstListAssignmentElementTarget,
    ListAssignmentTarget as AstListAssignmentTarget, MagicConstantKind as AstMagicConstantKind,
    Program, ReferenceTarget as AstReferenceTarget, Statement, StringPart as AstStringPart,
    TypeHint as AstTypeHint, UnaryOp as AstUnaryOp, UnsetTarget as AstUnsetTarget,
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
    pub return_by_ref: bool,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_hint: Option<TypeHint>,
    pub by_ref: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeHint {
    Null,
    Int,
    Float,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Store {
        name: String,
        value: ValueExpr,
    },
    StoreRef {
        name: String,
        target: ReferenceTarget,
    },
    StoreArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        value: ValueExpr,
        compound_op: Option<BinaryOp>,
        line: usize,
    },
    StoreArrayDimRef {
        target: ArrayDimTarget,
        source: ReferenceTarget,
    },
    Increment {
        name: String,
        op: IncDecOp,
        line: usize,
    },
    UnsetVariable {
        name: String,
    },
    UnsetArrayDim {
        array: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    DefineConstant {
        name: String,
        value: ValueExpr,
        line: usize,
    },
    Expression(ValueExpr),
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
    Try {
        body: Vec<Instruction>,
        catches: Vec<CatchClause>,
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
    Foreach {
        iterable: ValueExpr,
        key: Option<String>,
        value: String,
        value_by_ref: bool,
        body: Vec<Instruction>,
        line: usize,
    },
    Switch {
        expression: ValueExpr,
        cases: Vec<SwitchCase>,
    },
    Break {
        level: usize,
        line: usize,
    },
    Continue {
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
pub struct CatchClause {
    pub type_name: String,
    pub variable: Option<String>,
    pub body: Vec<Instruction>,
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
    Load {
        name: String,
        line: usize,
    },
    Assign {
        target: AssignmentTarget,
        value: Box<ValueExpr>,
    },
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
    Isset {
        targets: Vec<ValueExpr>,
    },
    Empty {
        target: Box<ValueExpr>,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        line: usize,
    },
    DynamicCall {
        callee: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        line: usize,
    },
    MethodCall {
        receiver: Box<ValueExpr>,
        name: String,
        arguments: Vec<ValueExpr>,
        line: usize,
    },
    Unary {
        op: UnaryOp,
        expr: Box<ValueExpr>,
        line: usize,
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
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<ValueExpr>,
    pub value: ArrayElementValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementValue {
    Value(ValueExpr),
    Reference(ReferenceTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDimTarget {
    pub array: String,
    pub index: Option<ValueExpr>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable {
        name: String,
        line: usize,
    },
    ArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    List(ListAssignmentTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentTarget {
    pub elements: Vec<ListAssignmentElement>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListAssignmentElement {
    pub key: Option<ValueExpr>,
    pub target: ListAssignmentElementTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListAssignmentElementTarget {
    Value(Box<AssignmentTarget>),
    Reference(ReferenceTarget),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceTarget {
    Variable { name: String, line: usize },
    ArrayDim(ArrayDimTarget),
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
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
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
        return_by_ref: function.return_by_ref,
        body: lower_statements(&function.body),
    }
}

fn lower_parameter(parameter: &AstFunctionParameter) -> FunctionParameter {
    FunctionParameter {
        name: parameter.name.clone(),
        type_hint: parameter.type_hint.map(lower_type_hint),
        by_ref: parameter.by_ref,
    }
}

fn lower_type_hint(type_hint: AstTypeHint) -> TypeHint {
    match type_hint {
        AstTypeHint::Null => TypeHint::Null,
        AstTypeHint::Int => TypeHint::Int,
        AstTypeHint::Float => TypeHint::Float,
        AstTypeHint::String => TypeHint::String,
        AstTypeHint::Bool => TypeHint::Bool,
    }
}

fn lower_statements(statements: &[Statement]) -> Vec<Instruction> {
    let mut instructions = Vec::new();
    for statement in statements {
        match statement {
            Statement::Assign {
                name,
                op,
                value,
                span,
            } => {
                instructions.push(Instruction::Store {
                    name: name.clone(),
                    value: lower_assignment_value(name, *op, value, span.line),
                });
            }
            Statement::AssignRef { name, target, .. } => {
                instructions.push(Instruction::StoreRef {
                    name: name.clone(),
                    target: lower_reference_target(target),
                });
            }
            Statement::ArrayAssign {
                target, op, value, ..
            } => {
                instructions.push(lower_array_dim_store(target, *op, value));
            }
            Statement::ArrayAssignRef { target, source, .. } => {
                instructions.push(Instruction::StoreArrayDimRef {
                    target: lower_array_dim_target(target),
                    source: lower_reference_target(source),
                });
            }
            Statement::Increment { name, op, span } => {
                instructions.push(Instruction::Increment {
                    name: name.clone(),
                    op: lower_inc_dec_op(*op),
                    line: span.line,
                });
            }
            Statement::Unset { targets, .. } => {
                for target in targets {
                    instructions.push(lower_unset_target(target));
                }
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
            Statement::Expression { expression, .. } => {
                instructions.push(Instruction::Expression(lower_expr(expression)));
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
            Statement::Foreach {
                iterable,
                key,
                value,
                value_by_ref,
                body,
                span,
            } => {
                instructions.push(Instruction::Foreach {
                    iterable: lower_expr(iterable),
                    key: key.clone(),
                    value: value.clone(),
                    value_by_ref: *value_by_ref,
                    body: lower_statements(body),
                    line: span.line,
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
            Statement::Continue { level, span } => {
                instructions.push(Instruction::Continue {
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
            Statement::Try { body, catches, .. } => {
                instructions.push(Instruction::Try {
                    body: lower_statements(body),
                    catches: catches.iter().map(lower_catch_clause).collect(),
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

fn lower_array_dim_store(
    target: &AstArrayDimTarget,
    op: AssignmentOp,
    value: &Expr,
) -> Instruction {
    Instruction::StoreArrayDim {
        array: target.array.clone(),
        dimensions: target
            .dimensions
            .iter()
            .map(|dimension| dimension.as_ref().map(lower_expr))
            .collect(),
        value: lower_expr(value),
        compound_op: assignment_op_binary_op(op),
        line: target.span.line,
    }
}

fn lower_array_dim_target(target: &AstArrayDimTarget) -> ArrayDimTarget {
    ArrayDimTarget {
        array: target.array.clone(),
        index: target
            .dimensions
            .first()
            .and_then(|dimension| dimension.as_ref())
            .map(lower_expr),
        line: target.span.line,
    }
}

fn lower_assignment_target(target: &AstAssignmentTarget) -> AssignmentTarget {
    match target {
        AstAssignmentTarget::Variable { name, span } => AssignmentTarget::Variable {
            name: name.clone(),
            line: span.line,
        },
        AstAssignmentTarget::ArrayDim(target) => AssignmentTarget::ArrayDim {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| dimension.as_ref().map(lower_expr))
                .collect(),
            line: target.span.line,
        },
        AstAssignmentTarget::List(target) => {
            AssignmentTarget::List(lower_list_assignment_target(target))
        }
    }
}

fn lower_list_assignment_target(target: &AstListAssignmentTarget) -> ListAssignmentTarget {
    ListAssignmentTarget {
        elements: target
            .elements
            .iter()
            .map(lower_list_assignment_element)
            .collect(),
        line: target.span.line,
    }
}

fn lower_list_assignment_element(element: &AstListAssignmentElement) -> ListAssignmentElement {
    ListAssignmentElement {
        key: element.key.as_ref().map(lower_expr),
        target: match &element.target {
            AstListAssignmentElementTarget::Value(target) => {
                ListAssignmentElementTarget::Value(Box::new(lower_assignment_target(target)))
            }
            AstListAssignmentElementTarget::Reference(target) => {
                ListAssignmentElementTarget::Reference(lower_reference_target(target))
            }
        },
    }
}

fn lower_reference_target(target: &AstReferenceTarget) -> ReferenceTarget {
    match target {
        AstReferenceTarget::Variable { name, span } => ReferenceTarget::Variable {
            name: name.clone(),
            line: span.line,
        },
        AstReferenceTarget::ArrayDim(target) => {
            ReferenceTarget::ArrayDim(lower_array_dim_target(target))
        }
    }
}

fn lower_unset_target(target: &AstUnsetTarget) -> Instruction {
    match target {
        AstUnsetTarget::Variable { name, .. } => Instruction::UnsetVariable { name: name.clone() },
        AstUnsetTarget::ArrayDim(target) => Instruction::UnsetArrayDim {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| {
                    lower_expr(
                        dimension
                            .as_ref()
                            .expect("parser rejects append syntax in unset targets"),
                    )
                })
                .collect(),
            line: target.span.line,
        },
    }
}

fn assignment_op_binary_op(op: AssignmentOp) -> Option<BinaryOp> {
    match op {
        AssignmentOp::Assign => None,
        AssignmentOp::AddAssign => Some(BinaryOp::Add),
        AssignmentOp::SubtractAssign => Some(BinaryOp::Subtract),
        AssignmentOp::MultiplyAssign => Some(BinaryOp::Multiply),
        AssignmentOp::PowerAssign => Some(BinaryOp::Power),
        AssignmentOp::DivideAssign => Some(BinaryOp::Divide),
        AssignmentOp::ModuloAssign => Some(BinaryOp::Modulo),
        AssignmentOp::ConcatAssign => Some(BinaryOp::Concat),
        AssignmentOp::BitwiseAndAssign => Some(BinaryOp::BitwiseAnd),
        AssignmentOp::BitwiseOrAssign => Some(BinaryOp::BitwiseOr),
        AssignmentOp::BitwiseXorAssign => Some(BinaryOp::BitwiseXor),
        AssignmentOp::ShiftLeftAssign => Some(BinaryOp::ShiftLeft),
        AssignmentOp::ShiftRightAssign => Some(BinaryOp::ShiftRight),
    }
}

fn lower_catch_clause(catch: &AstCatchClause) -> CatchClause {
    CatchClause {
        type_name: catch.type_name.clone(),
        variable: catch.variable.clone(),
        body: lower_statements(&catch.body),
    }
}

fn lower_assignment_value(name: &str, op: AssignmentOp, value: &Expr, line: usize) -> ValueExpr {
    let right = lower_expr(value);
    match op {
        AssignmentOp::Assign => right,
        AssignmentOp::AddAssign => lower_compound_assignment(name, line, BinaryOp::Add, right),
        AssignmentOp::SubtractAssign => {
            lower_compound_assignment(name, line, BinaryOp::Subtract, right)
        }
        AssignmentOp::MultiplyAssign => {
            lower_compound_assignment(name, line, BinaryOp::Multiply, right)
        }
        AssignmentOp::PowerAssign => lower_compound_assignment(name, line, BinaryOp::Power, right),
        AssignmentOp::DivideAssign => {
            lower_compound_assignment(name, line, BinaryOp::Divide, right)
        }
        AssignmentOp::ModuloAssign => {
            lower_compound_assignment(name, line, BinaryOp::Modulo, right)
        }
        AssignmentOp::ConcatAssign => {
            lower_compound_assignment(name, line, BinaryOp::Concat, right)
        }
        AssignmentOp::BitwiseAndAssign => {
            lower_compound_assignment(name, line, BinaryOp::BitwiseAnd, right)
        }
        AssignmentOp::BitwiseOrAssign => {
            lower_compound_assignment(name, line, BinaryOp::BitwiseOr, right)
        }
        AssignmentOp::BitwiseXorAssign => {
            lower_compound_assignment(name, line, BinaryOp::BitwiseXor, right)
        }
        AssignmentOp::ShiftLeftAssign => {
            lower_compound_assignment(name, line, BinaryOp::ShiftLeft, right)
        }
        AssignmentOp::ShiftRightAssign => {
            lower_compound_assignment(name, line, BinaryOp::ShiftRight, right)
        }
    }
}

fn lower_compound_assignment(name: &str, line: usize, op: BinaryOp, right: ValueExpr) -> ValueExpr {
    ValueExpr::Binary {
        op,
        left: Box::new(ValueExpr::Load {
            name: name.to_string(),
            line,
        }),
        right: Box::new(right),
        line,
    }
}

fn lower_expr(expr: &Expr) -> ValueExpr {
    match expr {
        Expr::String(value, _) => ValueExpr::String(value.clone()),
        Expr::InterpolatedString(parts, span) => lower_interpolated_string(parts, span.line),
        Expr::Int(value, _) => ValueExpr::Int(*value),
        Expr::Float(value, _) => ValueExpr::Float(*value),
        Expr::Bool(value, _) => ValueExpr::Bool(*value),
        Expr::Null(_) => ValueExpr::Null,
        Expr::Variable(name, span) => ValueExpr::Load {
            name: name.clone(),
            line: span.line,
        },
        Expr::Assign { target, value, .. } => ValueExpr::Assign {
            target: lower_assignment_target(target),
            value: Box::new(lower_expr(value)),
        },
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
            index: Box::new(lower_expr(
                index
                    .as_ref()
                    .expect("parser rejects append array reads outside assignment targets"),
            )),
            line: span.line,
        },
        Expr::Isset { targets, .. } => ValueExpr::Isset {
            targets: targets.iter().map(lower_expr).collect(),
        },
        Expr::Empty { target, .. } => ValueExpr::Empty {
            target: Box::new(lower_expr(target)),
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
        Expr::DynamicCall {
            callee,
            arguments,
            span,
        } => ValueExpr::DynamicCall {
            callee: Box::new(lower_expr(callee)),
            arguments: arguments.iter().map(lower_expr).collect(),
            line: span.line,
        },
        Expr::MethodCall {
            receiver,
            name,
            arguments,
            span,
        } => ValueExpr::MethodCall {
            receiver: Box::new(lower_expr(receiver)),
            name: name.clone(),
            arguments: arguments.iter().map(lower_expr).collect(),
            line: span.line,
        },
        Expr::Unary { op, expr, span } => ValueExpr::Unary {
            op: lower_unary_op(*op),
            expr: Box::new(lower_expr(expr)),
            line: span.line,
        },
        Expr::Cast { kind, expr, span } => ValueExpr::Cast {
            kind: lower_cast_kind(*kind),
            expr: Box::new(lower_expr(expr)),
            line: span.line,
        },
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => ValueExpr::Binary {
            op: lower_binary_op(*op),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
            line: span.line,
        },
        Expr::Grouped { expr, .. } => lower_expr(expr),
    }
}

fn lower_array_element(element: &AstArrayElement) -> ArrayElement {
    ArrayElement {
        key: element.key.as_ref().map(lower_expr),
        value: lower_array_element_value(&element.value),
    }
}

fn lower_array_element_value(value: &AstArrayElementValue) -> ArrayElementValue {
    match value {
        AstArrayElementValue::Value(value) => ArrayElementValue::Value(lower_expr(value)),
        AstArrayElementValue::Reference(target) => {
            ArrayElementValue::Reference(lower_reference_target(target))
        }
    }
}

fn lower_interpolated_string(parts: &[AstStringPart], line: usize) -> ValueExpr {
    let mut values = parts.iter().filter_map(|part| match part {
        AstStringPart::Literal(value) if value.is_empty() => None,
        AstStringPart::Literal(value) => Some(ValueExpr::String(value.clone())),
        AstStringPart::Variable(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::Load {
                name: name.clone(),
                line,
            }),
            line,
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
            line,
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
        AstUnaryOp::ErrorSuppress => UnaryOp::ErrorSuppress,
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
        AstBinaryOp::Coalesce => BinaryOp::Coalesce,
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
        AstBinaryOp::Xor => BinaryOp::Xor,
        AstBinaryOp::Or => BinaryOp::Or,
    }
}
