use std::collections::HashMap;

use crate::ast::{
    AnonymousFunction as AstAnonymousFunction, ArrayDimTarget as AstArrayDimTarget,
    ArrayElement as AstArrayElement, ArrayElementValue as AstArrayElementValue, AssignmentOp,
    AssignmentTarget as AstAssignmentTarget, AttributeConstantReference, AttributeMetadata,
    BinaryOp as AstBinaryOp, CastKind as AstCastKind, CatchClause as AstCatchClause,
    ClassDecl as AstClassDecl, ClosureUseCapture as AstClosureUseCapture,
    CompileWarning as AstCompileWarning, CompileWarningKind as AstCompileWarningKind, Expr,
    FunctionDecl as AstFunctionDecl, FunctionParameter as AstFunctionParameter,
    IncDecOp as AstIncDecOp, IncDecResult as AstIncDecResult, IncDecTarget as AstIncDecTarget,
    IncludeKind as AstIncludeKind, InstanceOfTarget as AstInstanceOfTarget,
    ListAssignmentElement as AstListAssignmentElement,
    ListAssignmentElementTarget as AstListAssignmentElementTarget,
    ListAssignmentTarget as AstListAssignmentTarget,
    ListExprElementTarget as AstListExprElementTarget, MagicConstantKind as AstMagicConstantKind,
    MatchArm as AstMatchArm, Program, PropertyTypeKind as AstPropertyTypeKind,
    PropertyVisibility as AstPropertyVisibility, ReferenceTarget as AstReferenceTarget, Statement,
    StringInterpolationIndex as AstStringInterpolationIndex, StringPart as AstStringPart,
    TypeHint as AstTypeHint, UnaryOp as AstUnaryOp, UnsetTarget as AstUnsetTarget,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub classes: Vec<ClassDecl>,
    pub traits: Vec<TraitDecl>,
    pub functions: Vec<FunctionDecl>,
    pub includes: Vec<IncludeFile>,
    pub instructions: Vec<Instruction>,
    pub compile_warnings: Vec<CompileWarning>,
    pub source_file: String,
    pub source_dir: String,
    pub strict_types: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileWarning {
    pub message: String,
    pub line: usize,
    pub kind: CompileWarningKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileWarningKind {
    Warning,
    UncaughtError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeFile {
    pub source_file: String,
    pub source_dir: String,
    pub path_aliases: Vec<String>,
    pub instructions: Vec<Instruction>,
    pub compile_warnings: Vec<CompileWarning>,
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
    pub interfaces: Vec<String>,
    pub trait_uses: Vec<TraitUseDecl>,
    pub line: usize,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_interface: bool,
    pub is_readonly: bool,
    pub properties: Vec<PropertyDecl>,
    pub static_properties: Vec<StaticPropertyDecl>,
    pub constants: Vec<ClassConstantDecl>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub trait_uses: Vec<TraitUseDecl>,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitUseDecl {
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub set_visibility: PropertyVisibility,
    pub is_final: bool,
    pub is_readonly: bool,
    pub has_hooks: bool,
    pub is_virtual: bool,
    pub hook_get_value: Option<ValueExpr>,
    pub type_hint: Option<PropertyTypeHint>,
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
    pub set_visibility: PropertyVisibility,
    pub is_final: bool,
    pub type_hint: Option<PropertyTypeHint>,
    pub value: Option<ValueExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyTypeHint {
    pub text: String,
    pub kind: PropertyTypeKind,
    pub allows_null: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyTypeKind {
    Null,
    Array,
    Int,
    Float,
    String,
    Bool,
    Mixed,
    Object,
    Class(String),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassConstantDecl {
    pub name: String,
    pub visibility: PropertyVisibility,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub deprecated_message_dependency: Option<DeprecatedMessageDependency>,
    pub deprecated_message_runtime_reference: Option<AttributeConstantReference>,
    pub is_enum_case: bool,
    pub value: ValueExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeprecatedMessageDependency {
    pub subject: String,
    pub message: Option<String>,
    pub since: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub function_index: usize,
    pub visibility: PropertyVisibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub display_name: String,
    pub class_name: Option<String>,
    pub trait_name: Option<String>,
    pub trait_method_name: Option<String>,
    pub method_name: Option<String>,
    pub deprecated_message: Option<String>,
    pub deprecated_since: Option<String>,
    pub no_discard_message: Option<String>,
    pub is_static: bool,
    pub line: usize,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeHint>,
    pub return_by_ref: bool,
    pub is_generator: bool,
    pub is_anonymous: bool,
    pub initially_declared: bool,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeHint {
    Null,
    Array,
    Callable,
    Int,
    Float,
    String,
    Bool,
    True,
    False,
    Object,
    Iterable,
    Mixed,
    Void,
    Never,
    Static,
    Nullable(Box<TypeHint>),
    Union(Vec<TypeHint>),
    Intersection(Vec<TypeHint>),
    Class(String),
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
    UnsetDynamicVariable {
        name: ValueExpr,
        line: usize,
    },
    BindGlobal {
        name: String,
    },
    DeclareFunction {
        function_index: usize,
    },
    BindStatic {
        name: String,
        value: Option<ValueExpr>,
        line: usize,
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
    UnsetPropertyArrayDim {
        receiver: ValueExpr,
        name: String,
        dimensions: Vec<ValueExpr>,
        line: usize,
    },
    UnsetProperty {
        receiver: ValueExpr,
        name: String,
        line: usize,
    },
    DefineConstant {
        name: String,
        value: ValueExpr,
        deprecated_message: Option<String>,
        deprecated_since: Option<String>,
        deprecated_message_dependency: Option<DeprecatedMessageDependency>,
        line: usize,
    },
    Expression(ValueExpr),
    Echo(ValueExpr),
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    Return {
        value: Option<ValueExpr>,
        line: usize,
    },
    Exit {
        value: Option<ValueExpr>,
        line: usize,
    },
    Throw {
        value: ValueExpr,
        line: usize,
    },
    Try {
        body: Vec<Instruction>,
        catches: Vec<CatchClause>,
        finally_body: Vec<Instruction>,
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
    pub type_names: Vec<String>,
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
    Constant {
        name: String,
        deprecated_message: Option<String>,
        deprecated_since: Option<String>,
        deprecated_message_dependency: Option<DeprecatedMessageDependency>,
        line: usize,
    },
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
    ArrayAppendAccess {
        array: Box<ValueExpr>,
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
    Throw {
        value: Box<ValueExpr>,
        line: usize,
    },
    Yield {
        key: Option<Box<ValueExpr>>,
        value: Option<Box<ValueExpr>>,
        line: usize,
    },
    InternalCall {
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    FirstClassCallable {
        callable: Box<ValueExpr>,
        line: usize,
    },
    DynamicCall {
        callee: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    MethodCall {
        receiver: Box<ValueExpr>,
        name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    DynamicMethodCall {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    NewObject {
        class_name: String,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    DynamicNewObject {
        class_name: Box<ValueExpr>,
        arguments: Vec<ValueExpr>,
        argument_names: Vec<Option<String>>,
        argument_unpacks: Vec<bool>,
        line: usize,
    },
    Clone {
        expr: Box<ValueExpr>,
        line: usize,
    },
    PropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    NullsafePropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicPropertyFetch {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
    StaticPropertyFetch {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicStaticPropertyFetch {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    ClassConstantFetch {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicClassNameFetch {
        receiver: Box<ValueExpr>,
        line: usize,
    },
    InstanceOf {
        expr: Box<ValueExpr>,
        target: InstanceOfTarget,
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
    PipeValue {
        expr: Box<ValueExpr>,
        line: usize,
    },
    Match {
        subject: Box<ValueExpr>,
        arms: Vec<MatchArm>,
        line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstanceOfTarget {
    ClassName(String),
    Expr(Box<ValueExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub conditions: Vec<ValueExpr>,
    pub value: ValueExpr,
    pub is_default: bool,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosureCapture {
    pub name: String,
    pub by_ref: bool,
    pub warn_if_missing: bool,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement {
    pub key: Option<ValueExpr>,
    pub value: ArrayElementValue,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementValue {
    Value(ValueExpr),
    Reference(ReferenceTarget),
    Unpack { value: ValueExpr, line: usize },
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
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    StaticPropertyArrayDim {
        class_name: String,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    DynamicStaticPropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    ValueArrayDim {
        array: Box<ValueExpr>,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicProperty {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
    StaticProperty {
        class_name: String,
        name: String,
        line: usize,
    },
    DynamicStaticProperty {
        receiver: Box<ValueExpr>,
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
    Variable {
        name: String,
        line: usize,
    },
    DynamicVariable {
        name: Box<ValueExpr>,
        line: usize,
    },
    ArrayDim(ArrayDimTarget),
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
        dimensions: Vec<Option<ValueExpr>>,
        line: usize,
    },
    Property {
        receiver: Box<ValueExpr>,
        name: String,
        line: usize,
    },
    DynamicProperty {
        receiver: Box<ValueExpr>,
        name: Box<ValueExpr>,
        line: usize,
    },
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
    Array,
    Object,
    Void,
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
    let include_function_ranges = context.declare_include_functions(&include_sources);
    for (index, function) in program.functions.iter().enumerate() {
        let body = context.lower_statements(&function.body);
        context.functions[index].body = body;
    }
    for (include, start_index) in include_sources.iter().zip(include_function_ranges.iter()) {
        context.lower_include_functions(include, *start_index);
    }
    let mut classes: Vec<_> = program
        .classes
        .iter()
        .map(|class| context.lower_class(class))
        .collect();
    let mut traits: Vec<_> = program.traits.iter().map(lower_trait).collect();
    for include in &include_sources {
        classes.extend(context.lower_include_classes(include));
        traits.extend(include.program.traits.iter().map(lower_trait));
    }
    let includes = include_sources
        .iter()
        .map(|include| context.lower_include_source(include))
        .collect();
    let instructions = context.lower_statements(&program.statements);
    Module {
        classes,
        traits,
        functions: context.functions,
        includes,
        instructions,
        compile_warnings: lower_compile_warnings(&program.compile_warnings),
        source_file,
        source_dir,
        strict_types: program.strict_types,
    }
}

struct LoweringContext<'a> {
    functions: Vec<FunctionDecl>,
    constant_deprecations: HashMap<String, DeprecatedMetadata>,
    constant_values: HashMap<String, String>,
    source_file: String,
    source_dir: String,
    include_resolutions: &'a IncludeResolutionMap,
    current_class_name: Option<String>,
    current_trait_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeprecatedMetadata {
    message: Option<String>,
    since: Option<String>,
    message_dependency: Option<DeprecatedMessageDependency>,
    message_runtime_reference: Option<AttributeConstantReference>,
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
    PropertyArrayDim {
        receiver: Box<ValueExpr>,
        name: String,
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
        let constant_values = collect_constant_values(program);
        let constant_deprecations = collect_constant_deprecations(program, &constant_values);
        let mut context = Self {
            functions: Vec::new(),
            constant_deprecations,
            constant_values,
            source_file,
            source_dir,
            include_resolutions,
            current_class_name: None,
            current_trait_name: None,
        };
        for function in &program.functions {
            context.declare_function(function);
        }
        context
    }

    fn declare_include_functions(&mut self, include_sources: &[IncludeSource]) -> Vec<usize> {
        let mut starts = Vec::with_capacity(include_sources.len());
        for include in include_sources {
            let previous_source_file =
                std::mem::replace(&mut self.source_file, include.source_file.clone());
            let previous_source_dir =
                std::mem::replace(&mut self.source_dir, include.source_dir.clone());
            starts.push(self.functions.len());
            for function in &include.program.functions {
                self.declare_function(function);
            }
            self.source_file = previous_source_file;
            self.source_dir = previous_source_dir;
        }
        starts
    }

    fn lower_include_functions(&mut self, include: &IncludeSource, start_index: usize) {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        for (offset, function) in include.program.functions.iter().enumerate() {
            let body = self.lower_statements(&function.body);
            self.functions[start_index + offset].body = body;
        }
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
    }

    fn lower_include_classes(&mut self, include: &IncludeSource) -> Vec<ClassDecl> {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let classes = include
            .program
            .classes
            .iter()
            .map(|class| self.lower_class(class))
            .collect();
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        classes
    }

    fn declare_function(&mut self, function: &AstFunctionDecl) {
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| self.lower_parameter(parameter))
            .collect();
        self.functions.push(FunctionDecl {
            name: function.name.clone(),
            display_name: function.name.clone(),
            class_name: None,
            trait_name: None,
            trait_method_name: None,
            method_name: None,
            deprecated_message: function.attributes.deprecated_message.clone(),
            deprecated_since: function.attributes.deprecated_since.clone(),
            no_discard_message: function.attributes.no_discard_message.clone(),
            is_static: false,
            line: function.span.line,
            parameters,
            return_type: function.return_type.clone().map(lower_type_hint),
            return_by_ref: function.return_by_ref,
            is_generator: statements_contain_yield(&function.body),
            is_anonymous: false,
            initially_declared: !function.is_conditionally_declared,
            body: Vec::new(),
        });
    }

    fn function_index_by_name(&self, name: &str) -> Option<usize> {
        self.functions.iter().position(|function| {
            function.class_name.is_none()
                && !function.is_anonymous
                && function.name.eq_ignore_ascii_case(name)
        })
    }

    fn lower_include_source(&mut self, include: &IncludeSource) -> IncludeFile {
        let previous_source_file =
            std::mem::replace(&mut self.source_file, include.source_file.clone());
        let previous_source_dir =
            std::mem::replace(&mut self.source_dir, include.source_dir.clone());
        let include_constant_values = collect_constant_values(&include.program);
        let include_constant_deprecations =
            collect_constant_deprecations(&include.program, &include_constant_values);
        let previous_constant_deprecations = std::mem::replace(
            &mut self.constant_deprecations,
            include_constant_deprecations,
        );
        let previous_constant_values =
            std::mem::replace(&mut self.constant_values, include_constant_values);
        let instructions = self.lower_statements(&include.program.statements);
        self.constant_deprecations = previous_constant_deprecations;
        self.constant_values = previous_constant_values;
        self.source_file = previous_source_file;
        self.source_dir = previous_source_dir;
        IncludeFile {
            source_file: include.source_file.clone(),
            source_dir: include.source_dir.clone(),
            path_aliases: include.path_aliases.clone(),
            instructions,
            compile_warnings: lower_compile_warnings(&include.program.compile_warnings),
        }
    }

    fn global_deprecated_metadata(
        &self,
        attributes: &AttributeMetadata,
        current_name: &str,
    ) -> DeprecatedMetadata {
        let current_subject = format!("Constant {}", current_name.trim_start_matches('\\'));
        let message = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => self
                    .constant_values
                    .get(&name.to_ascii_lowercase())
                    .cloned(),
                AttributeConstantReference::ClassConstant { .. } => None,
            })
            .or_else(|| attributes.deprecated_message.clone());
        let message_dependency = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    let subject = format!("Constant {}", name.trim_start_matches('\\'));
                    (subject != current_subject)
                        .then(|| {
                            self.constant_deprecations
                                .get(&name.to_ascii_lowercase())
                                .map(|metadata| (subject, metadata))
                        })
                        .flatten()
                }
                AttributeConstantReference::ClassConstant { .. } => None,
            })
            .map(|(subject, metadata)| DeprecatedMessageDependency {
                subject,
                message: metadata.message.clone(),
                since: metadata.since.clone(),
            });
        DeprecatedMetadata {
            message,
            since: attributes.deprecated_since.clone(),
            message_dependency,
            message_runtime_reference: None,
        }
    }

    fn class_deprecated_metadata(
        &self,
        attributes: &AttributeMetadata,
        class_name: &str,
        current_name: &str,
        class_constant_values: &HashMap<String, String>,
        class_constant_deprecations: &HashMap<String, DeprecatedMetadata>,
    ) -> DeprecatedMetadata {
        let current_subject = format!("Constant {class_name}::{current_name}");
        let referenced_message =
            attributes
                .deprecated_message_constant
                .as_ref()
                .and_then(|reference| match reference {
                    AttributeConstantReference::Constant(name) => self
                        .constant_values
                        .get(&name.to_ascii_lowercase())
                        .cloned(),
                    AttributeConstantReference::ClassConstant {
                        class_name: referenced_class,
                        name,
                    } => self
                        .resolve_attribute_class_name(referenced_class, class_name)
                        .filter(|resolved| resolved.eq_ignore_ascii_case(class_name))
                        .and_then(|_| {
                            class_constant_values
                                .get(&name.to_ascii_lowercase())
                                .cloned()
                        }),
                });
        let message = referenced_message.clone().or_else(|| {
            attributes
                .deprecated_message_constant
                .is_none()
                .then(|| attributes.deprecated_message.clone())
                .flatten()
        });
        let message_runtime_reference = (referenced_message.is_none())
            .then(|| attributes.deprecated_message_constant.clone())
            .flatten();
        let message_dependency = attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    let subject = format!("Constant {}", name.trim_start_matches('\\'));
                    (subject != current_subject)
                        .then(|| {
                            self.constant_deprecations
                                .get(&name.to_ascii_lowercase())
                                .map(|metadata| (subject, metadata))
                        })
                        .flatten()
                }
                AttributeConstantReference::ClassConstant {
                    class_name: referenced_class,
                    name,
                } => self
                    .resolve_attribute_class_name(referenced_class, class_name)
                    .and_then(|resolved_class| {
                        let subject = format!("Constant {resolved_class}::{name}");
                        if subject == current_subject
                            || !resolved_class.eq_ignore_ascii_case(class_name)
                        {
                            return None;
                        }
                        class_constant_deprecations
                            .get(&name.to_ascii_lowercase())
                            .map(|metadata| (subject, metadata))
                    }),
            })
            .map(|(subject, metadata)| DeprecatedMessageDependency {
                subject,
                message: metadata.message.clone(),
                since: metadata.since.clone(),
            });
        DeprecatedMetadata {
            message,
            since: attributes.deprecated_since.clone(),
            message_dependency,
            message_runtime_reference,
        }
    }

    fn resolve_attribute_class_name(
        &self,
        referenced_class: &str,
        current_class: &str,
    ) -> Option<String> {
        if referenced_class.eq_ignore_ascii_case("self")
            || referenced_class.eq_ignore_ascii_case("static")
        {
            Some(current_class.to_string())
        } else if referenced_class.eq_ignore_ascii_case(current_class) {
            Some(current_class.to_string())
        } else {
            Some(referenced_class.trim_start_matches('\\').to_string())
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
            display_name: format!("{{closure:{}:{}}}", self.source_file, function.span.line),
            class_name: self.current_class_name.clone(),
            trait_name: self.current_trait_name.clone(),
            trait_method_name: None,
            method_name: None,
            deprecated_message: function.attributes.deprecated_message.clone(),
            deprecated_since: function.attributes.deprecated_since.clone(),
            no_discard_message: function.attributes.no_discard_message.clone(),
            is_static: function.is_static,
            line: function.span.line,
            parameters,
            return_type: function.return_type.clone().map(lower_type_hint),
            return_by_ref: function.return_by_ref,
            is_generator: statements_contain_yield(&function.body),
            is_anonymous: true,
            initially_declared: true,
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
                set_visibility: lower_property_visibility(property.set_visibility),
                is_final: property.is_final,
                is_readonly: property.is_readonly,
                has_hooks: property.has_hooks,
                is_virtual: property.is_virtual,
                hook_get_value: property
                    .hook_get_value
                    .as_ref()
                    .map(|value| self.lower_expr(value)),
                type_hint: property.type_hint.as_ref().map(lower_property_type_hint),
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
                set_visibility: lower_property_visibility(property.set_visibility),
                is_final: property.is_final,
                type_hint: property.type_hint.as_ref().map(lower_property_type_hint),
                value: property.value.as_ref().map(|value| self.lower_expr(value)),
            })
            .collect();
        let class_constant_values =
            collect_class_constant_values(&class.constants, &self.constant_values);
        let class_constant_deprecations = collect_class_constant_deprecations(
            &class.constants,
            &self.constant_values,
            &class_constant_values,
        );
        let constants = class
            .constants
            .iter()
            .map(|constant| {
                let metadata = self.class_deprecated_metadata(
                    &constant.attributes,
                    &class.name,
                    &constant.name,
                    &class_constant_values,
                    &class_constant_deprecations,
                );
                ClassConstantDecl {
                    name: constant.name.clone(),
                    visibility: lower_property_visibility(constant.visibility),
                    deprecated_message: metadata.message,
                    deprecated_since: metadata.since,
                    deprecated_message_dependency: metadata.message_dependency,
                    deprecated_message_runtime_reference: metadata.message_runtime_reference,
                    is_enum_case: constant.is_enum_case,
                    value: self.lower_expr(&constant.value),
                }
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
                    display_name: format!("{}::{}", class.name, method.name),
                    class_name: Some(class.name.clone()),
                    trait_name: method.trait_name.clone(),
                    trait_method_name: method.trait_method_name.clone(),
                    method_name: Some(method.name.clone()),
                    deprecated_message: method.attributes.deprecated_message.clone(),
                    deprecated_since: method.attributes.deprecated_since.clone(),
                    no_discard_message: method.attributes.no_discard_message.clone(),
                    is_static: method.is_static,
                    line: method.span.line,
                    parameters,
                    return_type: method.return_type.clone().map(lower_type_hint),
                    return_by_ref: method.return_by_ref,
                    is_generator: statements_contain_yield(&method.body),
                    is_anonymous: false,
                    initially_declared: true,
                    body: Vec::new(),
                });
                let previous_class_name =
                    std::mem::replace(&mut self.current_class_name, Some(class.name.clone()));
                let previous_trait_name =
                    std::mem::replace(&mut self.current_trait_name, method.trait_name.clone());
                let body = self.lower_statements(&method.body);
                self.current_class_name = previous_class_name;
                self.current_trait_name = previous_trait_name;
                self.functions[function_index].body = body;
                MethodDecl {
                    name: method.name.clone(),
                    function_index,
                    visibility: lower_property_visibility(method.visibility),
                    is_static: method.is_static,
                    is_abstract: method.is_abstract,
                    line: method.span.line,
                }
            })
            .collect();
        ClassDecl {
            name: class.name.clone(),
            parent_name: class.parent_name.clone(),
            interfaces: class.interfaces.clone(),
            trait_uses: lower_trait_uses(&class.trait_uses),
            line: class.span.line,
            is_abstract: class.is_abstract,
            is_final: class.is_final,
            is_interface: class.is_interface,
            is_readonly: class.is_readonly,
            properties,
            static_properties,
            constants,
            methods,
        }
    }

    fn lower_statements(&mut self, statements: &[Statement]) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        for statement in statements {
            match statement {
                Statement::Empty { .. } | Statement::ClassDeclaration { .. } => {}
                Statement::FunctionDeclaration { name, .. } => {
                    if let Some(function_index) = self.function_index_by_name(name) {
                        instructions.push(Instruction::DeclareFunction { function_index });
                    }
                }
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
                Statement::Global { names, .. } => {
                    for name in names {
                        instructions.push(Instruction::BindGlobal { name: name.clone() });
                    }
                }
                Statement::Static { declarations, .. } => {
                    for declaration in declarations {
                        instructions.push(Instruction::BindStatic {
                            name: declaration.name.clone(),
                            value: declaration
                                .value
                                .as_ref()
                                .map(|value| self.lower_expr(value)),
                            line: declaration.span.line,
                        });
                    }
                }
                Statement::Const { declarations, .. } => {
                    for declaration in declarations {
                        let metadata = self
                            .global_deprecated_metadata(&declaration.attributes, &declaration.name);
                        instructions.push(Instruction::DefineConstant {
                            name: declaration.name.clone(),
                            value: self.lower_expr(&declaration.value),
                            deprecated_message: metadata.message,
                            deprecated_since: metadata.since,
                            deprecated_message_dependency: metadata.message_dependency,
                            line: declaration.span.line,
                        });
                    }
                }
                Statement::Call {
                    name,
                    arguments,
                    argument_names,
                    argument_unpacks,
                    span,
                } => {
                    let (arguments, argument_names) =
                        self.lower_internal_call_arguments(name, arguments, argument_names);
                    instructions.push(Instruction::InternalCall {
                        name: name.clone(),
                        arguments,
                        argument_names,
                        argument_unpacks: argument_unpacks.clone(),
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
                Statement::Exit { value, span } => {
                    instructions.push(Instruction::Exit {
                        value: value.as_ref().map(|value| self.lower_expr(value)),
                        line: span.line,
                    });
                }
                Statement::Throw { value, span } => {
                    instructions.push(Instruction::Throw {
                        value: self.lower_expr(value),
                        line: span.line,
                    });
                }
                Statement::Try {
                    body,
                    catches,
                    finally_body,
                    ..
                } => {
                    instructions.push(Instruction::Try {
                        body: self.lower_statements(body),
                        catches: catches
                            .iter()
                            .map(|catch| self.lower_catch_clause(catch))
                            .collect(),
                        finally_body: self.lower_statements(finally_body),
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
            type_hint: parameter.type_hint.clone().map(lower_type_hint),
            by_ref: parameter.by_ref,
            is_variadic: parameter.is_variadic,
            default_value: parameter
                .default_value
                .as_ref()
                .map(|value| self.lower_expr(value)),
        }
    }
}

fn lower_trait(trait_decl: &crate::ast::TraitDecl) -> TraitDecl {
    TraitDecl {
        name: trait_decl.name.clone(),
        trait_uses: lower_trait_uses(&trait_decl.trait_uses),
        deprecated_message: trait_decl.attributes.deprecated_message.clone(),
        deprecated_since: trait_decl.attributes.deprecated_since.clone(),
        line: trait_decl.span.line,
    }
}

fn collect_constant_deprecations(
    program: &Program,
    constant_values: &HashMap<String, String>,
) -> HashMap<String, DeprecatedMetadata> {
    let mut constants = HashMap::new();
    collect_constant_deprecations_in(&program.statements, constant_values, &mut constants);
    constants
}

fn collect_constant_deprecations_in(
    statements: &[Statement],
    constant_values: &HashMap<String, String>,
    constants: &mut HashMap<String, DeprecatedMetadata>,
) {
    for statement in statements {
        match statement {
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    if declaration.attributes.deprecated_message.is_some()
                        || declaration.attributes.deprecated_since.is_some()
                    {
                        let metadata = DeprecatedMetadata {
                            message: declaration
                                .attributes
                                .deprecated_message_constant
                                .as_ref()
                                .and_then(|reference| match reference {
                                    AttributeConstantReference::Constant(name) => {
                                        constant_values.get(&name.to_ascii_lowercase()).cloned()
                                    }
                                    AttributeConstantReference::ClassConstant { .. } => None,
                                })
                                .or_else(|| declaration.attributes.deprecated_message.clone()),
                            since: declaration.attributes.deprecated_since.clone(),
                            message_dependency: declaration
                                .attributes
                                .deprecated_message_constant
                                .as_ref()
                                .and_then(|reference| match reference {
                                    AttributeConstantReference::Constant(name) => {
                                        let subject =
                                            format!("Constant {}", name.trim_start_matches('\\'));
                                        let current_subject = format!(
                                            "Constant {}",
                                            declaration.name.trim_start_matches('\\')
                                        );
                                        (subject != current_subject)
                                            .then(|| {
                                                constants
                                                    .get(&name.to_ascii_lowercase())
                                                    .map(|metadata| (subject, metadata))
                                            })
                                            .flatten()
                                    }
                                    AttributeConstantReference::ClassConstant { .. } => None,
                                })
                                .map(|(subject, metadata)| DeprecatedMessageDependency {
                                    subject,
                                    message: metadata.message.clone(),
                                    since: metadata.since.clone(),
                                }),
                            message_runtime_reference: None,
                        };
                        constants.insert(declaration.name.to_ascii_lowercase(), metadata);
                    }
                }
            }
            Statement::Block { statements, .. } => {
                collect_constant_deprecations_in(statements, constant_values, constants);
            }
            _ => {}
        }
    }
}

fn collect_constant_values(program: &Program) -> HashMap<String, String> {
    let mut constants = HashMap::new();
    collect_constant_values_in(&program.statements, &mut constants);
    constants
}

fn collect_constant_values_in(statements: &[Statement], constants: &mut HashMap<String, String>) {
    for statement in statements {
        match statement {
            Statement::Const { declarations, .. } => {
                for declaration in declarations {
                    if let Some(value) = string_constant_value(&declaration.value, constants, None)
                    {
                        constants.insert(declaration.name.to_ascii_lowercase(), value);
                    }
                }
            }
            Statement::Block { statements, .. } => {
                collect_constant_values_in(statements, constants);
            }
            _ => {}
        }
    }
}

fn collect_class_constant_values(
    constants: &[crate::ast::ClassConstantDecl],
    global_constants: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for constant in constants {
        if let Some(value) = string_constant_value(&constant.value, global_constants, Some(&values))
        {
            values.insert(constant.name.to_ascii_lowercase(), value);
        }
    }
    values
}

fn collect_class_constant_deprecations(
    constants: &[crate::ast::ClassConstantDecl],
    global_constants: &HashMap<String, String>,
    class_constant_values: &HashMap<String, String>,
) -> HashMap<String, DeprecatedMetadata> {
    let mut deprecations = HashMap::new();
    for constant in constants {
        if constant.attributes.deprecated_message.is_none()
            && constant.attributes.deprecated_since.is_none()
        {
            continue;
        }
        let message = constant
            .attributes
            .deprecated_message_constant
            .as_ref()
            .and_then(|reference| match reference {
                AttributeConstantReference::Constant(name) => {
                    global_constants.get(&name.to_ascii_lowercase()).cloned()
                }
                AttributeConstantReference::ClassConstant { name, .. } => class_constant_values
                    .get(&name.to_ascii_lowercase())
                    .cloned(),
            })
            .or_else(|| constant.attributes.deprecated_message.clone());
        deprecations.insert(
            constant.name.to_ascii_lowercase(),
            DeprecatedMetadata {
                message,
                since: constant.attributes.deprecated_since.clone(),
                message_dependency: None,
                message_runtime_reference: None,
            },
        );
    }
    deprecations
}

fn string_constant_value(
    expr: &Expr,
    global_constants: &HashMap<String, String>,
    class_constants: Option<&HashMap<String, String>>,
) -> Option<String> {
    match expr {
        Expr::String(value, _) => Some(value.clone()),
        Expr::Constant(name, _) => global_constants.get(&name.to_ascii_lowercase()).cloned(),
        Expr::ClassConstantFetch { name, .. } => {
            class_constants.and_then(|constants| constants.get(&name.to_ascii_lowercase()).cloned())
        }
        Expr::Grouped { expr, .. } => {
            string_constant_value(expr, global_constants, class_constants)
        }
        _ => None,
    }
}

fn lower_compile_warnings(warnings: &[AstCompileWarning]) -> Vec<CompileWarning> {
    warnings
        .iter()
        .map(|warning| CompileWarning {
            message: warning.message.clone(),
            line: warning.span.line,
            kind: match warning.kind {
                AstCompileWarningKind::Warning => CompileWarningKind::Warning,
                AstCompileWarningKind::UncaughtError => CompileWarningKind::UncaughtError,
            },
        })
        .collect()
}

fn lower_trait_uses(trait_uses: &[crate::ast::TraitUseDecl]) -> Vec<TraitUseDecl> {
    trait_uses
        .iter()
        .map(|trait_use| TraitUseDecl {
            name: trait_use.name.clone(),
            line: trait_use.span.line,
        })
        .collect()
}

fn lower_closure_capture(capture: &AstClosureUseCapture) -> ClosureCapture {
    ClosureCapture {
        name: capture.name.clone(),
        by_ref: capture.by_ref,
        warn_if_missing: capture.warn_if_missing,
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

fn lower_property_type_hint(type_hint: &crate::ast::PropertyTypeHint) -> PropertyTypeHint {
    PropertyTypeHint {
        text: type_hint.text.clone(),
        kind: match &type_hint.kind {
            AstPropertyTypeKind::Null => PropertyTypeKind::Null,
            AstPropertyTypeKind::Array => PropertyTypeKind::Array,
            AstPropertyTypeKind::Int => PropertyTypeKind::Int,
            AstPropertyTypeKind::Float => PropertyTypeKind::Float,
            AstPropertyTypeKind::String => PropertyTypeKind::String,
            AstPropertyTypeKind::Bool => PropertyTypeKind::Bool,
            AstPropertyTypeKind::Mixed => PropertyTypeKind::Mixed,
            AstPropertyTypeKind::Object => PropertyTypeKind::Object,
            AstPropertyTypeKind::Class(name) => PropertyTypeKind::Class(name.clone()),
            AstPropertyTypeKind::Unsupported => PropertyTypeKind::Unsupported,
        },
        allows_null: type_hint.allows_null,
    }
}

fn lower_type_hint(type_hint: AstTypeHint) -> TypeHint {
    match type_hint {
        AstTypeHint::Null => TypeHint::Null,
        AstTypeHint::Array => TypeHint::Array,
        AstTypeHint::Callable => TypeHint::Callable,
        AstTypeHint::Int => TypeHint::Int,
        AstTypeHint::Float => TypeHint::Float,
        AstTypeHint::String => TypeHint::String,
        AstTypeHint::Bool => TypeHint::Bool,
        AstTypeHint::True => TypeHint::True,
        AstTypeHint::False => TypeHint::False,
        AstTypeHint::Object => TypeHint::Object,
        AstTypeHint::Iterable => TypeHint::Iterable,
        AstTypeHint::Mixed => TypeHint::Mixed,
        AstTypeHint::Void => TypeHint::Void,
        AstTypeHint::Never => TypeHint::Never,
        AstTypeHint::Static => TypeHint::Static,
        AstTypeHint::Nullable(inner) => TypeHint::Nullable(Box::new(lower_type_hint(*inner))),
        AstTypeHint::Union(types) => {
            TypeHint::Union(types.into_iter().map(lower_type_hint).collect())
        }
        AstTypeHint::Intersection(types) => {
            TypeHint::Intersection(types.into_iter().map(lower_type_hint).collect())
        }
        AstTypeHint::Class(name) => TypeHint::Class(name),
    }
}

fn statements_contain_yield(statements: &[Statement]) -> bool {
    statements.iter().any(statement_contains_yield)
}

fn statement_contains_yield(statement: &Statement) -> bool {
    match statement {
        Statement::Assign { value, .. }
        | Statement::AssignRef { source: value, .. }
        | Statement::ArrayAssign { value, .. }
        | Statement::ArrayAssignRef { source: value, .. }
        | Statement::Print {
            expression: value, ..
        }
        | Statement::Expression {
            expression: value, ..
        }
        | Statement::Return {
            value: Some(value), ..
        }
        | Statement::Exit {
            value: Some(value), ..
        }
        | Statement::Throw { value, .. } => expr_contains_yield(value),
        Statement::Call { arguments, .. }
        | Statement::Echo {
            expressions: arguments,
            ..
        } => arguments.iter().any(expr_contains_yield),
        Statement::Const { declarations, .. } => declarations
            .iter()
            .any(|declaration| expr_contains_yield(&declaration.value)),
        Statement::Static { declarations, .. } => declarations
            .iter()
            .filter_map(|declaration| declaration.value.as_ref())
            .any(expr_contains_yield),
        Statement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_yield(condition)
                || statements_contain_yield(then_body)
                || statements_contain_yield(else_body)
        }
        Statement::Block { statements, .. }
        | Statement::While {
            body: statements, ..
        }
        | Statement::DoWhile {
            body: statements, ..
        } => statements_contain_yield(statements),
        Statement::For {
            initializers,
            condition,
            updates,
            body,
            ..
        } => {
            statements_contain_yield(initializers)
                || condition.as_ref().is_some_and(expr_contains_yield)
                || statements_contain_yield(updates)
                || statements_contain_yield(body)
        }
        Statement::Foreach { iterable, body, .. } => {
            expr_contains_yield(iterable) || statements_contain_yield(body)
        }
        Statement::Switch {
            expression, cases, ..
        } => {
            expr_contains_yield(expression)
                || cases.iter().any(|case| {
                    case.condition.as_ref().is_some_and(expr_contains_yield)
                        || statements_contain_yield(&case.body)
                })
        }
        Statement::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            statements_contain_yield(body)
                || catches
                    .iter()
                    .any(|catch| statements_contain_yield(&catch.body))
                || statements_contain_yield(finally_body)
        }
        Statement::Empty { .. }
        | Statement::ClassDeclaration { .. }
        | Statement::FunctionDeclaration { .. }
        | Statement::Return { value: None, .. }
        | Statement::Exit { value: None, .. }
        | Statement::Increment { .. }
        | Statement::Unset { .. }
        | Statement::Global { .. }
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::Label { .. }
        | Statement::Goto { .. }
        | Statement::InlineHtml { .. } => false,
    }
}

fn expr_contains_yield(expr: &Expr) -> bool {
    match expr {
        Expr::Yield { .. } => true,
        Expr::AnonymousFunction(_) => false,
        Expr::DynamicVariable { name, .. }
        | Expr::Print {
            expression: name, ..
        }
        | Expr::Include { path: name, .. }
        | Expr::Throw { value: name, .. }
        | Expr::Unary { expr: name, .. }
        | Expr::Cast { expr: name, .. }
        | Expr::Clone { expr: name, .. }
        | Expr::FirstClassCallable { callable: name, .. }
        | Expr::Grouped { expr: name, .. }
        | Expr::PipeValue { expr: name, .. } => expr_contains_yield(name),
        Expr::String(_, _)
        | Expr::InterpolatedString(_, _)
        | Expr::ShellExec { .. }
        | Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_)
        | Expr::Variable(_, _)
        | Expr::IncDec { .. }
        | Expr::Constant(_, _)
        | Expr::MagicConstant(_, _)
        | Expr::StaticPropertyFetch { .. }
        | Expr::ClassConstantFetch { .. } => false,
        Expr::DynamicStaticPropertyFetch { receiver, .. } => expr_contains_yield(receiver),
        Expr::Assign { target, value, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(value)
        }
        Expr::AssignRef { target, source, .. } => {
            assignment_target_contains_yield(target) || expr_contains_yield(source)
        }
        Expr::Call { arguments, .. }
        | Expr::DynamicCall { arguments, .. }
        | Expr::MethodCall { arguments, .. }
        | Expr::DynamicMethodCall { arguments, .. }
        | Expr::NewObject { arguments, .. }
        | Expr::DynamicNewObject { arguments, .. } => arguments.iter().any(expr_contains_yield),
        Expr::PropertyFetch { receiver, .. }
        | Expr::NullsafePropertyFetch { receiver, .. }
        | Expr::DynamicClassNameFetch { receiver, .. } => expr_contains_yield(receiver),
        Expr::InstanceOf { expr, target, .. } => {
            expr_contains_yield(expr)
                || matches!(target, AstInstanceOfTarget::Expr(target) if expr_contains_yield(target))
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        Expr::Array { elements, .. } => elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || match &element.value {
                    AstArrayElementValue::Hole(_) => false,
                    AstArrayElementValue::Value(value) | AstArrayElementValue::Unpack(value) => {
                        expr_contains_yield(value)
                    }
                    AstArrayElementValue::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                }
        }),
        Expr::List(list) => list.elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || element.target.as_ref().is_some_and(|target| match target {
                    AstListExprElementTarget::Value(value) => expr_contains_yield(value),
                    AstListExprElementTarget::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                })
        }),
        Expr::ArrayAccess { array, index, .. } => {
            expr_contains_yield(array)
                || index
                    .as_ref()
                    .is_some_and(|index| expr_contains_yield(index))
        }
        Expr::Isset { targets, .. } => targets.iter().any(expr_contains_yield),
        Expr::Empty { target, .. } => expr_contains_yield(target),
        Expr::Binary { left, right, .. } => expr_contains_yield(left) || expr_contains_yield(right),
        Expr::Ternary {
            condition,
            if_true,
            if_false,
            ..
        } => {
            expr_contains_yield(condition)
                || if_true
                    .as_ref()
                    .is_some_and(|if_true| expr_contains_yield(if_true))
                || expr_contains_yield(if_false)
        }
        Expr::Match { subject, arms, .. } => {
            expr_contains_yield(subject)
                || arms.iter().any(|arm| {
                    arm.conditions.iter().any(expr_contains_yield)
                        || expr_contains_yield(&arm.value)
                })
        }
    }
}

fn assignment_target_contains_yield(target: &AstAssignmentTarget) -> bool {
    match target {
        AstAssignmentTarget::Variable { .. }
        | AstAssignmentTarget::ArrayDim(_)
        | AstAssignmentTarget::StaticProperty { .. } => false,
        AstAssignmentTarget::StaticPropertyArrayDim { dimensions, .. } => {
            dimensions.iter().flatten().any(expr_contains_yield)
        }
        AstAssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => expr_contains_yield(array) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        AstAssignmentTarget::DynamicArrayDim {
            name, dimensions, ..
        } => expr_contains_yield(name) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstAssignmentTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        AstAssignmentTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
        AstAssignmentTarget::DynamicStaticProperty { receiver, .. } => {
            expr_contains_yield(receiver)
        }
        AstAssignmentTarget::List(list) => list.elements.iter().any(|element| {
            element.key.as_ref().is_some_and(expr_contains_yield)
                || match &element.target {
                    AstListAssignmentElementTarget::Value(target) => {
                        assignment_target_contains_yield(target)
                    }
                    AstListAssignmentElementTarget::Reference(target) => {
                        reference_target_contains_yield(target)
                    }
                }
        }),
    }
}

fn reference_target_contains_yield(target: &AstReferenceTarget) -> bool {
    match target {
        AstReferenceTarget::Variable { .. } | AstReferenceTarget::ArrayDim(_) => false,
        AstReferenceTarget::DynamicVariable { name, .. } => expr_contains_yield(name),
        AstReferenceTarget::PropertyArrayDim {
            receiver,
            dimensions,
            ..
        } => expr_contains_yield(receiver) || dimensions.iter().flatten().any(expr_contains_yield),
        AstReferenceTarget::Property { receiver, .. } => expr_contains_yield(receiver),
        AstReferenceTarget::DynamicProperty { receiver, name, .. } => {
            expr_contains_yield(receiver) || expr_contains_yield(name)
        }
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
            AstAssignmentTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => AssignmentTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
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
            AstAssignmentTarget::StaticPropertyArrayDim {
                class_name,
                name,
                dimensions,
                span,
            } => AssignmentTarget::StaticPropertyArrayDim {
                class_name: class_name.clone(),
                name: name.clone(),
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
            AstAssignmentTarget::DynamicStaticPropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => AssignmentTarget::DynamicStaticPropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
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
            AstAssignmentTarget::ValueArrayDim {
                array,
                dimensions,
                span,
            } => AssignmentTarget::ValueArrayDim {
                array: Box::new(self.lower_expr(array)),
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
            AstAssignmentTarget::Property {
                receiver,
                name,
                span,
            } => AssignmentTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstAssignmentTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => AssignmentTarget::DynamicProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
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
            AstAssignmentTarget::DynamicStaticProperty {
                receiver,
                name,
                span,
            } => AssignmentTarget::DynamicStaticProperty {
                receiver: Box::new(self.lower_expr(receiver)),
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
            AstIncDecTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => IncDecTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
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
            AstReferenceTarget::DynamicVariable { name, span } => {
                ReferenceTarget::DynamicVariable {
                    name: Box::new(self.lower_expr(name)),
                    line: span.line,
                }
            }
            AstReferenceTarget::ArrayDim(target) => {
                ReferenceTarget::ArrayDim(self.lower_array_dim_target(target))
            }
            AstReferenceTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => ReferenceTarget::PropertyArrayDim {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
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
            AstReferenceTarget::Property {
                receiver,
                name,
                span,
            } => ReferenceTarget::Property {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            AstReferenceTarget::DynamicProperty {
                receiver,
                name,
                span,
            } => ReferenceTarget::DynamicProperty {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                line: span.line,
            },
        }
    }

    fn lower_unset_target(&mut self, target: &AstUnsetTarget) -> Instruction {
        match target {
            AstUnsetTarget::Variable { name, .. } => {
                Instruction::UnsetVariable { name: name.clone() }
            }
            AstUnsetTarget::DynamicVariable { name, span } => Instruction::UnsetDynamicVariable {
                name: self.lower_expr(name),
                line: span.line,
            },
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
            AstUnsetTarget::PropertyArrayDim {
                receiver,
                name,
                dimensions,
                span,
            } => Instruction::UnsetPropertyArrayDim {
                receiver: self.lower_expr(receiver),
                name: name.clone(),
                dimensions: dimensions
                    .iter()
                    .map(|dimension| self.lower_expr(dimension))
                    .collect(),
                line: span.line,
            },
            AstUnsetTarget::Property {
                receiver,
                name,
                span,
            } => Instruction::UnsetProperty {
                receiver: self.lower_expr(receiver),
                name: name.clone(),
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
            type_names: catch.type_names.clone(),
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
                AssignmentTarget::ArrayDim { .. }
                | AssignmentTarget::DynamicArrayDim { .. }
                | AssignmentTarget::PropertyArrayDim { .. }
                | AssignmentTarget::StaticPropertyArrayDim { .. }
                | AssignmentTarget::DynamicStaticPropertyArrayDim { .. }
                | AssignmentTarget::ValueArrayDim { .. }
                | AssignmentTarget::Property { .. }
                | AssignmentTarget::DynamicProperty { .. }
                | AssignmentTarget::StaticProperty { .. }
                | AssignmentTarget::DynamicStaticProperty { .. } => (op, self.lower_expr(value)),
                AssignmentTarget::DynamicVariable { .. } | AssignmentTarget::List(_) => {
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
    fn lower_instanceof_target(&mut self, target: &AstInstanceOfTarget) -> InstanceOfTarget {
        match target {
            AstInstanceOfTarget::ClassName { name, .. } => {
                InstanceOfTarget::ClassName(name.clone())
            }
            AstInstanceOfTarget::Expr(expr) => {
                InstanceOfTarget::Expr(Box::new(self.lower_expr(expr)))
            }
        }
    }

    fn lower_expr(&mut self, expr: &Expr) -> ValueExpr {
        match expr {
            Expr::String(value, _) => ValueExpr::String(value.clone()),
            Expr::InterpolatedString(parts, span) => lower_interpolated_string(parts, span.line),
            Expr::ShellExec { command, span } => ValueExpr::InternalCall {
                name: "shell_exec".to_string(),
                arguments: vec![ValueExpr::String(command.clone())],
                argument_names: vec![None],
                argument_unpacks: vec![false],
                line: span.line,
            },
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
            Expr::Constant(name, span) => {
                let metadata = self.constant_deprecations.get(&name.to_ascii_lowercase());
                ValueExpr::Constant {
                    name: name.clone(),
                    deprecated_message: metadata.and_then(|metadata| metadata.message.clone()),
                    deprecated_since: metadata.and_then(|metadata| metadata.since.clone()),
                    deprecated_message_dependency: metadata
                        .and_then(|metadata| metadata.message_dependency.clone()),
                    line: span.line,
                }
            }
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
            Expr::List(_) => {
                unreachable!("list destructuring syntax must lower through assignment targets")
            }
            Expr::ArrayAccess { array, index, span } => match index {
                Some(index) => ValueExpr::ArrayAccess {
                    array: Box::new(self.lower_expr(array)),
                    index: Box::new(self.lower_expr(index)),
                    line: span.line,
                },
                None => ValueExpr::ArrayAppendAccess {
                    array: Box::new(self.lower_expr(array)),
                    line: span.line,
                },
            },
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
            Expr::Throw { value, span } => ValueExpr::Throw {
                value: Box::new(self.lower_expr(value)),
                line: span.line,
            },
            Expr::Yield { key, value, span } => ValueExpr::Yield {
                key: key.as_ref().map(|key| Box::new(self.lower_expr(key))),
                value: value.as_ref().map(|value| Box::new(self.lower_expr(value))),
                line: span.line,
            },
            Expr::Call {
                name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => {
                let (arguments, argument_names) =
                    self.lower_internal_call_arguments(name, arguments, argument_names);
                ValueExpr::InternalCall {
                    name: name.clone(),
                    arguments,
                    argument_names,
                    argument_unpacks: argument_unpacks.clone(),
                    line: span.line,
                }
            }
            Expr::FirstClassCallable { callable, span } => ValueExpr::FirstClassCallable {
                callable: Box::new(self.lower_expr(callable)),
                line: span.line,
            },
            Expr::DynamicCall {
                callee,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => ValueExpr::DynamicCall {
                callee: Box::new(self.lower_expr(callee)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::MethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => ValueExpr::MethodCall {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::DynamicMethodCall {
                receiver,
                name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => ValueExpr::DynamicMethodCall {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::NewObject {
                class_name,
                source_name: _,
                arguments,
                argument_names,
                argument_unpacks,
                anonymous_class_source: _,
                span,
            } => ValueExpr::NewObject {
                class_name: class_name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::DynamicNewObject {
                class_name,
                arguments,
                argument_names,
                argument_unpacks,
                span,
            } => ValueExpr::DynamicNewObject {
                class_name: Box::new(self.lower_expr(class_name)),
                arguments: arguments
                    .iter()
                    .map(|argument| self.lower_expr(argument))
                    .collect(),
                argument_names: argument_names.clone(),
                argument_unpacks: argument_unpacks.clone(),
                line: span.line,
            },
            Expr::Clone { expr, span } => ValueExpr::Clone {
                expr: Box::new(self.lower_expr(expr)),
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
            Expr::NullsafePropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::NullsafePropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicPropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::DynamicPropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: Box::new(self.lower_expr(name)),
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
            Expr::DynamicStaticPropertyFetch {
                receiver,
                name,
                span,
            } => ValueExpr::DynamicStaticPropertyFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                name: name.clone(),
                line: span.line,
            },
            Expr::ClassConstantFetch {
                class_name,
                name,
                span,
            } => ValueExpr::ClassConstantFetch {
                class_name: class_name.clone(),
                name: name.clone(),
                line: span.line,
            },
            Expr::DynamicClassNameFetch { receiver, span } => ValueExpr::DynamicClassNameFetch {
                receiver: Box::new(self.lower_expr(receiver)),
                line: span.line,
            },
            Expr::InstanceOf { expr, target, span } => ValueExpr::InstanceOf {
                expr: Box::new(self.lower_expr(expr)),
                target: self.lower_instanceof_target(target),
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
            Expr::Match {
                subject,
                arms,
                span,
            } => ValueExpr::Match {
                subject: Box::new(self.lower_expr(subject)),
                arms: arms.iter().map(|arm| self.lower_match_arm(arm)).collect(),
                line: span.line,
            },
            Expr::Grouped { expr, .. } => self.lower_expr(expr),
            Expr::PipeValue { expr, span } => ValueExpr::PipeValue {
                expr: Box::new(self.lower_expr(expr)),
                line: span.line,
            },
        }
    }

    fn lower_match_arm(&mut self, arm: &AstMatchArm) -> MatchArm {
        MatchArm {
            conditions: arm
                .conditions
                .iter()
                .map(|condition| self.lower_expr(condition))
                .collect(),
            value: self.lower_expr(&arm.value),
            is_default: arm.is_default,
            line: arm.span.line,
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
            line: element.line,
        }
    }

    fn lower_array_element_value(&mut self, value: &AstArrayElementValue) -> ArrayElementValue {
        match value {
            AstArrayElementValue::Hole(_) => {
                unreachable!("array holes are rejected outside list destructuring")
            }
            AstArrayElementValue::Value(value) => ArrayElementValue::Value(self.lower_expr(value)),
            AstArrayElementValue::Reference(target) => {
                ArrayElementValue::Reference(self.lower_reference_target(target))
            }
            AstArrayElementValue::Unpack(value) => ArrayElementValue::Unpack {
                value: self.lower_expr(value),
                line: value.span().line,
            },
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
        AstStringPart::PropertyFetch { variable, property } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::PropertyFetch {
                receiver: Box::new(ValueExpr::Load {
                    name: variable.clone(),
                    line,
                }),
                name: property.clone(),
                line,
            }),
            line,
        }),
        AstStringPart::PropertyChain {
            variable,
            properties,
        } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(lower_interpolated_property_chain(
                variable, properties, line,
            )),
            line,
        }),
        AstStringPart::MethodCall { variable, method } => Some(ValueExpr::Cast {
            kind: CastKind::String,
            expr: Box::new(ValueExpr::MethodCall {
                receiver: Box::new(ValueExpr::Load {
                    name: variable.clone(),
                    line,
                }),
                name: method.clone(),
                arguments: Vec::new(),
                argument_names: Vec::new(),
                argument_unpacks: Vec::new(),
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
        Expr::String(value, _) => assertion_string_text(value),
        Expr::InterpolatedString(_, _) => "\"\"".to_string(),
        Expr::ShellExec { command, .. } => format!("`{}`", command.replace('`', "\\`")),
        Expr::Int(value, _) => value.to_string(),
        Expr::Float(value, _) => assertion_float_text(*value),
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
        Expr::FirstClassCallable { callable, .. } => {
            format!("{}(...)", assertion_expr_text(callable))
        }
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
        Expr::DynamicMethodCall {
            receiver,
            name,
            arguments,
            ..
        } => format!(
            "{}->{{{}}}({})",
            assertion_expr_text(receiver),
            assertion_expr_text(name),
            assertion_argument_list_text(arguments)
        ),
        Expr::NewObject {
            source_name,
            arguments,
            anonymous_class_source,
            ..
        } => {
            if let Some(source) = anonymous_class_source {
                assertion_anonymous_class_source_text(source)
            } else {
                format!(
                    "new {source_name}({})",
                    assertion_argument_list_text(arguments)
                )
            }
        }
        Expr::DynamicNewObject {
            class_name,
            arguments,
            ..
        } => format!(
            "new {}({})",
            assertion_expr_text(class_name),
            assertion_argument_list_text(arguments)
        ),
        Expr::Clone { expr, .. } => format!("clone {}", assertion_expr_text(expr)),
        Expr::PropertyFetch { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        Expr::NullsafePropertyFetch { receiver, name, .. } => {
            format!("{}?->{name}", assertion_expr_text(receiver))
        }
        Expr::DynamicPropertyFetch { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
        Expr::StaticPropertyFetch {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        Expr::DynamicStaticPropertyFetch { receiver, name, .. } => {
            format!("{}::${name}", assertion_expr_text(receiver))
        }
        Expr::ClassConstantFetch {
            class_name, name, ..
        } => format!("{class_name}::{name}"),
        Expr::DynamicClassNameFetch { receiver, .. } => {
            format!("{}::class", assertion_expr_text(receiver))
        }
        Expr::InstanceOf { expr, target, .. } => {
            let target_text = match target {
                AstInstanceOfTarget::ClassName { source_name, .. } => source_name.clone(),
                AstInstanceOfTarget::Expr(target) => assertion_expr_text(target),
            };
            format!("{} instanceof {target_text}", assertion_expr_text(expr))
        }
        Expr::Array { elements, .. } => format!(
            "[{}]",
            elements
                .iter()
                .map(assertion_array_element_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::List(list) => format!(
            "list({})",
            list.elements
                .iter()
                .map(assertion_list_expr_element_text)
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
        Expr::Throw { value, .. } => format!("throw {}", assertion_expr_text(value)),
        Expr::Yield { key, value, .. } => match (key, value) {
            (Some(key), Some(value)) => {
                format!(
                    "yield {} => {}",
                    assertion_expr_text(key),
                    assertion_expr_text(value)
                )
            }
            (None, Some(value)) => format!("yield {}", assertion_expr_text(value)),
            (None, None) => "yield".to_string(),
            (Some(key), None) => format!("yield {} => null", assertion_expr_text(key)),
        },
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
        Expr::Match { subject, arms, .. } => format!(
            "match ({}) {{ {} }}",
            assertion_expr_text(subject),
            arms.iter()
                .map(assertion_match_arm_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Grouped { expr, .. } => format!("({})", assertion_expr_text(expr)),
        Expr::PipeValue { expr, .. } => assertion_expr_text(expr),
        Expr::AnonymousFunction(function) => assertion_anonymous_function_text(function),
    }
}

fn assertion_match_arm_text(arm: &AstMatchArm) -> String {
    let conditions = if arm.is_default {
        "default".to_string()
    } else {
        arm.conditions
            .iter()
            .map(assertion_expr_text)
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!("{} => {}", conditions, assertion_expr_text(&arm.value))
}

fn assertion_string_text(value: &str) -> String {
    let mut text = String::with_capacity(value.len() + 2);
    text.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => text.push_str("\\\\"),
            '\'' => text.push_str("\\'"),
            _ => text.push(ch),
        }
    }
    text.push('\'');
    text
}

fn assertion_anonymous_function_text(function: &AstAnonymousFunction) -> String {
    if function.is_arrow {
        let return_by_ref = if function.return_by_ref { "&" } else { "" };
        let parameters = function
            .parameters
            .iter()
            .map(assertion_function_parameter_text)
            .collect::<Vec<_>>()
            .join(", ");
        let return_type = function
            .return_type
            .as_ref()
            .map(|hint| format!(": {}", assertion_type_hint_text(hint)))
            .unwrap_or_default();
        let body = match function.body.as_slice() {
            [Statement::Return {
                value: Some(value), ..
            }] => assertion_expr_text(value),
            _ => "null".to_string(),
        };

        let static_prefix = if function.is_static { "static " } else { "" };
        return format!("{static_prefix}fn{return_by_ref}({parameters}){return_type} => {body}");
    }

    let body = function
        .body
        .iter()
        .filter_map(|statement| assertion_statement_text(statement, "    "))
        .collect::<Vec<_>>();
    if body.is_empty() {
        return "function()".to_string();
    }
    let static_prefix = if function.is_static { "static " } else { "" };
    format!("{static_prefix}function () {{\n{}\n}}", body.join("\n"))
}

fn assertion_statement_text(statement: &Statement, indent: &str) -> Option<String> {
    match statement {
        Statement::ClassDeclaration { source, span } => Some(format!(
            "{}\n",
            assertion_source_block_text(source, span.column, indent)
        )),
        Statement::Expression { expression, .. } => {
            Some(assertion_expression_statement_text(expression, indent))
        }
        Statement::Empty { .. } => None,
        _ => None,
    }
}

fn assertion_expression_statement_text(expr: &Expr, indent: &str) -> String {
    match expr {
        Expr::Match { subject, arms, .. } => {
            assertion_match_expression_statement_text(subject, arms, indent)
        }
        _ => format!("{indent}{};", assertion_expr_text(expr)),
    }
}

fn assertion_match_expression_statement_text(
    subject: &Expr,
    arms: &[AstMatchArm],
    indent: &str,
) -> String {
    let arm_indent = format!("{indent}    ");
    let mut text = format!("{indent}match ({}) {{", assertion_expr_text(subject));
    for arm in arms {
        text.push('\n');
        text.push_str(&arm_indent);
        text.push_str(&assertion_match_arm_text(arm));
        text.push(',');
    }
    text.push('\n');
    text.push_str(indent);
    text.push_str("};");
    text
}

fn assertion_source_block_text(source: &str, source_column: usize, indent: &str) -> String {
    let source_indent = source_column.saturating_sub(1);
    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line = if index == 0 {
                line
            } else {
                line.get(source_indent..).unwrap_or(line)
            };
            format!("{indent}{line}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assertion_anonymous_class_source_text(source: &str) -> String {
    let mut lines = source.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        return first.to_string();
    }
    let strip = rest
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut normalized = Vec::with_capacity(rest.len() + 1);
    normalized.push(first.to_string());
    normalized.extend(rest.into_iter().map(|line| {
        if line.len() >= strip {
            line[strip..].to_string()
        } else {
            line.to_string()
        }
    }));
    normalized.join("\n")
}

fn assertion_float_text(value: f64) -> String {
    let text = value.to_string();
    if value.is_finite()
        && value.fract() == 0.0
        && !text.contains('.')
        && !text.contains('e')
        && !text.contains('E')
    {
        format!("{text}.0")
    } else {
        text
    }
}

fn assertion_argument_list_text(arguments: &[Expr]) -> String {
    arguments
        .iter()
        .map(assertion_expr_text)
        .collect::<Vec<_>>()
        .join(", ")
}

fn assertion_function_parameter_text(parameter: &AstFunctionParameter) -> String {
    let mut variable = String::new();
    if parameter.by_ref {
        variable.push('&');
    }
    if parameter.is_variadic {
        variable.push_str("...");
    }
    variable.push('$');
    variable.push_str(&parameter.name);

    if let Some(type_hint) = &parameter.type_hint {
        format!("{} {variable}", assertion_type_hint_text(type_hint))
    } else {
        variable
    }
}

fn assertion_type_hint_text(type_hint: &AstTypeHint) -> String {
    match type_hint {
        AstTypeHint::Null => "null".to_string(),
        AstTypeHint::Array => "array".to_string(),
        AstTypeHint::Callable => "callable".to_string(),
        AstTypeHint::Int => "int".to_string(),
        AstTypeHint::Float => "float".to_string(),
        AstTypeHint::String => "string".to_string(),
        AstTypeHint::Bool => "bool".to_string(),
        AstTypeHint::True => "true".to_string(),
        AstTypeHint::False => "false".to_string(),
        AstTypeHint::Object => "object".to_string(),
        AstTypeHint::Iterable => "Traversable|array".to_string(),
        AstTypeHint::Mixed => "mixed".to_string(),
        AstTypeHint::Void => "void".to_string(),
        AstTypeHint::Never => "never".to_string(),
        AstTypeHint::Static => "static".to_string(),
        AstTypeHint::Nullable(inner) => format!("?{}", assertion_type_hint_text(inner)),
        AstTypeHint::Union(types) => types
            .iter()
            .map(assertion_type_hint_text)
            .collect::<Vec<_>>()
            .join("|"),
        AstTypeHint::Intersection(types) => types
            .iter()
            .map(assertion_type_hint_text)
            .collect::<Vec<_>>()
            .join("&"),
        AstTypeHint::Class(name) => name.clone(),
    }
}

fn assertion_array_element_text(element: &AstArrayElement) -> String {
    let value = match &element.value {
        AstArrayElementValue::Hole(_) => String::new(),
        AstArrayElementValue::Value(value) => assertion_expr_text(value),
        AstArrayElementValue::Reference(target) => {
            format!("&{}", assertion_reference_target_text(target))
        }
        AstArrayElementValue::Unpack(value) => format!("...{}", assertion_expr_text(value)),
    };
    if let Some(key) = &element.key {
        format!("{} => {value}", assertion_expr_text(key))
    } else {
        value
    }
}

fn assertion_list_expr_element_text(element: &crate::ast::ListExprElement) -> String {
    let Some(target) = &element.target else {
        return String::new();
    };
    let value = match target {
        AstListExprElementTarget::Value(value) => assertion_expr_text(value),
        AstListExprElementTarget::Reference(target) => {
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
        AstAssignmentTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
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
        AstAssignmentTarget::StaticPropertyArrayDim {
            class_name,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{class_name}::${name}");
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
        AstAssignmentTarget::DynamicStaticPropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}::${name}", assertion_expr_text(receiver));
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
        AstAssignmentTarget::ValueArrayDim {
            array, dimensions, ..
        } => {
            let mut text = assertion_expr_text(array);
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
        AstAssignmentTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstAssignmentTarget::DynamicProperty { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
        AstAssignmentTarget::StaticProperty {
            class_name, name, ..
        } => format!("{class_name}::${name}"),
        AstAssignmentTarget::DynamicStaticProperty { receiver, name, .. } => {
            format!("{}::${name}", assertion_expr_text(receiver))
        }
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
        AstReferenceTarget::DynamicVariable { name, .. } => {
            format!("${{{}}}", assertion_expr_text(name))
        }
        AstReferenceTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstReferenceTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
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
        AstReferenceTarget::Property { receiver, name, .. } => {
            format!("{}->{name}", assertion_expr_text(receiver))
        }
        AstReferenceTarget::DynamicProperty { receiver, name, .. } => {
            format!(
                "{}->{{{}}}",
                assertion_expr_text(receiver),
                assertion_expr_text(name)
            )
        }
    }
}

fn assertion_inc_dec_target_text(target: &AstIncDecTarget) -> String {
    match target {
        AstIncDecTarget::Variable { name, .. } => format!("${name}"),
        AstIncDecTarget::DynamicVariable { .. } => "${...}".to_string(),
        AstIncDecTarget::DynamicArrayDim { .. } => "${...}[...]".to_string(),
        AstIncDecTarget::ArrayDim(target) => assertion_array_dim_target_text(target),
        AstIncDecTarget::PropertyArrayDim {
            receiver,
            name,
            dimensions,
            ..
        } => {
            let mut text = format!("{}->{name}", assertion_expr_text(receiver));
            for dimension in dimensions {
                text.push('[');
                if let Some(dimension) = dimension {
                    text.push_str(&assertion_expr_text(dimension));
                }
                text.push(']');
            }
            text
        }
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
        AstCastKind::Array => "array",
        AstCastKind::Object => "object",
        AstCastKind::Void => "void",
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

fn lower_interpolated_property_chain(
    variable: &str,
    properties: &[String],
    line: usize,
) -> ValueExpr {
    let mut expr = ValueExpr::Load {
        name: variable.to_string(),
        line,
    };
    for property in properties {
        expr = ValueExpr::PropertyFetch {
            receiver: Box::new(expr),
            name: property.clone(),
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
        AstCastKind::Array => CastKind::Array,
        AstCastKind::Object => CastKind::Object,
        AstCastKind::Void => CastKind::Void,
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
