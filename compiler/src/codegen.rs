use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::ast::{AssignTarget, BinaryOp, Expr, Program, Span, Stmt, UnaryOp};
use crate::error::{CompileResult, Diagnostic, Phase};

const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects ternary and null coalescing expressions until native PHP truthiness, null-aware lookup, and branch side-effect ordering exist; phpc run handles current conditional expression behavior";
const ASSEMBLY_CONDITIONAL_REJECTION: &str = "assembly conditional lowering rejects ternary and null coalescing expressions until native PHP truthiness, null-aware lookup, and branch side-effect ordering exist; phpc run handles current conditional expression behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const ASSEMBLY_FUNCTION_CALL_REJECTION: &str = "assembly function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_FUNCTION_DECLARATION_REJECTION: &str = "LLVM user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const ASSEMBLY_FUNCTION_DECLARATION_REJECTION: &str = "assembly user-function lowering rejects function declarations and return statements until native function symbol tables, stack-frame layout, default parameter binding, recursion guards, return-value flow, and exact native error behavior exist; phpc run handles current user-function declaration and return behavior";
const LLVM_MAGIC_CONSTANT_REJECTION: &str = "LLVM magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, and __FUNCTION__ until native source mapping, path canonicalization, and function-context lowering exist; phpc run handles current magic constant behavior";
const ASSEMBLY_MAGIC_CONSTANT_REJECTION: &str = "assembly magic-constant lowering rejects executable magic constants __LINE__, __FILE__, __DIR__, and __FUNCTION__ until native source mapping, path canonicalization, and function-context lowering exist; phpc run handles current magic constant behavior";
const LLVM_GLOBAL_CONSTANT_REJECTION: &str = "LLVM global-constant lowering rejects built-in constants, runtime-defined constants, bare constant reads, top-level const declarations, and define()/constant()/defined() until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const ASSEMBLY_GLOBAL_CONSTANT_REJECTION: &str = "assembly global-constant lowering rejects built-in constants, runtime-defined constants, bare constant reads, top-level const declarations, and define()/constant()/defined() until native constant tables, source-order definitions, namespace-aware lookup, and exact native error behavior exist; phpc run handles current global constant behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, object instantiation, public property reads/writes, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const ASSEMBLY_OBJECT_CLASS_REJECTION: &str = "assembly object/class lowering rejects class declarations, object instantiation, public property reads/writes, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const ASSEMBLY_ARRAY_REJECTION: &str = "assembly array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const ASSEMBLY_CONTROL_FLOW_REJECTION: &str = "assembly control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";

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

#[derive(Default)]
struct LlvmGenerator {
    strings: Vec<(String, String)>,
    body: Vec<String>,
    variables: HashMap<String, IrValue>,
    next_temp: usize,
    next_string: usize,
}

#[derive(Debug, Clone)]
enum IrValue {
    Int(String),
    Float(String),
    String(String),
    Bool(bool),
    Null,
}

