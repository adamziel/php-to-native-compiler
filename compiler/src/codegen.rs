use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::ast::{AssignTarget, BinaryOp, Expr, Program, Span, Stmt, UnaryOp};
use crate::error::{CompileResult, Diagnostic, Phase};

const MAX_KNOWN_INT_VALUES: usize = 4;
const MAX_KNOWN_FLOAT_VALUES: usize = 4;
const MAX_KNOWN_STRING_VALUES: usize = 4;
const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const ASSEMBLY_FUNCTION_DECLARATION_REJECTION: &str = "assembly user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_MAGIC_CONSTANT_REJECTION: &str = "LLVM magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, and __FUNCTION__ until native source mapping, path canonicalization, and function-context lowering exist; phpc run handles current magic constant behavior";
const ASSEMBLY_MAGIC_CONSTANT_REJECTION: &str = "assembly magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, and __FUNCTION__ until native source mapping, path canonicalization, and function-context lowering exist; phpc run handles current magic constant behavior";
const LLVM_GLOBAL_CONSTANT_REJECTION: &str = "LLVM global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const ASSEMBLY_GLOBAL_CONSTANT_REJECTION: &str = "assembly global-constant lowering rejects built-in constant values, runtime-defined constants, bare constant reads, top-level const declarations, define()/constant(), and unsupported defined() forms until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const ASSEMBLY_OBJECT_CLASS_REJECTION: &str = "assembly object/class lowering rejects class declarations, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const ASSEMBLY_ARRAY_REJECTION: &str = "assembly array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const ASSEMBLY_CONTROL_FLOW_REJECTION: &str = "assembly control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const ASSEMBLY_MUTATION_REJECTION: &str = "assembly mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_ISSET_REJECTION: &str = "LLVM isset lowering rejects array offset operands, object property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const ASSEMBLY_ISSET_REJECTION: &str = "assembly isset lowering rejects array offset operands, object property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const LLVM_EMPTY_REJECTION: &str = "LLVM empty lowering rejects array offset operands, object property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const ASSEMBLY_EMPTY_REJECTION: &str = "assembly empty lowering rejects array offset operands, object property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";
const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unsupported unary operators or operands until native PHP numeric coercion, truthiness conversion, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary behavior";
const ASSEMBLY_UNARY_REJECTION: &str = "assembly unary lowering rejects unsupported unary operators or operands until native PHP numeric coercion, truthiness conversion, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary behavior";
const LLVM_ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const ASSEMBLY_ARITHMETIC_REJECTION: &str = "assembly arithmetic lowering rejects unsupported binary arithmetic operators or operands until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const LLVM_MIXED_NUMERIC_ARITHMETIC_REJECTION: &str = "LLVM mixed numeric arithmetic lowering rejects int/float operands until native PHP numeric promotion, result typing, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current mixed numeric arithmetic behavior";
const ASSEMBLY_MIXED_NUMERIC_ARITHMETIC_REJECTION: &str = "assembly mixed numeric arithmetic lowering rejects int/float operands until native PHP numeric promotion, result typing, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current mixed numeric arithmetic behavior";
const LLVM_SCALAR_COERCION_ARITHMETIC_REJECTION: &str = "LLVM scalar-coercion arithmetic lowering rejects booleans, nulls, and strings in +, -, and * until native PHP numeric coercion, string numeric parsing, warnings/recovery behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current scalar-coercion arithmetic behavior";
const ASSEMBLY_SCALAR_COERCION_ARITHMETIC_REJECTION: &str = "assembly scalar-coercion arithmetic lowering rejects booleans, nulls, and strings in +, -, and * until native PHP numeric coercion, string numeric parsing, warnings/recovery behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current scalar-coercion arithmetic behavior";
const LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "LLVM integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";
const ASSEMBLY_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "assembly integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";
const LLVM_DIVISION_REJECTION: &str = "LLVM division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";
const ASSEMBLY_DIVISION_REJECTION: &str = "assembly division lowering rejects / until native PHP division semantics, zero-divisor runtime checks, avoidance of misleading integer truncation, overflow/INF/NAN behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current division behavior";
const LLVM_MODULO_RUNTIME_CHECK_REJECTION: &str = "LLVM modulo lowering rejects dynamic, zero, or non-positive integer divisors until native modulo runtime checks, PHP modulo diagnostics, negative-divisor/min-int edge behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current modulo behavior";
const ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION: &str = "assembly modulo lowering rejects dynamic, zero, or non-positive integer divisors until native modulo runtime checks, PHP modulo diagnostics, negative-divisor/min-int edge behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current modulo behavior";
const LLVM_CONCAT_REJECTION: &str = "LLVM concatenation lowering rejects unsupported concatenation operands until native PHP scalar-to-string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const ASSEMBLY_CONCAT_REJECTION: &str = "assembly concatenation lowering rejects unsupported concatenation operands until native PHP scalar-to-string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const LLVM_BITWISE_REJECTION: &str = "LLVM bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior";
const ASSEMBLY_BITWISE_REJECTION: &str = "assembly bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior";
const LLVM_VARIABLE_READ_REJECTION: &str = "LLVM variable-read lowering rejects reads that are not statically assigned earlier in the same straight-line native subset until native symbol-table storage, undefined-variable diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current variable-read behavior";
const ASSEMBLY_VARIABLE_READ_REJECTION: &str = "assembly variable-read lowering rejects reads that are not statically assigned earlier in the same straight-line native subset until native symbol-table storage, undefined-variable diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current variable-read behavior";

pub fn emit_llvm_ir(program: &Program) -> CompileResult<String> {
    let mut generator = LlvmGenerator::default();
    generator.emit_program(program)
}

pub fn emit_assembly(program: &Program) -> CompileResult<String> {
    let ir = emit_llvm_ir(program)?;
    if command_available("clang") {
        return clang_assembly_from_ir(&ir);
    }
    if command_available("llc") {
        return llc_assembly_from_ir(&ir);
    }
    if command_available("cc") {
        let c_source = emit_c_source_for_assembly(program)?;
        return cc_assembly_from_c(&c_source);
    }

    Err(Diagnostic::new(
        Phase::Codegen,
        0,
        0,
        "no assembly backend found; install clang, llc, or cc",
    ))
}

pub fn native_runtime_scalar_echo_probe_ir() -> String {
    [
        "; generated by phpc native runtime helper probe",
        "; this is a dependency sketch, not production lowering or linked execution",
        "%phpc.NativeScalarValue = type { i8, i8, [6 x i8], i64, double }",
        "declare i64 @phpc_native_scalar_echo_len(%phpc.NativeScalarValue)",
        "declare i64 @phpc_native_scalar_echo_write(%phpc.NativeScalarValue, ptr, i64)",
        "",
        "define i64 @phpc_probe_scalar_echo_len() {",
        "entry:",
        "  %value = insertvalue %phpc.NativeScalarValue zeroinitializer, i8 2, 0",
        "  %with_payload = insertvalue %phpc.NativeScalarValue %value, i64 42, 3",
        "  %len = call i64 @phpc_native_scalar_echo_len(%phpc.NativeScalarValue %with_payload)",
        "  ret i64 %len",
        "}",
        "",
    ]
    .join("\n")
}

#[derive(Default)]
struct LlvmGenerator {
    strings: Vec<(String, String)>,
    body: Vec<String>,
    variables: HashMap<String, IrValue>,
    known_ints: HashMap<String, KnownInt>,
    known_floats: HashMap<String, KnownFloat>,
    known_strings: HashMap<String, KnownString>,
    known_bools: HashMap<String, KnownBool>,
    next_string: usize,
    next_temp: usize,
    uses_strcmp: bool,
}

#[derive(Debug, Clone)]
enum IrValue {
    Int(String),
    Float(String),
    String(String),
    StringPtr(String),
    Bool(bool),
    BoolExpr(String),
    Null,
}

#[derive(Debug, Clone)]
struct KnownInt {
    values: Vec<i64>,
}

impl KnownInt {
    fn one(value: i64) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = i64>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_INT_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[i64] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }

    fn is_single_value(&self, expected: i64) -> bool {
        matches!(self.values.as_slice(), [value] if *value == expected)
    }
}

#[derive(Debug, Clone)]
struct KnownFloat {
    values: Vec<f64>,
}

