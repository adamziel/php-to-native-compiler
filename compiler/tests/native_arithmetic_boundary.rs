use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{
    codegen::emit_native_executable_c_source, emit_asm_source, emit_ir_source, parse, run_source,
};

const LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "LLVM integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";
const LLVM_MODULO_RUNTIME_CHECK_REJECTION: &str = "LLVM modulo lowering rejects dynamic, zero, or non-positive integer divisors until native modulo runtime checks, PHP modulo diagnostics, negative-divisor/min-int edge behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current modulo behavior";

#[test]
fn phpc_run_still_handles_current_binary_arithmetic_subset() {
    let execution = run_source(
        r#"<?php
echo 1 + 2, "\n";
echo 8 - 2.5, "\n";
echo true * 6, "\n";
echo 9 / "3", "\n";
echo "8" % 3;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\n5.5\n6\n3\n2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_routes_non_integer_modulo_coercion_through_value_result_boundary() {
    let ir = emit_ir_source("<?php\necho 7.5 % 3;\n").unwrap();

    assert!(
        ir.contains("declare %phpc.NativeValueOperationResult @phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueOperationResult @phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("i8 4"),
        "float modulo should use the shared native value-operation modulo tag:\n{ir}"
    );
    assert!(
        ir.contains("extractvalue %phpc.NativeValueOperationResult"),
        "{ir}"
    );
}

#[test]
fn emit_ir_lowers_static_primitive_scalar_coercion_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
echo true * 4, "\n";
echo null + 2, "\n";
echo "3" * 2, "\n";
echo 1.5 + 2, "\n";
echo -"3";
"#,
    )
    .unwrap();

    assert!(ir.contains("@printf(ptr @.fmt_int, i64 4)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 2)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 3.5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -3)"), "{ir}");
}

#[test]
fn native_c_lowers_static_primitive_scalar_coercion_arithmetic() {
    let program = parse(
        r#"<?php
echo true * 4, "\n";
echo null + 2, "\n";
echo "3" * 2, "\n";
echo 1.5 + 2, "\n";
echo -"3";
"#,
    )
    .unwrap();

    let source = emit_native_executable_c_source(&program).unwrap();

    assert!(
        !source.contains("phpc_native_value_binary_result"),
        "primitive arithmetic should be resolved before the generic value-operation ABI:\n{source}"
    );
    assert!(
        !source.contains("phpc_native_value_unary_result"),
        "primitive unary arithmetic should be resolved before the generic value-operation ABI:\n{source}"
    );
    assert!(source.contains("int_value = 4;"), "{source}");
    assert!(source.contains("int_value = 2;"), "{source}");
    assert!(source.contains("int_value = 6;"), "{source}");
    assert!(source.contains("float_value = 3.5;"), "{source}");
    assert!(source.contains("int_value = -3;"), "{source}");
}

#[test]
fn emit_ir_routes_scalar_and_native_value_arithmetic_through_operation_results() {
    let ir = emit_ir_source(
        r#"<?php
echo "8tail" + 2, "|";
echo 10 - "3tail", "|";
echo "2.5tail" * 4, "|";
echo "9tail" / 3, "|";
echo "9tail" % 4, "|";
echo strpos("abc", "b") + 1, "|";
echo -strrev("6");
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%phpc.NativeValueOperationResult = type { i8, %phpc.NativeValueHandle, %phpc.NativeDiagnosticHandle }"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueOperationResult @phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %phpc.NativeValueOperationResult @phpc_native_value_unary_result"),
        "{ir}"
    );
    assert!(
        ir.matches("call %phpc.NativeValueOperationResult @phpc_native_value_binary_result")
            .count()
            >= 6,
        "{ir}"
    );
    assert!(
        ir.contains("call %phpc.NativeValueOperationResult @phpc_native_value_unary_result"),
        "{ir}"
    );
    for tag in [0, 1, 2, 3, 4] {
        assert!(
            ir.contains(&format!("i8 {tag}")),
            "missing op tag {tag}:\n{ir}"
        );
    }
    assert!(
        ir.matches("extractvalue %phpc.NativeValueOperationResult")
            .count()
            >= 18,
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_diagnostic_message_stderr"),
        "{ir}"
    );
    assert!(
        ir.contains("native_value_operation_error")
            && ir.contains("native_value_operation_ok")
            && ir.contains("ret i32 1"),
        "operation-result errors should have a generated native failure edge:\n{ir}"
    );
    assert!(
        ir.matches("call i64 @phpc_native_value_format_stdout_with_diagnostic")
            .count()
            >= 7,
        "{ir}"
    );
}

