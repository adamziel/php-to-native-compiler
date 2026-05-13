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
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const ASSEMBLY_MUTATION_REJECTION: &str = "assembly mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";
const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unary minus and logical not until native PHP numeric coercion, truthiness conversion, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary behavior";
const ASSEMBLY_UNARY_REJECTION: &str = "assembly unary lowering rejects unary minus and logical not until native PHP numeric coercion, truthiness conversion, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary behavior";
const LLVM_ARITHMETIC_REJECTION: &str = "LLVM arithmetic lowering rejects binary arithmetic operators until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const ASSEMBLY_ARITHMETIC_REJECTION: &str = "assembly arithmetic lowering rejects binary arithmetic operators until native PHP numeric coercion, division/modulo zero checks, modulo coercions, references/copy-on-write, and exact native error behavior exist; phpc run handles current arithmetic behavior";
const LLVM_CONCAT_REJECTION: &str = "LLVM concatenation lowering rejects string concatenation until native PHP string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
const ASSEMBLY_CONCAT_REJECTION: &str = "assembly concatenation lowering rejects string concatenation until native PHP string conversion, dynamic allocation, references/copy-on-write, and exact native error behavior exist; phpc run handles current concatenation behavior";
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

#[derive(Default)]
struct LlvmGenerator {
    strings: Vec<(String, String)>,
    body: Vec<String>,
    variables: HashMap<String, IrValue>,
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
            Expr::Variable(name, span) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| self.unsupported(*span, LLVM_VARIABLE_READ_REJECTION)),
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(*span, LLVM_GLOBAL_CONSTANT_REJECTION))
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
            Expr::Unary { op, span, .. } => {
                if matches!(op, UnaryOp::BitwiseNot) {
                    return Err(self.unsupported(
                        *span,
                        "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                Err(self.unsupported(*span, LLVM_UNARY_REJECTION))
            }
            Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::NullCoalesceAssign { span, .. }
            | Expr::IncrementDecrement { span, .. } => {
                Err(self.unsupported(*span, LLVM_MUTATION_REJECTION))
            }
            Expr::Ternary { span, .. } | Expr::ShortTernary { span, .. } => {
                Err(self.unsupported(*span, LLVM_CONDITIONAL_REJECTION))
            }
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
                    return Err(self.unsupported(*span, LLVM_CONDITIONAL_REJECTION));
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
                if is_binary_arithmetic_op(*op) {
                    return Err(self.unsupported(*span, LLVM_ARITHMETIC_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return Err(self.unsupported(*span, LLVM_CONCAT_REJECTION));
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
        _left: IrValue,
        op: BinaryOp,
        _right: IrValue,
        span: Span,
    ) -> CompileResult<IrValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION))
            }
            BinaryOp::Div | BinaryOp::Mod => Err(self.unsupported(span, LLVM_ARITHMETIC_REJECTION)),
            BinaryOp::Concat => Err(self.unsupported(span, LLVM_CONCAT_REJECTION)),
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

    fn add_string(&mut self, value: &str) -> String {
        let name = format!(".str.{}", self.next_string);
        self.next_string += 1;
        self.strings.push((name.clone(), value.to_string()));
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
            Expr::Variable(name, span) => self
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| self.unsupported(*span, ASSEMBLY_VARIABLE_READ_REJECTION)),
            Expr::Call { name, span, .. } if is_global_constant_builtin(name) => {
                Err(self.unsupported(*span, ASSEMBLY_GLOBAL_CONSTANT_REJECTION))
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
            Expr::Unary { op, span, .. } => {
                if matches!(op, UnaryOp::BitwiseNot) {
                    return Err(self.unsupported(
                        *span,
                        "assembly bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior",
                    ));
                }
                Err(self.unsupported(*span, ASSEMBLY_UNARY_REJECTION))
            }
            Expr::Assign { span, .. }
            | Expr::CompoundAssign { span, .. }
            | Expr::NullCoalesceAssign { span, .. }
            | Expr::IncrementDecrement { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_MUTATION_REJECTION))
            }
            Expr::Ternary { span, .. } | Expr::ShortTernary { span, .. } => {
                Err(self.unsupported(*span, ASSEMBLY_CONDITIONAL_REJECTION))
            }
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
                    return Err(self.unsupported(*span, ASSEMBLY_CONDITIONAL_REJECTION));
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
                if is_binary_arithmetic_op(*op) {
                    return Err(self.unsupported(*span, ASSEMBLY_ARITHMETIC_REJECTION));
                }
                if matches!(op, BinaryOp::Concat) {
                    return Err(self.unsupported(*span, ASSEMBLY_CONCAT_REJECTION));
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
        _left: CValue,
        op: BinaryOp,
        _right: CValue,
        span: Span,
    ) -> CompileResult<CValue> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION))
            }
            BinaryOp::Div | BinaryOp::Mod => {
                Err(self.unsupported(span, ASSEMBLY_ARITHMETIC_REJECTION))
            }
            BinaryOp::Concat => Err(self.unsupported(span, ASSEMBLY_CONCAT_REJECTION)),
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

fn is_binary_arithmetic_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
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