impl KnownFloat {
    fn one(value: f64) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = f64>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.iter().any(|existing| existing == &value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_FLOAT_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[f64] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

#[derive(Debug, Clone)]
struct KnownString {
    values: Vec<String>,
}

impl KnownString {
    fn one(value: String) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = String>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
            if unique.len() > MAX_KNOWN_STRING_VALUES {
                return None;
            }
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[String] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

#[derive(Debug, Clone)]
struct KnownBool {
    values: Vec<bool>,
}

impl KnownBool {
    fn one(value: bool) -> Self {
        Self {
            values: vec![value],
        }
    }

    fn from_values(values: impl IntoIterator<Item = bool>) -> Option<Self> {
        let mut unique = Vec::new();
        for value in values {
            if unique.contains(&value) {
                continue;
            }
            unique.push(value);
        }
        if unique.is_empty() {
            None
        } else {
            Some(Self { values: unique })
        }
    }

    fn values(&self) -> &[bool] {
        &self.values
    }

    fn is_single(&self) -> bool {
        self.values.len() == 1
    }
}

impl LlvmGenerator {
    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        output.push_str("; generated by phpc milestone 1\n");
        output.push_str("declare i32 @printf(ptr, ...)\n");
        if self.uses_strcmp {
            output.push_str("declare i32 @strcmp(ptr, ptr)\n");
        }
        output.push('\n');
        output.push_str("@.fmt_int = private unnamed_addr constant [5 x i8] c\"%lld\\00\"\n");
        output.push_str("@.fmt_float = private unnamed_addr constant [3 x i8] c\"%g\\00\"\n");
        output.push_str("@.fmt_str = private unnamed_addr constant [3 x i8] c\"%s\\00\"\n");

        for (name, text) in &self.strings {
            output.push_str(&format!(
                "@{name} = private unnamed_addr constant [{} x i8] c\"{}\"\n",
                text.as_bytes().len() + 1,
                llvm_c_string(text)
            ));
        }

        output.push_str("\ndefine i32 @main() {\nentry:\n");
        for line in &self.body {
            output.push_str("  ");
            output.push_str(line);
            output.push('\n');
        }
        output.push_str("  ret i32 0\n}\n");
        Ok(output)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.emit_expr(expr)?;
                    self.emit_echo(value);
                }
                Ok(())
            }
            Stmt::Print { expr, .. } => {
                let value = self.emit_expr(expr)?;
                self.emit_echo(value);
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => self.emit_assignment(target, expr),
            Stmt::CompoundAssign { span, .. }
            | Stmt::IncrementDecrement { span, .. }
            | Stmt::NullCoalesceAssign { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => {
                Err(self.unsupported(function.span, LLVM_FUNCTION_DECLARATION_REJECTION))
            }
            Stmt::Class(class) => Err(self.unsupported(class.span, LLVM_OBJECT_CLASS_REJECTION)),
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Break { span }
            | Stmt::Continue { span } => Err(self.unsupported(*span, LLVM_CONTROL_FLOW_REJECTION)),
            Stmt::Foreach { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Stmt::UnsetVariable { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { span, .. } => Err(self.unsupported(*span, LLVM_MUTATION_REJECTION)),
            Stmt::ConstDeclaration { span, .. } => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Return { span, .. } => {
                Err(self.unsupported(*span, LLVM_FUNCTION_DECLARATION_REJECTION))
            }
            Stmt::Global { span, .. } => Err(self.unsupported(
                *span,
                "global declarations are not supported by LLVM IR emission yet",
            )),
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> CompileResult<IrValue> {
        match expr {
            Expr::Null(_) => Ok(IrValue::Null),
            Expr::Bool(value, _) => Ok(IrValue::Bool(*value)),
            Expr::Int(value, _) => Ok(IrValue::Int(value.to_string())),
            Expr::Float(value, _) => Ok(IrValue::Float(format_float_literal(*value))),
            Expr::String(value, _) => Ok(IrValue::String(value.clone())),
            Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span } => {
                Err(self.unsupported(*span, LLVM_MAGIC_CONSTANT_REJECTION))
            }
            Expr::GlobalConstant { span, .. } => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Array { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Expr::Index { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Expr::AppendIndex { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Expr::Property { span, .. } => {
                Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION))
            }
            Expr::MethodCall { span, .. } => {
                Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION))
            }
            Expr::Variable(name, span) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| self.unsupported(*span, LLVM_VARIABLE_READ_REJECTION)),
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("isset") => {
                self.emit_isset_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("empty") => {
                self.emit_empty_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("strlen") => {
                self.emit_strlen_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("function_exists") => {
                self.emit_function_exists_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_callable") => {
                self.emit_is_callable_call(args, *span)
            }
            Expr::Call { name, args, span } if is_native_type_introspection_builtin(name) => {
                self.emit_native_type_introspection_call(name, args, *span)
            }
            Expr::Call { name, span, .. } if is_object_metadata_builtin(name) => {
                Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION))
            }
            Expr::Call { name, span, .. } if is_array_builtin(name) => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::Call { span, .. } | Expr::DynamicCall { span, .. } => {
                Err(self.unsupported(*span, LLVM_FUNCTION_CALL_REJECTION))
            }
            Expr::New { span, .. } => Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION)),
            Expr::Unary { op, expr, span } => {
                if matches!(op, UnaryOp::Not) {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        let value = self.emit_expr(expr)?;
                        if matches!(value, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                            return Ok(value);
                        }
                        let inverted = self.emit_bool_not(value, *span)?;
                        return self.emit_bool_not(inverted, *span);
                    }
                }
                if matches!(op, UnaryOp::BitwiseNot) {
                    if let Expr::Unary {
                        op: UnaryOp::BitwiseNot,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        return match self.emit_expr(expr)? {
                            value @ IrValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, LLVM_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::NullCoalesceAssign { span, .. }
            | Expr::IncrementDecrement { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => self.emit_ternary_expr(condition, if_true, if_false, *span),
            Expr::ShortTernary {
                condition,
                if_false,
                span,
            } => self.emit_short_ternary(condition, if_false, *span),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) && !matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
                    return self.emit_scalar_comparison_expr(left, *op, right, *span);
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.unsupported(*span, LLVM_CONDITIONAL_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return self.emit_static_string_concat_expr(left, right, *span);
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return self.emit_logical_expr(left, *op, right, *span);
                }
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, LLVM_ISSET_REJECTION));
        };

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), LLVM_ISSET_REJECTION));
        };

        Ok(IrValue::Bool(!matches!(
            self.variables.get(name),
            None | Some(IrValue::Null)
        )))
    }

    fn emit_empty_call(&self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, LLVM_EMPTY_REJECTION));
        };

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), LLVM_EMPTY_REJECTION));
        };

        let Some(value) = self.variables.get(name) else {
            return Ok(IrValue::Bool(true));
        };

        self.known_truthiness_for_value(value)
            .map(|truthy| IrValue::Bool(!truthy))
            .ok_or_else(|| self.unsupported(arg.span(), LLVM_EMPTY_REJECTION))
    }

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.strlen_result_for_value(&value)
            .map(|length| IrValue::Int(length.to_string()))
            .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION))
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.function_exists_result_for_value(&value)
            .map(IrValue::Bool)
            .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION))
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                IrValue::Bool(value) => value,
                _ => return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
            }
        } else {
            false
        };

        self.is_callable_result_for_value(&value, syntax_only)
            .map(IrValue::Bool)
            .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION))
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<IrValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_GLOBAL_CONSTANT_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.defined_result_for_value(&value)
            .map(IrValue::Bool)
            .ok_or_else(|| self.unsupported(span, LLVM_GLOBAL_CONSTANT_REJECTION))
    }

    fn emit_native_type_introspection_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if is_native_metadata_exists_builtin(name) {
            return self.emit_native_metadata_exists_call(args, span);
        }
        if is_native_member_metadata_exists_builtin(name) {
            return self.emit_native_member_metadata_exists_call(args, span);
        }
        if is_native_relationship_metadata_builtin(name) {
            return self.emit_native_relationship_metadata_call(args, span);
        }

        if args.len() != 1 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        match name.to_ascii_lowercase().as_str() {
            "gettype" => Ok(IrValue::String(llvm_gettype_name(&value).to_string())),
            "get_debug_type" => Ok(IrValue::String(llvm_debug_type_name(&value).to_string())),
            "is_null" => Ok(IrValue::Bool(matches!(value, IrValue::Null))),
            "is_bool" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::Bool(_) | IrValue::BoolExpr(_)
            ))),
            "is_int" | "is_integer" | "is_long" => {
                Ok(IrValue::Bool(matches!(value, IrValue::Int(_))))
            }
            "is_float" | "is_double" => Ok(IrValue::Bool(matches!(value, IrValue::Float(_)))),
            "is_string" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::String(_) | IrValue::StringPtr(_)
            ))),
            "is_array" => Ok(IrValue::Bool(false)),
            "is_scalar" => Ok(IrValue::Bool(matches!(
                value,
                IrValue::Bool(_)
                    | IrValue::BoolExpr(_)
                    | IrValue::Int(_)
                    | IrValue::Float(_)
                    | IrValue::String(_)
                    | IrValue::StringPtr(_)
            ))),
            "is_numeric" => self
                .is_numeric_result_for_value(&value)
                .map(IrValue::Bool)
                .ok_or_else(|| self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
            "is_countable" | "is_iterable" => Ok(IrValue::Bool(false)),
            "is_object" => Ok(IrValue::Bool(false)),
            _ => Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION)),
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, IrValue::String(_) | IrValue::StringPtr(_)) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
            }
        }

        Ok(IrValue::Bool(false))
    }

    fn emit_native_member_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if args.len() != 2 {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(member, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        Ok(IrValue::Bool(false))
    }

    fn emit_native_relationship_metadata_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<IrValue> {
        if !(2..=3).contains(&args.len()) {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, IrValue::String(_) | IrValue::StringPtr(_))
            || !matches!(class_name, IrValue::String(_) | IrValue::StringPtr(_))
        {
            return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                return Err(self.unsupported(span, LLVM_FUNCTION_CALL_REJECTION));
            }
        }

        Ok(IrValue::Bool(false))
    }

    fn is_numeric_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Int(_) | IrValue::Float(_) => Some(true),
            IrValue::Null | IrValue::Bool(_) | IrValue::BoolExpr(_) => Some(false),
            IrValue::String(value) => Some(is_php_numeric_string_literal(value)),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_numeric_result(&values)
            }
        }
    }

    fn function_exists_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::String(value) => Some(is_native_known_function_name(value)),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_function_exists_result(&values)
            }
            _ => None,
        }
    }

    fn strlen_result_for_value(&self, value: &IrValue) -> Option<usize> {
        match value {
            IrValue::String(value) => Some(value.len()),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_byte_length(&values)
            }
            _ => None,
        }
    }

    fn is_callable_result_for_value(&self, value: &IrValue, syntax_only: bool) -> Option<bool> {
        match value {
            IrValue::String(_) | IrValue::StringPtr(_) if syntax_only => Some(true),
            IrValue::Null
            | IrValue::Bool(_)
            | IrValue::BoolExpr(_)
            | IrValue::Int(_)
            | IrValue::Float(_) => Some(false),
            _ => self.function_exists_result_for_value(value),
        }
    }

    fn defined_result_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::String(value) => native_defined_result(value),
            IrValue::StringPtr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_defined_result(&values)
            }
            _ => None,
        }
    }

    fn emit_assignment(&mut self, target: &AssignTarget, expr: &Expr) -> CompileResult<()> {
        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.emit_expr(expr)?;
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            AssignTarget::ArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            AssignTarget::Property { span, .. } => {
                Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION))
            }
        }
    }

    fn emit_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Mod => {
                self.emit_arithmetic_binary(left, op, right, span)
            }
            BinaryOp::Div => Err(self.unsupported(span, LLVM_DIVISION_REJECTION)),
            BinaryOp::Concat => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => self.emit_scalar_comparison(left, op, right, span),
            BinaryOp::StrictEq | BinaryOp::StrictNe => {
                self.emit_static_strict_identity(left, op, right, span)
            }
            BinaryOp::NullCoalesce => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => {
                self.emit_bool_binary(left, op, right, span)
            }
            BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor => {
                self.emit_integer_bitwise_binary(left, op, right, span)
            }
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                self.emit_integer_shift_binary(left, op, right, span)
            }
        }
    }

    fn emit_arithmetic_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match (left, right) {
            (IrValue::Int(left), IrValue::Int(right)) => match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                    if matches!(op, BinaryOp::Add) {
                        if right == "0" {
                            return Ok(IrValue::Int(left));
                        }
                        if left == "0" {
                            return Ok(IrValue::Int(right));
                        }
                    }
                    if matches!(op, BinaryOp::Sub) && right == "0" {
                        return Ok(IrValue::Int(left));
                    }
                    if matches!(op, BinaryOp::Sub) && left == right {
                        return Ok(IrValue::Int("0".to_string()));
                    }
                    if matches!(op, BinaryOp::Mul) {
                        if right == "0" || left == "0" {
                            return Ok(IrValue::Int("0".to_string()));
                        }
                        if right == "1" {
                            return Ok(IrValue::Int(left));
                        }
                        if left == "1" {
                            return Ok(IrValue::Int(right));
                        }
                    }
                    let left_is_tracked = self.is_tracked_integer_value(&left);
                    let right_is_tracked = self.is_tracked_integer_value(&right);
                    let Some(result) = self.checked_static_integer_arithmetic(&left, op, &right)
                    else {
                        return Err(
                            self.unsupported(span, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION)
                        );
                    };
                    if (left_is_tracked || right_is_tracked) && result.is_single() {
                        return Ok(IrValue::Int(result.values()[0].to_string()));
                    }
                    let instruction = match op {
                        BinaryOp::Add => "add",
                        BinaryOp::Sub => "sub",
                        BinaryOp::Mul => "mul",
                        _ => unreachable!("operator matched above"),
                    };
                    let temp = self.next_temp();
                    self.body
                        .push(format!("{temp} = {instruction} i64 {left}, {right}"));
                    self.known_ints.insert(temp.clone(), result);
                    return Ok(IrValue::Int(temp));
                }
                BinaryOp::Mod => {
                    let Ok(divisor) = right.parse::<i64>() else {
                        return Err(self.unsupported(span, LLVM_MODULO_RUNTIME_CHECK_REJECTION));
                    };
                    if divisor <= 0 {
                        return Err(self.unsupported(span, LLVM_MODULO_RUNTIME_CHECK_REJECTION));
                    }
                    if divisor == 1 {
                        return Ok(IrValue::Int("0".to_string()));
                    }
                    let modulo_result = self.static_integer_modulo(&left, divisor);
                    if let (Some(left_values), Some(result)) =
                        (self.known_integer_values(&left), modulo_result.as_ref())
                    {
                        if !left_values.is_single() && result.is_single() {
                            return Ok(IrValue::Int(result.values()[0].to_string()));
                        }
                    }
                    let temp = self.next_temp();
                    self.body.push(format!("{temp} = srem i64 {left}, {right}"));
                    if let Some(result) = modulo_result {
                        self.known_ints.insert(temp.clone(), result);
                    }
                    return Ok(IrValue::Int(temp));
                }
                _ => return Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
            },
            (IrValue::Float(left), IrValue::Float(right)) => {
                if matches!(op, BinaryOp::Add) {
                    if right == "0.0" && self.known_finite_nonzero_float_values(&left) {
                        return Ok(IrValue::Float(left));
                    }
                    if left == "0.0" && self.known_finite_nonzero_float_values(&right) {
                        return Ok(IrValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && right == "0.0"
                    && self.known_finite_nonzero_float_values(&left)
                {
                    return Ok(IrValue::Float(left));
                }
                if matches!(op, BinaryOp::Sub) && left == "0.0" {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                if matches!(op, BinaryOp::Mul) {
                    if (right == "0.0" && self.known_finite_positive_float_values(&left))
                        || (left == "0.0" && self.known_finite_positive_float_values(&right))
                    {
                        return Ok(IrValue::Float("0.0".to_string()));
                    }
                    if right == "-1.0" {
                        if let Some(result) = self.static_float_negate(&left) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(IrValue::Float(format_float_literal(
                                    result.values()[0],
                                )));
                            }
                        }
                    }
                    if left == "-1.0" {
                        if let Some(result) = self.static_float_negate(&right) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(IrValue::Float(format_float_literal(
                                    result.values()[0],
                                )));
                            }
                        }
                    }
                    if right == "1.0" && self.known_float_values(&left).is_some() {
                        return Ok(IrValue::Float(left));
                    }
                    if left == "1.0" && self.known_float_values(&right).is_some() {
                        return Ok(IrValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && left == right
                    && self
                        .known_float_values(&left)
                        .is_some_and(|values| values.values().iter().all(|value| value.is_finite()))
                {
                    return Ok(IrValue::Float("0.0".to_string()));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul)
                    && (left_is_tracked || right_is_tracked)
                {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                let instruction = match op {
                    BinaryOp::Add => "fadd",
                    BinaryOp::Sub => "fsub",
                    BinaryOp::Mul => "fmul",
                    _ => return Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = {instruction} double {left}, {right}"));
                if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            (IrValue::Int(_), IrValue::Float(_)) | (IrValue::Float(_), IrValue::Int(_))
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) =>
            {
                Err(self.unsupported(span, LLVM_MIXED_NUMERIC_ARITHMETIC_REJECTION))
            }
            (
                IrValue::Null
                | IrValue::Bool(_)
                | IrValue::BoolExpr(_)
                | IrValue::String(_)
                | IrValue::StringPtr(_),
                _,
            )
            | (
                _,
                IrValue::Null
                | IrValue::Bool(_)
                | IrValue::BoolExpr(_)
                | IrValue::String(_)
                | IrValue::StringPtr(_),
            ) if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) => {
                Err(self.unsupported(span, LLVM_SCALAR_COERCION_ARITHMETIC_REJECTION))
            }
            _ => Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
        }
    }

    fn emit_scalar_comparison_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let left = self
            .emit_expr(left)
            .map_err(|_| self.unsupported(span, llvm_comparison_rejection()))?;
        let right = self
            .emit_expr(right)
            .map_err(|_| self.unsupported(span, llvm_comparison_rejection()))?;
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match (left, right) {
            (IrValue::Null, IrValue::Null) => {
                let Some(result) = null_comparison_result(op) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::Bool(left), IrValue::Bool(right)) => {
                let Some(result) = bool_comparison_result(left, op, right) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::BoolExpr(left), IrValue::Bool(right)) => {
                let right = if right { "true" } else { "false" };
                self.emit_bool_scalar_comparison(left, op, right.to_string(), span)
            }
            (IrValue::Bool(left), IrValue::BoolExpr(right)) => {
                let left = if left { "true" } else { "false" };
                self.emit_bool_scalar_comparison(left.to_string(), op, right, span)
            }
            (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) => {
                self.emit_bool_scalar_comparison(left, op, right, span)
            }
            (IrValue::String(left), IrValue::String(right)) => {
                let Some(result) = static_safe_string_comparison_result(
                    Some(KnownString::one(left)),
                    op,
                    Some(KnownString::one(right)),
                ) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                Ok(IrValue::Bool(result))
            }
            (IrValue::StringPtr(left), IrValue::StringPtr(right)) => {
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::StringPtr(left), IrValue::String(right)) => {
                let right = self.string_pointer_operand(IrValue::String(right));
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::String(left), IrValue::StringPtr(right)) => {
                let left = self.string_pointer_operand(IrValue::String(left));
                self.emit_string_comparison(left, op, right, span)
            }
            (IrValue::Int(left), IrValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    let Some(result) = integer_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                if left == right {
                    let Some(result) = integer_comparison_result(0, op, 0) else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_integer_value(&left);
                let right_is_tracked = self.is_tracked_integer_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<i64>().is_ok() || right.parse::<i64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_integer_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) =
                            self.static_integer_comparison_result(&left, op, &right)
                        {
                            return Ok(IrValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::Eq => "eq",
                    BinaryOp::Ne => "ne",
                    BinaryOp::Lt => "slt",
                    BinaryOp::Le => "sle",
                    BinaryOp::Gt => "sgt",
                    BinaryOp::Ge => "sge",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = icmp {predicate} i64 {left}, {right}"));
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                Ok(IrValue::BoolExpr(temp))
            }
            (IrValue::Float(left), IrValue::Float(right)) => {
                let Some(left_values) = self.known_float_values(&left) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                let Some(right_values) = self.known_float_values(&right) else {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                };
                if !left_values.values().iter().all(|value| value.is_finite())
                    || !right_values.values().iter().all(|value| value.is_finite())
                {
                    return Err(self.unsupported(span, llvm_comparison_rejection()));
                }
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    let Some(result) = float_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, llvm_comparison_rejection()));
                    };
                    return Ok(IrValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<f64>().is_ok() || right.parse::<f64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_float_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) = self.static_float_comparison_result(&left, op, &right)
                        {
                            return Ok(IrValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::Eq => "oeq",
                    BinaryOp::Ne => "une",
                    BinaryOp::Lt => "olt",
                    BinaryOp::Le => "ole",
                    BinaryOp::Gt => "ogt",
                    BinaryOp::Ge => "oge",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = fcmp {predicate} double {left}, {right}"));
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                Ok(IrValue::BoolExpr(temp))
            }
            _ => Err(self.unsupported(span, llvm_comparison_rejection())),
        }
    }

    fn emit_bool_scalar_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        if let Some(fold) = bool_literal_comparison_fold(&left, op, &right, "true", "false") {
            return match fold {
                BoolLiteralComparisonFold::Static(value) => Ok(IrValue::Bool(value)),
                BoolLiteralComparisonFold::Reuse(value) => Ok(IrValue::BoolExpr(value)),
                BoolLiteralComparisonFold::Invert(value) => {
                    self.emit_bool_not(IrValue::BoolExpr(value), span)
                }
            };
        }
        if left == right {
            let Some(result) = bool_comparison_result(false, op, false) else {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            };
            return Ok(IrValue::Bool(result));
        }
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            return Ok(IrValue::Bool(result));
        }
        let predicate = match op {
            BinaryOp::Eq => "eq",
            BinaryOp::Ne => "ne",
            BinaryOp::Lt => "ult",
            BinaryOp::Le => "ule",
            BinaryOp::Gt => "ugt",
            BinaryOp::Ge => "uge",
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = icmp {predicate} i1 {left}, {right}"));
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            self.known_bools
                .insert(temp.clone(), KnownBool::one(result));
        }
        Ok(IrValue::BoolExpr(temp))
    }

    fn checked_static_integer_arithmetic(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left.checked_add(*right),
                    BinaryOp::Sub => left.checked_sub(*right),
                    BinaryOp::Mul => left.checked_mul(*right),
                    _ => None,
                }?;
                results.push(result);
            }
        }
        KnownInt::from_values(results)
    }

    fn known_integer_values(&self, value: &str) -> Option<KnownInt> {
        value
            .parse::<i64>()
            .ok()
            .map(KnownInt::one)
            .or_else(|| self.known_ints.get(value).cloned())
    }

    fn is_tracked_integer_value(&self, value: &str) -> bool {
        self.known_ints.contains_key(value)
    }

    fn static_integer_modulo(&self, left: &str, divisor: i64) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let values = left.values().iter().map(|value| value % divisor);
        KnownInt::from_values(values)
    }

    fn static_integer_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = integer_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_float_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        if !left_values.values().iter().all(|value| value.is_finite())
            || !right_values.values().iter().all(|value| value.is_finite())
        {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = float_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_bool_comparison_result(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = bool_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_integer_shift_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let (IrValue::Int(left), IrValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        let Some(count) = self.static_integer_shift_count(&right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if count == 0 {
            return Ok(IrValue::Int(left));
        }
        if self.is_tracked_integer_value(&left) {
            if let Some(result) = self.static_integer_shift(&left, op, count) {
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let instruction = match op {
            BinaryOp::ShiftLeft => "shl",
            BinaryOp::ShiftRight => "ashr",
            _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = {instruction} i64 {left}, {count}"));
        if let Some(result) = self.static_integer_shift(&left, op, count) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn static_integer_shift(&self, left: &str, op: BinaryOp, count: u32) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let factor = if matches!(op, BinaryOp::ShiftLeft) {
            Some(1_i64.checked_shl(count)?)
        } else {
            None
        };
        let values = left.values().iter().map(|value| match op {
            BinaryOp::ShiftLeft => value.checked_mul(factor.expect("left shift has a factor")),
            BinaryOp::ShiftRight => Some(value >> count),
            _ => None,
        });
        let mut results = Vec::new();
        for value in values {
            results.push(value?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_shift_count(&self, right: &str) -> Option<u32> {
        if let Ok(count) = right.parse::<u32>() {
            return (count < 64).then_some(count);
        }
        let values = self.known_integer_values(right)?;
        if !values.is_single() {
            return None;
        }
        let count = u32::try_from(values.values()[0]).ok()?;
        (count < 64).then_some(count)
    }

    fn static_integer_bitwise(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::BitwiseAnd => left & right,
                    BinaryOp::BitwiseOr => left | right,
                    BinaryOp::BitwiseXor => left ^ right,
                    _ => return None,
                });
            }
        }
        KnownInt::from_values(results)
    }

    fn static_integer_negate(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            results.push(value.checked_neg()?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_bitwise_not(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        KnownInt::from_values(value.values().iter().map(|value| !value))
    }

    fn emit_integer_bitwise_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let (IrValue::Int(left), IrValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if left == right {
            return Ok(match op {
                BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr => IrValue::Int(left),
                BinaryOp::BitwiseXor => IrValue::Int("0".to_string()),
                _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
            });
        }
        if matches!(op, BinaryOp::BitwiseAnd) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(IrValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(IrValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr | BinaryOp::BitwiseXor) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(IrValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr)
            && (self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                || self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1)))
        {
            return Ok(IrValue::Int("-1".to_string()));
        }
        if matches!(op, BinaryOp::BitwiseXor)
            && ((self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                && self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single()))
                || (self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1))
                    && self
                        .known_integer_values(&left)
                        .is_some_and(|values| values.is_single())))
        {
            let result = self
                .static_integer_bitwise(&left, op, &right)
                .expect("single known integer XOR all-ones result is known");
            return Ok(IrValue::Int(result.values()[0].to_string()));
        }
        let left_is_tracked = self.is_tracked_integer_value(&left);
        let right_is_tracked = self.is_tracked_integer_value(&right);
        if left_is_tracked || right_is_tracked {
            if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let instruction = match op {
            BinaryOp::BitwiseAnd => "and",
            BinaryOp::BitwiseOr => "or",
            BinaryOp::BitwiseXor => "xor",
            _ => return Err(self.unsupported(span, LLVM_BITWISE_REJECTION)),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = {instruction} i64 {left}, {right}"));
        if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn emit_static_string_concat_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        if is_empty_string_literal(left) {
            let right = self.emit_expr(right)?;
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = self.emit_expr(left)?;
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = self.emit_static_string_concat_operand(left, span)?;
        let right = self.emit_static_string_concat_operand(right, span)?;
        Ok(IrValue::String(format!("{left}{right}")))
    }

    fn emit_empty_string_concat_identity(
        &self,
        value: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match value {
            IrValue::String(_) | IrValue::StringPtr(_) => Ok(value),
            _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
        }
    }

    fn emit_static_string_concat_operand(
        &mut self,
        expr: &Expr,
        span: Span,
    ) -> CompileResult<String> {
        match expr {
            Expr::String(value, _) => Ok(value.clone()),
            Expr::Variable(name, variable_span) => match self.variables.get(name).cloned() {
                Some(IrValue::String(value)) => Ok(value),
                Some(_) => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
                None => Err(self.unsupported(*variable_span, LLVM_VARIABLE_READ_REJECTION)),
            },
            Expr::Binary {
                left,
                op: BinaryOp::Concat,
                right,
                span: concat_span,
            } => match self.emit_static_string_concat_expr(left, right, *concat_span)? {
                IrValue::String(value) => Ok(value),
                _ => unreachable!("static string concatenation returns a string"),
            },
            Expr::Ternary { .. } => match self.emit_expr(expr)? {
                IrValue::String(value) => Ok(value),
                IrValue::StringPtr(value) => {
                    let values = self
                        .known_string_values(&value)
                        .ok_or_else(|| self.unsupported(span, LLVM_CONCAT_REJECTION))?;
                    if values.is_single() {
                        Ok(values.values()[0].clone())
                    } else {
                        Err(self.unsupported(span, LLVM_CONCAT_REJECTION))
                    }
                }
                _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
            },
            _ => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
        }
    }

    fn emit_static_strict_identity(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let is_identical = match (left, right) {
            (IrValue::Null, IrValue::Null) => true,
            (IrValue::Bool(left), IrValue::Bool(right)) => left == right,
            (IrValue::BoolExpr(left), IrValue::Bool(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    Some(KnownBool::one(right)),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(IrValue::BoolExpr(left));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(IrValue::BoolExpr(left), span);
                }
                let right = if right { "true" } else { "false" };
                return self.emit_bool_comparison(left, op, right.to_string(), span);
            }
            (IrValue::Bool(left), IrValue::BoolExpr(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    Some(KnownBool::one(left)),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(IrValue::BoolExpr(right));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(IrValue::BoolExpr(right), span);
                }
                let left = if left { "true" } else { "false" };
                return self.emit_bool_comparison(left.to_string(), op, right, span);
            }
            (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) => {
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                return self.emit_bool_comparison(left, op, right, span);
            }
            (IrValue::String(left), IrValue::String(right)) => left == right,
            (IrValue::StringPtr(left), IrValue::StringPtr(right)) => {
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::StringPtr(left), IrValue::String(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    Some(KnownString::one(right.clone())),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                let right = self.string_pointer_operand(IrValue::String(right));
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::String(left), IrValue::StringPtr(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    Some(KnownString::one(left.clone())),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(IrValue::Bool(result));
                }
                let left = self.string_pointer_operand(IrValue::String(left));
                return self.emit_string_comparison(left, op, right, span);
            }
            (IrValue::Float(left), IrValue::Float(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(IrValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                    }));
                }
                if left == right {
                    if let Some(result) = self.static_same_float_strict_identity(&left, op) {
                        return Ok(IrValue::Bool(result));
                    }
                }
                if let Some(result) = self.static_float_strict_identity(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::StrictEq => "oeq",
                    BinaryOp::StrictNe => "une",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = fcmp {predicate} double {left}, {right}"));
                return Ok(IrValue::BoolExpr(temp));
            }
            (IrValue::Int(left), IrValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    return Ok(IrValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                    }));
                }
                if left == right {
                    return Ok(IrValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_integer_strict_identity(&left, op, &right) {
                    return Ok(IrValue::Bool(result));
                }
                let predicate = match op {
                    BinaryOp::StrictEq => "eq",
                    BinaryOp::StrictNe => "ne",
                    _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = icmp {predicate} i64 {left}, {right}"));
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(temp.clone(), KnownBool::one(result));
                }
                return Ok(IrValue::BoolExpr(temp));
            }
            _ => false,
        };
        let result = match op {
            BinaryOp::StrictEq => is_identical,
            BinaryOp::StrictNe => !is_identical,
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        Ok(IrValue::Bool(result))
    }

    fn static_integer_strict_identity(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        if left_values.is_single() && right_values.is_single() {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_integer_strict_identity_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_string_values(&self, value: &str) -> Option<KnownString> {
        self.known_strings.get(value).cloned()
    }

    fn known_bool_values(&self, value: &str) -> Option<KnownBool> {
        match value {
            "true" => Some(KnownBool::one(true)),
            "false" => Some(KnownBool::one(false)),
            _ => self.known_bools.get(value).cloned(),
        }
    }

    fn static_bool_strict_identity(
        &self,
        left_values: Option<KnownBool>,
        right_values: Option<KnownBool>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_float_values(&self, value: &str) -> Option<KnownFloat> {
        value
            .parse::<f64>()
            .ok()
            .map(KnownFloat::one)
            .or_else(|| self.known_floats.get(value).cloned())
    }

    fn is_tracked_float_value(&self, value: &str) -> bool {
        self.known_floats.contains_key(value)
    }

    fn known_finite_nonzero_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value != 0.0)
        })
    }

    fn known_finite_positive_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
        })
    }

    fn static_float_strict_identity(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_same_float_strict_identity(&self, value: &str, op: BinaryOp) -> Option<bool> {
        let values = self.known_float_values(value)?;
        if !values.values().iter().all(|value| value.is_finite()) {
            return None;
        }
        Some(static_strict_identity_result(true, op))
    }

    fn static_float_arithmetic(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownFloat> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    _ => return None,
                };
                if !result.is_finite() {
                    return None;
                }
                results.push(result);
            }
        }
        KnownFloat::from_values(results)
    }

    fn known_string_values_for_value(&self, value: &IrValue) -> Option<KnownString> {
        match value {
            IrValue::String(value) => Some(KnownString::one(value.clone())),
            IrValue::StringPtr(value) => self.known_string_values(value),
            _ => None,
        }
    }

    fn static_string_strict_identity(
        &self,
        left_values: Option<KnownString>,
        right_values: Option<KnownString>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_bool_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        if let (Some(left), Some(right)) = (
            self.known_truthiness_for_value(&left),
            self.known_truthiness_for_value(&right),
        ) {
            return Ok(IrValue::Bool(logical_truthiness_result(left, op, right)?));
        }
        match (left, right) {
            (IrValue::Bool(left), IrValue::Bool(right)) => Ok(IrValue::Bool(match op {
                BinaryOp::LogicalAnd => left && right,
                BinaryOp::LogicalOr => left || right,
                BinaryOp::LogicalXor => left ^ right,
                _ => return Err(self.unsupported(span, llvm_logical_rejection())),
            })),
            (IrValue::Bool(left), right) => match op {
                BinaryOp::LogicalAnd if left => self.require_bool_value(right, span),
                BinaryOp::LogicalAnd => Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if left => Ok(IrValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(right, span),
                BinaryOp::LogicalXor if left => {
                    let right = self.require_bool_value(right, span)?;
                    self.emit_bool_not(right, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(right, span),
                _ => Err(self.unsupported(span, llvm_logical_rejection())),
            },
            (left, IrValue::Bool(right)) => match op {
                BinaryOp::LogicalAnd if right => self.require_bool_value(left, span),
                BinaryOp::LogicalAnd => Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if right => Ok(IrValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(left, span),
                BinaryOp::LogicalXor if right => {
                    let left = self.require_bool_value(left, span)?;
                    self.emit_bool_not(left, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(left, span),
                _ => Err(self.unsupported(span, llvm_logical_rejection())),
            },
            (left, right) => {
                let Some(left) = llvm_bool_operand(left) else {
                    return Err(self.unsupported(span, llvm_logical_rejection()));
                };
                let Some(right) = llvm_bool_operand(right) else {
                    return Err(self.unsupported(span, llvm_logical_rejection()));
                };
                if left == right && matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                    return Ok(IrValue::BoolExpr(left));
                }
                if left == right && matches!(op, BinaryOp::LogicalXor) {
                    return Ok(IrValue::Bool(false));
                }
                let result = self.static_bool_binary(&left, op, &right);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let instruction = match op {
                    BinaryOp::LogicalAnd => "and",
                    BinaryOp::LogicalOr => "or",
                    BinaryOp::LogicalXor => "xor",
                    _ => return Err(self.unsupported(span, llvm_logical_rejection())),
                };
                let temp = self.next_temp();
                self.body
                    .push(format!("{temp} = {instruction} i1 {left}, {right}"));
                if let Some(result) = result {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
        }
    }

    fn emit_logical_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let left = self.emit_expr(left)?;
        if let Some(left_truthy) = self.known_truthiness_for_value(&left) {
            match op {
                BinaryOp::LogicalAnd if !left_truthy => return Ok(IrValue::Bool(false)),
                BinaryOp::LogicalOr if left_truthy => return Ok(IrValue::Bool(true)),
                _ => {}
            }
        }
        let right = self.emit_expr(right)?;
        self.emit_bool_binary(left, op, right, span)
    }

    fn require_bool_value(&self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match value {
            IrValue::Bool(_) | IrValue::BoolExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, llvm_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &IrValue) -> Option<bool> {
        match value {
            IrValue::Bool(value) => Some(*value),
            IrValue::BoolExpr(_) => None,
            IrValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            IrValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            IrValue::String(value) => Some(php_string_truthy(value)),
            IrValue::StringPtr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
            IrValue::Null => Some(false),
        }
    }

    fn emit_bool_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        let predicate = match op {
            BinaryOp::StrictEq => "eq",
            BinaryOp::StrictNe => "ne",
            _ => return Err(self.unsupported(span, llvm_comparison_rejection())),
        };
        let temp = self.next_temp();
        self.body
            .push(format!("{temp} = icmp {predicate} i1 {left}, {right}"));
        Ok(IrValue::BoolExpr(temp))
    }

    fn static_bool_binary(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownBool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::LogicalAnd => *left && *right,
                    BinaryOp::LogicalOr => *left || *right,
                    BinaryOp::LogicalXor => *left ^ *right,
                    _ => return None,
                });
            }
        }
        KnownBool::from_values(results)
    }

    fn emit_string_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<IrValue> {
        let predicate = llvm_string_comparison_predicate(op)
            .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
        if left == right {
            let Some(result) = reflexive_string_comparison_result(op) else {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            };
            return Ok(IrValue::Bool(result));
        }
        let known_result = if matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
            self.static_string_strict_identity(
                self.known_string_values(&left),
                self.known_string_values(&right),
                op,
            )
        } else {
            let left_values = self
                .known_string_values(&left)
                .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
            let right_values = self
                .known_string_values(&right)
                .ok_or_else(|| self.unsupported(span, llvm_comparison_rejection()))?;
            if !known_strings_are_safe_for_native_comparison(&left_values)
                || !known_strings_are_safe_for_native_comparison(&right_values)
            {
                return Err(self.unsupported(span, llvm_comparison_rejection()));
            }
            string_comparison_result_for_known_values(&left_values, op, &right_values)
        };
        if let Some(known_result) = known_result {
            return Ok(IrValue::Bool(known_result));
        }
        self.uses_strcmp = true;
        let comparison = self.next_temp();
        self.body.push(format!(
            "{comparison} = call i32 @strcmp(ptr {left}, ptr {right})"
        ));
        let result = self.next_temp();
        self.body
            .push(format!("{result} = icmp {predicate} i32 {comparison}, 0"));
        if let Some(known_result) = known_result {
            self.known_bools
                .insert(result.clone(), KnownBool::one(known_result));
        }
        Ok(IrValue::BoolExpr(result))
    }

    fn emit_ternary(
        &mut self,
        condition: IrValue,
        if_true: IrValue,
        if_false: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match condition {
            IrValue::Bool(true) => return Ok(if_true),
            IrValue::Bool(false) => return Ok(if_false),
            IrValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    return Ok(if_true);
                }
                if values.is_single_value(0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
            }
            IrValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    return Ok(if_true);
                }
                if matches!(values.values(), [value] if *value == 0.0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
            }
            IrValue::String(value) => {
                if php_string_truthy(&value) {
                    Ok(if_true)
                } else {
                    Ok(if_false)
                }
            }
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(if_true),
                    Some(false) => Ok(if_false),
                    None => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
                }
            }
            IrValue::Null => Ok(if_false),
            condition => self.emit_dynamic_ternary(condition, if_true, if_false, span),
        }
    }

    fn emit_ternary_expr(
        &mut self,
        condition: &Expr,
        if_true: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_ternary_expr(condition, if_true, if_false) {
            return Ok(condition_value);
        }
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.emit_expr(if_true)
            } else {
                self.emit_expr(if_false)
            };
        }
        if !matches!(condition_value, IrValue::BoolExpr(_)) {
            return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
        }
        let if_true = self.emit_expr(if_true)?;
        let if_false = self.emit_expr(if_false)?;
        self.emit_ternary(condition_value, if_true, if_false, span)
    }

    fn emit_short_ternary(
        &mut self,
        condition: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<IrValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_expr(condition, if_false) {
            if matches!(
                condition_value,
                IrValue::BoolExpr(_) | IrValue::Int(_) | IrValue::Float(_) | IrValue::StringPtr(_)
            ) {
                return Ok(condition_value);
            }
        }
        match condition_value {
            IrValue::Bool(true) => Ok(IrValue::Bool(true)),
            IrValue::Bool(false) => {
                let if_false = self.emit_expr(if_false)?;
                Ok(if_false)
            }
            IrValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    Ok(IrValue::Int(value))
                } else if values.is_single_value(0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
                }
            }
            IrValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    Ok(IrValue::Float(value))
                } else if matches!(values.values(), [value] if *value == 0.0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION))
                }
            }
            IrValue::String(value) => {
                if php_string_truthy(&value) {
                    Ok(IrValue::String(value))
                } else {
                    self.emit_expr(if_false)
                }
            }
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(IrValue::StringPtr(value)),
                    Some(false) => self.emit_expr(if_false),
                    None => Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION)),
                }
            }
            IrValue::Null => self.emit_expr(if_false),
            condition @ IrValue::BoolExpr(_) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(if_false, IrValue::Bool(_) | IrValue::BoolExpr(_)) {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                }
                self.emit_ternary(condition, IrValue::Bool(true), if_false, span)
            }
        }
    }

    fn emit_dynamic_ternary(
        &mut self,
        condition: IrValue,
        if_true: IrValue,
        if_false: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        let Some(condition) = llvm_bool_operand(condition) else {
            return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
        };
        match (if_true, if_false) {
            (IrValue::Null, IrValue::Null) => Ok(IrValue::Null),
            (IrValue::Int(if_true), IrValue::Int(if_false)) => {
                if if_true == if_false {
                    return Ok(IrValue::Int(if_true));
                }
                let result = self.static_integer_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Int(result.values()[0].to_string()));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, i64 {if_true}, i64 {if_false}"
                ));
                if let Some(result) = result {
                    self.known_ints.insert(temp.clone(), result);
                }
                Ok(IrValue::Int(temp))
            }
            (IrValue::Float(if_true), IrValue::Float(if_false)) => {
                if if_true == if_false {
                    return Ok(IrValue::Float(if_true));
                }
                let result = self.static_float_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, double {if_true}, double {if_false}"
                ));
                if let Some(result) = result {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            (if_true, if_false) => {
                if matches!(
                    (&if_true, &if_false),
                    (
                        IrValue::String(_) | IrValue::StringPtr(_),
                        IrValue::String(_) | IrValue::StringPtr(_)
                    )
                ) {
                    if let Some(result) = identical_string_ternary_branch(&if_true, &if_false) {
                        return Ok(result);
                    }
                    let result = self.static_string_ternary(&if_true, &if_false);
                    if let Some(result) = result.as_ref() {
                        if result.is_single() {
                            return Ok(IrValue::String(result.values()[0].clone()));
                        }
                    }
                    let if_true = self.string_pointer_operand(if_true);
                    let if_false = self.string_pointer_operand(if_false);
                    let temp = self.next_temp();
                    self.body.push(format!(
                        "{temp} = select i1 {condition}, ptr {if_true}, ptr {if_false}"
                    ));
                    if let Some(result) = result {
                        self.known_strings.insert(temp.clone(), result);
                    }
                    return Ok(IrValue::StringPtr(temp));
                }
                if let Some(result) = identical_bool_expr_ternary_branch(&if_true, &if_false) {
                    return Ok(result);
                }
                if let Some(result) = bool_literal_ternary_branch(&condition, &if_true, &if_false) {
                    return match result {
                        BoolLiteralTernaryBranch::Static(value) => Ok(IrValue::Bool(value)),
                        BoolLiteralTernaryBranch::Reuse(value) => Ok(IrValue::BoolExpr(value)),
                        BoolLiteralTernaryBranch::Invert(value) => {
                            self.emit_bool_not(IrValue::BoolExpr(value), span)
                        }
                    };
                }
                let Some(if_true) = llvm_bool_operand(if_true) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                let Some(if_false) = llvm_bool_operand(if_false) else {
                    return Err(self.unsupported(span, LLVM_CONDITIONAL_REJECTION));
                };
                let result = self.static_bool_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {condition}, i1 {if_true}, i1 {if_false}"
                ));
                if let Some(result) = result {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
        }
    }

    fn static_integer_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownInt> {
        let if_true = self.known_integer_values(if_true)?;
        let if_false = self.known_integer_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownInt::from_values(values)
    }

    fn static_float_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownFloat> {
        let if_true = self.known_float_values(if_true)?;
        let if_false = self.known_float_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownFloat::from_values(values)
    }

    fn static_string_ternary(&self, if_true: &IrValue, if_false: &IrValue) -> Option<KnownString> {
        let if_true = self.known_string_values_for_value(if_true)?;
        let if_false = self.known_string_values_for_value(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values().iter().cloned());
        KnownString::from_values(values)
    }

    fn static_bool_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownBool> {
        let if_true = self.known_bool_values(if_true)?;
        let if_false = self.known_bool_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownBool::from_values(values)
    }

    fn emit_unary(&mut self, op: UnaryOp, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match op {
            UnaryOp::Negate => self.emit_numeric_negate(value, span),
            UnaryOp::Not => self.emit_bool_not(value, span),
            UnaryOp::BitwiseNot => self.emit_integer_bitwise_not(value, span),
        }
    }

    fn emit_numeric_negate(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match value {
            IrValue::Int(value) => {
                let Some(result) = self.static_integer_negate(&value) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                if result.is_single() {
                    return Ok(IrValue::Int(result.values()[0].to_string()));
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = sub i64 0, {value}"));
                self.known_ints.insert(temp.clone(), result);
                Ok(IrValue::Int(temp))
            }
            IrValue::Float(value) => {
                if let Some(result) = self.static_float_negate(&value) {
                    if result.is_single() && result.values()[0] != 0.0 {
                        return Ok(IrValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = fsub double 0.0, {value}"));
                if let Some(result) = self.static_float_negate(&value) {
                    self.known_floats.insert(temp.clone(), result);
                }
                Ok(IrValue::Float(temp))
            }
            _ => Err(self.unsupported(span, LLVM_UNARY_REJECTION)),
        }
    }

    fn emit_integer_bitwise_not(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        let IrValue::Int(value) = value else {
            return Err(self.unsupported(span, LLVM_BITWISE_REJECTION));
        };
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            if result.is_single() {
                return Ok(IrValue::Int(result.values()[0].to_string()));
            }
        }
        let temp = self.next_temp();
        self.body.push(format!("{temp} = xor i64 {value}, -1"));
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            self.known_ints.insert(temp.clone(), result);
        }
        Ok(IrValue::Int(temp))
    }

    fn emit_bool_not(&mut self, value: IrValue, span: Span) -> CompileResult<IrValue> {
        match value {
            IrValue::Bool(value) => Ok(IrValue::Bool(!value)),
            IrValue::BoolExpr(value) => {
                if let Some(result) = self.static_bool_not(&value) {
                    if result.is_single() {
                        return Ok(IrValue::Bool(result.values()[0]));
                    }
                }
                let temp = self.next_temp();
                self.body.push(format!("{temp} = xor i1 {value}, true"));
                if let Some(result) = self.static_bool_not(&value) {
                    self.known_bools.insert(temp.clone(), result);
                }
                Ok(IrValue::BoolExpr(temp))
            }
            IrValue::Int(value) => {
                let Some(truthy) = known_integer_truthiness(&self.known_integer_values(&value))
                else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                Ok(IrValue::Bool(!truthy))
            }
            IrValue::Float(value) => {
                let Some(truthy) = known_float_truthiness(&self.known_float_values(&value)) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                Ok(IrValue::Bool(!truthy))
            }
            IrValue::String(value) => Ok(IrValue::Bool(!php_string_truthy(&value))),
            IrValue::StringPtr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, LLVM_UNARY_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(value) => Ok(IrValue::Bool(!value)),
                    None => Err(self.unsupported(span, LLVM_UNARY_REJECTION)),
                }
            }
            IrValue::Null => Ok(IrValue::Bool(true)),
        }
    }

    fn static_bool_not(&self, value: &str) -> Option<KnownBool> {
        let value = self.known_bool_values(value)?;
        KnownBool::from_values(value.values().iter().map(|value| !value))
    }

    fn static_float_negate(&self, value: &str) -> Option<KnownFloat> {
        let value = self.known_float_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            let result = -value;
            if !result.is_finite() {
                return None;
            }
            results.push(result);
        }
        KnownFloat::from_values(results)
    }

    fn emit_echo(&mut self, value: IrValue) {
        match value {
            IrValue::Null | IrValue::Bool(false) => {}
            IrValue::Bool(true) => {
                let global = self.add_string("1");
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @{global})"
                ));
            }
            IrValue::BoolExpr(value) => {
                let true_global = self.add_string("1");
                let false_global = self.add_string("");
                let temp = self.next_temp();
                self.body.push(format!(
                    "{temp} = select i1 {value}, ptr @{true_global}, ptr @{false_global}"
                ));
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {temp})"
                ));
            }
            IrValue::Int(value) => {
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 {value})"
                ));
            }
            IrValue::Float(value) => {
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_float, double {value})"
                ));
            }
            IrValue::String(value) => {
                let global = self.add_string(&value);
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @{global})"
                ));
            }
            IrValue::StringPtr(value) => {
                self.body.push(format!(
                    "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {value})"
                ));
            }
        }
    }

    fn string_pointer_operand(&mut self, value: IrValue) -> String {
        match value {
            IrValue::String(value) => {
                let name = format!("@{}", self.add_string(&value));
                self.known_strings
                    .insert(name.clone(), KnownString::one(value));
                name
            }
            IrValue::StringPtr(value) => value,
            _ => unreachable!("string pointer operands are prefiltered"),
        }
    }

    fn add_string(&mut self, value: &str) -> String {
        let name = format!(".str.{}", self.next_string);
        self.next_string += 1;
        self.strings.push((name.clone(), value.to_string()));
        name
    }

    fn next_temp(&mut self) -> String {
        let name = format!("%tmp{}", self.next_temp);
        self.next_temp += 1;
        name
    }

    fn unsupported(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Codegen, span.line, span.column, message)
    }
}