#[test]
fn emit_ir_rejects_integer_arithmetic_overflow_with_specific_boundary() {
    for source in [
        "<?php\necho 9223372036854775807 + 1;\n",
        "<?php\necho -9223372036854775807 - 2;\n",
        "<?php\necho 3037000500 * 3037000500;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_not_statically_proven_integer_arithmetic_overflow() {
    let error = emit_ir_source(
        r#"<?php
$flag = 1 + 2;
$condition = $flag === 3;
$left = $condition ? 1 : 2;
$right_flag = $flag === 2;
$right = $right_flag ? 10 : 20;
$third_flag = $flag === 1;
$third = $third_flag ? 100 : 200;
echo $left + $right + $third;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION);
}

#[test]
fn emit_ir_lowers_static_mixed_numeric_primitive_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
echo 1.5 + 2, "\n";
echo 8 - 2.5, "\n";
echo 3 * 2.5;
"#,
    )
    .unwrap();

    assert!(ir.contains("@printf(ptr @.fmt_float, double 3.5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 5.5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 7.5)"), "{ir}");
}

#[test]
fn emit_asm_routes_non_numeric_arithmetic_through_backend_value_result_boundary() {
    let asm = emit_asm_source("<?php\necho \"two\" + 2;\n").unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn emit_asm_rejects_integer_modulo_without_runtime_divisor_checks() {
    let error = emit_asm_source("<?php\n$divisor = 4 - 2;\necho 8 % $divisor;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_MODULO_RUNTIME_CHECK_REJECTION);
}

#[test]
fn emit_asm_rejects_integer_arithmetic_overflow_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 9223372036854775807 + 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION);
}

#[test]
fn emit_ir_routes_leading_numeric_arithmetic_through_value_result_boundary() {
    let ir = emit_ir_source("<?php\necho \"1foo\" + 2;\n").unwrap();

    assert!(
        ir.contains("call %phpc.NativeValueOperationResult @phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_diagnostic_message_stderr"),
        "{ir}"
    );
    assert!(
        ir.contains("native_value_operation_error"),
        "runtime arithmetic errors should branch to the shared native error exit:\n{ir}"
    );
}

#[test]
fn emit_asm_routes_string_arithmetic_through_backend_value_result_boundary() {
    let asm = emit_asm_source("<?php\necho \"two\" * 4;\n").unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn emit_ir_lowers_static_integer_add_subtract_and_multiply() {
    let ir = emit_ir_source(
        r#"<?php
$a = 1 + 2;
$b = $a * 4;
$c = $b - 5;
echo $a, "\n", $b, "\n", $c;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(
        !ir.contains("mul i64 %tmp0, 4"),
        "tracked single-result integer multiplication by a literal should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = sub i64 12, 5"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 12)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp1)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_tracked_integer_expression_subtraction_to_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$same = $value - $value;
echo $same + 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("sub i64 %tmp0, %tmp0"),
        "identical tracked integer expression subtraction should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 5"),
        "later literal additive identity should fold after subtraction to zero:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 5)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_untracked_identical_integer_expression_subtraction_to_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value, "\n";
echo $value - $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("sub i64 %tmp0, %tmp0"),
        "untracked identical integer expression subtraction should fold to zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
}

#[test]
fn emit_ir_folds_tracked_integer_expression_additive_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$plus_right = $value + 0;
$plus_left = 0 + $value;
$minus_zero = $value - 0;

echo $plus_right + $plus_left + $minus_zero;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("add i64 %tmp0, 0"),
        "tracked integer expression plus right zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, %tmp0"),
        "tracked integer expression plus left zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("sub i64 %tmp0, 0"),
        "tracked integer expression minus zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "tracked integer expression addition should fold when the result is known:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp1, %tmp0"),
        "chained tracked integer expression addition should fold when the result is known:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 24)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_integer_expression_multiplicative_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$times_right = $value * 1;
$times_left = 1 * $value;

echo $times_right + $times_left;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("mul i64 %tmp0, 1"),
        "tracked integer expression times right one should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("mul i64 1, %tmp0"),
        "tracked integer expression times left one should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "tracked integer expression addition should fold when the result is known:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 16)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_integer_expression_multiplication_by_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$zero_right = $value * 0;
