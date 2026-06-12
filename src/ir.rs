use std::collections::HashMap;

use crate::ast::{
    AnonymousFunction as AstAnonymousFunction, ArrayDimTarget as AstArrayDimTarget,
    ArrayElement as AstArrayElement, ArrayElementValue as AstArrayElementValue, AssignmentOp,
    AssignmentTarget as AstAssignmentTarget, BinaryOp as AstBinaryOp, CastKind as AstCastKind,
    CatchClause as AstCatchClause, ClassDecl as AstClassDecl,
    ClosureUseCapture as AstClosureUseCapture, Expr, FunctionParameter as AstFunctionParameter,
    IncDecOp as AstIncDecOp, IncDecResult as AstIncDecResult, IncDecTarget as AstIncDecTarget,
    IncludeKind as AstIncludeKind, ListAssignmentElement as AstListAssignmentElement,
    ListAssignmentElementTarget as AstListAssignmentElementTarget,
    ListAssignmentTarget as AstListAssignmentTarget, MagicConstantKind as AstMagicConstantKind,
    Program, PropertyVisibility as AstPropertyVisibility, ReferenceTarget as AstReferenceTarget,
    Statement, StringInterpolationIndex as AstStringInterpolationIndex,
    StringPart as AstStringPart, TypeHint as AstTypeHint, UnaryOp as AstUnaryOp,
    UnsetTarget as AstUnsetTarget,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub classes: Vec<ClassDecl>,
    pub functions: Vec<FunctionDecl>,
    pub includes: Vec<IncludeFile>,
    pub instructions: Vec<Instruction>,
    pub source_file: String,
    pub source_dir: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeFile {
    pub source_file: String,
    pub source_dir: String,
    pub path_aliases: Vec<String>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeSource {
    pub source_file: String,
    pub source_dir: String,
    pub path_aliases: Vec<String>,
    pub program: Program,
}

pub type IncludeResolutionMap = HashMap<(String, usize, usize), Vec<usize>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent_name: Option<String>,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub value: Option<ValueExpr>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyVisibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticPropertyDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub value: Option<ValueExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub function_index: usize,
    pub is_static: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub is_static: bool,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_anonymous: bool,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub type_hint: Option<TypeHint>,
    pub by_ref: bool,
    pub is_variadic: bool,
    pub default_value: Option<ValueExpr>,
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
        source: ValueExpr,
        line: usize,
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
        source: ValueExpr,
    },
    Increment {
        target: IncDecTarget,
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
    UnsetDynamicArrayDim {
        name: ValueExpr,
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
        argument_names: Vec<Option<String>>,
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
        key: Option<AssignmentTarget>,
        value: AssignmentTarget,
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
    Closure {
        function_index: usize,
        captures: Vec<ClosureCapture>,
        line: usize,
    },
    Load {
        name: String,
        line: usize,
    },
    LegacyDollarBraceStringVariable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    IncDec {
        target: IncDecTarget,
        op: IncDecOp,
        result: IncDecResult,
        line: usize,
    },
    Assign {
        target: AssignmentTarget,
        op: AssignmentOp,
        value: Box<ValueExpr>,
    },
    AssignRef {
        target: AssignmentTarget,
        source: Box<ValueExpr>,
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
    Print {
        expression: Box<ValueExpr>,
    },
    Include {
        kind: AstIncludeKind,
        path: Box<ValueExpr>,
        candidates: Vec<usize>,
        line: usize,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        line: usize,
    },
    DynamicCall {
        callee: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        line: usize,
    },
    MethodCall {
        receiver: Box<ValueExpr>,
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        line: usize,
    },
    NewObject {
        class_name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        line: usize,
    },
    PropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    StaticPropertyFetch {
        class_name: String,
        name: String,
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
    Ternary {
        condition: Box<ValueExpr>,
        if_true: Option<Box<ValueExpr>>,
        if_false: Box<ValueExpr>,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureCapture {
    pub name: String,
    pub by_ref: bool,
    pub line: usize,
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
    pub dimensions: Vec<Option<ValueExpr>>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicArrayDim {
        name: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    StaticProperty {
        class_name: String,
        name: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncDecResult {
    Pre,
    Post,
}

pub fn lower(program: &Program) -> Module {
    lower_with_source(program, String::new(), String::new())
}

pub fn lower_with_source(program: &Program, source_file: String, source_dir: String) -> Module {
    lower_with_source_and_includes(
        program,
        source_file,
        source_dir,
        Vec::new(),
        &IncludeResolutionMap::new(),
    )
}

pub fn lower_with_source_and_includes(
    program: &Program,
    source_file: String,
    source_dir: String,
    include_sources: Vec<IncludeSource>,
    include_resolutions: &IncludeResolutionMap,
) -> Module {
    let mut context = LoweringContext::new(
        program,
        source_file.clone(),
        source_dir.clone(),
        include_resolutions,
    );
    for (index, function) in program.functions.iter().enumerate() {
        let body = context.lower_statements(&function.body);
        context.functions[index].body = body;
    }
    let classes = program
        .classes
        .iter()
        .map(|class| context.lower_class(class))
        .collect();
    let includes = include_sources
        .iter()
        .map(|include| context.lower_include_source(include))
        .collect();
    let instructions = context.lower_statements(&program.statements);
    Module {
        classes,
        functions: context.functions,
        includes,
        instructions,
        source_file,
        source_dir,
    }
}

struct LoweringContext<'a> {
    functions: Vec<FunctionDecl>,
    source_file: String,
    source_dir: String,
    include_resolutions: &'a IncludeResolutionMap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncDecTarget {
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    DynamicArrayDim {
        name: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ArrayDim {
        array: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    StaticProperty {
        class_name: String,
        name: String,
        line: usize,
    },
}

impl<'a> LoweringContext<'a> {
    fn new(
        program: &Program,
        source_file: String,
        source_dir: String,
        include_resolutions: &'a IncludeResolutionMap,
    ) -> Self {
        let mut context = Self {
            functions: Vec::new(),
            source_file,
            source_dir,
            include_resolutions,
        };
        for function in &program.functions {
            let parameters = function
                .parameters
                .iter()
                .map(|parameter| context.lower_parameter(parameter))
                .collect();
            context.functions.push(FunctionDecl {
                name: function.name.clone(),
                class_name: None,
                method_name: None,
                is_static: false,
                parameters,
                return_type: function.return_type.map(lower_type_hint),
                return_by_ref: function.return_by_ref,
                is_anonymous: false,
                body: Vec::new(),
            });
        }
        context
    }

    fn lower_include_source(&mut self, include: &IncludeSource) -> IncludeFile {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let instructions = self.lower_statements(&include.program.statements);
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        IncludeFile {
            source_file: include.source_file.clone(),
            source_dir: include.source_dir.clone(),
            path_aliases: include.path_aliases.clone(),
            instructions,
        }
    }

    fn lower_anonymous_function(&mut self, function: &AstAnonymousFunction) -> ValueExpr {
        let function_index = self.functions.len();
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| self.lower_parameter(parameter))
            .collect();
        self.functions.push(FunctionDecl {
            name: "{closure}".to_string(),
            class_name: None,
            method_name: None,
            is_static: false,
            parameters,
            return_type: function.return_type.map(lower_type_hint),
            return_by_ref: function.return_by_ref,
            is_anonymous: true,
            body: Vec::new(),
        });
        let body = self.lower_statements(&function.body);
        self.functions[function_index].body = body;
        ValueExpr::Closure {
            function_index,
            captures: function
                .captures
                .iter()
                .map(lower_closure_capture)
                .collect(),
            line: function.span.line,
        }
    }

    fn lower_class(&mut self, class: &AstClassDecl) -> ClassDecl {
        let properties = class
            .properties
            .iter()
            .map(|property| PropertyDecl {
                name: property.name.clone(),
                visibility: lower_property_visibility(property.visibility),
                value: property.value.as_ref().map(|value| self.lower_expr(value)),
                line: property.span.line,
            })
            .collect();
        let static_properties = class
            .static_properties
            .iter()
            .map(|property| StaticPropertyDecl {
                name: property.name.clone(),
                visibility: lower_property_visibility(property.visibility),
                value: property.value.as_ref().map(|value| self.lower_expr(value)),
            })
            .collect();
        let methods = class
            .methods
            .iter()
            .map(|method| {
                let function_index = self.functions.len();
                let parameters = method
                    .parameters
                    .iter()
                    .map(|parameter| self.lower_parameter(parameter))
                    .collect();
                self.functions.push(FunctionDecl {
                    name: format!("{}::{}", class.name, method.name),
                    class_name: Some(class.name.clone()),
                    method_name: Some(method.name.clone()),
                    is_static: method.is_static,
                    parameters,
                    return_type: method.return_type.map(lower_type_hint),
                    return_by_ref: method.return_by_ref,
                    is_anonymous: false,
                    body: Vec::new(),
                });
                let body = self.lower_statements(&method.body);
                self.functions[function_index].body = body;
                MethodDecl {
                    name: method.name.clone(),
                    function_index,
                    is_static: method.is_static,
                }
            })
            .collect();
        ClassDecl {
            name: class.name.clone(),
            parent_name: class.parent_name.clone(),
            properties,
            static_properties,
            methods,
        }
    }

    fn lower_statements(&mut self, statements: &[Statement]) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        for statement in statements {
            match statement {
                Statement::Empty { .. } => {}
                Statement::Assign {
                    name,
                    op,
                    value,
                    span,
                } => {
                    if matches!(op, AssignmentOp::CoalesceAssign) {
                        instructions.push(Instruction::Expression(ValueExpr::Assign {
                            target: AssignmentTarget::Variable {
                                name: name.clone(),
                                line: span.line,
                            },
                            op: *op,
                            value: Box::new(self.lower_expr(value)),
                        }));
                    } else {
                        instructions.push(Instruction::Store {
                            name: name.clone(),
                            value: self.lower_assignment_value(name, *op, value, span.line),
                        });
                    }
                }
                Statement::AssignRef { name, source, span } => {
                    instructions.push(Instruction::StoreRef {
                        name: name.clone(),
                        source: self.lower_expr(source),
                        line: span.line,
                    });
                }
                Statement::ArrayAssign {
                    target, op, value, ..
                } => {
                    if matches!(op, AssignmentOp::CoalesceAssign) {
                        instructions.push(Instruction::Expression(ValueExpr::Assign {
                            target: self.lower_assignment_target(&AstAssignmentTarget::ArrayDim(
                                target.clone(),
                            )),
                            op: *op,
                            value: Box::new(self.lower_expr(value)),
                        }));
                    } else {
                        instructions.push(self.lower_array_dim_store(target, *op, value));
                    }
                }
                Statement::ArrayAssignRef { target, source, .. } => {
                    instructions.push(Instruction::StoreArrayDimRef {
                        target: self.lower_array_dim_target(target),
                        source: self.lower_expr(source),
                    });
                }
                Statement::Increment { target, op, span } => {
                    instructions.push(Instruction::Increment {
                        target: self.lower_inc_dec_target(target),
                        op: lower_inc_dec_op(*op),
                        line: span.line,
                    });
                }
                Statement::Unset { targets, .. } => {
                    for target in targets {
                        instructions.push(self.lower_unset_target(target));
                    }
                }
                Statement::Const { declarations, .. } => {
                    for declaration in declarations {
                        instructions.push(Instruction::DefineConstant {
                            name: declaration.name.clone(),
                            value: self.lower_expr(&declaration.value),
                            line: declaration.span.line,
                        });
                    }
                }
                Statement::Call {
                    name,
                    arguments,
                    argument_names,
                    span,
                } => {
                    let (arguments, argument_names) =
                        self.lower_internal_call_arguments(name, arguments, argument_names);
                    instructions.push(Instruction::InternalCall {
                        name: name.clone(),
                        arguments,
                        argument_names,
                        line: span.line,
                    });
                }
                Statement::Echo { expressions, .. } => {
                    for expression in expressions {
                        instructions.push(Instruction::Echo(self.lower_expr(expression)));
                    }
                }
                Statement::Print { expression, .. } => {
                    instructions.push(Instruction::Expression(ValueExpr::Print {
                        expression: Box::new(self.lower_expr(expression)),
                    }));
                }
                Statement::Expression { expression, .. } => {
                    instructions.push(Instruction::Expression(self.lower_expr(expression)));
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
                        condition: self.lower_expr(condition),
                        then_body: self.lower_statements(then_body),
                        else_body: self.lower_statements(else_body),
                    });
                }
                Statement::Block { statements, .. } => {
                    instructions.extend(self.lower_statements(statements));
                }
                Statement::While {
                    condition, body, ..
                } => {
                    instructions.push(Instruction::While {
                        condition: self.lower_expr(condition),
                        body: self.lower_statements(body),
                    });
                }
                Statement::DoWhile {
                    body, condition, ..
                } => {
                    instructions.push(Instruction::DoWhile {
                        body: self.lower_statements(body),
                        condition: self.lower_expr(condition),
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
                        initializers: self.lower_statements(initializers),
                        condition: condition
                            .as_ref()
                            .map(|condition| self.lower_expr(condition)),
                        updates: self.lower_statements(updates),
                        body: self.lower_statements(body),
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
                        iterable: self.lower_expr(iterable),
                        key: key
                            .as_ref()
                            .map(|target| self.lower_assignment_target(target)),
                        value: self.lower_assignment_target(value),
                        value_by_ref: *value_by_ref,
                        body: self.lower_statements(body),
                        line: span.line,
                    });
                }
                Statement::Switch {
                    expression, cases, ..
                } => {
                    instructions.push(Instruction::Switch {
                        expression: self.lower_expr(expression),
                        cases: cases
                            .iter()
                            .map(|case| SwitchCase {
                                condition: case
                                    .condition
                                    .as_ref()
                                    .map(|condition| self.lower_expr(condition)),
                                body: self.lower_statements(&case.body),
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
                        value: value.as_ref().map(|value| self.lower_expr(value)),
                        line: span.line,
                    });
                }
                Statement::Try { body, catches, .. } => {
                    instructions.push(Instruction::Try {
                        body: self.lower_statements(body),
                        catches: catches
                            .iter()
                            .map(|catch| self.lower_catch_clause(catch))
                            .collect(),
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
        &mut self,
        target: &AstArrayDimTarget,
        op: AssignmentOp,
        value: &Expr,
    ) -> Instruction {
        Instruction::StoreArrayDim {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| {
                    dimension
                        .as_ref()
                        .map(|dimension| self.lower_expr(dimension))
                })
                .collect(),
            value: self.lower_expr(value),
            compound_op: assignment_op_binary_op(op),
            line: target.span.line,
        }
    }

    fn lower_array_dim_target(&mut self, target: &AstArrayDimTarget) -> ArrayDimTarget {
        ArrayDimTarget {
            array: target.array.clone(),
            dimensions: target
                .dimensions
                .iter()
                .map(|dimension| {
                    dimension
                        .as_ref()
                        .map(|dimension| self.lower_expr(dimension))
                })
                .collect(),
            line: target.span.line,
        }
    }

    fn lower_parameter(&mut self, parameter: &AstFunctionParameter) -> FunctionParameter {
        FunctionParameter {
            name: parameter.name.clone(),
            type_hint: parameter.type_hint.map(lower_type_hint),
            by_ref: parameter.by_ref,
            is_variadic: parameter.is_variadic,
            default_value: parameter
                .default_value
                .as_ref()
                .map(|value| self.lower_expr(value)),
        }
    }
}

fn lower_closure_capture(capture: &AstClosureUseCapture) -> ClosureCapture {
    ClosureCapture {
        name: capture.name.clone(),
        by_ref: capture.by_ref,
        line: capture.span.line,
    }
}

fn lower_property_visibility(visibility: AstPropertyVisibility) -> PropertyVisibility {
    match visibility {
        AstPropertyVisibility::Public => PropertyVisibility::Public,
        AstPropertyVisibility::Protected => PropertyVisibility::Protected,
        AstPropertyVisibility::Private => PropertyVisibility::Private,
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

impl<'a> LoweringContext<'a> {
    fn lower_assignment_target(&mut self, target: &AstAssignmentTarget) -> AssignmentTarget {
        match target {
            AstAssignmentTarget::Variable { name, span } => AssignmentTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicVariable { name, span } => {
                AssignmentTarget::DynamicVariable {
                    name: Box::new(self.lower_expr(name)),
                    line: span.line,
                }
            }
            AstAssignmentTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => AssignmentTarget::DynamicArrayDim {
                name: Box::new(self.lower_expr(name)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstAssignmentTarget::ArrayDim(target) => AssignmentTarget::ArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: target.span.line,
            },
            AstAssignmentTarget::Property {
                receiver,
                name,
                span,
            } => AssignmentTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::StaticProperty {
                class_name,
                name,
                span,
            } => AssignmentTarget::StaticProperty {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::List(target) => {
                AssignmentTarget::List(self.lower_list_assignment_target(target))
            }
        }
    }

    fn lower_inc_dec_target(&mut self, target: &AstIncDecTarget) -> IncDecTarget {
        match target {
            AstIncDecTarget::Variable { name, span } => IncDecTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstIncDecTarget::DynamicVariable { name, span } => IncDecTarget::DynamicVariable {
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            AstIncDecTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => IncDecTarget::DynamicArrayDim {
                name: Box::new(self.lower_expr(name)),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: span.line,
            },
            AstIncDecTarget::ArrayDim(target) => IncDecTarget::ArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        dimension
                            .as_ref()
                            .map(|dimension| self.lower_expr(dimension))
                    })
                    .collect(),
                line: target.span.line,
            },
            AstIncDecTarget::Property {
                receiver,
                name,
                span,
            } => IncDecTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstIncDecTarget::StaticProperty {
                class_name,
                name,
                span,
            } => IncDecTarget::StaticProperty {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
        }
    }

    fn lower_list_assignment_target(
        &mut self,
        target: &AstListAssignmentTarget,
    ) -> ListAssignmentTarget {
        ListAssignmentTarget {
            elements: target
                .elements
                .iter()
                .map(|element| self.lower_list_assignment_element(element))
                .collect(),
            line: target.span.line,
        }
    }

    fn lower_list_assignment_element(
        &mut self,
        element: &AstListAssignmentElement,
    ) -> ListAssignmentElement {
        ListAssignmentElement {
            key: element.key.as_ref().map(|key| self.lower_expr(key)),
            target: match &element.target {
                AstListAssignmentElementTarget::Value(target) => {
                    ListAssignmentElementTarget::Value(Box::new(
                        self.lower_assignment_target(target),
                    ))
                }
                AstListAssignmentElementTarget::Reference(target) => {
                    ListAssignmentElementTarget::Reference(self.lower_reference_target(target))
                }
            },
        }
    }

    fn lower_reference_target(&mut self, target: &AstReferenceTarget) -> ReferenceTarget {
        match target {
            AstReferenceTarget::Variable { name, span } => ReferenceTarget::Variable {
                name: name.clone(),
                line: span.line,
            },
            AstReferenceTarget::ArrayDim(target) => {
                ReferenceTarget::ArrayDim(self.lower_array_dim_target(target))
            }
        }
    }

    fn lower_unset_target(&mut self, target: &AstUnsetTarget) -> Instruction {
        match target {
            AstUnsetTarget::Variable { name, .. } => {
                Instruction::UnsetVariable { name: name.clone() }
            }
            AstUnsetTarget::ArrayDim(target) => Instruction::UnsetArrayDim {
                array: target.array.clone(),
                dimensions: target
                    .dimensions
                    .iter()
                    .map(|dimension| {
                        self.lower_expr(
                            dimension
                                .as_ref()
                                .expect("parser rejects append syntax in unset targets"),
                        )
                    })
                    .collect(),
                line: target.span.line,
            },
            AstUnsetTarget::DynamicArrayDim {
                name,
                dimensions,
                span,
            } => Instruction::UnsetDynamicArrayDim {
                name: self.lower_expr(name),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
        }
    }
}

fn assignment_op_binary_op(op: AssignmentOp) -> Option<BinaryOp> {
    match op {
        AssignmentOp::Assign => None,
        AssignmentOp::CoalesceAssign => {
            unreachable!("null coalescing assignment lowers through ValueExpr::Assign")
        }
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

impl<'a> LoweringContext<'a> {
    fn lower_catch_clause(&mut self, catch: &AstCatchClause) -> CatchClause {
        CatchClause {
            type_name: catch.type_name.clone(),
            variable: catch.variable.clone(),
            body: self.lower_statements(&catch.body),
        }
    }

    fn lower_assignment_value(
        &mut self,
        name: &str,
        op: AssignmentOp,
        value: &Expr,
        line: usize,
    ) -> ValueExpr {
        let right = self.lower_expr(value);
        match op {
            AssignmentOp::Assign => right,
            AssignmentOp::CoalesceAssign => {
                unreachable!("direct null coalescing assignment lowers through ValueExpr::Assign")
            }
            AssignmentOp::AddAssign => lower_compound_assignment(name, line, BinaryOp::Add, right),
            AssignmentOp::SubtractAssign => {
                lower_compound_assignment(name, line, BinaryOp::Subtract, right)
            }
            AssignmentOp::MultiplyAssign => {
                lower_compound_assignment(name, line, BinaryOp::Multiply, right)
            }
            AssignmentOp::PowerAssign => {
                lower_compound_assignment(name, line, BinaryOp::Power, right)
            }
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

    fn lower_assignment_expr_value(
        &mut self,
        target: AssignmentTarget,
        op: AssignmentOp,
        value: &Expr,
    ) -> (AssignmentOp, ValueExpr) {
        match op {
            AssignmentOp::Assign | AssignmentOp::CoalesceAssign => (op, self.lower_expr(value)),
            AssignmentOp::AddAssign
            | AssignmentOp::SubtractAssign
            | AssignmentOp::MultiplyAssign
            | AssignmentOp::PowerAssign
            | AssignmentOp::DivideAssign
            | AssignmentOp::ModuloAssign
            | AssignmentOp::ConcatAssign
            | AssignmentOp::BitwiseAndAssign
            | AssignmentOp::BitwiseOrAssign
            | AssignmentOp::BitwiseXorAssign
            | AssignmentOp::ShiftLeftAssign
            | AssignmentOp::ShiftRightAssign => match target {
                AssignmentTarget::Variable { name, line } => (
                    AssignmentOp::Assign,
                    self.lower_assignment_value(&name, op, value, line),
                ),
                AssignmentTarget::ArrayDim { .. } => (op, self.lower_expr(value)),
                AssignmentTarget::DynamicVariable { .. }
                | AssignmentTarget::DynamicArrayDim { .. }
                | AssignmentTarget::Property { .. }
                | AssignmentTarget::StaticProperty { .. }
                | AssignmentTarget::List(_) => {
                    unreachable!("parser rejects compound assignment expression targets")
                }
            },
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

impl<'a> LoweringContext<'a> {
    fn lower_expr(&mut self, expr: &Expr) -> ValueExpr {
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
            Expr::DynamicVariable { name, span } => ValueExpr::DynamicVariable {
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
            Expr::AnonymousFunction(function) => self.lower_anonymous_function(function),
            Expr::IncDec {
                target,
                op,
                result,
                span,
            } => ValueExpr::IncDec {
                target: self.lower_inc_dec_target(target),
                op: lower_inc_dec_op(*op),
                result: lower_inc_dec_result(*result),
                line: span.line,
            },
            Expr::Assign {
                target, op, value, ..
            } => {
                let target = self.lower_assignment_target(target);
                let (op, value) = self.lower_assignment_expr_value(target.clone(), *op, value);
                ValueExpr::Assign {
                    target,
                    op,
                    value: Box::new(value),
                }
            }
            Expr::AssignRef { target, source, .. } => ValueExpr::AssignRef {
                target: self.lower_assignment_target(target),
                source: Box::new(self.lower_expr(source)),
            },
            Expr::Constant(name, _) => ValueExpr::Constant(name.clone()),
            Expr::MagicConstant(kind, span) => ValueExpr::MagicConstant {
                kind: lower_magic_constant_kind(*kind),
                line: span.line,
            },
            Expr::Array { elements, .. } => ValueExpr::Array(
                elements
                    .iter()
                    .map(|element| self.lower_array_element(element))
                    .collect(),
            ),
            Expr::ArrayAccess { array, index, span } => {
                ValueExpr::ArrayAccess {
                    array: Box::new(self.lower_expr(array)),
                    index: Box::new(
                        self.lower_expr(index.as_ref().expect(
                            "parser rejects append array reads outside assignment targets",
                        )),
                    ),
                    line: span.line,
                }
            }
            Expr::Isset { targets, .. } => ValueExpr::Isset {
                targets: targets
                    .iter()
                    .map(|target| self.lower_expr(target))
                    .collect(),
            },
            Expr::Empty { target, .. } => ValueExpr::Empty {
                target: Box::new(self.lower_expr(target)),
            },
            Expr::Print { expression, .. } => ValueExpr::Print {
                expression: Box::new(self.lower_expr(expression)),
            },
            Expr::Include { kind, path, span } => ValueExpr::Include {
                kind: *kind,
                path: Box::new(self.lower_expr(path)),
                candidates: self.include_candidates(*span),
                line: span.line,
            },
            Expr::Call {
                name,
                arguments,
                argument_names,
                span,
            } => {
                let (arguments, argument_names) =
                    self.lower_internal_call_arguments(name, arguments, argument_names);
                ValueExpr::InternalCall {
                    name: name.clone(),
                    arguments,
                    argument_names,
                    line: span.line,
                }
            }
            Expr::DynamicCall {
                callee,
                arguments,
                argument_names,
                span,
            } => ValueExpr::DynamicCall {
                callee: Box::new(self.lower_expr(callee)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                line: span.line,
            },
            Expr::MethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                span,
            } => ValueExpr::MethodCall {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                line: span.line,
            },
            Expr::NewObject {
                class_name,
                arguments,
                argument_names,
                span,
            } => ValueExpr::NewObject {
                class_name: class_name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                line: span.line,
            },
            Expr::PropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::PropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::StaticPropertyFetch {
                class_name,
                name,
                span,
            } => ValueExpr::StaticPropertyFetch {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            Expr::Unary { op, expr, span } => ValueExpr::Unary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
            Expr::Cast { kind, expr, span } => ValueExpr::Cast {
                kind: lower_cast_kind(*kind),
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => ValueExpr::Binary {
                op: lower_binary_op(*op),
                left: Box::new(self.lower_expr(left)),
                right: Box::new(self.lower_expr(right)),
                line: span.line,
            },
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => ValueExpr::Ternary {
                condition: Box::new(self.lower_expr(condition)),
                if_true: if_true
                    .as_ref()
                    .map(|if_true| Box::new(self.lower_expr(if_true))),
                if_false: Box::new(self.lower_expr(if_false)),
                line: span.line,
            },
            Expr::Grouped { expr, .. } => self.lower_expr(expr),
        }
    }

    fn include_candidates(&self, span: crate::diagnostic::SourceSpan) -> Vec<usize> {
        self.include_resolutions
            .get(&(self.source_file.clone(), span.byte_start, span.byte_end))
            .cloned()
            .expect("include expressions require include-aware lowering")
    }

    fn lower_array_element(&mut self, element: &AstArrayElement) -> ArrayElement {
        ArrayElement {
            key: element.key.as_ref().map(|key| self.lower_expr(key)),
            value: self.lower_array_element_value(&element.value),
        }
    }

    fn lower_array_element_value(&mut self, value: &AstArrayElementValue) -> ArrayElementValue {
        match value {
            AstArrayElementValue::Value(value) => ArrayElementValue::Value(self.lower_expr(value)),
            AstArrayElementValue::Reference(target) => {
                ArrayElementValue::Reference(self.lower_reference_target(target))
            }
        }
    }

    fn lower_internal_call_arguments(
        &mut self,
        name: &str,
        arguments: &[Expr],
        argument_names: &[Option<String>],
    ) -> (Vec<ValueExpr>, Vec<Option<String>>) {
        let mut lowered_arguments: Vec<_> = arguments
            .iter()
            .map(|argument| self.lower_expr(argument))
            .collect();
        let mut lowered_names = argument_names.to_vec();

        if name.eq_ignore_ascii_case("assert")
            && arguments.len() == 1
            && argument_names.iter().all(Option::is_none)
        {
            lowered_arguments.push(ValueExpr::String(format!(
                "assert({})",
                assertion_expr_text(&arguments[0])
            )));
            lowered_names.push(None);
        }

        (lowered_arguments, lowered_names)
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
        AstStringPart::LegacyDollarBraceVariable(name) => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::LegacyDollarBraceStringVariable {
                name: name.clone(),
                line,
            }),
            line,
        }),
        AstStringPart::ArrayAccess { array, indices } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(lower_interpolated_array_access(array, indices, line)),
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

fn assertion_expr_text(expr: &Expr) -> String {
    match expr {
        Expr::String(value, _) => format!("{value:?}"),
        Expr::InterpolatedString(_, _) => "\"\"".to_string(),
        Expr::Int(value, _) => value.to_string(),
        Expr::Float(value, _) => value.to_string(),
        Expr::Bool(value, _) => value.to_string(),
        Expr::Null(_) => "null".to_string(),
        Expr::Variable(name, _) => format!("${name}"),
        Expr::DynamicVariable { name, .. } => format!("$${}", assertion_expr_text(name)),
        Expr::Constant(name, _) => name.clone(),
        Expr::MagicConstant(kind, _) => assertion_magic_constant_text(*kind).to_string(),
        Expr::IncDec {
            target, op, result, ..
        } => assertion_inc_dec_text(target, *op, *result),
        Expr::Assign {
            target, op, value, ..
        } => format!(
            "{} {} {}",
            assertion_assignment_target_text(target),
            assertion_assignment_op_text(*op),
            assertion_expr_text(value)
        ),
        Expr::AssignRef { target, source, .. } => format!(
            "{} =& {}",
            assertion_assignment_target_text(target),
            assertion_expr_text(source)
        ),
        Expr::Call {
            name, arguments, ..
        } => format!("{}({})", name, assertion_argument_list_text(arguments)),
        Expr::DynamicCall {
            callee, arguments, ..
        } => format!(
            "{}({})",
            assertion_expr_text(callee),
            assertion_argument_list_text(arguments)
        ),
        Expr::MethodCall {
            receiver,
            name,
            arguments,
            ..
        } => format!(
            "{}->{}({})",
            assertion_expr_text(receiver),
            name,
            assertion_argument_list_text(arguments)
        ),
        Expr::NewObject {
            class_name,
            arguments,
            ..
        } => format!(
            "new {class_name}({})",
            assertion_argument_list_text(arguments)
        ),
        Expr::PropertyFetch { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        Expr::StaticPropertyFetch {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        Expr::Array { elements, .. } => format!(
            "[{}]",
            elements
                .iter()
                .map(assertion_array_element_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::ArrayAccess { array, index, .. } => {
            let index = index
                .as_ref()
                .map(|index| assertion_expr_text(index))
                .unwrap_or_default();
            format!("{}[{index}]", assertion_expr_text(array))
        }
        Expr::Isset { targets, .. } => {
            format!("isset({})", assertion_argument_list_text(targets))
        }
        Expr::Empty { target, .. } => format!("empty({})", assertion_expr_text(target)),
        Expr::Print { expression, .. } => format!("print {}", assertion_expr_text(expression)),
        Expr::Include { kind, path, .. } => {
            format!(
                "{} {}",
                assertion_include_kind_text(*kind),
                assertion_expr_text(path)
            )
        }
        Expr::Unary { op, expr, .. } => {
            format!(
                "{}{}",
                assertion_unary_op_text(*op),
                assertion_expr_text(expr)
            )
        }
        Expr::Cast { kind, expr, .. } => {
            format!(
                "({}) {}",
                assertion_cast_kind_text(*kind),
                assertion_expr_text(expr)
            )
        }
        Expr::Binary {
            op, left, right, ..
        } => format!(
            "{} {} {}",
            assertion_expr_text(left),
            assertion_binary_op_text(*op),
            assertion_expr_text(right)
        ),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            if let Some(if_true) = if_true {
                format!(
                    "{} ? {} : {}",
                    assertion_expr_text(condition),
                    assertion_expr_text(if_true),
                    assertion_expr_text(if_false)
                )
            } else {
                format!(
                    "{} ?: {}",
                    assertion_expr_text(condition),
                    assertion_expr_text(if_false)
                )
            }
        }
        Expr::Grouped { expr, .. } => format!("({})", assertion_expr_text(expr)),
        Expr::AnonymousFunction(_) => "function()".to_string(),
    }
}

fn assertion_argument_list_text(arguments: &[Expr]) -> String {
    arguments
        .iter()
        .map(assertion_expr_text)
        .collect::<Vec<_>>()
        .join(", ")
}

fn assertion_array_element_text(element: &AstArrayElement) -> String {
    let value = match &element.value {
        AstArrayElementValue::Value(value) => assertion_expr_text(value),
        AstArrayElementValue::Reference(target) => {
            format!("&{}", assertion_reference_target_text(target))
        }
    };
    if let Some(key) = &element.key {
        format!("{} => {value}", assertion_expr_text(key))
    } else {
        value
    }
}

fn assertion_assignment_target_text(target: &AstAssignmentTarget) -> String {
    match target {
        AstAssignmentTarget::Variable { name, .. } => format!("${name}"),
        AstAssignmentTarget::DynamicVariable { name, .. } => {
            format!("$${}", assertion_expr_text(name))
        }
        AstAssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => {
            let mut text = format!("$${}", assertion_expr_text(name));
            for dimension in dimensions {
                let index = dimension
                    .as_ref()
                    .map(assertion_expr_text)
                    .unwrap_or_default();
                text.push('[');
                text.push_str(&index);
                text.push(']');
            }
            text
        }
        AstAssignmentTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstAssignmentTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstAssignmentTarget::StaticProperty {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        AstAssignmentTarget::List(_) => "list(...)".to_string(),
    }
}

fn assertion_array_dim_target_text(target: &AstArrayDimTarget) -> String {
    let mut text = format!("${}", target.array);
    for dimension in &target.dimensions {
        let index = dimension
            .as_ref()
            .map(assertion_expr_text)
            .unwrap_or_default();
        text.push('[');
        text.push_str(&index);
        text.push(']');
    }
    text
}

fn assertion_reference_target_text(target: &AstReferenceTarget) -> String {
    match target {
        AstReferenceTarget::Variable { name, .. } => format!("${name}"),
        AstReferenceTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
    }
}

fn assertion_inc_dec_target_text(target: &AstIncDecTarget) -> String {
    match target {
        AstIncDecTarget::Variable { name, .. } => format!("${name}"),
        AstIncDecTarget::DynamicVariable { .. } => "${...}".to_string(),
        AstIncDecTarget::DynamicArrayDim { .. } => "${...}[...]".to_string(),
        AstIncDecTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstIncDecTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstIncDecTarget::StaticProperty {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
    }
}

fn assertion_assignment_op_text(op: AssignmentOp) -> &'static str {
    match op {
        AssignmentOp::Assign => "=",
        AssignmentOp::AddAssign => "+=",
        AssignmentOp::SubtractAssign => "-=",
        AssignmentOp::MultiplyAssign => "*=",
        AssignmentOp::PowerAssign => "**=",
        AssignmentOp::DivideAssign => "/=",
        AssignmentOp::ModuloAssign => "%=",
        AssignmentOp::ConcatAssign => ".=",
        AssignmentOp::BitwiseAndAssign => "&=",
        AssignmentOp::BitwiseOrAssign => "|=",
        AssignmentOp::BitwiseXorAssign => "^=",
        AssignmentOp::ShiftLeftAssign => "<<=",
        AssignmentOp::ShiftRightAssign => ">>=",
        AssignmentOp::CoalesceAssign => "??=",
    }
}

fn assertion_binary_op_text(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::Add => "+",
        AstBinaryOp::Subtract => "-",
        AstBinaryOp::Multiply => "*",
        AstBinaryOp::Power => "**",
        AstBinaryOp::Divide => "/",
        AstBinaryOp::Modulo => "%",
        AstBinaryOp::Concat => ".",
        AstBinaryOp::Coalesce => "??",
        AstBinaryOp::ShiftLeft => "<<",
        AstBinaryOp::ShiftRight => ">>",
        AstBinaryOp::Equal => "==",
        AstBinaryOp::NotEqual => "!=",
        AstBinaryOp::Spaceship => "<=>",
        AstBinaryOp::Identical => "===",
        AstBinaryOp::NotIdentical => "!==",
        AstBinaryOp::Less => "<",
        AstBinaryOp::LessEqual => "<=",
        AstBinaryOp::Greater => ">",
        AstBinaryOp::GreaterEqual => ">=",
        AstBinaryOp::BitwiseAnd => "&",
        AstBinaryOp::BitwiseXor => "^",
        AstBinaryOp::BitwiseOr => "|",
        AstBinaryOp::And => "&&",
        AstBinaryOp::Xor => "xor",
        AstBinaryOp::Or => "||",
    }
}

fn assertion_unary_op_text(op: AstUnaryOp) -> &'static str {
    match op {
        AstUnaryOp::Positive => "+",
        AstUnaryOp::Negate => "-",
        AstUnaryOp::Not => "!",
        AstUnaryOp::BitwiseNot => "~",
        AstUnaryOp::ErrorSuppress => "@",
    }
}

fn assertion_inc_dec_text(
    target: &AstIncDecTarget,
    op: AstIncDecOp,
    result: AstIncDecResult,
) -> String {
    let op = match op {
        AstIncDecOp::Increment => "++",
        AstIncDecOp::Decrement => "--",
    };
    let target = assertion_inc_dec_target_text(target);
    match result {
        AstIncDecResult::Pre => format!("{op}{target}"),
        AstIncDecResult::Post => format!("{target}{op}"),
    }
}

fn assertion_cast_kind_text(kind: AstCastKind) -> &'static str {
    match kind {
        AstCastKind::Int => "int",
        AstCastKind::Integer => "integer",
        AstCastKind::Float => "float",
        AstCastKind::Double => "double",
        AstCastKind::String => "string",
        AstCastKind::Binary => "binary",
        AstCastKind::Bool => "bool",
        AstCastKind::Boolean => "boolean",
    }
}

fn assertion_magic_constant_text(kind: AstMagicConstantKind) -> &'static str {
    match kind {
        AstMagicConstantKind::File => "__FILE__",
        AstMagicConstantKind::Dir => "__DIR__",
        AstMagicConstantKind::Line => "__LINE__",
        AstMagicConstantKind::Function => "__FUNCTION__",
        AstMagicConstantKind::Class => "__CLASS__",
        AstMagicConstantKind::Method => "__METHOD__",
        AstMagicConstantKind::Trait => "__TRAIT__",
        AstMagicConstantKind::Namespace => "__NAMESPACE__",
    }
}

fn assertion_include_kind_text(kind: crate::ast::IncludeKind) -> &'static str {
    match kind {
        crate::ast::IncludeKind::Include => "include",
        crate::ast::IncludeKind::IncludeOnce => "include_once",
        crate::ast::IncludeKind::Require => "require",
        crate::ast::IncludeKind::RequireOnce => "require_once",
    }
}

fn lower_interpolated_array_access(
    array: &str,
    indices: &[AstStringInterpolationIndex],
    line: usize,
) -> ValueExpr {
    let mut expr = ValueExpr::Load {
        name: array.to_string(),
        line,
    };
    for index in indices {
        expr = ValueExpr::ArrayAccess {
            array: Box::new(expr),
            index: Box::new(lower_interpolated_array_index(index, line)),
            line,
        };
    }
    expr
}

fn lower_interpolated_array_index(index: &AstStringInterpolationIndex, line: usize) -> ValueExpr {
    match index {
        AstStringInterpolationIndex::String(value) => ValueExpr::String(value.clone()),
        AstStringInterpolationIndex::Int(value) => ValueExpr::Int(*value),
        AstStringInterpolationIndex::Variable(name) => ValueExpr::Load {
            name: name.clone(),
            line,
        },
    }
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

fn lower_inc_dec_result(result: AstIncDecResult) -> IncDecResult {
    match result {
        AstIncDecResult::Pre => IncDecResult::Pre,
        AstIncDecResult::Post => IncDecResult::Post,
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