fn clang_assembly_from_ir(ir: &str) -> CompileResult<String> {
    let mut child = Command::new("clang")
        .args(["-x", "ir", "-S", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start clang for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open clang stdin"))?;
        stdin.write_all(ir.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write LLVM IR to clang: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for clang: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("clang", &output.stderr),
        ));
    }

    assembly_backend_success_output("clang", &output)
}

fn llc_assembly_from_ir(ir: &str) -> CompileResult<String> {
    let mut child = Command::new("llc")
        .args(["-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start llc for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open llc stdin"))?;
        stdin.write_all(ir.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write LLVM IR to llc: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for llc: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("llc", &output.stderr),
        ));
    }

    assembly_backend_success_output("llc", &output)
}

fn cc_assembly_from_c(source: &str) -> CompileResult<String> {
    let mut child = Command::new("cc")
        .args(["-x", "c", "-S", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to start cc for assembly emission: {error}"),
            )
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Diagnostic::new(Phase::Codegen, 0, 0, "failed to open cc stdin"))?;
        stdin.write_all(source.as_bytes()).map_err(|error| {
            Diagnostic::new(
                Phase::Codegen,
                0,
                0,
                format!("failed to write C fallback source to cc: {error}"),
            )
        })?;
    }

    let output = child.wait_with_output().map_err(|error| {
        Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("failed to wait for cc: {error}"),
        )
    })?;

    if !output.status.success() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            assembly_backend_failure_message("cc", &output.stderr),
        ));
    }

    assembly_backend_success_output("cc", &output)
}