$zero_left = 0 * $value;

echo $zero_right + 5, "\n", $zero_left + 7;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("mul i64 %tmp0, 0"),
        "tracked integer expression times right zero should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("mul i64 0, %tmp0"),
        "tracked integer expression times left zero should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 5"),
        "later literal additive identity should fold after multiplication by zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 7"),
        "later literal additive identity should fold after multiplication by zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 7)"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_integer_expression_arithmetic_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value + 0, "\n";
echo 0 + $value, "\n";
echo $value - 0, "\n";
echo $value * 1, "\n";
echo 1 * $value, "\n";
echo $value * 0, "\n";
echo 0 * $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    for redundant in [
        "add i64 %tmp0, 0",
        "add i64 0, %tmp0",
        "sub i64 %tmp0, 0",
        "mul i64 %tmp0, 1",
        "mul i64 1, %tmp0",
        "mul i64 %tmp0, 0",
        "mul i64 0, %tmp0",
    ] {
        assert!(
            !ir.contains(redundant),
            "untracked integer arithmetic identity should fold `{redundant}`:\n{ir}"
        );
    }
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 %tmp0)").count(),
        5,
        "{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 0)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_single_result_integer_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$base = 1 + 2;
$other = 4 + 5;
$flag = $base === 3;
$left = $flag ? 3 : 4;
$right = $flag ? 5 : 6;

echo $base + 4, "\n";
echo 10 - $base, "\n";
echo $base * 5, "\n";
echo 1 + 2, "\n";
echo $base + $other, "\n";
echo $left + $right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 4, 5"), "{ir}");
    assert!(
        !ir.contains("add i64 %tmp0, 4"),
        "tracked single-result integer addition should fold:\n{ir}"
    );
    assert!(
        !ir.contains("sub i64 10, %tmp0"),
        "tracked single-result integer subtraction should fold:\n{ir}"
    );
    assert!(
        !ir.contains("mul i64 %tmp0, 5"),
        "tracked single-result integer multiplication should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 7)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 15)"),
        "{ir}"
    );
    assert!(
        ir.contains("add i64 1, 2"),
        "literal-only integer arithmetic should still be emitted:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp1"),
        "tracked-expression plus tracked-expression single-result arithmetic should fold:\n{ir}"
    );
    assert!(
        ir.contains("add i64 %tmp3, %tmp4"),
        "ambiguous tracked-expression plus tracked-expression arithmetic should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_expression_integer_arithmetic_when_result_is_single() {
    let ir = emit_ir_source(
        r#"<?php
$left = 1 + 2;
$right = 4 + 5;
$mul_left = 2 * 3;
$mul_right = 1 + 4;
$flag = $left === 3;
$amb_left = $flag ? 3 : 4;
$amb_right = $flag ? 5 : 6;

echo $left + $right, "\n";
echo $right - $left, "\n";
echo $mul_left * $mul_right, "\n";
echo $amb_left + $amb_right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 4, 5"), "{ir}");
    assert!(ir.contains("%tmp2 = mul i64 2, 3"), "{ir}");
    assert!(ir.contains("%tmp3 = add i64 1, 4"), "{ir}");
    assert!(
        !ir.contains("add i64 %tmp0, %tmp1"),
        "tracked-expression plus tracked-expression addition should fold:\n{ir}"
    );
    assert!(
        !ir.contains("sub i64 %tmp1, %tmp0"),
        "tracked-expression plus tracked-expression subtraction should fold:\n{ir}"
    );
    assert!(
        !ir.contains("mul i64 %tmp2, %tmp3"),
        "tracked-expression plus tracked-expression multiplication should fold:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 12)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 30)"), "{ir}");
    assert!(
        ir.contains("add i64 %tmp5, %tmp6"),
        "ambiguous tracked-expression integer arithmetic should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_numeric_literal_arithmetic_identities() {
    let ir = emit_ir_source(
        r#"<?php
$plus_right = 5 + 0;
$plus_left = 0 + 6;
$minus_zero = 7 - 0;
$same_int = 8 - 8;
$times_right = 9 * 1;
$times_left = 1 * 10;
$zero_right = 11 * 0;
$zero_left = 0 * 12;
$same_float = 2.5 - 2.5;
$float_times_right = 3.5 * 1.0;
$float_times_left = 1.0 * 4.5;

echo $plus_right, "\n";
echo $plus_left, "\n";
echo $minus_zero, "\n";
echo $same_int, "\n";
echo $times_right, "\n";
echo $times_left, "\n";
echo $zero_right, "\n";
echo $zero_left, "\n";
echo $same_float, "\n";
echo $float_times_right, "\n";
echo $float_times_left;
"#,
    )
    .unwrap();

    for redundant in [
        "add i64 5, 0",
        "add i64 0, 6",
        "sub i64 7, 0",
        "sub i64 8, 8",
        "mul i64 9, 1",
        "mul i64 1, 10",
        "mul i64 11, 0",
        "mul i64 0, 12",
        "fsub double 2.5, 2.5",
        "fmul double 3.5, 1.0",
        "fmul double 1.0, 4.5",
    ] {
        assert!(
            !ir.contains(redundant),
            "numeric literal identity should fold `{redundant}`:\n{ir}"
        );
    }
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 7)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 9)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 10)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 0.0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 3.5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_float, double 4.5)"), "{ir}");
}