impl LlvmGenerator {
    fn emit_program(&mut self, program: &Program) -> CompileResult<String> {
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        let mut output = String::new();
        output.push_str("; generated by phpc milestone 1\n");
        output.push_str("declare i32 @printf(ptr, ...)\n\n");
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
            Stmt::CompoundAssign { span, .. } => Err(self.unsupported(
                *span,
                "compound assignment is supported by phpc run for direct static variables, direct array offsets, and direct object properties but not LLVM IR emission yet",
            )),
            Stmt::IncrementDecrement { span, .. } => Err(self.unsupported(
                *span,
                "increment/decrement is supported by phpc run for direct static int/float variables, direct array int/float offsets, and direct object int/float properties but not LLVM IR emission yet",
            )),
            Stmt::NullCoalesceAssign { span, .. } => Err(self.unsupported(
                *span,
                "null coalescing assignment is supported by phpc run for direct variables, direct array offsets, and direct object properties but not LLVM IR emission yet",
            )),
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => Err(self.unsupported(
                function.span,
                LLVM_FUNCTION_DECLARATION_REJECTION,
            )),
            Stmt::Class(class) => Err(self.unsupported(
                class.span,
                LLVM_OBJECT_CLASS_REJECTION,
            )),
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Break { span }
            | Stmt::Continue { span } => {
                Err(self.unsupported(*span, LLVM_CONTROL_FLOW_REJECTION))
            }
            Stmt::Foreach { span, .. } => Err(self.unsupported(*span, LLVM_ARRAY_REJECTION)),
            Stmt::UnsetVariable { span, .. } => Err(self.unsupported(
                *span,
                "variable unset is supported by phpc run but not LLVM IR emission yet",
            )),
            Stmt::UnsetArrayIndex { span, .. } => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { span, .. } => Err(self.unsupported(
                *span,
                "multiple-operand unset is supported by phpc run but not LLVM IR emission yet",
            )),
            Stmt::ConstDeclaration { span, .. } => Err(self.unsupported(
                *span,
                LLVM_GLOBAL_CONSTANT_REJECTION,
            )),
            Stmt::Return { span, .. } => Err(self.unsupported(
                *span,
                LLVM_FUNCTION_DECLARATION_REJECTION,
            )),
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
            Expr::Property { span, .. } => Err(self.unsupported(
                *span,
                LLVM_OBJECT_CLASS_REJECTION,
            )),
            Expr::Variable(name, span) => self.variables.get(name).cloned().ok_or_else(|| {
                self.unsupported(
                    *span,
                    format!("variable '${name}' is not known in LLVM lowering"),
                )
            }),
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(
                    *span,
                    LLVM_GLOBAL_CONSTANT_REJECTION,
                ))
            }
            Expr::Call { name, span, .. } if is_object_metadata_builtin(name) => {
                Err(self.unsupported(*span, LLVM_OBJECT_CLASS_REJECTION))
            }
            Expr::Call { name, span, .. } if is_array_builtin(name) => {
                Err(self.unsupported(*span, LLVM_ARRAY_REJECTION))
            }
            Expr::Call { span, .. } | Expr::DynamicCall { span, .. } => Err(self.unsupported(
                *span,
                LLVM_FUNCTION_CALL_REJECTION,
            )),
            Expr::New { span, .. } => Err(self.unsupported(
                *span,
                LLVM_OBJECT_CLASS_REJECTION,
            )),
            Expr::Unary { op, span, .. } => {
                if matches!(op, UnaryOp::BitwiseNot) {
                    return Err(self.unsupported(
                        *span,
                        "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                Err(self.unsupported(
                    *span,
                    "unary expressions are supported by phpc run but not LLVM IR emission yet",
                ))
            }
            Expr::Assign { span, .. } => Err(self.unsupported(
                *span,
                "assignment expressions are supported by phpc run for direct static variables, direct array offsets, direct append offsets, and direct object properties but not LLVM IR emission yet",
            )),
            Expr::CompoundAssign { span, .. } => Err(self.unsupported(
                *span,
                "compound assignment expressions are supported by phpc run for direct static variables, direct array offsets, and direct object properties but not LLVM IR emission yet",
            )),
            Expr::NullCoalesceAssign { span, .. } => Err(self.unsupported(
                *span,
                "null coalescing assignment expressions are supported by phpc run for direct variables, direct array offsets, and direct object properties but not LLVM IR emission yet",
            )),
            Expr::IncrementDecrement { span, .. } => Err(self.unsupported(
                *span,
                "increment/decrement expressions are supported by phpc run for direct static int/float variables, direct array int/float offsets, and direct object int/float properties but not LLVM IR emission yet",
            )),
            Expr::Ternary { span, .. } | Expr::ShortTernary { span, .. } => Err(self.unsupported(
                *span,
                LLVM_CONDITIONAL_REJECTION,
            )),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) {
                    return Err(self.unsupported(
                        *span,
                        "LLVM comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics",
                    ));
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.unsupported(
                        *span,
                        LLVM_CONDITIONAL_REJECTION,
                    ));
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return Err(self.unsupported(
                        *span,
                        "LLVM logical lowering rejects logical operators until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior",
                    ));
                }
                if is_bitwise_or_shift_op(*op) {
                    return Err(self.unsupported(
                        *span,
                        "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
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
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                self.emit_numeric_binary(left, op, right, span)
            }
            BinaryOp::Div => self.emit_div(left, right, span),
            BinaryOp::Mod => self.emit_mod(left, right, span),
            BinaryOp::Concat => self.emit_concat(left, right, span),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::StrictEq
            | BinaryOp::StrictNe
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Err(self.unsupported(
                span,
                "LLVM comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics",
            )),
            BinaryOp::NullCoalesce => Err(self.unsupported(
                span,
                LLVM_CONDITIONAL_REJECTION,
            )),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => Err(self.unsupported(
                span,
                "LLVM logical lowering rejects logical operators until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior",
            )),
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => Err(self.unsupported(
                span,
                "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
            )),
        }
    }

    fn emit_numeric_binary(
        &mut self,
        left: IrValue,
        op: BinaryOp,
        right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        if matches!(left, IrValue::String(_)) || matches!(right, IrValue::String(_)) {
            return Err(self.unsupported(
                span,
                "LLVM arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        match (left, right) {
            (IrValue::Int(left), IrValue::Int(right)) => {
                let temp = self.temp();
                let instr = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Sub => "sub",
                    BinaryOp::Mul => "mul",
                    _ => unreachable!("caller restricts op"),
                };
                self.body
                    .push(format!("{temp} = {instr} i64 {left}, {right}"));
                Ok(IrValue::Int(temp))
            }
            (left, right) => {
                let left = self.into_float(left, span)?;
                let right = self.into_float(right, span)?;
                let temp = self.temp();
                let instr = match op {
                    BinaryOp::Add => "fadd",
                    BinaryOp::Sub => "fsub",
                    BinaryOp::Mul => "fmul",
                    _ => unreachable!("caller restricts op"),
                };
                self.body
                    .push(format!("{temp} = {instr} double {left}, {right}"));
                Ok(IrValue::Float(temp))
            }
        }
    }

    fn emit_div(&mut self, left: IrValue, right: IrValue, span: Span) -> CompileResult<IrValue> {
        if matches!(left, IrValue::String(_)) || matches!(right, IrValue::String(_)) {
            return Err(self.unsupported(
                span,
                "LLVM arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        match classify_ir_divisor(&right) {
            NativeDivisorStatus::KnownZero => {
                return Err(self.unsupported(
                    span,
                    "LLVM division lowering rejects statically known division by zero; phpc run reports a runtime diagnostic",
                ));
            }
            NativeDivisorStatus::Dynamic => {
                return Err(self.unsupported(
                    span,
                    "LLVM division lowering rejects dynamic divisors until native runtime zero checks exist; phpc run handles runtime division diagnostics",
                ));
            }
            NativeDivisorStatus::KnownNonZero | NativeDivisorStatus::UnsupportedCoercion => {}
        }
        let left = self.into_float(left, span)?;
        let right = self.into_float(right, span)?;
        let temp = self.temp();
        self.body
            .push(format!("{temp} = fdiv double {left}, {right}"));
        Ok(IrValue::Float(temp))
    }

    fn emit_mod(&mut self, left: IrValue, right: IrValue, span: Span) -> CompileResult<IrValue> {
        if matches!(left, IrValue::String(_)) || matches!(right, IrValue::String(_)) {
            return Err(self.unsupported(
                span,
                "LLVM arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        match (left, right) {
            (IrValue::Int(left), IrValue::Int(right)) => {
                let divisor = right.parse::<i64>().map_err(|_| {
                    self.unsupported(
                        span,
                        "LLVM modulo lowering requires an integer divisor known at compile time",
                    )
                })?;
                if divisor == 0 {
                    return Err(self.unsupported(
                        span,
                        "LLVM modulo lowering rejects modulo by zero; phpc run reports a runtime diagnostic",
                    ));
                }
                let temp = self.temp();
                self.body.push(format!("{temp} = srem i64 {left}, {right}"));
                Ok(IrValue::Int(temp))
            }
            _ => Err(self.unsupported(
                span,
                "LLVM modulo lowering currently requires integer operands; phpc run handles the broader int-coercion subset",
            )),
        }
    }

    fn emit_concat(&mut self, left: IrValue, right: IrValue, span: Span) -> CompileResult<IrValue> {
        let left = self.const_echo_string(left).ok_or_else(|| {
            self.unsupported(
                span,
                "LLVM concat currently requires compile-time scalar operands",
            )
        })?;
        let right = self.const_echo_string(right).ok_or_else(|| {
            self.unsupported(
                span,
                "LLVM concat currently requires compile-time scalar operands",
            )
        })?;
        Ok(IrValue::String(format!("{left}{right}")))
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
        }
    }

    fn into_float(&mut self, value: IrValue, span: Span) -> CompileResult<String> {
        match value {
            IrValue::Float(value) => Ok(value),
            IrValue::Int(value) => {
                let temp = self.temp();
                self.body
                    .push(format!("{temp} = sitofp i64 {value} to double"));
                Ok(temp)
            }
            IrValue::Bool(value) => Ok(if value {
                "1.0".to_string()
            } else {
                "0.0".to_string()
            }),
            IrValue::Null => Ok("0.0".to_string()),
            IrValue::String(_) => Err(self.unsupported(
                span,
                "LLVM arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            )),
        }
    }

    fn const_echo_string(&self, value: IrValue) -> Option<String> {
        match value {
            IrValue::Null => Some(String::new()),
            IrValue::Bool(false) => Some(String::new()),
            IrValue::Bool(true) => Some("1".to_string()),
            IrValue::Int(value) if value.parse::<i64>().is_ok() => Some(value),
            IrValue::Float(value) if value.parse::<f64>().is_ok() => Some(value),
            IrValue::String(value) => Some(value),
            IrValue::Int(_) | IrValue::Float(_) => None,
        }
    }

    fn add_string(&mut self, value: &str) -> String {
        let name = format!(".str.{}", self.next_string);
        self.next_string += 1;
        self.strings.push((name.clone(), value.to_string()));
        name
    }

    fn temp(&mut self) -> String {
        let temp = format!("%t{}", self.next_temp);
        self.next_temp += 1;
        temp
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("clang failed to emit assembly: {}", stderr.trim()),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("llc failed to emit assembly: {}", stderr.trim()),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Diagnostic::new(
            Phase::Codegen,
            0,
            0,
            format!("cc failed to emit assembly: {}", stderr.trim()),
        ));
    }

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
    next_temp: usize,
}

#[derive(Debug, Clone)]
enum CValue {
    Int(String),
    Float(String),
    String(String),
    Bool(bool),
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
            Stmt::CompoundAssign { span, .. } => Err(self.unsupported(
                *span,
                "compound assignment is supported by phpc run for direct static variables, direct array offsets, and direct object properties but not assembly emission yet",
            )),
            Stmt::IncrementDecrement { span, .. } => Err(self.unsupported(
                *span,
                "increment/decrement is supported by phpc run for direct static int/float variables, direct array int/float offsets, and direct object int/float properties but not assembly emission yet",
            )),
            Stmt::NullCoalesceAssign { span, .. } => Err(self.unsupported(
                *span,
                "null coalescing assignment is supported by phpc run for direct variables, direct array offsets, and direct object properties but not assembly emission yet",
            )),
            Stmt::Expr { expr, .. } => {
                self.emit_expr(expr)?;
                Ok(())
            }
            Stmt::Function(function) => Err(self.unsupported(
                function.span,
                ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
            )),
            Stmt::Class(class) => Err(self.unsupported(
                class.span,
                ASSEMBLY_OBJECT_CLASS_REJECTION,
            )),
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
            Stmt::UnsetVariable { span, .. } => Err(self.unsupported(
                *span,
                "variable unset is supported by phpc run but not assembly emission yet",
            )),
            Stmt::UnsetArrayIndex { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Stmt::UnsetMany { span, .. } => Err(self.unsupported(
                *span,
                "multiple-operand unset is supported by phpc run but not assembly emission yet",
            )),
            Stmt::ConstDeclaration { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_GLOBAL_CONSTANT_REJECTION,
            )),
            Stmt::Return { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_FUNCTION_DECLARATION_REJECTION,
            )),
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
            Expr::Property { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_OBJECT_CLASS_REJECTION,
            )),
            Expr::Variable(name, span) => self.variables.get(name).cloned().ok_or_else(|| {
                self.unsupported(
                    *span,
                    format!("variable '${name}' is not known in assembly lowering"),
                )
            }),
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
            }
            Expr::Call { name, span, .. } if is_object_metadata_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_OBJECT_CLASS_REJECTION))
            }
            Expr::Call { name, span, .. } if is_array_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_ARRAY_REJECTION))
            }
            Expr::Call { span, .. } | Expr::DynamicCall { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_FUNCTION_CALL_REJECTION,
            )),
            Expr::New { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_OBJECT_CLASS_REJECTION,
            )),
            Expr::Unary { op, span, .. } => {
                if matches!(op, UnaryOp::BitwiseNot) {
                    return Err(self.unsupported(
                        *span,
                        "assembly bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                Err(self.unsupported(
                    *span,
                    "unary expressions are supported by phpc run but not assembly emission yet",
                ))
            }
            Expr::Assign { span, .. } => Err(self.unsupported(
                *span,
                "assignment expressions are supported by phpc run for direct static variables, direct array offsets, direct append offsets, and direct object properties but not assembly emission yet",
            )),
            Expr::CompoundAssign { span, .. } => Err(self.unsupported(
                *span,
                "compound assignment expressions are supported by phpc run for direct static variables, direct array offsets, and direct object properties but not assembly emission yet",
            )),
            Expr::NullCoalesceAssign { span, .. } => Err(self.unsupported(
                *span,
                "null coalescing assignment expressions are supported by phpc run for direct variables, direct array offsets, and direct object properties but not assembly emission yet",
            )),
            Expr::IncrementDecrement { span, .. } => Err(self.unsupported(
                *span,
                "increment/decrement expressions are supported by phpc run for direct static int/float variables, direct array int/float offsets, and direct object int/float properties but not assembly emission yet",
            )),
            Expr::Ternary { span, .. } | Expr::ShortTernary { span, .. } => Err(self.unsupported(
                *span,
                ASSEMBLY_CONDITIONAL_REJECTION,
            )),
            Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                if is_comparison_op(*op) {
                    return Err(self.unsupported(
                        *span,
                        "assembly comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics",
                    ));
                }
                if matches!(op, BinaryOp::NullCoalesce) {
                    return Err(self.unsupported(
                        *span,
                        ASSEMBLY_CONDITIONAL_REJECTION,
                    ));
                }
                if matches!(
                    op,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor
                ) {
                    return Err(self.unsupported(
                        *span,
                        "assembly logical lowering rejects logical operators until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior",
                    ));
                }
                if is_bitwise_or_shift_op(*op) {
                    return Err(self.unsupported(
                        *span,
                        "assembly bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                let left = self.emit_expr(left)?;
                let right = self.emit_expr(right)?;
                self.emit_binary(left, *op, right, *span)
            }
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
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                self.emit_numeric_binary(left, op, right, span)
            }
            BinaryOp::Div => self.emit_div(left, right, span),
            BinaryOp::Mod => self.emit_mod(left, right, span),
            BinaryOp::Concat => self.emit_concat(left, right, span),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::StrictEq
            | BinaryOp::StrictNe
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Err(self.unsupported(
                span,
                "assembly comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics",
            )),
            BinaryOp::NullCoalesce => Err(self.unsupported(
                span,
                ASSEMBLY_CONDITIONAL_REJECTION,
            )),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::LogicalXor => Err(self.unsupported(
                span,
                "assembly logical lowering rejects logical operators until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior",
            )),
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => Err(self.unsupported(
                span,
                "assembly bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
            )),
        }
    }

    fn emit_numeric_binary(
        &mut self,
        left: CValue,
        op: BinaryOp,
        right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        if matches!(left, CValue::String(_)) || matches!(right, CValue::String(_)) {
            return Err(self.unsupported(
                span,
                "assembly arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        let operator = match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            _ => unreachable!("caller restricts op"),
        };

        match (left, right) {
            (CValue::Int(left), CValue::Int(right)) => {
                let temp = self.temp();
                self.body
                    .push(format!("long long {temp} = {left} {operator} {right};"));
                Ok(CValue::Int(temp))
            }
            (left, right) => {
                let left = self.into_float(left, span)?;
                let right = self.into_float(right, span)?;
                let temp = self.temp();
                self.body
                    .push(format!("double {temp} = {left} {operator} {right};"));
                Ok(CValue::Float(temp))
            }
        }
    }

    fn emit_div(&mut self, left: CValue, right: CValue, span: Span) -> CompileResult<CValue> {
        if matches!(left, CValue::String(_)) || matches!(right, CValue::String(_)) {
            return Err(self.unsupported(
                span,
                "assembly arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        match classify_c_divisor(&right) {
            NativeDivisorStatus::KnownZero => {
                return Err(self.unsupported(
                    span,
                    "assembly division lowering rejects statically known division by zero; phpc run reports a runtime diagnostic",
                ));
            }
            NativeDivisorStatus::Dynamic => {
                return Err(self.unsupported(
                    span,
                    "assembly division lowering rejects dynamic divisors until native runtime zero checks exist; phpc run handles runtime division diagnostics",
                ));
            }
            NativeDivisorStatus::KnownNonZero | NativeDivisorStatus::UnsupportedCoercion => {}
        }
        let left = self.into_float(left, span)?;
        let right = self.into_float(right, span)?;
        let temp = self.temp();
        self.body.push(format!("double {temp} = {left} / {right};"));
        Ok(CValue::Float(temp))
    }

    fn emit_mod(&mut self, left: CValue, right: CValue, span: Span) -> CompileResult<CValue> {
        if matches!(left, CValue::String(_)) || matches!(right, CValue::String(_)) {
            return Err(self.unsupported(
                span,
                "assembly arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            ));
        }

        match (left, right) {
            (CValue::Int(left), CValue::Int(right)) => {
                let divisor = right.parse::<i64>().map_err(|_| {
                    self.unsupported(
                        span,
                        "assembly modulo lowering requires an integer divisor known at compile time",
                    )
                })?;
                if divisor == 0 {
                    return Err(self.unsupported(
                        span,
                        "assembly modulo lowering rejects modulo by zero; phpc run reports a runtime diagnostic",
                    ));
                }
                let temp = self.temp();
                self.body
                    .push(format!("long long {temp} = {left} % {right};"));
                Ok(CValue::Int(temp))
            }
            _ => Err(self.unsupported(
                span,
                "assembly modulo lowering currently requires integer operands; phpc run handles the broader int-coercion subset",
            )),
        }
    }

    fn emit_concat(&mut self, left: CValue, right: CValue, span: Span) -> CompileResult<CValue> {
        let left = self.const_echo_string(left).ok_or_else(|| {
            self.unsupported(
                span,
                "assembly concat currently requires compile-time scalar operands",
            )
        })?;
        let right = self.const_echo_string(right).ok_or_else(|| {
            self.unsupported(
                span,
                "assembly concat currently requires compile-time scalar operands",
            )
        })?;
        Ok(CValue::String(format!("{left}{right}")))
    }

    fn emit_echo(&mut self, value: CValue) {
        match value {
            CValue::Null | CValue::Bool(false) => {}
            CValue::Bool(true) => self.body.push("printf(\"%s\", \"1\");".to_string()),
            CValue::Int(value) => self.body.push(format!("printf(\"%lld\", {value});")),
            CValue::Float(value) => self.body.push(format!("printf(\"%g\", {value});")),
            CValue::String(value) => self
                .body
                .push(format!("printf(\"%s\", \"{}\");", c_string(&value))),
        }
    }

    fn into_float(&self, value: CValue, span: Span) -> CompileResult<String> {
        match value {
            CValue::Float(value) => Ok(value),
            CValue::Int(value) => Ok(format!("(double)({value})")),
            CValue::Bool(value) => Ok(if value {
                "1.0".to_string()
            } else {
                "0.0".to_string()
            }),
            CValue::Null => Ok("0.0".to_string()),
            CValue::String(_) => Err(self.unsupported(
                span,
                "assembly arithmetic lowering rejects string operands until native numeric-string coercion exists; phpc run handles numeric strings and non-numeric string diagnostics",
            )),
        }
    }

    fn const_echo_string(&self, value: CValue) -> Option<String> {
        match value {
            CValue::Null => Some(String::new()),
            CValue::Bool(false) => Some(String::new()),
            CValue::Bool(true) => Some("1".to_string()),
            CValue::Int(value) if value.parse::<i64>().is_ok() => Some(value),
            CValue::Float(value) if value.parse::<f64>().is_ok() => Some(value),
            CValue::String(value) => Some(value),
            CValue::Int(_) | CValue::Float(_) => None,
        }
    }

    fn temp(&mut self) -> String {
        let temp = format!("t{}", self.next_temp);
        self.next_temp += 1;
        temp
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

enum NativeDivisorStatus {
    KnownZero,
    KnownNonZero,
    Dynamic,
    UnsupportedCoercion,
}

fn classify_ir_divisor(value: &IrValue) -> NativeDivisorStatus {
    match value {
        IrValue::Null => NativeDivisorStatus::KnownZero,
        IrValue::Bool(value) => {
            if *value {
                NativeDivisorStatus::KnownNonZero
            } else {
                NativeDivisorStatus::KnownZero
            }
        }
        IrValue::Int(value) => match value.parse::<i64>() {
            Ok(0) => NativeDivisorStatus::KnownZero,
            Ok(_) => NativeDivisorStatus::KnownNonZero,
            Err(_) => NativeDivisorStatus::Dynamic,
        },
        IrValue::Float(value) => match value.parse::<f64>() {
            Ok(value) if value == 0.0 => NativeDivisorStatus::KnownZero,
            Ok(_) => NativeDivisorStatus::KnownNonZero,
            Err(_) => NativeDivisorStatus::Dynamic,
        },
        IrValue::String(_) => NativeDivisorStatus::UnsupportedCoercion,
    }
}

fn classify_c_divisor(value: &CValue) -> NativeDivisorStatus {
    match value {
        CValue::Null => NativeDivisorStatus::KnownZero,
        CValue::Bool(value) => {
            if *value {
                NativeDivisorStatus::KnownNonZero
            } else {
                NativeDivisorStatus::KnownZero
            }
        }
        CValue::Int(value) => match value.parse::<i64>() {
            Ok(0) => NativeDivisorStatus::KnownZero,
            Ok(_) => NativeDivisorStatus::KnownNonZero,
            Err(_) => NativeDivisorStatus::Dynamic,
        },
        CValue::Float(value) => match value.parse::<f64>() {
            Ok(value) if value == 0.0 => NativeDivisorStatus::KnownZero,
            Ok(_) => NativeDivisorStatus::KnownNonZero,
            Err(_) => NativeDivisorStatus::Dynamic,
        },
        CValue::String(_) => NativeDivisorStatus::UnsupportedCoercion,
    }
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

fn is_bitwise_or_shift_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
    )
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

fn format_float_literal(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}