fn assembly_backend_failure_message(command: &str, stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    let detail = stderr.trim();
    if detail.is_empty() {
        format!("{command} failed to emit assembly: backend exited without stderr")
    } else {
        format!("{command} failed to emit assembly: {detail}")
    }
}

fn assembly_backend_success_output(
    command: &str,
    output: &std::process::Output,
) -> CompileResult<String> {
    if output.stdout.is_empty() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("{command} emitted empty assembly output"),
        ));
    }

    if String::from_utf8_lossy(&output.stdout).trim().is_empty() {
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("{command} emitted whitespace-only assembly output"),
        ));
    }

    // Successful backends may emit warnings or notes to stderr; assembly is
    // taken only from stdout and process stderr is not surfaced by phpc.
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn emit_c_source_for_assembly(program: &Program) -> CompileResult<String> {
    let mut generator = CGenerator::default();
    generator.emit_program(program)
}

#[derive(Default)]
struct CGenerator {
    body: Vec<String>,
    variables: HashMap<String, CValue>,
    known_ints: HashMap<String, KnownInt>,
    known_floats: HashMap<String, KnownFloat>,
    known_strings: HashMap<String, KnownString>,
    known_bools: HashMap<String, KnownBool>,
    uses_strcmp: bool,
}

#[derive(Debug, Clone)]
enum CValue {
    Int(String),
    Float(String),
    String(String),
    StringExpr(String),
    Bool(bool),
    BoolExpr(String),
    Null,
}