#[test]
fn emit_ir_lowers_static_float_add_subtract_and_multiply() {
    let ir = emit_ir_source(
        r#"<?php
$a = 1.5 + 2.25;
$b = $a * 2.5;
$c = $b - 1.25;
echo $a, "\n", $b, "\n", $c;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(
        !ir.contains("fmul double %tmp0, 2.5"),
        "tracked single-result float multiplication by a literal should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = fsub double 9.375, 1.25"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double %tmp0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 9.375)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double %tmp1)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_tracked_float_expression_subtraction_to_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.5 + 2.5;
$same = $value - $value;
echo $same + 1.25;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.5"), "{ir}");
    assert!(
        !ir.contains("fsub double %tmp0, %tmp0"),
        "identical tracked finite float expression subtraction should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("fadd double 0.0, 1.25"),
        "nonzero float additive identity after subtraction-to-zero should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 1.25)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_float_expression_multiplicative_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.5 + 2.5;
$times_right = $value * 1.0;
$times_left = 1.0 * $value;

echo $times_right + $times_left;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.5"), "{ir}");
    assert!(
        !ir.contains("fmul double %tmp0, 1.0"),
        "tracked float expression times right one should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("fmul double 1.0, %tmp0"),
        "tracked float expression times left one should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("fadd double %tmp0, %tmp0"),
        "known tracked float identity results should fold through later nonzero addition:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 8.0)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_nonzero_float_additive_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.5 + 2.5;

echo $value + 0.0, "\n";
echo 0.0 + $value, "\n";
echo $value - 0.0, "\n";
echo (0.0 - 0.0) + 0.0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.5"), "{ir}");
    assert!(
        !ir.contains("fadd double %tmp0, 0.0"),
        "tracked nonzero float expression plus right zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("fadd double 0.0, %tmp0"),
        "tracked nonzero float expression plus left zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("fsub double %tmp0, 0.0"),
        "tracked nonzero float expression minus right zero should reuse the expression:\n{ir}"
    );
    assert!(
        ir.contains("fadd double 0.0, 0.0"),
        "possible signed-zero float identity should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_single_known_nonzero_float_left_zero_subtraction() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.5 + 2.25;
$zero = 0.0 + 0.0;

echo 0.0 - $value, "\n";
echo 0.0 - 2.5, "\n";
echo 0.0 - $zero;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(
        !ir.contains("fsub double 0.0, %tmp0"),
        "single-known nonzero float left-zero subtraction should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fsub double 0.0, 2.5"),
        "nonzero float literal left-zero subtraction should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -3.75)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -2.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("fsub double 0.0, %tmp1"),
        "possible signed-zero float subtraction should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_positive_float_multiplication_by_zero() {
    let ir = emit_ir_source(
        r#"<?php
$positive = 1.5 + 2.5;
$negative = 0.0 - $positive;

echo $positive * 0.0, "\n";
echo 0.0 * $positive, "\n";
echo $negative * 0.0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.5"), "{ir}");
    assert!(
        !ir.contains("fmul double %tmp0, 0.0"),
        "tracked positive float expression times right zero should fold to positive zero:\n{ir}"
    );
    assert!(
        !ir.contains("fmul double 0.0, %tmp0"),
        "tracked positive float expression times left zero should fold to positive zero:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 0.0)"),
        "{ir}"
    );
    assert!(
        ir.contains("fmul double -4.0, 0.0"),
        "negative float multiplication by zero should stay emitted to preserve signed-zero behavior:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_single_known_nonzero_float_multiplication_by_negative_one() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.5 + 2.25;
$zero = 0.0 + 0.0;

echo $value * -1.0, "\n";
echo -1.0 * 2.5, "\n";
echo $zero * -1.0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(
        !ir.contains("fmul double %tmp0, -1.0"),
        "single-known nonzero float times right negative one should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fmul double -1.0, 2.5"),
        "nonzero float literal times left negative one should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -3.75)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -2.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("fmul double %tmp1, -1.0"),
        "possible signed-zero float multiplication by negative one should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_single_result_nonzero_float_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$base = 1.5 + 2.25;

echo $base + 1.25, "\n";
echo $base - 0.25, "\n";
echo $base * 2.0, "\n";
echo 1.5 + 2.25, "\n";
echo (0.0 + 0.0) + 0.0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(
        !ir.contains("fadd double %tmp0, 1.25"),
        "tracked single-result float addition should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fsub double %tmp0, 0.25"),
        "tracked single-result float subtraction should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fmul double %tmp0, 2.0"),
        "tracked single-result float multiplication should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 5.0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 3.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 7.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("fadd double 1.5, 2.25"),
        "literal-only float arithmetic should still be emitted:\n{ir}"
    );
    assert!(
        ir.contains("fadd double 0.0, 0.0"),
        "zero-result float arithmetic should stay emitted for signed-zero sensitivity:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_expression_nonzero_float_arithmetic_when_result_is_single() {
    let ir = emit_ir_source(
        r#"<?php
$left = 1.5 + 2.25;
$right = 4.0 + 0.5;
$mul_left = 1.25 + 0.25;
$mul_right = 2.0 + 1.0;
$zero_left = 1.5 + 2.25;
$zero_right = 3.0 + 0.75;
$seed = 1 + 2;
$flag = $seed === 3;
$amb_left = $flag ? 1.25 : 2.25;
$amb_right = $flag ? 2.75 : 3.75;

echo $left + $right, "\n";
echo $right - $left, "\n";
echo $mul_left * $mul_right, "\n";
echo 1.5 + 2.25, "\n";
echo $zero_left - $zero_right, "\n";
echo $amb_left + $amb_right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(ir.contains("%tmp1 = fadd double 4.0, 0.5"), "{ir}");
    assert!(ir.contains("%tmp2 = fadd double 1.25, 0.25"), "{ir}");
    assert!(ir.contains("%tmp3 = fadd double 2.0, 1.0"), "{ir}");
    assert!(
        !ir.contains("fadd double %tmp0, %tmp1"),
        "tracked-expression float addition should fold when the nonzero result is known:\n{ir}"
    );
    assert!(
        !ir.contains("fsub double %tmp1, %tmp0"),
        "tracked-expression float subtraction should fold when the nonzero result is known:\n{ir}"
    );
    assert!(
        !ir.contains("fmul double %tmp2, %tmp3"),
        "tracked-expression float multiplication should fold when the nonzero result is known:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 8.25)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 0.75)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double 4.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("fadd double 1.5, 2.25"),
        "literal-only float arithmetic should still be emitted:\n{ir}"
    );
    assert!(
        ir.contains("fsub double %tmp4, %tmp5"),
        "zero-result tracked-expression float arithmetic should stay emitted for signed-zero sensitivity:\n{ir}"
    );
    assert!(
        ir.contains("fadd double %tmp8, %tmp9"),
        "ambiguous tracked-expression float arithmetic should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_tracks_bounded_float_arithmetic_results_for_later_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 1.25 : 1.25;
$offset = $flag ? 2.75 : 2.75;
$total = $value + $offset;
$ambiguous = ($flag ? 1.25 : 2.25) + 2.75;

echo ($total === 4.0) ? 10 : 20, "\n";
echo ($ambiguous === 4.0) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp2 = fadd double 1.25, 2.75"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 10)"),
        "{ir}"
    );
    assert!(
        !ir.contains("fcmp oeq double %tmp2, 4.0"),
        "proven float arithmetic identity should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp4 = fadd double %tmp3, 2.75"), "{ir}");
    assert!(ir.contains("fcmp oeq double %tmp"), "{ir}");
    assert!(ir.contains(", 4.0"), "{ir}");
    assert!(ir.contains("select i1 %tmp"), "{ir}");
    assert!(ir.contains(", i64 1, i64 0"), "{ir}");
}

#[test]
fn emit_ir_lowers_integer_modulo_with_static_positive_divisor() {
    let ir = emit_ir_source(
        r#"<?php
$value = 10 + 5;
$remainder = $value % 4;
echo $remainder, "\n";
echo 17 % 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 10, 5"), "{ir}");
    assert!(ir.contains("%tmp1 = srem i64 %tmp0, 4"), "{ir}");
    assert!(ir.contains("srem i64 17, 5"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp1)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp"),
        "{ir}"
    );
}

#[test]
fn emit_ir_tracks_integer_modulo_results_for_later_checked_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$condition = $seed === 3;
$value = $condition ? 10 : 11;
$remainder = $value % 3;
echo $remainder + 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, i64 10, i64 11"),
        "{ir}"
    );
    assert!(ir.contains("%tmp3 = srem i64 %tmp2, 3"), "{ir}");
    assert!(ir.contains("%tmp4 = add i64 %tmp3, 5"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp4)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_integer_modulo_by_one_to_zero() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 10 : 11;
$tracked = $value % 1;
$literal = 17 % 1;

echo $tracked + 5, "\n";
echo $literal + 7;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("srem i64 %tmp2, 1"),
        "tracked integer modulo by one should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("srem i64 17, 1"),
        "literal integer modulo by one should fold to zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 7)"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_integer_modulo_by_one_to_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value, "\n";
echo $value % 1;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("srem i64 %tmp0, 1"),
        "untracked integer modulo by one should fold to zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
}

#[test]
fn emit_ir_folds_bounded_integer_modulo_when_all_remainders_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 10 : 13;
$remainder = $value % 3;

echo $remainder + 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("srem i64 %tmp2, 3"),
        "bounded integer modulo should fold when every remainder matches:\n{ir}"
    );
    assert!(ir.contains("%tmp3 = add i64 1, 5"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp3)"), "{ir}");
}

#[test]
fn emit_ir_rejects_integer_modulo_without_static_positive_divisor() {
    for source in [
        "<?php\necho 8 % 0;\n",
        "<?php\necho 8 % -2;\n",
        "<?php\n$divisor = 4 - 2;\necho 8 % $divisor;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_MODULO_RUNTIME_CHECK_REJECTION);
    }
}

#[test]
fn native_arithmetic_emit_ir_cli_routes_string_division_and_modulo_value_results() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone178/native_arithmetic_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    assert!(
        output.status.success(),
        "compile stdout:\n{}\ncompile stderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let ir = String::from_utf8_lossy(&output.stdout);
    assert!(
        ir.contains("%phpc.NativeValueOperationResult = type")
            && ir.contains("@phpc_native_value_binary_result"),
        "{ir}"
    );
    assert!(
        ir.contains("i8 3") && ir.contains("i8 4"),
        "string division and modulo should use shared value-operation tags:\n{ir}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "unexpected CLI stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn native_integer_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone233/native_integer_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone233/native_integer_arithmetic_emit_ir.cli"),
    )
    .expect("native integer arithmetic IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone293/native_float_arithmetic_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone293/native_float_arithmetic_emit_ir.cli"),
    )
    .expect("native float arithmetic IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_modulo_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone314/native_integer_modulo_emit_ir.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone314/native_integer_modulo_emit_ir.cli"),
    )
    .expect("native integer modulo IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_modulo_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone344/native_integer_modulo_result_tracking.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone344/native_integer_modulo_result_tracking_emit_ir.cli"),
    )
    .expect("native integer modulo result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_modulo_by_one_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone452/native_integer_modulo_by_one_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone452/native_integer_modulo_by_one_folding_emit_ir.cli"),
    )
    .expect("native integer modulo-by-one folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_integer_modulo_by_one_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone507/native_untracked_integer_modulo_by_one.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone507/native_untracked_integer_modulo_by_one_emit_ir.cli"),
    )
    .expect("native untracked integer modulo-by-one IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_integer_modulo_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone453/native_bounded_integer_modulo_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone453/native_bounded_integer_modulo_folding_emit_ir.cli"),
    )
    .expect("native bounded integer modulo folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_additive_identity_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone461/native_float_additive_identity_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone461/native_float_additive_identity_folding_emit_ir.cli"),
    )
    .expect("native float additive identity folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_left_zero_subtraction_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone462/native_float_left_zero_subtraction_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone462/native_float_left_zero_subtraction_folding_emit_ir.cli",
    ))
    .expect("native float left-zero subtraction folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_positive_float_multiplication_by_zero_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone463/native_positive_float_multiplication_by_zero_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone463/native_positive_float_multiplication_by_zero_folding_emit_ir.cli",
    ))
    .expect("native positive float multiplication-by-zero folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_multiplication_by_negative_one_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone464/native_float_multiplication_by_negative_one_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone464/native_float_multiplication_by_negative_one_folding_emit_ir.cli",
    ))
    .expect("native float multiplication-by-negative-one folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_float_arithmetic_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone465/native_tracked_float_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone465/native_tracked_float_arithmetic_folding_emit_ir.cli",
        ))
        .expect("native tracked float arithmetic folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_expression_float_arithmetic_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone480/native_tracked_expression_float_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone480/native_tracked_expression_float_arithmetic_folding_emit_ir.cli",
    ))
    .expect("native tracked-expression float arithmetic folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_integer_arithmetic_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone466/native_tracked_integer_arithmetic_folding.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone466/native_tracked_integer_arithmetic_folding_emit_ir.cli",
        ))
        .expect("native tracked integer arithmetic folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_expression_integer_arithmetic_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone477/native_tracked_expression_integer_arithmetic_folding.php",
    );
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone477/native_tracked_expression_integer_arithmetic_folding_emit_ir.cli",
    ))
    .expect("native tracked-expression integer arithmetic folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_integer_arithmetic_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone502/native_untracked_integer_arithmetic_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone502/native_untracked_integer_arithmetic_identity_emit_ir.cli",
    ))
    .expect("native untracked integer arithmetic identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_identical_integer_subtraction_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone506/native_untracked_identical_integer_subtraction.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone506/native_untracked_identical_integer_subtraction_emit_ir.cli",
    ))
    .expect("native untracked identical integer subtraction IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mixed_numeric_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone323/native_mixed_numeric_arithmetic.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone323/native_mixed_numeric_arithmetic_emit_ir.cli"),
    )
    .expect("native mixed numeric arithmetic IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_scalar_coercion_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone326/native_scalar_coercion_arithmetic.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone326/native_scalar_coercion_arithmetic_emit_ir.cli"),
    )
    .expect("native scalar-coercion arithmetic IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_overflow_arithmetic_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone329/native_integer_overflow_arithmetic.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone329/native_integer_overflow_arithmetic_emit_ir.cli"),
    )
    .expect("native integer overflow arithmetic IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_modulo_runtime_check_boundary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone317/native_modulo_runtime_check_boundary.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone317/native_modulo_runtime_check_boundary_emit_ir.cli"),
    )
    .expect("native modulo runtime-check boundary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_float_arithmetic_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone368/native_bounded_float_arithmetic_result_tracking.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone368/native_bounded_float_arithmetic_result_tracking_emit_ir.cli",
    ))
    .expect("native bounded float arithmetic result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_integer_expr_subtraction_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone416/native_identical_integer_expr_subtraction.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone416/native_identical_integer_expr_subtraction_emit_ir.cli",
        ))
        .expect("native identical integer expression subtraction IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_float_expr_subtraction_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone419/native_identical_float_expr_subtraction.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone419/native_identical_float_expr_subtraction_emit_ir.cli",
        ))
        .expect("native identical float expression subtraction IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_multiplicative_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone434/native_float_multiplicative_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone434/native_float_multiplicative_identity_emit_ir.cli"),
    )
    .expect("native float multiplicative identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_additive_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone422/native_integer_additive_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone422/native_integer_additive_identity_emit_ir.cli"),
    )
    .expect("native integer additive identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_multiplicative_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone425/native_integer_multiplicative_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone425/native_integer_multiplicative_identity_emit_ir.cli"),
    )
    .expect("native integer multiplicative identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_multiplication_by_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone428/native_integer_multiplication_by_zero.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(
        workspace_root
            .join("tests/fixtures/milestone428/native_integer_multiplication_by_zero_emit_ir.cli"),
    )
    .expect("native integer multiplication-by-zero IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_numeric_literal_arithmetic_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone451/native_numeric_literal_arithmetic_identity.php");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-ir"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected = fs::read_to_string(workspace_root.join(
        "tests/fixtures/milestone451/native_numeric_literal_arithmetic_identity_emit_ir.cli",
    ))
    .expect("native numeric literal arithmetic identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