impl CGenerator {
    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        output.push_str("/* generated by phpc milestone 1 C assembly fallback */\n");
        output.push_str("#include <stdio.h>\n\n");
        if self.uses_strcmp {
            output.push_str("#include <string.h>\n\n");
        }
        output.push_str("int main(void) {\n");
        for line in &self.body {
            output.push_str("  ");
            output.push_str(line);
            output.push('\n');
        }
        output.push_str("  return 0;\n");
        output.push_str("}\n");
        Ok(output)
    }

    fn emit_statement(&mut self, stmt: &Stmt) -> CompileResult<()> {
        match stmt {
            Stmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.emit_expr(expr)?;
                    self.emit_echo(value);
                }
                Ok(())
            }
            Stmt::Print { expr, .. } => {
                let value = self.emit_expr(expr)?;
                self.emit_echo(value);
                Ok(())
            }
            Stmt::Assign { target, expr, .. } => self.emit_assignment(target, expr),
            Stmt::CompoundAssign { span, .. }
            | Stmt::IncrementDecrement { span, .. }
            | Stmt::NullCoalesceAssign { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => {
                Err(self.unsupported(function.span, ASSEMBLY_FUNCTION_DECLARATION_REJECTION))
            }
            Stmt::Class(class) => {
                Err(self.unsupported(class.span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Break { span }
            | Stmt::Continue { span } => {
                Err(self.unsupported(*span, ASSEMBLY_CONTROL_FLOW_REJECTION))
            }
            Stmt::Foreach { span, .. } => Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION)),
            Stmt::UnsetVariable { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::UnsetArrayIndex { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Stmt::ConstDeclaration { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Stmt::Return { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_FUNCTION_DECLARATION_REJECTION))
            }
            Stmt::Global { span, .. } => Err(self.unsupported(
                *span,
                "global declarations are not supported by assembly emission yet",
            )),
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> CompileResult<CValue> {
        match expr {
            Expr::Null(_) => Ok(CValue::Null),
            Expr::Bool(value, _) => Ok(CValue::Bool(*value)),
            Expr::Int(value, _) => Ok(CValue::Int(value.to_string())),
            Expr::Float(value, _) => Ok(CValue::Float(format_float_literal(*value))),
            Expr::String(value, _) => Ok(CValue::String(value.clone())),
            Expr::MagicLine { span }
            | Expr::MagicFile { span }
            | Expr::MagicDir { span }
            | Expr::MagicFunction { span } => {
                Err(self.unsupported(*span, ASSEMBLY_MAGIC_CONSTANT_REJECTION))
            }
            Expr::GlobalConstant { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Array { span, .. } => Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION)),
            Expr::Index { span, .. } => Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION)),
            Expr::AppendIndex { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Property { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Expr::MethodCall { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Expr::Variable(name, span) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| self.unsupported(*span, ASSEMBLY_VARIABLE_READ_REJECTION)),
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("defined") => {
                self.emit_defined_call(args, *span)
            }
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("isset") => {
                self.emit_isset_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("empty") => {
                self.emit_empty_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("strlen") => {
                self.emit_strlen_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("function_exists") => {
                self.emit_function_exists_call(args, *span)
            }
            Expr::Call { name, args, span } if name.eq_ignore_ascii_case("is_callable") => {
                self.emit_is_callable_call(args, *span)
            }
            Expr::Call { name, args, span } if is_native_type_introspection_builtin(name) => {
                self.emit_native_type_introspection_call(name, args, *span)
            }
            Expr::Call { name, span, .. } if is_object_metadata_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Expr::Call { name, span, .. } if is_array_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Call { span, .. } | Expr::DynamicCall { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_FUNCTION_CALL_REJECTION))
            }
            Expr::New { span, .. } => Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION)),
            Expr::Unary { op, expr, span } => {
                if matches!(op, UnaryOp::Not) {
                    if let Expr::Unary {
                        op: UnaryOp::Not,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        let value = self.emit_expr(expr)?;
                        if matches!(value, CValue::Bool(_) | CValue::BoolExpr(_)) {
                            return Ok(value);
                        }
                        let inverted = self.emit_bool_not(value, *span)?;
                        return self.emit_bool_not(inverted, *span);
                    }
                }
                if matches!(op, UnaryOp::BitwiseNot) {
                    if let Expr::Unary {
                        op: UnaryOp::BitwiseNot,
                        expr,
                        ..
                    } = expr.as_ref()
                    {
                        return match self.emit_expr(expr)? {
                            value @ CValue::Int(_) => Ok(value),
                            _ => Err(self.unsupported(*span, ASSEMBLY_BITWISE_REJECTION)),
                        };
                    }
                }
                let value = self.emit_expr(expr)?;
                self.emit_unary(*op, value, *span)
            }
            Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::NullCoalesceAssign { span, .. }
            | Expr::IncrementDecrement { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::Ternary {
                condition,
                if_true,
                if_false,
                span,
            } => self.emit_ternary_expr(condition, if_true, if_false, *span),
            Expr::ShortTernary {
                condition,
                if_false,
                span,
            } => self.emit_short_ternary(condition, if_false, *span),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) && !matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
                    return self.emit_scalar_comparison_expr(left, *op, right, *span);
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.unsupported(*span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return self.emit_static_string_concat_expr(left, right, *span);
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return self.emit_logical_expr(left, *op, right, *span);
                }
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
        }
    }

    fn emit_isset_call(&self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, ASSEMBLY_ISSET_REJECTION));
        };

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), ASSEMBLY_ISSET_REJECTION));
        };

        Ok(CValue::Bool(!matches!(
            self.variables.get(name),
            None | Some(CValue::Null)
        )))
    }

    fn emit_empty_call(&self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        let [arg] = args else {
            return Err(self.unsupported(span, ASSEMBLY_EMPTY_REJECTION));
        };

        let Expr::Variable(name, _) = arg else {
            return Err(self.unsupported(arg.span(), ASSEMBLY_EMPTY_REJECTION));
        };

        let Some(value) = self.variables.get(name) else {
            return Ok(CValue::Bool(true));
        };

        self.known_truthiness_for_value(value)
            .map(|truthy| CValue::Bool(!truthy))
            .ok_or_else(|| self.unsupported(arg.span(), ASSEMBLY_EMPTY_REJECTION))
    }

    fn emit_strlen_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.strlen_result_for_value(&value)
            .map(|length| CValue::Int(length.to_string()))
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_function_exists_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.function_exists_result_for_value(&value)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_is_callable_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        let syntax_only = if let Some(arg) = args.get(1) {
            match self.emit_expr(arg)? {
                CValue::Bool(value) => value,
                _ => return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
            }
        } else {
            false
        };

        self.is_callable_result_for_value(&value, syntax_only)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION))
    }

    fn emit_defined_call(&mut self, args: &[Expr], span: Span) -> CompileResult<CValue> {
        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        self.defined_result_for_value(&value)
            .map(CValue::Bool)
            .ok_or_else(|| self.unsupported(span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
    }

    fn emit_native_type_introspection_call(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if is_native_metadata_exists_builtin(name) {
            return self.emit_native_metadata_exists_call(args, span);
        }
        if is_native_member_metadata_exists_builtin(name) {
            return self.emit_native_member_metadata_exists_call(args, span);
        }
        if is_native_relationship_metadata_builtin(name) {
            return self.emit_native_relationship_metadata_call(args, span);
        }

        if args.len() != 1 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let value = self.emit_expr(&args[0])?;
        match name.to_ascii_lowercase().as_str() {
            "gettype" => Ok(CValue::String(c_gettype_name(&value).to_string())),
            "get_debug_type" => Ok(CValue::String(c_debug_type_name(&value).to_string())),
            "is_null" => Ok(CValue::Bool(matches!(value, CValue::Null))),
            "is_bool" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_) | CValue::BoolExpr(_)
            ))),
            "is_int" | "is_integer" | "is_long" => {
                Ok(CValue::Bool(matches!(value, CValue::Int(_))))
            }
            "is_float" | "is_double" => Ok(CValue::Bool(matches!(value, CValue::Float(_)))),
            "is_string" => Ok(CValue::Bool(matches!(
                value,
                CValue::String(_) | CValue::StringExpr(_)
            ))),
            "is_array" => Ok(CValue::Bool(false)),
            "is_scalar" => Ok(CValue::Bool(matches!(
                value,
                CValue::Bool(_)
                    | CValue::BoolExpr(_)
                    | CValue::Int(_)
                    | CValue::Float(_)
                    | CValue::String(_)
                    | CValue::StringExpr(_)
            ))),
            "is_numeric" => self
                .is_numeric_result_for_value(&value)
                .map(CValue::Bool)
                .ok_or_else(|| self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
            "is_countable" | "is_iterable" => Ok(CValue::Bool(false)),
            "is_object" => Ok(CValue::Bool(false)),
            _ => Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION)),
        }
    }

    fn emit_native_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !(1..=2).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let name = self.emit_expr(&args[0])?;
        if !matches!(name, CValue::String(_) | CValue::StringExpr(_)) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        if let Some(autoload) = args.get(1) {
            let autoload = self.emit_expr(autoload)?;
            if !matches!(autoload, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
            }
        }

        Ok(CValue::Bool(false))
    }

    fn emit_native_member_metadata_exists_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if args.len() != 2 {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let member = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(member, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        Ok(CValue::Bool(false))
    }

    fn emit_native_relationship_metadata_call(
        &mut self,
        args: &[Expr],
        span: Span,
    ) -> CompileResult<CValue> {
        if !(2..=3).contains(&args.len()) {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        let object_or_class = self.emit_expr(&args[0])?;
        let class_name = self.emit_expr(&args[1])?;
        if !matches!(object_or_class, CValue::String(_) | CValue::StringExpr(_))
            || !matches!(class_name, CValue::String(_) | CValue::StringExpr(_))
        {
            return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
        }

        if let Some(allow_string) = args.get(2) {
            let allow_string = self.emit_expr(allow_string)?;
            if !matches!(allow_string, CValue::Bool(_) | CValue::BoolExpr(_)) {
                return Err(self.unsupported(span, ASSEMBLY_FUNCTION_CALL_REJECTION));
            }
        }

        Ok(CValue::Bool(false))
    }

    fn is_numeric_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::Int(_) | CValue::Float(_) => Some(true),
            CValue::Null | CValue::Bool(_) | CValue::BoolExpr(_) => Some(false),
            CValue::String(value) => Some(is_php_numeric_string_literal(value)),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_numeric_result(&values)
            }
        }
    }

    fn function_exists_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::String(value) => Some(is_native_known_function_name(value)),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_function_exists_result(&values)
            }
            _ => None,
        }
    }

    fn strlen_result_for_value(&self, value: &CValue) -> Option<usize> {
        match value {
            CValue::String(value) => Some(value.len()),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_byte_length(&values)
            }
            _ => None,
        }
    }

    fn is_callable_result_for_value(&self, value: &CValue, syntax_only: bool) -> Option<bool> {
        match value {
            CValue::String(_) | CValue::StringExpr(_) if syntax_only => Some(true),
            CValue::Null
            | CValue::Bool(_)
            | CValue::BoolExpr(_)
            | CValue::Int(_)
            | CValue::Float(_) => Some(false),
            _ => self.function_exists_result_for_value(value),
        }
    }

    fn defined_result_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::String(value) => native_defined_result(value),
            CValue::StringExpr(_) => {
                let values = self.known_string_values_for_value(value)?;
                known_strings_have_uniform_defined_result(&values)
            }
            _ => None,
        }
    }

    fn emit_assignment(&mut self, target: &AssignTarget, expr: &Expr) -> CompileResult<()> {
        match target {
            AssignTarget::Variable { name, .. } => {
                let value = self.emit_expr(expr)?;
                self.variables.insert(name.clone(), value);
                Ok(())
            }
            AssignTarget::ArrayIndex { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            AssignTarget::Property { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
        }
    }

    fn emit_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Mod => {
                self.emit_arithmetic_binary(left, op, right, span)
            }
            BinaryOp::Div => Err(self.unsupported(span, ASSEMBLY_DIVISION_REJECTION)),
            BinaryOp::Concat => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => self.emit_scalar_comparison(left, op, right, span),
            BinaryOp::StrictEq | BinaryOp::StrictNe => {
                self.emit_static_strict_identity(left, op, right, span)
            }
            BinaryOp::NullCoalesce => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => {
                self.emit_bool_binary(left, op, right, span)
            }
            BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr | BinaryOp::BitwiseXor => {
                self.emit_integer_bitwise_binary(left, op, right, span)
            }
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => {
                self.emit_integer_shift_binary(left, op, right, span)
            }
        }
    }

    fn emit_arithmetic_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Mod => "%",
            _ => return Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION)),
        };
        match (left, right) {
            (CValue::Int(left), CValue::Int(right)) => {
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) {
                    if matches!(op, BinaryOp::Add) {
                        if right == "0" {
                            return Ok(CValue::Int(left));
                        }
                        if left == "0" {
                            return Ok(CValue::Int(right));
                        }
                    }
                    if matches!(op, BinaryOp::Sub) && right == "0" {
                        return Ok(CValue::Int(left));
                    }
                    if matches!(op, BinaryOp::Sub) && left == right {
                        return Ok(CValue::Int("0".to_string()));
                    }
                    if matches!(op, BinaryOp::Mul) {
                        if right == "0" || left == "0" {
                            return Ok(CValue::Int("0".to_string()));
                        }
                        if right == "1" {
                            return Ok(CValue::Int(left));
                        }
                        if left == "1" {
                            return Ok(CValue::Int(right));
                        }
                    }
                    let left_is_tracked = self.is_tracked_integer_value(&left);
                    let right_is_tracked = self.is_tracked_integer_value(&right);
                    let Some(result) = self.checked_static_integer_arithmetic(&left, op, &right)
                    else {
                        return Err(
                            self.unsupported(span, ASSEMBLY_INTEGER_OVERFLOW_ARITHMETIC_REJECTION)
                        );
                    };
                    if (left_is_tracked || right_is_tracked) && result.is_single() {
                        return Ok(CValue::Int(result.values()[0].to_string()));
                    }
                    let expression = format!("({left} {operator} {right})");
                    self.known_ints.insert(expression.clone(), result);
                    return Ok(CValue::Int(expression));
                }
                if matches!(op, BinaryOp::Mod) {
                    let Ok(divisor) = right.parse::<i64>() else {
                        return Err(self.unsupported(span, ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION));
                    };
                    if divisor <= 0 {
                        return Err(self.unsupported(span, ASSEMBLY_MODULO_RUNTIME_CHECK_REJECTION));
                    }
                    if divisor == 1 {
                        return Ok(CValue::Int("0".to_string()));
                    }
                    let modulo_result = self.static_integer_modulo(&left, divisor);
                    if let (Some(left_values), Some(result)) =
                        (self.known_integer_values(&left), modulo_result.as_ref())
                    {
                        if !left_values.is_single() && result.is_single() {
                            return Ok(CValue::Int(result.values()[0].to_string()));
                        }
                    }
                    let expression = format!("({left} {operator} {right})");
                    if let Some(result) = modulo_result {
                        self.known_ints.insert(expression.clone(), result);
                    }
                    return Ok(CValue::Int(expression));
                }
                Ok(CValue::Int(format!("({left} {operator} {right})")))
            }
            (CValue::Float(left), CValue::Float(right)) => {
                if matches!(op, BinaryOp::Mod) {
                    return Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION));
                }
                if matches!(op, BinaryOp::Add) {
                    if right == "0.0" && self.known_finite_nonzero_float_values(&left) {
                        return Ok(CValue::Float(left));
                    }
                    if left == "0.0" && self.known_finite_nonzero_float_values(&right) {
                        return Ok(CValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && right == "0.0"
                    && self.known_finite_nonzero_float_values(&left)
                {
                    return Ok(CValue::Float(left));
                }
                if matches!(op, BinaryOp::Sub) && left == "0.0" {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(CValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                if matches!(op, BinaryOp::Mul) {
                    if (right == "0.0" && self.known_finite_positive_float_values(&left))
                        || (left == "0.0" && self.known_finite_positive_float_values(&right))
                    {
                        return Ok(CValue::Float("0.0".to_string()));
                    }
                    if right == "-1.0" {
                        if let Some(result) = self.static_float_negate(&left) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(CValue::Float(format_float_literal(result.values()[0])));
                            }
                        }
                    }
                    if left == "-1.0" {
                        if let Some(result) = self.static_float_negate(&right) {
                            if result.is_single() && result.values()[0] != 0.0 {
                                return Ok(CValue::Float(format_float_literal(result.values()[0])));
                            }
                        }
                    }
                    if right == "1.0" && self.known_float_values(&left).is_some() {
                        return Ok(CValue::Float(left));
                    }
                    if left == "1.0" && self.known_float_values(&right).is_some() {
                        return Ok(CValue::Float(right));
                    }
                }
                if matches!(op, BinaryOp::Sub)
                    && left == right
                    && self
                        .known_float_values(&left)
                        .is_some_and(|values| values.values().iter().all(|value| value.is_finite()))
                {
                    return Ok(CValue::Float("0.0".to_string()));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul)
                    && (left_is_tracked || right_is_tracked)
                {
                    if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                        if result.is_single() && result.values()[0] != 0.0 {
                            return Ok(CValue::Float(format_float_literal(result.values()[0])));
                        }
                    }
                }
                let expression = format!("({left} {operator} {right})");
                if let Some(result) = self.static_float_arithmetic(&left, op, &right) {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            (CValue::Int(_), CValue::Float(_)) | (CValue::Float(_), CValue::Int(_))
                if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) =>
            {
                Err(self.unsupported(span, ASSEMBLY_MIXED_NUMERIC_ARITHMETIC_REJECTION))
            }
            (
                CValue::Null
                | CValue::Bool(_)
                | CValue::BoolExpr(_)
                | CValue::String(_)
                | CValue::StringExpr(_),
                _,
            )
            | (
                _,
                CValue::Null
                | CValue::Bool(_)
                | CValue::BoolExpr(_)
                | CValue::String(_)
                | CValue::StringExpr(_),
            ) if matches!(op, BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul) => {
                Err(self.unsupported(span, ASSEMBLY_SCALAR_COERCION_ARITHMETIC_REJECTION))
            }
            _ => Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION)),
        }
    }

    fn emit_scalar_comparison_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let left = self
            .emit_expr(left)
            .map_err(|_| self.unsupported(span, assembly_comparison_rejection()))?;
        let right = self
            .emit_expr(right)
            .map_err(|_| self.unsupported(span, assembly_comparison_rejection()))?;
        self.emit_scalar_comparison(left, op, right, span)
    }

    fn emit_scalar_comparison(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match (left, right) {
            (CValue::Null, CValue::Null) => {
                let Some(result) = null_comparison_result(op) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::Bool(left), CValue::Bool(right)) => {
                let Some(result) = bool_comparison_result(left, op, right) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::BoolExpr(left), CValue::Bool(right)) => {
                let right = if right { "1" } else { "0" };
                self.emit_bool_scalar_comparison(left, op, right.to_string(), span)
            }
            (CValue::Bool(left), CValue::BoolExpr(right)) => {
                let left = if left { "1" } else { "0" };
                self.emit_bool_scalar_comparison(left.to_string(), op, right, span)
            }
            (CValue::BoolExpr(left), CValue::BoolExpr(right)) => {
                self.emit_bool_scalar_comparison(left, op, right, span)
            }
            (CValue::String(left), CValue::String(right)) => {
                let Some(result) = static_safe_string_comparison_result(
                    Some(KnownString::one(left)),
                    op,
                    Some(KnownString::one(right)),
                ) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                Ok(CValue::Bool(result))
            }
            (CValue::StringExpr(left), CValue::StringExpr(right)) => {
                self.emit_string_comparison(left, op, right, span)
            }
            (CValue::StringExpr(left), CValue::String(right)) => {
                let right_operand = c_string_operand(CValue::String(right.clone()));
                self.known_strings
                    .insert(right_operand.clone(), KnownString::one(right));
                self.emit_string_comparison(left, op, right_operand, span)
            }
            (CValue::String(left), CValue::StringExpr(right)) => {
                let left_operand = c_string_operand(CValue::String(left.clone()));
                self.known_strings
                    .insert(left_operand.clone(), KnownString::one(left));
                self.emit_string_comparison(left_operand, op, right, span)
            }
            (CValue::Int(left), CValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    let Some(result) = integer_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                if left == right {
                    let Some(result) = integer_comparison_result(0, op, 0) else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_integer_value(&left);
                let right_is_tracked = self.is_tracked_integer_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<i64>().is_ok() || right.parse::<i64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_integer_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) =
                            self.static_integer_comparison_result(&left, op, &right)
                        {
                            return Ok(CValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = c_comparison_operator(op)
                    .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                Ok(CValue::BoolExpr(expression))
            }
            (CValue::Float(left), CValue::Float(right)) => {
                let Some(left_values) = self.known_float_values(&left) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                let Some(right_values) = self.known_float_values(&right) else {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                };
                if !left_values.values().iter().all(|value| value.is_finite())
                    || !right_values.values().iter().all(|value| value.is_finite())
                {
                    return Err(self.unsupported(span, assembly_comparison_rejection()));
                }
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    let Some(result) = float_comparison_result(left_literal, op, right_literal)
                    else {
                        return Err(self.unsupported(span, assembly_comparison_rejection()));
                    };
                    return Ok(CValue::Bool(result));
                }
                let left_is_tracked = self.is_tracked_float_value(&left);
                let right_is_tracked = self.is_tracked_float_value(&right);
                if left_is_tracked != right_is_tracked
                    && (left.parse::<f64>().is_ok() || right.parse::<f64>().is_ok())
                {
                    let tracked = if left_is_tracked { &left } else { &right };
                    if self
                        .known_float_values(tracked)
                        .is_some_and(|values| values.is_single())
                    {
                        if let Some(result) = self.static_float_comparison_result(&left, op, &right)
                        {
                            return Ok(CValue::Bool(result));
                        }
                    }
                }
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = c_comparison_operator(op)
                    .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_float_comparison_result(&left, op, &right) {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                Ok(CValue::BoolExpr(expression))
            }
            _ => Err(self.unsupported(span, assembly_comparison_rejection())),
        }
    }

    fn emit_bool_scalar_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        if let Some(fold) = bool_literal_comparison_fold(&left, op, &right, "1", "0") {
            return match fold {
                BoolLiteralComparisonFold::Static(value) => Ok(CValue::Bool(value)),
                BoolLiteralComparisonFold::Reuse(value) => Ok(CValue::BoolExpr(value)),
                BoolLiteralComparisonFold::Invert(value) => {
                    self.emit_bool_not(CValue::BoolExpr(value), span)
                }
            };
        }
        if left == right {
            let Some(result) = bool_comparison_result(false, op, false) else {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            };
            return Ok(CValue::Bool(result));
        }
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            return Ok(CValue::Bool(result));
        }
        let operator = c_comparison_operator(op)
            .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
        let expression = format!("(({left}) {operator} ({right}))");
        if let Some(result) = self.static_bool_comparison_result(&left, op, &right) {
            self.known_bools
                .insert(expression.clone(), KnownBool::one(result));
        }
        Ok(CValue::BoolExpr(expression))
    }

    fn checked_static_integer_arithmetic(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left.checked_add(*right),
                    BinaryOp::Sub => left.checked_sub(*right),
                    BinaryOp::Mul => left.checked_mul(*right),
                    _ => None,
                }?;
                results.push(result);
            }
        }
        KnownInt::from_values(results)
    }

    fn known_integer_values(&self, value: &str) -> Option<KnownInt> {
        value
            .parse::<i64>()
            .ok()
            .map(KnownInt::one)
            .or_else(|| self.known_ints.get(value).cloned())
    }

    fn is_tracked_integer_value(&self, value: &str) -> bool {
        self.known_ints.contains_key(value)
    }

    fn static_integer_modulo(&self, left: &str, divisor: i64) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let values = left.values().iter().map(|value| value % divisor);
        KnownInt::from_values(values)
    }

    fn static_integer_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = integer_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_float_comparison_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        if !left_values.values().iter().all(|value| value.is_finite())
            || !right_values.values().iter().all(|value| value.is_finite())
        {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = float_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_bool_comparison_result(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = bool_comparison_result(*left, op, *right)?;
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_integer_shift_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let (CValue::Int(left), CValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        let Some(count) = self.static_integer_shift_count(&right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if count == 0 {
            return Ok(CValue::Int(left));
        }
        if self.is_tracked_integer_value(&left) {
            if let Some(result) = self.static_integer_shift(&left, op, count) {
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let operator = match op {
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
            _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
        };
        let expression = format!("({left} {operator} {count})");
        if let Some(result) = self.static_integer_shift(&left, op, count) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn static_integer_shift(&self, left: &str, op: BinaryOp, count: u32) -> Option<KnownInt> {
        let left = self.known_integer_values(left)?;
        let factor = if matches!(op, BinaryOp::ShiftLeft) {
            Some(1_i64.checked_shl(count)?)
        } else {
            None
        };
        let values = left.values().iter().map(|value| match op {
            BinaryOp::ShiftLeft => value.checked_mul(factor.expect("left shift has a factor")),
            BinaryOp::ShiftRight => Some(value >> count),
            _ => None,
        });
        let mut results = Vec::new();
        for value in values {
            results.push(value?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_shift_count(&self, right: &str) -> Option<u32> {
        if let Ok(count) = right.parse::<u32>() {
            return (count < 64).then_some(count);
        }
        let values = self.known_integer_values(right)?;
        if !values.is_single() {
            return None;
        }
        let count = u32::try_from(values.values()[0]).ok()?;
        (count < 64).then_some(count)
    }

    fn emit_integer_bitwise_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let (CValue::Int(left), CValue::Int(right)) = (left, right) else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if left == right {
            return Ok(match op {
                BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr => CValue::Int(left),
                BinaryOp::BitwiseXor => CValue::Int("0".to_string()),
                _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
            });
        }
        if matches!(op, BinaryOp::BitwiseAnd) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int("0".to_string()));
            }
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(CValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
            {
                return Ok(CValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr | BinaryOp::BitwiseXor) {
            if self
                .known_integer_values(&right)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int(left));
            }
            if self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(0))
            {
                return Ok(CValue::Int(right));
            }
        }
        if matches!(op, BinaryOp::BitwiseOr)
            && (self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                || self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1)))
        {
            return Ok(CValue::Int("-1".to_string()));
        }
        if matches!(op, BinaryOp::BitwiseXor)
            && ((self
                .known_integer_values(&left)
                .is_some_and(|values| values.is_single_value(-1))
                && self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single()))
                || (self
                    .known_integer_values(&right)
                    .is_some_and(|values| values.is_single_value(-1))
                    && self
                        .known_integer_values(&left)
                        .is_some_and(|values| values.is_single())))
        {
            let result = self
                .static_integer_bitwise(&left, op, &right)
                .expect("single known integer XOR all-ones result is known");
            return Ok(CValue::Int(result.values()[0].to_string()));
        }
        let left_is_tracked = self.is_tracked_integer_value(&left);
        let right_is_tracked = self.is_tracked_integer_value(&right);
        if left_is_tracked || right_is_tracked {
            if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
            }
        }
        let operator = match op {
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            _ => return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION)),
        };
        let expression = format!("({left} {operator} {right})");
        if let Some(result) = self.static_integer_bitwise(&left, op, &right) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn static_integer_bitwise(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownInt> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::BitwiseAnd => left & right,
                    BinaryOp::BitwiseOr => left | right,
                    BinaryOp::BitwiseXor => left ^ right,
                    _ => return None,
                });
            }
        }
        KnownInt::from_values(results)
    }

    fn static_integer_negate(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            results.push(value.checked_neg()?);
        }
        KnownInt::from_values(results)
    }

    fn static_integer_bitwise_not(&self, value: &str) -> Option<KnownInt> {
        let value = self.known_integer_values(value)?;
        KnownInt::from_values(value.values().iter().map(|value| !value))
    }

    fn emit_static_string_concat_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        if is_empty_string_literal(left) {
            let right = self.emit_expr(right)?;
            return self.emit_empty_string_concat_identity(right, span);
        }
        if is_empty_string_literal(right) {
            let left = self.emit_expr(left)?;
            return self.emit_empty_string_concat_identity(left, span);
        }
        let left = self.emit_static_string_concat_operand(left, span)?;
        let right = self.emit_static_string_concat_operand(right, span)?;
        Ok(CValue::String(format!("{left}{right}")))
    }

    fn emit_empty_string_concat_identity(
        &self,
        value: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match value {
            CValue::String(_) | CValue::StringExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
        }
    }

    fn emit_static_string_concat_operand(
        &mut self,
        expr: &Expr,
        span: Span,
    ) -> CompileResult<String> {
        match expr {
            Expr::String(value, _) => Ok(value.clone()),
            Expr::Variable(name, variable_span) => match self.variables.get(name).cloned() {
                Some(CValue::String(value)) => Ok(value),
                Some(_) => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
                None => Err(self.unsupported(*variable_span, ASSEMBLY_VARIABLE_READ_REJECTION)),
            },
            Expr::Binary {
                left,
                op: BinaryOp::Concat,
                right,
                span: concat_span,
            } => match self.emit_static_string_concat_expr(left, right, *concat_span)? {
                CValue::String(value) => Ok(value),
                _ => unreachable!("static string concatenation returns a string"),
            },
            Expr::Ternary { .. } => match self.emit_expr(expr)? {
                CValue::String(value) => Ok(value),
                CValue::StringExpr(value) => {
                    let values = self
                        .known_string_values(&value)
                        .ok_or_else(|| self.unsupported(span, ASSEMBLY_CONCAT_REJECTION))?;
                    if values.is_single() {
                        Ok(values.values()[0].clone())
                    } else {
                        Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION))
                    }
                }
                _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
            },
            _ => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
        }
    }

    fn emit_static_strict_identity(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let is_identical = match (left, right) {
            (CValue::Null, CValue::Null) => true,
            (CValue::Bool(left), CValue::Bool(right)) => left == right,
            (CValue::BoolExpr(left), CValue::Bool(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    Some(KnownBool::one(right)),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(CValue::BoolExpr(left));
                }
                if matches!(
                    (op, right),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(CValue::BoolExpr(left), span);
                }
                let right = if right { "1" } else { "0" };
                return self.emit_bool_comparison(left, op, right.to_string(), span);
            }
            (CValue::Bool(left), CValue::BoolExpr(right)) => {
                if let Some(result) = self.static_bool_strict_identity(
                    Some(KnownBool::one(left)),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, true) | (BinaryOp::StrictNe, false)
                ) {
                    return Ok(CValue::BoolExpr(right));
                }
                if matches!(
                    (op, left),
                    (BinaryOp::StrictEq, false) | (BinaryOp::StrictNe, true)
                ) {
                    return self.emit_bool_not(CValue::BoolExpr(right), span);
                }
                let left = if left { "1" } else { "0" };
                return self.emit_bool_comparison(left.to_string(), op, right, span);
            }
            (CValue::BoolExpr(left), CValue::BoolExpr(right)) => {
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_bool_strict_identity(
                    self.known_bool_values(&left),
                    self.known_bool_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                return self.emit_bool_comparison(left, op, right, span);
            }
            (CValue::String(left), CValue::String(right)) => left == right,
            (CValue::StringExpr(left), CValue::StringExpr(right)) => {
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::StringExpr(left), CValue::String(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    self.known_string_values(&left),
                    Some(KnownString::one(right.clone())),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                let right = c_string_operand(CValue::String(right));
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::String(left), CValue::StringExpr(right)) => {
                if let Some(result) = self.static_string_strict_identity(
                    Some(KnownString::one(left.clone())),
                    self.known_string_values(&right),
                    op,
                ) {
                    return Ok(CValue::Bool(result));
                }
                let left = c_string_operand(CValue::String(left));
                return self.emit_string_comparison(left, op, right, span);
            }
            (CValue::Float(left), CValue::Float(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(CValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                    }));
                }
                if left == right {
                    if let Some(result) = self.static_same_float_strict_identity(&left, op) {
                        return Ok(CValue::Bool(result));
                    }
                }
                if let Some(result) = self.static_float_strict_identity(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = match op {
                    BinaryOp::StrictEq => "==",
                    BinaryOp::StrictNe => "!=",
                    _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                return Ok(CValue::BoolExpr(expression));
            }
            (CValue::Int(left), CValue::Int(right)) => {
                if let (Ok(left_literal), Ok(right_literal)) =
                    (left.parse::<i64>(), right.parse::<i64>())
                {
                    return Ok(CValue::Bool(match op {
                        BinaryOp::StrictEq => left_literal == right_literal,
                        BinaryOp::StrictNe => left_literal != right_literal,
                        _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                    }));
                }
                if left == right {
                    return Ok(CValue::Bool(static_strict_identity_result(true, op)));
                }
                if let Some(result) = self.static_integer_strict_identity(&left, op, &right) {
                    return Ok(CValue::Bool(result));
                }
                let operator = match op {
                    BinaryOp::StrictEq => "==",
                    BinaryOp::StrictNe => "!=",
                    _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = self.static_integer_strict_identity_result(&left, op, &right)
                {
                    self.known_bools
                        .insert(expression.clone(), KnownBool::one(result));
                }
                return Ok(CValue::BoolExpr(expression));
            }
            _ => false,
        };
        let result = match op {
            BinaryOp::StrictEq => is_identical,
            BinaryOp::StrictNe => !is_identical,
            _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
        };
        Ok(CValue::Bool(result))
    }

    fn static_integer_strict_identity(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        if left_values.is_single() && right_values.is_single() {
            return None;
        }
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_integer_strict_identity_result(
        &self,
        left: &str,
        op: BinaryOp,
        right: &str,
    ) -> Option<bool> {
        let left_values = self.known_integer_values(left)?;
        let right_values = self.known_integer_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_string_values(&self, value: &str) -> Option<KnownString> {
        self.known_strings.get(value).cloned()
    }

    fn known_bool_values(&self, value: &str) -> Option<KnownBool> {
        match value {
            "1" => Some(KnownBool::one(true)),
            "0" => Some(KnownBool::one(false)),
            _ => self.known_bools.get(value).cloned(),
        }
    }

    fn static_bool_strict_identity(
        &self,
        left_values: Option<KnownBool>,
        right_values: Option<KnownBool>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn known_float_values(&self, value: &str) -> Option<KnownFloat> {
        value
            .parse::<f64>()
            .ok()
            .map(KnownFloat::one)
            .or_else(|| self.known_floats.get(value).cloned())
    }

    fn is_tracked_float_value(&self, value: &str) -> bool {
        self.known_floats.contains_key(value)
    }

    fn known_finite_nonzero_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value != 0.0)
        })
    }

    fn known_finite_positive_float_values(&self, value: &str) -> bool {
        self.known_float_values(value).is_some_and(|values| {
            values
                .values()
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
        })
    }

    fn static_float_strict_identity(&self, left: &str, op: BinaryOp, right: &str) -> Option<bool> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn static_same_float_strict_identity(&self, value: &str, op: BinaryOp) -> Option<bool> {
        let values = self.known_float_values(value)?;
        if !values.values().iter().all(|value| value.is_finite()) {
            return None;
        }
        Some(static_strict_identity_result(true, op))
    }

    fn static_float_arithmetic(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownFloat> {
        let left_values = self.known_float_values(left)?;
        let right_values = self.known_float_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                let result = match op {
                    BinaryOp::Add => left + right,
                    BinaryOp::Sub => left - right,
                    BinaryOp::Mul => left * right,
                    _ => return None,
                };
                if !result.is_finite() {
                    return None;
                }
                results.push(result);
            }
        }
        KnownFloat::from_values(results)
    }

    fn known_string_values_for_value(&self, value: &CValue) -> Option<KnownString> {
        match value {
            CValue::String(value) => Some(KnownString::one(value.clone())),
            CValue::StringExpr(value) => self.known_string_values(value),
            _ => None,
        }
    }

    fn static_string_strict_identity(
        &self,
        left_values: Option<KnownString>,
        right_values: Option<KnownString>,
        op: BinaryOp,
    ) -> Option<bool> {
        let left_values = left_values?;
        let right_values = right_values?;
        let mut result = None;
        for left in left_values.values() {
            for right in right_values.values() {
                let current = match op {
                    BinaryOp::StrictEq => left == right,
                    BinaryOp::StrictNe => left != right,
                    _ => return None,
                };
                if result.is_some_and(|result| result != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        result
    }

    fn emit_bool_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        if let (Some(left), Some(right)) = (
            self.known_truthiness_for_value(&left),
            self.known_truthiness_for_value(&right),
        ) {
            return Ok(CValue::Bool(logical_truthiness_result(left, op, right)?));
        }
        match (left, right) {
            (CValue::Bool(left), CValue::Bool(right)) => Ok(CValue::Bool(match op {
                BinaryOp::LogicalAnd => left && right,
                BinaryOp::LogicalOr => left || right,
                BinaryOp::LogicalXor => left ^ right,
                _ => return Err(self.unsupported(span, assembly_logical_rejection())),
            })),
            (CValue::Bool(left), right) => match op {
                BinaryOp::LogicalAnd if left => self.require_bool_value(right, span),
                BinaryOp::LogicalAnd => Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if left => Ok(CValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(right, span),
                BinaryOp::LogicalXor if left => {
                    let right = self.require_bool_value(right, span)?;
                    self.emit_bool_not(right, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(right, span),
                _ => Err(self.unsupported(span, assembly_logical_rejection())),
            },
            (left, CValue::Bool(right)) => match op {
                BinaryOp::LogicalAnd if right => self.require_bool_value(left, span),
                BinaryOp::LogicalAnd => Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if right => Ok(CValue::Bool(true)),
                BinaryOp::LogicalOr => self.require_bool_value(left, span),
                BinaryOp::LogicalXor if right => {
                    let left = self.require_bool_value(left, span)?;
                    self.emit_bool_not(left, span)
                }
                BinaryOp::LogicalXor => self.require_bool_value(left, span),
                _ => Err(self.unsupported(span, assembly_logical_rejection())),
            },
            (left, right) => {
                let Some(left) = c_bool_operand(left) else {
                    return Err(self.unsupported(span, assembly_logical_rejection()));
                };
                let Some(right) = c_bool_operand(right) else {
                    return Err(self.unsupported(span, assembly_logical_rejection()));
                };
                if left == right && matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
                    return Ok(CValue::BoolExpr(left));
                }
                if left == right && matches!(op, BinaryOp::LogicalXor) {
                    return Ok(CValue::Bool(false));
                }
                let result = self.static_bool_binary(&left, op, &right);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let operator = match op {
                    BinaryOp::LogicalAnd => "&&",
                    BinaryOp::LogicalOr => "||",
                    BinaryOp::LogicalXor => "!=",
                    _ => return Err(self.unsupported(span, assembly_logical_rejection())),
                };
                let expression = format!("(({left}) {operator} ({right}))");
                if let Some(result) = result {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
        }
    }

    fn emit_logical_expr(
        &mut self,
        left: &Expr,
        op: BinaryOp,
        right: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let left = self.emit_expr(left)?;
        if let Some(left_truthy) = self.known_truthiness_for_value(&left) {
            match op {
                BinaryOp::LogicalAnd if !left_truthy => return Ok(CValue::Bool(false)),
                BinaryOp::LogicalOr if left_truthy => return Ok(CValue::Bool(true)),
                _ => {}
            }
        }
        let right = self.emit_expr(right)?;
        self.emit_bool_binary(left, op, right, span)
    }

    fn require_bool_value(&self, value: CValue, span: Span) -> CompileResult<CValue> {
        match value {
            CValue::Bool(_) | CValue::BoolExpr(_) => Ok(value),
            _ => Err(self.unsupported(span, assembly_logical_rejection())),
        }
    }

    fn known_truthiness_for_value(&self, value: &CValue) -> Option<bool> {
        match value {
            CValue::Bool(value) => Some(*value),
            CValue::BoolExpr(_) => None,
            CValue::Int(value) => known_integer_truthiness(&self.known_integer_values(value)),
            CValue::Float(value) => known_float_truthiness(&self.known_float_values(value)),
            CValue::String(value) => Some(php_string_truthy(value)),
            CValue::StringExpr(value) => self
                .known_string_values(value)
                .and_then(|values| known_string_truthiness(&values)),
            CValue::Null => Some(false),
        }
    }

    fn emit_bool_comparison(
        &self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = match op {
            BinaryOp::StrictEq => "==",
            BinaryOp::StrictNe => "!=",
            _ => return Err(self.unsupported(span, assembly_comparison_rejection())),
        };
        Ok(CValue::BoolExpr(format!("(({left}) {operator} ({right}))")))
    }

    fn static_bool_binary(&self, left: &str, op: BinaryOp, right: &str) -> Option<KnownBool> {
        let left_values = self.known_bool_values(left)?;
        let right_values = self.known_bool_values(right)?;
        let mut results = Vec::new();
        for left in left_values.values() {
            for right in right_values.values() {
                results.push(match op {
                    BinaryOp::LogicalAnd => *left && *right,
                    BinaryOp::LogicalOr => *left || *right,
                    BinaryOp::LogicalXor => *left ^ *right,
                    _ => return None,
                });
            }
        }
        KnownBool::from_values(results)
    }

    fn emit_string_comparison(
        &mut self,
        left: String,
        op: BinaryOp,
        right: String,
        span: Span,
    ) -> CompileResult<CValue> {
        let operator = c_string_comparison_operator(op)
            .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
        if left == right {
            let Some(result) = reflexive_string_comparison_result(op) else {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            };
            return Ok(CValue::Bool(result));
        }
        let known_result = if matches!(op, BinaryOp::StrictEq | BinaryOp::StrictNe) {
            self.static_string_strict_identity(
                self.known_string_values(&left),
                self.known_string_values(&right),
                op,
            )
        } else {
            let left_values = self
                .known_string_values(&left)
                .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
            let right_values = self
                .known_string_values(&right)
                .ok_or_else(|| self.unsupported(span, assembly_comparison_rejection()))?;
            if !known_strings_are_safe_for_native_comparison(&left_values)
                || !known_strings_are_safe_for_native_comparison(&right_values)
            {
                return Err(self.unsupported(span, assembly_comparison_rejection()));
            }
            string_comparison_result_for_known_values(&left_values, op, &right_values)
        };
        if let Some(known_result) = known_result {
            return Ok(CValue::Bool(known_result));
        }
        self.uses_strcmp = true;
        let expression = format!("(strcmp({left}, {right}) {operator} 0)");
        if let Some(known_result) = known_result {
            self.known_bools
                .insert(expression.clone(), KnownBool::one(known_result));
        }
        Ok(CValue::BoolExpr(expression))
    }

    fn emit_ternary(
        &mut self,
        condition: CValue,
        if_true: CValue,
        if_false: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match condition {
            CValue::Bool(true) => return Ok(if_true),
            CValue::Bool(false) => return Ok(if_false),
            CValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    return Ok(if_true);
                }
                if values.is_single_value(0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
            CValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    return Ok(if_true);
                }
                if matches!(values.values(), [value] if *value == 0.0) {
                    return Ok(if_false);
                }
                Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
            CValue::String(value) => {
                if php_string_truthy(&value) {
                    Ok(if_true)
                } else {
                    Ok(if_false)
                }
            }
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(if_true),
                    Some(false) => Ok(if_false),
                    None => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
                }
            }
            CValue::Null => Ok(if_false),
            condition => self.emit_dynamic_ternary(condition, if_true, if_false, span),
        }
    }

    fn emit_ternary_expr(
        &mut self,
        condition: &Expr,
        if_true: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_ternary_expr(condition, if_true, if_false) {
            return Ok(condition_value);
        }
        if let Some(truthy) = self.known_truthiness_for_value(&condition_value) {
            return if truthy {
                self.emit_expr(if_true)
            } else {
                self.emit_expr(if_false)
            };
        }
        if !matches!(condition_value, CValue::BoolExpr(_)) {
            return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
        }
        let if_true = self.emit_expr(if_true)?;
        let if_false = self.emit_expr(if_false)?;
        self.emit_ternary(condition_value, if_true, if_false, span)
    }

    fn emit_short_ternary(
        &mut self,
        condition: &Expr,
        if_false: &Expr,
        span: Span,
    ) -> CompileResult<CValue> {
        let condition_value = self.emit_expr(condition)?;
        if same_direct_variable_expr(condition, if_false) {
            if matches!(
                condition_value,
                CValue::BoolExpr(_) | CValue::Int(_) | CValue::Float(_) | CValue::StringExpr(_)
            ) {
                return Ok(condition_value);
            }
        }
        match condition_value {
            CValue::Bool(true) => Ok(CValue::Bool(true)),
            CValue::Bool(false) => {
                let if_false = self.emit_expr(if_false)?;
                Ok(if_false)
            }
            CValue::Int(value) => {
                let Some(values) = self.known_integer_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if values.values().iter().all(|value| *value != 0) {
                    Ok(CValue::Int(value))
                } else if values.is_single_value(0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
                }
            }
            CValue::Float(value) => {
                let Some(values) = self.known_float_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                if !values.values().iter().all(|value| value.is_finite()) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                if values.values().iter().all(|value| *value != 0.0) {
                    Ok(CValue::Float(value))
                } else if matches!(values.values(), [value] if *value == 0.0) {
                    self.emit_expr(if_false)
                } else {
                    Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION))
                }
            }
            CValue::String(value) => {
                if php_string_truthy(&value) {
                    Ok(CValue::String(value))
                } else {
                    self.emit_expr(if_false)
                }
            }
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(true) => Ok(CValue::StringExpr(value)),
                    Some(false) => self.emit_expr(if_false),
                    None => Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION)),
                }
            }
            CValue::Null => self.emit_expr(if_false),
            condition @ CValue::BoolExpr(_) => {
                let if_false = self.emit_expr(if_false)?;
                if !matches!(if_false, CValue::Bool(_) | CValue::BoolExpr(_)) {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                }
                self.emit_ternary(condition, CValue::Bool(true), if_false, span)
            }
        }
    }

    fn emit_dynamic_ternary(
        &mut self,
        condition: CValue,
        if_true: CValue,
        if_false: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        let Some(condition) = c_bool_operand(condition) else {
            return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
        };
        match (if_true, if_false) {
            (CValue::Null, CValue::Null) => Ok(CValue::Null),
            (CValue::Int(if_true), CValue::Int(if_false)) => {
                if if_true == if_false {
                    return Ok(CValue::Int(if_true));
                }
                let result = self.static_integer_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Int(result.values()[0].to_string()));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_ints.insert(expression.clone(), result);
                }
                Ok(CValue::Int(expression))
            }
            (CValue::Float(if_true), CValue::Float(if_false)) => {
                if if_true == if_false {
                    return Ok(CValue::Float(if_true));
                }
                let result = self.static_float_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            (if_true, if_false) => {
                if matches!(
                    (&if_true, &if_false),
                    (
                        CValue::String(_) | CValue::StringExpr(_),
                        CValue::String(_) | CValue::StringExpr(_)
                    )
                ) {
                    if let Some(result) = identical_c_string_ternary_branch(&if_true, &if_false) {
                        return Ok(result);
                    }
                    let result = self.static_string_ternary(&if_true, &if_false);
                    if let Some(result) = result.as_ref() {
                        if result.is_single() {
                            return Ok(CValue::String(result.values()[0].clone()));
                        }
                    }
                    let if_true = c_string_operand(if_true);
                    let if_false = c_string_operand(if_false);
                    let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                    if let Some(result) = result {
                        self.known_strings.insert(expression.clone(), result);
                    }
                    return Ok(CValue::StringExpr(expression));
                }
                if let Some(result) = identical_c_bool_expr_ternary_branch(&if_true, &if_false) {
                    return Ok(result);
                }
                if let Some(result) = c_bool_literal_ternary_branch(&condition, &if_true, &if_false)
                {
                    return match result {
                        BoolLiteralTernaryBranch::Static(value) => Ok(CValue::Bool(value)),
                        BoolLiteralTernaryBranch::Reuse(value) => Ok(CValue::BoolExpr(value)),
                        BoolLiteralTernaryBranch::Invert(value) => {
                            self.emit_bool_not(CValue::BoolExpr(value), span)
                        }
                    };
                }
                let Some(if_true) = c_bool_operand(if_true) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                let Some(if_false) = c_bool_operand(if_false) else {
                    return Err(self.unsupported(span, ASSEMBLY_CONDITIONAL_REJECTION));
                };
                let result = self.static_bool_ternary(&if_true, &if_false);
                if let Some(result) = result.as_ref() {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let expression = format!("(({condition}) ? ({if_true}) : ({if_false}))");
                if let Some(result) = result {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
        }
    }

    fn static_integer_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownInt> {
        let if_true = self.known_integer_values(if_true)?;
        let if_false = self.known_integer_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownInt::from_values(values)
    }

    fn static_float_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownFloat> {
        let if_true = self.known_float_values(if_true)?;
        let if_false = self.known_float_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownFloat::from_values(values)
    }

    fn static_string_ternary(&self, if_true: &CValue, if_false: &CValue) -> Option<KnownString> {
        let if_true = self.known_string_values_for_value(if_true)?;
        let if_false = self.known_string_values_for_value(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values().iter().cloned());
        KnownString::from_values(values)
    }

    fn static_bool_ternary(&self, if_true: &str, if_false: &str) -> Option<KnownBool> {
        let if_true = self.known_bool_values(if_true)?;
        let if_false = self.known_bool_values(if_false)?;
        let mut values = if_true.values().to_vec();
        values.extend(if_false.values());
        KnownBool::from_values(values)
    }

    fn emit_unary(&mut self, op: UnaryOp, value: CValue, span: Span) -> CompileResult<CValue> {
        match op {
            UnaryOp::Negate => self.emit_numeric_negate(value, span),
            UnaryOp::Not => self.emit_bool_not(value, span),
            UnaryOp::BitwiseNot => self.emit_integer_bitwise_not(value, span),
        }
    }

    fn emit_numeric_negate(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        match value {
            CValue::Int(value) => {
                let Some(result) = self.static_integer_negate(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                if result.is_single() {
                    return Ok(CValue::Int(result.values()[0].to_string()));
                }
                let expression = format!("(-{value})");
                self.known_ints.insert(expression.clone(), result);
                Ok(CValue::Int(expression))
            }
            CValue::Float(value) => {
                if let Some(result) = self.static_float_negate(&value) {
                    if result.is_single() && result.values()[0] != 0.0 {
                        return Ok(CValue::Float(format_float_literal(result.values()[0])));
                    }
                }
                let expression = format!("(-{value})");
                if let Some(result) = self.static_float_negate(&value) {
                    self.known_floats.insert(expression.clone(), result);
                }
                Ok(CValue::Float(expression))
            }
            _ => Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION)),
        }
    }

    fn emit_integer_bitwise_not(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        let CValue::Int(value) = value else {
            return Err(self.unsupported(span, ASSEMBLY_BITWISE_REJECTION));
        };
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            if result.is_single() {
                return Ok(CValue::Int(result.values()[0].to_string()));
            }
        }
        let expression = format!("(~{value})");
        if let Some(result) = self.static_integer_bitwise_not(&value) {
            self.known_ints.insert(expression.clone(), result);
        }
        Ok(CValue::Int(expression))
    }

    fn emit_bool_not(&mut self, value: CValue, span: Span) -> CompileResult<CValue> {
        match value {
            CValue::Bool(value) => Ok(CValue::Bool(!value)),
            CValue::BoolExpr(value) => {
                if let Some(result) = self.static_bool_not(&value) {
                    if result.is_single() {
                        return Ok(CValue::Bool(result.values()[0]));
                    }
                }
                let expression = format!("!({value})");
                if let Some(result) = self.static_bool_not(&value) {
                    self.known_bools.insert(expression.clone(), result);
                }
                Ok(CValue::BoolExpr(expression))
            }
            CValue::Int(value) => {
                let Some(truthy) = known_integer_truthiness(&self.known_integer_values(&value))
                else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                Ok(CValue::Bool(!truthy))
            }
            CValue::Float(value) => {
                let Some(truthy) = known_float_truthiness(&self.known_float_values(&value)) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                Ok(CValue::Bool(!truthy))
            }
            CValue::String(value) => Ok(CValue::Bool(!php_string_truthy(&value))),
            CValue::StringExpr(value) => {
                let Some(values) = self.known_string_values(&value) else {
                    return Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION));
                };
                match known_string_truthiness(&values) {
                    Some(value) => Ok(CValue::Bool(!value)),
                    None => Err(self.unsupported(span, ASSEMBLY_UNARY_REJECTION)),
                }
            }
            CValue::Null => Ok(CValue::Bool(true)),
        }
    }

    fn static_bool_not(&self, value: &str) -> Option<KnownBool> {
        let value = self.known_bool_values(value)?;
        KnownBool::from_values(value.values().iter().map(|value| !value))
    }

    fn static_float_negate(&self, value: &str) -> Option<KnownFloat> {
        let value = self.known_float_values(value)?;
        let mut results = Vec::new();
        for value in value.values() {
            let result = -value;
            if !result.is_finite() {
                return None;
            }
            results.push(result);
        }
        KnownFloat::from_values(results)
    }

    fn emit_echo(&mut self, value: CValue) {
        match value {
            CValue::Null | CValue::Bool(false) => {}
            CValue::Bool(true) => self.body.push("printf(\"%s\", \"1\");".to_string()),
            CValue::BoolExpr(value) => self
                .body
                .push(format!("if ({value}) {{ printf(\"%s\", \"1\"); }}")),
            CValue::Int(value) => self.body.push(format!("printf(\"%lld\", {value});")),
            CValue::Float(value) => self.body.push(format!("printf(\"%g\", {value});")),
            CValue::String(value) => self
                .body
                .push(format!("printf(\"%s\", \"{}\");", c_string(&value))),
            CValue::StringExpr(value) => self.body.push(format!("printf(\"%s\", {value});")),
        }
    }

    fn unsupported(&self, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Codegen, span.line, span.column, message)
    }
}

fn llvm_c_string(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'\\' => escaped.push_str("\\5C"),
            b'"' => escaped.push_str("\\22"),
            b'\n' => escaped.push_str("\\0A"),
            b'\r' => escaped.push_str("\\0D"),
            b'\t' => escaped.push_str("\\09"),
            0x20..=0x7e => escaped.push(*byte as char),
            other => escaped.push_str(&format!("\\{other:02X}")),
        }
    }
    escaped.push_str("\\00");
    escaped
}

fn c_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_ascii_graphic() || ch == ' ' => escaped.push(ch),
            ch => escaped.push_str(&format!("\\x{:02X}", ch as u32)),
        }
    }
    escaped
}

fn is_comparison_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::StrictEq
            | BinaryOp::StrictNe
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge
    )
}

fn is_empty_string_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::String(value, _) if value.is_empty())
}

fn same_direct_variable_expr(left: &Expr, right: &Expr) -> bool {
    matches!((left, right), (Expr::Variable(left, _), Expr::Variable(right, _)) if left == right)
}

fn same_direct_variable_ternary_expr(condition: &Expr, if_true: &Expr, if_false: &Expr) -> bool {
    same_direct_variable_expr(condition, if_true) && same_direct_variable_expr(condition, if_false)
}

fn llvm_comparison_rejection() -> &'static str {
    "LLVM comparison lowering rejects unsupported comparison operands until native PHP comparison coercions exist; same-type null, boolean, integer, finite float, known ASCII nonnumeric string comparisons, and identical string-pointer self-comparisons are lowered for the current native subset; phpc run handles current scalar comparison diagnostics"
}

fn assembly_comparison_rejection() -> &'static str {
    "assembly comparison lowering rejects unsupported comparison operands until native PHP comparison coercions exist; same-type null, boolean, integer, finite float, known ASCII nonnumeric string comparisons, and identical string-pointer self-comparisons are lowered for the current native subset; phpc run handles current scalar comparison diagnostics"
}

fn c_comparison_operator(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => return None,
    })
}

fn identical_string_ternary_branch(if_true: &IrValue, if_false: &IrValue) -> Option<IrValue> {
    match (if_true, if_false) {
        (IrValue::String(left), IrValue::String(right)) if left == right => {
            Some(IrValue::String(left.clone()))
        }
        (IrValue::StringPtr(left), IrValue::StringPtr(right)) if left == right => {
            Some(IrValue::StringPtr(left.clone()))
        }
        _ => None,
    }
}

fn identical_bool_expr_ternary_branch(if_true: &IrValue, if_false: &IrValue) -> Option<IrValue> {
    match (if_true, if_false) {
        (IrValue::BoolExpr(left), IrValue::BoolExpr(right)) if left == right => {
            Some(IrValue::BoolExpr(left.clone()))
        }
        _ => None,
    }
}

fn identical_c_string_ternary_branch(if_true: &CValue, if_false: &CValue) -> Option<CValue> {
    match (if_true, if_false) {
        (CValue::String(left), CValue::String(right)) if left == right => {
            Some(CValue::String(left.clone()))
        }
        (CValue::StringExpr(left), CValue::StringExpr(right)) if left == right => {
            Some(CValue::StringExpr(left.clone()))
        }
        _ => None,
    }
}

fn identical_c_bool_expr_ternary_branch(if_true: &CValue, if_false: &CValue) -> Option<CValue> {
    match (if_true, if_false) {
        (CValue::BoolExpr(left), CValue::BoolExpr(right)) if left == right => {
            Some(CValue::BoolExpr(left.clone()))
        }
        _ => None,
    }
}

enum BoolLiteralTernaryBranch {
    Static(bool),
    Reuse(String),
    Invert(String),
}

fn bool_literal_ternary_branch(
    condition: &str,
    if_true: &IrValue,
    if_false: &IrValue,
) -> Option<BoolLiteralTernaryBranch> {
    match (if_true, if_false) {
        (IrValue::Bool(true), IrValue::Bool(true)) => Some(BoolLiteralTernaryBranch::Static(true)),
        (IrValue::Bool(false), IrValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Static(false))
        }
        (IrValue::Bool(true), IrValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Reuse(condition.to_string()))
        }
        (IrValue::Bool(false), IrValue::Bool(true)) => {
            Some(BoolLiteralTernaryBranch::Invert(condition.to_string()))
        }
        _ => None,
    }
}

fn c_bool_literal_ternary_branch(
    condition: &str,
    if_true: &CValue,
    if_false: &CValue,
) -> Option<BoolLiteralTernaryBranch> {
    match (if_true, if_false) {
        (CValue::Bool(true), CValue::Bool(true)) => Some(BoolLiteralTernaryBranch::Static(true)),
        (CValue::Bool(false), CValue::Bool(false)) => Some(BoolLiteralTernaryBranch::Static(false)),
        (CValue::Bool(true), CValue::Bool(false)) => {
            Some(BoolLiteralTernaryBranch::Reuse(condition.to_string()))
        }
        (CValue::Bool(false), CValue::Bool(true)) => {
            Some(BoolLiteralTernaryBranch::Invert(condition.to_string()))
        }
        _ => None,
    }
}

fn llvm_string_comparison_predicate(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::StrictEq => "eq",
        BinaryOp::Ne | BinaryOp::StrictNe => "ne",
        BinaryOp::Lt => "slt",
        BinaryOp::Le => "sle",
        BinaryOp::Gt => "sgt",
        BinaryOp::Ge => "sge",
        _ => return None,
    })
}

fn c_string_comparison_operator(op: BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::StrictEq => "==",
        BinaryOp::Ne | BinaryOp::StrictNe => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => return None,
    })
}

fn known_strings_are_safe_for_native_comparison(values: &KnownString) -> bool {
    values.values().iter().all(|value| {
        value.bytes().all(|byte| byte.is_ascii() && byte != 0)
            && !string_looks_numeric_for_native_comparison(value)
    })
}

fn string_looks_numeric_for_native_comparison(value: &str) -> bool {
    let first = value.bytes().find(|byte| !byte.is_ascii_whitespace());
    matches!(first, Some(b'+' | b'-' | b'.' | b'0'..=b'9'))
}

fn string_comparison_result_for_known_values(
    left_values: &KnownString,
    op: BinaryOp,
    right_values: &KnownString,
) -> Option<bool> {
    let mut result = None;
    for left in left_values.values() {
        for right in right_values.values() {
            let ordering = left.cmp(right);
            let current = match op {
                BinaryOp::Eq => ordering.is_eq(),
                BinaryOp::Ne => !ordering.is_eq(),
                BinaryOp::Lt => ordering.is_lt(),
                BinaryOp::Le => ordering.is_lt() || ordering.is_eq(),
                BinaryOp::Gt => ordering.is_gt(),
                BinaryOp::Ge => ordering.is_gt() || ordering.is_eq(),
                _ => return None,
            };
            if result.is_some_and(|result| result != current) {
                return None;
            }
            result = Some(current);
        }
    }
    result
}

fn static_safe_string_comparison_result(
    left_values: Option<KnownString>,
    op: BinaryOp,
    right_values: Option<KnownString>,
) -> Option<bool> {
    let left_values = left_values?;
    let right_values = right_values?;
    if !known_strings_are_safe_for_native_comparison(&left_values)
        || !known_strings_are_safe_for_native_comparison(&right_values)
    {
        return None;
    }
    string_comparison_result_for_known_values(&left_values, op, &right_values)
}

fn llvm_logical_rejection() -> &'static str {
    "LLVM logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior"
}

fn assembly_logical_rejection() -> &'static str {
    "assembly logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior"
}

fn static_strict_identity_result(is_identical: bool, op: BinaryOp) -> bool {
    match op {
        BinaryOp::StrictEq => is_identical,
        BinaryOp::StrictNe => !is_identical,
        _ => unreachable!("strict identity helper only accepts strict identity operators"),
    }
}

fn reflexive_string_comparison_result(op: BinaryOp) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::Le | BinaryOp::Ge | BinaryOp::StrictEq => true,
        BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::StrictNe => false,
        _ => return None,
    })
}

enum BoolLiteralComparisonFold {
    Static(bool),
    Reuse(String),
    Invert(String),
}

fn bool_literal_comparison_fold(
    left: &str,
    op: BinaryOp,
    right: &str,
    true_literal: &str,
    false_literal: &str,
) -> Option<BoolLiteralComparisonFold> {
    let left_literal = bool_literal_value(left, true_literal, false_literal);
    let right_literal = bool_literal_value(right, true_literal, false_literal);
    match (left_literal, right_literal) {
        (Some(_), Some(_)) | (None, None) => return None,
        (None, Some(literal)) => bool_comparison_with_right_literal_fold(left, op, literal),
        (Some(literal), None) => bool_comparison_with_left_literal_fold(literal, op, right),
    }
}

fn bool_comparison_with_right_literal_fold(
    dynamic: &str,
    op: BinaryOp,
    literal: bool,
) -> Option<BoolLiteralComparisonFold> {
    match (op, literal) {
        (BinaryOp::Eq, true)
        | (BinaryOp::Ne, false)
        | (BinaryOp::Gt, false)
        | (BinaryOp::Ge, true) => Some(BoolLiteralComparisonFold::Reuse(dynamic.to_string())),
        (BinaryOp::Eq, false)
        | (BinaryOp::Ne, true)
        | (BinaryOp::Lt, true)
        | (BinaryOp::Le, false) => Some(BoolLiteralComparisonFold::Invert(dynamic.to_string())),
        (BinaryOp::Le, true) | (BinaryOp::Ge, false) => {
            Some(BoolLiteralComparisonFold::Static(true))
        }
        (BinaryOp::Lt, false) | (BinaryOp::Gt, true) => {
            Some(BoolLiteralComparisonFold::Static(false))
        }
        _ => None,
    }
}

fn bool_comparison_with_left_literal_fold(
    literal: bool,
    op: BinaryOp,
    dynamic: &str,
) -> Option<BoolLiteralComparisonFold> {
    match (op, literal) {
        (BinaryOp::Eq, true)
        | (BinaryOp::Ne, false)
        | (BinaryOp::Lt, false)
        | (BinaryOp::Le, true) => Some(BoolLiteralComparisonFold::Reuse(dynamic.to_string())),
        (BinaryOp::Eq, false)
        | (BinaryOp::Ne, true)
        | (BinaryOp::Gt, true)
        | (BinaryOp::Ge, false) => Some(BoolLiteralComparisonFold::Invert(dynamic.to_string())),
        (BinaryOp::Le, false) | (BinaryOp::Ge, true) => {
            Some(BoolLiteralComparisonFold::Static(true))
        }
        (BinaryOp::Lt, true) | (BinaryOp::Gt, false) => {
            Some(BoolLiteralComparisonFold::Static(false))
        }
        _ => None,
    }
}

fn bool_literal_value(value: &str, true_literal: &str, false_literal: &str) -> Option<bool> {
    if value == true_literal {
        Some(true)
    } else if value == false_literal {
        Some(false)
    } else {
        None
    }
}

fn null_comparison_result(op: BinaryOp) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq | BinaryOp::Le | BinaryOp::Ge => true,
        BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt => false,
        _ => return None,
    })
}

fn bool_comparison_result(left: bool, op: BinaryOp, right: bool) -> Option<bool> {
    let left = u8::from(left);
    let right = u8::from(right);
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn integer_comparison_result(left: i64, op: BinaryOp, right: i64) -> Option<bool> {
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn float_comparison_result(left: f64, op: BinaryOp, right: f64) -> Option<bool> {
    if !left.is_finite() || !right.is_finite() {
        return None;
    }
    Some(match op {
        BinaryOp::Eq => left == right,
        BinaryOp::Ne => left != right,
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => return None,
    })
}

fn llvm_bool_operand(value: IrValue) -> Option<String> {
    match value {
        IrValue::Bool(true) => Some("true".to_string()),
        IrValue::Bool(false) => Some("false".to_string()),
        IrValue::BoolExpr(value) => Some(value),
        _ => None,
    }
}

fn c_bool_operand(value: CValue) -> Option<String> {
    match value {
        CValue::Bool(true) => Some("1".to_string()),
        CValue::Bool(false) => Some("0".to_string()),
        CValue::BoolExpr(value) => Some(value),
        _ => None,
    }
}

fn c_string_operand(value: CValue) -> String {
    match value {
        CValue::String(value) => format!("\"{}\"", c_string(&value)),
        CValue::StringExpr(value) => value,
        _ => unreachable!("string operands are prefiltered"),
    }
}

fn php_string_truthy(value: &str) -> bool {
    !value.is_empty() && value != "0"
}

fn logical_truthiness_result(left: bool, op: BinaryOp, right: bool) -> CompileResult<bool> {
    Ok(match op {
        BinaryOp::LogicalAnd => left && right,
        BinaryOp::LogicalOr => left || right,
        BinaryOp::LogicalXor => left ^ right,
        _ => unreachable!("logical operands are prefiltered"),
    })
}

fn known_integer_truthiness(values: &Option<KnownInt>) -> Option<bool> {
    let values = values.as_ref()?;
    known_truthiness(values.values().iter().map(|value| *value != 0))
}

fn known_float_truthiness(values: &Option<KnownFloat>) -> Option<bool> {
    let values = values.as_ref()?;
    if !values.values().iter().all(|value| value.is_finite()) {
        return None;
    }
    known_truthiness(values.values().iter().map(|value| *value != 0.0))
}

fn known_string_truthiness(values: &KnownString) -> Option<bool> {
    known_truthiness(values.values().iter().map(|value| php_string_truthy(value)))
}

fn known_truthiness(values: impl IntoIterator<Item = bool>) -> Option<bool> {
    let mut result = None;
    for current in values {
        if result.is_some_and(|result| result != current) {
            return None;
        }
        result = Some(current);
    }
    result
}

fn is_global_constant_builtin(name: &str) -> bool {
    name.eq_ignore_ascii_case("define")
        || name.eq_ignore_ascii_case("constant")
        || name.eq_ignore_ascii_case("defined")
}

fn is_object_metadata_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "get_class"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "get_declared_classes"
            | "get_declared_interfaces"
            | "get_declared_traits"
            | "get_called_class"
            | "spl_object_id"
            | "spl_object_hash"
            | "property_exists"
            | "method_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_object_vars"
            | "get_mangled_object_vars"
            | "is_a"
            | "is_subclass_of"
            | "get_parent_class"
    )
}

fn is_array_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "count"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "array_is_list"
            | "array_keys"
            | "array_reverse"
            | "array_slice"
            | "array_chunk"
            | "array_pad"
            | "array_merge"
            | "array_replace"
            | "array_flip"
            | "array_change_key_case"
            | "array_column"
            | "array_fill_keys"
            | "array_combine"
            | "array_intersect_key"
            | "array_diff_key"
            | "array_diff"
            | "array_intersect"
            | "array_unique"
            | "array_count_values"
            | "array_sum"
            | "array_product"
            | "array_reduce"
            | "array_filter"
            | "array_map"
            | "in_array"
            | "array_search"
    )
}

fn is_native_type_introspection_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "gettype"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_array"
            | "is_scalar"
            | "is_numeric"
            | "is_countable"
            | "is_iterable"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "property_exists"
            | "method_exists"
            | "is_a"
            | "is_subclass_of"
    )
}

fn is_native_metadata_exists_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "class_exists" | "interface_exists" | "trait_exists" | "enum_exists"
    )
}

fn is_native_member_metadata_exists_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "property_exists" | "method_exists"
    )
}

fn is_native_relationship_metadata_builtin(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "is_a" | "is_subclass_of"
    )
}

fn known_strings_have_uniform_function_exists_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = is_native_known_function_name(value);
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn known_strings_have_uniform_byte_length(values: &KnownString) -> Option<usize> {
    let mut result = None;
    for value in values.values() {
        let current = value.len();
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn known_strings_have_uniform_defined_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = native_defined_result(value)?;
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn native_defined_result(name: &str) -> Option<bool> {
    if !is_supported_native_constant_name(name) {
        return None;
    }

    Some(builtin_global_constant_value(name).is_some())
}

fn builtin_global_constant_value(name: &str) -> Option<i64> {
    match name {
        "CASE_LOWER" => Some(0),
        "CASE_UPPER" => Some(1),
        "ARRAY_FILTER_USE_BOTH" => Some(1),
        "ARRAY_FILTER_USE_KEY" => Some(2),
        "SORT_REGULAR" => Some(0),
        "SORT_NUMERIC" => Some(1),
        "SORT_STRING" => Some(2),
        _ => None,
    }
}

fn is_supported_native_constant_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn is_native_known_function_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "define"
            | "strlen"
            | "count"
            | "constant"
            | "defined"
            | "array_key_exists"
            | "array_values"
            | "array_key_first"
            | "array_key_last"
            | "array_is_list"
            | "array_keys"
            | "array_reverse"
            | "array_slice"
            | "array_chunk"
            | "array_pad"
            | "array_merge"
            | "array_replace"
            | "array_flip"
            | "array_change_key_case"
            | "array_column"
            | "array_fill_keys"
            | "array_combine"
            | "array_intersect_key"
            | "array_diff_key"
            | "array_diff"
            | "array_intersect"
            | "array_unique"
            | "array_count_values"
            | "array_sum"
            | "array_product"
            | "array_reduce"
            | "array_filter"
            | "array_map"
            | "in_array"
            | "array_search"
            | "gettype"
            | "is_null"
            | "is_bool"
            | "is_int"
            | "is_integer"
            | "is_long"
            | "is_float"
            | "is_double"
            | "is_string"
            | "is_array"
            | "is_scalar"
            | "is_numeric"
            | "is_countable"
            | "is_iterable"
            | "is_callable"
            | "function_exists"
            | "get_class"
            | "is_object"
            | "get_debug_type"
            | "class_exists"
            | "interface_exists"
            | "trait_exists"
            | "enum_exists"
            | "get_declared_classes"
            | "get_declared_interfaces"
            | "get_declared_traits"
            | "get_called_class"
            | "spl_object_id"
            | "spl_object_hash"
            | "property_exists"
            | "method_exists"
            | "get_class_methods"
            | "get_class_vars"
            | "get_object_vars"
            | "get_mangled_object_vars"
            | "is_a"
            | "is_subclass_of"
            | "get_parent_class"
            | "var_dump"
            | "print_r"
    )
}

fn known_strings_have_uniform_numeric_result(values: &KnownString) -> Option<bool> {
    let mut result = None;
    for value in values.values() {
        let current = is_php_numeric_string_literal(value);
        if let Some(previous) = result {
            if previous != current {
                return None;
            }
        } else {
            result = Some(current);
        }
    }
    result
}

fn is_php_numeric_string_literal(value: &str) -> bool {
    let trimmed = value.trim_matches(|ch: char| ch.is_ascii_whitespace());
    !trimmed.is_empty() && is_well_formed_php_numeric_string(trimmed)
}

fn is_well_formed_php_numeric_string(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }

    let digits_before_decimal = consume_ascii_digits(bytes, &mut index);
    if matches!(bytes.get(index), Some(b'.')) {
        index += 1;
        let digits_after_decimal = consume_ascii_digits(bytes, &mut index);
        if digits_before_decimal == 0 && digits_after_decimal == 0 {
            return false;
        }
    } else if digits_before_decimal == 0 {
        return false;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) == 0 {
            return false;
        }
    }

    index == bytes.len()
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while matches!(bytes.get(*index), Some(b'0'..=b'9')) {
        *index += 1;
    }
    *index - start
}

fn llvm_gettype_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "NULL",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "boolean",
        IrValue::Int(_) => "integer",
        IrValue::Float(_) => "double",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
    }
}

fn llvm_debug_type_name(value: &IrValue) -> &'static str {
    match value {
        IrValue::Null => "null",
        IrValue::Bool(_) | IrValue::BoolExpr(_) => "bool",
        IrValue::Int(_) => "int",
        IrValue::Float(_) => "float",
        IrValue::String(_) | IrValue::StringPtr(_) => "string",
    }
}

fn c_gettype_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "NULL",
        CValue::Bool(_) | CValue::BoolExpr(_) => "boolean",
        CValue::Int(_) => "integer",
        CValue::Float(_) => "double",
        CValue::String(_) | CValue::StringExpr(_) => "string",
    }
}

fn c_debug_type_name(value: &CValue) -> &'static str {
    match value {
        CValue::Null => "null",
        CValue::Bool(_) | CValue::BoolExpr(_) => "bool",
        CValue::Int(_) => "int",
        CValue::Float(_) => "float",
        CValue::String(_) | CValue::StringExpr(_) => "string",
    }
}

fn format_float_literal(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
