use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_BITWISE_REJECTION: &str = "LLVM bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior";
const LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION: &str = "LLVM integer arithmetic lowering rejects overflow-sensitive or not-statically-proven integer +, -, and * until native PHP integer overflow promotion, runtime checks, references/copy-on-write, and exact native error behavior exist; phpc run handles current integer overflow arithmetic behavior";

#[test]
fn phpc_run_still_handles_current_bitwise_and_shift_operators() {
    let execution = run_source(
        r#"<?php
var_dump(6 & 3);
var_dump(6 | 3);
var_dump(6 ^ 3);
var_dump(~5);
var_dump(8 << 1);
var_dump(8 >> 1);
var_dump("ab" & "AB");
var_dump("8" << true);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(2)\nint(7)\nint(5)\nint(-6)\nint(16)\nint(4)\nstring(2) \"AB\"\nint(16)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_lowers_integer_bitwise_operators() {
    let ir = emit_ir_source(
        r#"<?php
$left = 6 + 2;
$right = 3;
$and = $left & $right;
$or = $left | $right;
$xor = $left ^ $right;
$not = ~$right;

echo $and, "\n";
echo $or, "\n";
echo $xor, "\n";
echo $not, "z";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, 3"),
        "tracked single-result integer bitwise AND should fold:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 %tmp0, 3"),
        "tracked single-result integer bitwise OR should fold:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, 3"),
        "tracked single-result integer bitwise XOR should fold:\n{ir}"
    );
    assert!(!ir.contains("xor i64 3, -1"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 11)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -4)"), "{ir}");
}

#[test]
fn emit_ir_tracks_static_integer_bitwise_results_for_later_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$left = 6 + 2;
$mask = 3;
$and = $left & $mask;
$or = $left | $mask;
$xor = $or ^ $mask;
$not = ~$mask;

echo $and + 5, "\n";
echo $xor + $not;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, 3"),
        "tracked single-result integer bitwise AND should fold:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 %tmp0, 3"),
        "tracked single-result integer bitwise OR should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = xor i64 11, 3"), "{ir}");
    assert!(!ir.contains("xor i64 3, -1"), "{ir}");
    assert!(
        !ir.contains("add i64 0, 5"),
        "later integer additive identity should fold after bitwise AND:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp1, -4"),
        "tracked single-result integer arithmetic should fold after bitwise XOR:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 4)"), "{ir}");
}

#[test]
fn emit_ir_folds_single_known_integer_bitwise_not() {
    let ir = emit_ir_source(
        r#"<?php
$literal = ~3;
$expr = ~(1 + 2);

echo $literal, "\n";
echo $expr;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(
        !ir.contains("xor i64 3, -1"),
        "literal integer bitwise-not should fold to the known result:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, -1"),
        "single known integer expression bitwise-not should fold to the known result:\n{ir}"
    );
    assert_eq!(ir.matches("@printf(ptr @.fmt_int, i64 -4)").count(), 2);
}

#[test]
fn emit_ir_folds_untracked_integer_double_bitwise_not() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;
$same = ~~$value;

echo $same;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, -1"),
        "double bitwise-not should not emit the first redundant invert:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp1, -1"),
        "double bitwise-not should not emit the second redundant invert:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp0)"), "{ir}");
}

#[test]
fn emit_ir_folds_single_known_integer_bitwise_xor_all_ones() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$all_ones = 0 - 1;

echo 5 ^ -1, "\n";
echo -1 ^ 7, "\n";
echo ($value ^ $all_ones) + 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = sub i64 0, 1"), "{ir}");
    assert!(
        !ir.contains("xor i64 5, -1"),
        "literal right all-ones XOR should fold to the known integer result:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 -1, 7"),
        "literal left all-ones XOR should fold to the known integer result:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, %tmp1"),
        "single known tracked-expression XOR all-ones should fold:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 -9, 0"),
        "known XOR-all-ones result should fold through later addition:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -8)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -9)"), "{ir}");
}

#[test]
fn emit_ir_folds_identical_integer_expression_bitwise_operands() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$same_and = $value & $value;
$same_or = $value | $value;
$same_xor = $value ^ $value;

echo $same_and + $same_or, "\n";
echo $same_xor + 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, %tmp0"),
        "identical integer expression & should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 %tmp0, %tmp0"),
        "identical integer expression | should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, %tmp0"),
        "identical integer expression ^ should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "known identical integer expression bitwise results should fold through later addition:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 5"),
        "later literal additive identity should fold after identical xor to zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 16)"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_identical_integer_expression_bitwise_operands() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value & $value, "\n";
echo $value | $value, "\n";
echo $value ^ $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    for redundant in [
        "and i64 %tmp0, %tmp0",
        "or i64 %tmp0, %tmp0",
        "xor i64 %tmp0, %tmp0",
    ] {
        assert!(
            !ir.contains(redundant),
            "untracked identical integer bitwise operation should fold `{redundant}`:\n{ir}"
        );
    }
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 %tmp0)").count(),
        2,
        "{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
}

#[test]
fn emit_ir_folds_tracked_integer_expression_bitwise_zero_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$or_right = $value | 0;
$or_left = 0 | $value;
$xor_right = $value ^ 0;
$xor_left = 0 ^ $value;

echo $or_right + $or_left + $xor_right + $xor_left;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("or i64 %tmp0, 0"),
        "tracked integer expression OR right zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 0, %tmp0"),
        "tracked integer expression OR left zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, 0"),
        "tracked integer expression XOR right zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 0, %tmp0"),
        "tracked integer expression XOR left zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "known tracked zero-identity bitwise results should fold through later addition:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 32)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_untracked_integer_expression_bitwise_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo ($value & -1), "\n";
echo (-1 & $value), "\n";
echo ($value | 0), "\n";
echo (0 | $value), "\n";
echo ($value ^ 0), "\n";
echo (0 ^ $value), "\n";
echo ($value & 0), "\n";
echo 0 & $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    for redundant in [
        "and i64 %tmp0, -1",
        "and i64 -1, %tmp0",
        "or i64 %tmp0, 0",
        "or i64 0, %tmp0",
        "xor i64 %tmp0, 0",
        "xor i64 0, %tmp0",
        "and i64 %tmp0, 0",
        "and i64 0, %tmp0",
    ] {
        assert!(
            !ir.contains(redundant),
            "untracked integer bitwise identity should fold `{redundant}`:\n{ir}"
        );
    }
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 %tmp0)").count(),
        6,
        "{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 0)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_integer_expression_bitwise_all_ones_identities() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$all_ones = 0 - 1;
$and_right = $value & $all_ones;
$and_left = $all_ones & $value;

echo $and_right + $and_left;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = sub i64 0, 1"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, %tmp1"),
        "tracked integer expression AND right all-ones should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("and i64 %tmp1, %tmp0"),
        "tracked integer expression AND left all-ones should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "known tracked all-ones identity bitwise results should fold through later addition:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 16)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_integer_expression_bitwise_or_all_ones() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$all_ones = 0 - 1;
$or_right = $value | $all_ones;
$or_left = $all_ones | $value;

echo $or_right + 0, "\n";
echo 0 + $or_left;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = sub i64 0, 1"), "{ir}");
    assert!(
        !ir.contains("or i64 %tmp0, %tmp1"),
        "tracked integer expression OR right all-ones should fold to -1:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 %tmp1, %tmp0"),
        "tracked integer expression OR left all-ones should fold to -1:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 -1, 0"),
        "known OR-all-ones result should fold through later addition:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, -1"),
        "known OR-all-ones result should fold through later addition:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 -1)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_integer_expression_bitwise_and_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$and_right = $value & 0;
$and_left = 0 & $value;

echo $and_right + 5, "\n", $and_left + 7;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, 0"),
        "tracked integer expression AND right zero should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("and i64 0, %tmp0"),
        "tracked integer expression AND left zero should fold to zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 5"),
        "later literal additive identity should fold after bitwise AND zero:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 0, 7"),
        "later literal additive identity should fold after bitwise AND zero:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 7)"), "{ir}");
}

#[test]
fn emit_ir_folds_tracked_single_result_integer_bitwise() {
    let ir = emit_ir_source(
        r#"<?php
$base = 6 + 2;
$other = 4 + 1;
$flag = $base === 8;
$left = $flag ? 12 : 10;
$right = $flag ? 5 : 3;

echo $base & 3, "\n";
echo 1 | $base, "\n";
echo $base ^ 5, "\n";
echo 6 & 3, "\n";
echo $base & $other, "\n";
echo $left & $right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 4, 1"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 8"), "{ir}");
    assert!(
        ir.contains("%tmp3 = select i1 %tmp2, i64 12, i64 10"),
        "{ir}"
    );
    assert!(ir.contains("%tmp4 = select i1 %tmp2, i64 5, i64 3"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, 3"),
        "tracked single-result integer bitwise AND should fold:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 1, %tmp0"),
        "tracked single-result integer bitwise OR should fold:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp0, 5"),
        "tracked single-result integer bitwise XOR should fold:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 9)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 13)"),
        "{ir}"
    );
    assert!(
        ir.contains("and i64 6, 3"),
        "literal-only integer bitwise should still be emitted:\n{ir}"
    );
    assert!(
        !ir.contains("and i64 %tmp0, %tmp1"),
        "tracked-expression plus tracked-expression bitwise should fold when the result is known:\n{ir}"
    );
    assert!(
        ir.contains("and i64 %tmp3, %tmp4"),
        "ambiguous tracked-expression plus tracked-expression bitwise should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_expression_integer_bitwise_when_result_is_single() {
    let ir = emit_ir_source(
        r#"<?php
$left = 6 + 2;
$right = 4 + 1;
$xor_left = 9 + 3;
$xor_right = 1 + 2;
$flag = $left === 8;
$amb_left = $flag ? 12 : 10;
$amb_right = $flag ? 5 : 3;

echo $left & $right, "\n";
echo $left | $right, "\n";
echo $xor_left ^ $xor_right, "\n";
echo 6 & 3, "\n";
echo $amb_left & $amb_right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 4, 1"), "{ir}");
    assert!(ir.contains("%tmp2 = add i64 9, 3"), "{ir}");
    assert!(ir.contains("%tmp3 = add i64 1, 2"), "{ir}");
    assert!(
        !ir.contains("and i64 %tmp0, %tmp1"),
        "tracked-expression integer bitwise AND should fold when the result is known:\n{ir}"
    );
    assert!(
        !ir.contains("or i64 %tmp0, %tmp1"),
        "tracked-expression integer bitwise OR should fold when the result is known:\n{ir}"
    );
    assert!(
        !ir.contains("xor i64 %tmp2, %tmp3"),
        "tracked-expression integer bitwise XOR should fold when the result is known:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 13)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 15)"),
        "{ir}"
    );
    assert!(
        ir.contains("and i64 6, 3"),
        "literal-only integer bitwise should still be emitted:\n{ir}"
    );
    assert!(
        ir.contains("and i64 %tmp5, %tmp6"),
        "ambiguous tracked-expression integer bitwise should stay emitted:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_integer_literal_bitwise_identities() {
    let ir = emit_ir_source(
        r#"<?php
$same_and = 5 & 5;
$same_or = 6 | 6;
$same_xor = 7 ^ 7;
$and_zero_right = 8 & 0;
$and_zero_left = 0 & 9;
$and_all_ones_right = 10 & -1;
$and_all_ones_left = -1 & 11;
$or_zero_right = 12 | 0;
$or_zero_left = 0 | 13;
$xor_zero_right = 14 ^ 0;
$xor_zero_left = 0 ^ 15;

echo $same_and, "\n";
echo $same_or, "\n";
echo $same_xor, "\n";
echo $and_zero_right, "\n";
echo $and_zero_left, "\n";
echo $and_all_ones_right, "\n";
echo $and_all_ones_left, "\n";
echo $or_zero_right, "\n";
echo $or_zero_left, "\n";
echo $xor_zero_right, "\n";
echo $xor_zero_left;
"#,
    )
    .unwrap();

    for redundant in [
        "and i64 5, 5",
        "or i64 6, 6",
        "xor i64 7, 7",
        "and i64 8, 0",
        "and i64 0, 9",
        "and i64 10, -1",
        "and i64 -1, 11",
        "or i64 12, 0",
        "or i64 0, 13",
        "xor i64 14, 0",
        "xor i64 0, 15",
    ] {
        assert!(
            !ir.contains(redundant),
            "integer literal bitwise identity should fold `{redundant}`:\n{ir}"
        );
    }
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 10)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 11)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 12)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 13)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 14)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 15)"), "{ir}");
}

#[test]
fn emit_ir_tracks_bounded_integer_bitwise_results_for_later_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;
$mask = $flag ? 3 : 1;
$and = $value & $mask;
$flipped = ~$mask;

echo $and + 10, "\n";
echo $flipped + 20;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 5, i64 6"), "{ir}");
    assert!(ir.contains("%tmp3 = select i1 %tmp1, i64 3, i64 1"), "{ir}");
    assert!(ir.contains("%tmp4 = and i64 %tmp2, %tmp3"), "{ir}");
    assert!(ir.contains("%tmp5 = xor i64 %tmp3, -1"), "{ir}");
    assert!(ir.contains("%tmp6 = add i64 %tmp4, 10"), "{ir}");
    assert!(ir.contains("%tmp9 = add i64 %tmp5, 20"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp6)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp9)"), "{ir}");
}

#[test]
fn emit_ir_lowers_integer_shifts_with_static_safe_counts() {
    let ir = emit_ir_source(
        r#"<?php
$left = 6 + 2;
$negative = -8;
$shift_left = $left << 2;
$shift_right = $left >> 1;
$shift_negative = $negative >> 1;

echo $shift_left, "\n";
echo $shift_right, "\n";
echo $shift_negative, "z";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(!ir.contains("sub i64 0, 8"), "{ir}");
    assert!(
        !ir.contains("shl i64 %tmp0, 2"),
        "tracked single-result integer left shift should fold:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 %tmp0, 1"),
        "tracked single-result integer right shift should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = ashr i64 -8, 1"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 32)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 4)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp1)"), "{ir}");
}

#[test]
fn emit_ir_folds_tracked_integer_expression_shift_by_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 6 + 2;
$shift_left = $value << 0;
$shift_right = $value >> 0;

echo $shift_left + $shift_right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("shl i64 %tmp0, 0"),
        "tracked integer expression left shift by zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 %tmp0, 0"),
        "tracked integer expression right shift by zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, %tmp0"),
        "known tracked shift-by-zero results should fold through later addition:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 16)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_integer_literal_shift_by_zero() {
    let ir = emit_ir_source(
        r#"<?php
$left = 8 << 0;
$right = 9 >> 0;

echo $left, "\n";
echo $right;
"#,
    )
    .unwrap();

    assert!(
        !ir.contains("shl i64 8, 0"),
        "integer literal left shift by zero should reuse the literal:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 9, 0"),
        "integer literal right shift by zero should reuse the literal:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 8)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 9)"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_integer_expression_shift_by_zero() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value << 0, "\n";
echo $value >> 0;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("shl i64 %tmp0, 0"),
        "untracked integer expression left shift by zero should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 %tmp0, 0"),
        "untracked integer expression right shift by zero should reuse the expression:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 %tmp0)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_single_result_integer_shifts() {
    let ir = emit_ir_source(
        r#"<?php
$base = 6 + 2;
$literal_left = 8 << 1;
$literal_right = 8 >> 1;
$seed = 1 + 2;
$flag = $seed === 3;
$bounded = $flag ? 5 : 6;

echo $base << 2, "\n";
echo $base >> 1, "\n";
echo $literal_left + $literal_right, "\n";
echo $bounded << 1;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("shl i64 %tmp0, 2"),
        "tracked single-result integer left shift should fold:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 %tmp0, 1"),
        "tracked single-result integer right shift should fold:\n{ir}"
    );
    assert!(
        ir.contains("shl i64 8, 1"),
        "literal-only integer left shift should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("ashr i64 8, 1"),
        "literal-only integer right shift should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("shl i64 %tmp"),
        "non-single tracked integer shift should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 32)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 4)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_accepts_tracked_single_result_integer_shift_counts() {
    let ir = emit_ir_source(
        r#"<?php
$count = 1 + 1;
$base = 6 + 2;

echo $base << $count, "\n";
echo $base >> $count, "\n";
echo 8 << $count;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 1"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("shl i64 %tmp1, %tmp0"),
        "tracked single-result shift count should lower as a static safe count:\n{ir}"
    );
    assert!(
        !ir.contains("ashr i64 %tmp1, %tmp0"),
        "tracked single-result shift count should lower as a static safe count:\n{ir}"
    );
    assert!(
        ir.contains("shl i64 8, 2"),
        "literal left operand should still emit with the proven static count:\n{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 32)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 2)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_rejects_ambiguous_tracked_integer_shift_counts() {
    let error = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$count = $flag ? 1 : 2;

echo 8 << $count;
"#,
    )
    .expect_err("ambiguous tracked shift counts should remain unsupported");

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn emit_ir_tracks_safe_integer_shift_results_for_later_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$left = 6 + 2;
$shift_left = $left << 2;
$shift_right = $shift_left >> 3;
$negative = -8;
$shift_negative = $negative >> 1;

echo $shift_left + 5, "\n";
echo $shift_right + $shift_negative;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 6, 2"), "{ir}");
    assert!(
        !ir.contains("shl i64 %tmp0, 2"),
        "tracked single-result integer left shift should fold:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = ashr i64 32, 3"), "{ir}");
    assert!(!ir.contains("sub i64 0, 8"), "{ir}");
    assert!(ir.contains("%tmp2 = ashr i64 -8, 1"), "{ir}");
    assert!(ir.contains("%tmp3 = add i64 32, 5"), "{ir}");
    assert!(
        !ir.contains("add i64 %tmp1, %tmp2"),
        "known tracked shift results should fold through later addition:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp3)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
}

#[test]
fn emit_ir_tracks_bounded_integer_shift_and_negation_results_for_later_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;
$shifted = $value << 1;
$negated = -$value;

echo $shifted + $negated;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 5, i64 6"), "{ir}");
    assert!(ir.contains("%tmp3 = shl i64 %tmp2, 1"), "{ir}");
    assert!(ir.contains("%tmp4 = sub i64 0, %tmp2"), "{ir}");
    assert!(ir.contains("%tmp5 = add i64 %tmp3, %tmp4"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 %tmp5)"), "{ir}");
}

#[test]
fn emit_ir_rejects_arithmetic_after_untracked_overflow_sensitive_shift() {
    let error = emit_ir_source("<?php\n$shifted = 4611686018427387904 << 1;\necho $shifted + 1;\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_INTEGER_OVERFLOW_ARITHMETIC_REJECTION);
}

#[test]
fn emit_ir_rejects_unsupported_shifts_with_specific_boundary() {
    for source in [
        "<?php\necho 8 << -1;\n",
        "<?php\necho 8 << 64;\n",
        "<?php\n$seed = 1 + 2;\n$flag = $seed === 3;\n$count = $flag ? 1 : 2;\necho 8 << $count;\n",
        "<?php\necho true << 1;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_BITWISE_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_unsupported_bitwise_operands() {
    let error = emit_ir_source("<?php\necho true & 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn emit_ir_rejects_unsupported_unary_bitwise_not_operands() {
    let error = emit_ir_source("<?php\necho ~true;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn emit_asm_rejects_bitwise_operators_before_backend_execution() {
    let error = emit_asm_source("<?php\necho true & 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_BITWISE_REJECTION);
}

#[test]
fn native_integer_bitwise_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone275/native_integer_bitwise_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone275/native_integer_bitwise_emit_ir.cli"),
    )
    .expect("native integer bitwise CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone335/native_integer_bitwise_result_tracking.php");
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
            .join("tests/fixtures/milestone335/native_integer_bitwise_result_tracking_emit_ir.cli"),
    )
    .expect("native integer bitwise result-tracking CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_integer_expr_bitwise_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone413/native_identical_integer_expr_bitwise.php");
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
            .join("tests/fixtures/milestone413/native_identical_integer_expr_bitwise_emit_ir.cli"),
    )
    .expect("native identical integer expression bitwise IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_zero_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone435/native_integer_bitwise_zero_identity.php");
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
            .join("tests/fixtures/milestone435/native_integer_bitwise_zero_identity_emit_ir.cli"),
    )
    .expect("native integer bitwise zero identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_integer_bitwise_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone501/native_untracked_integer_bitwise_identity.php");
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
            "tests/fixtures/milestone501/native_untracked_integer_bitwise_identity_emit_ir.cli",
        ))
        .expect("native untracked integer bitwise identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_identical_integer_bitwise_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone505/native_untracked_identical_integer_bitwise.php");
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
        "tests/fixtures/milestone505/native_untracked_identical_integer_bitwise_emit_ir.cli",
    ))
    .expect("native untracked identical integer bitwise IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_all_ones_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone436/native_integer_bitwise_all_ones_identity.php");
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
            "tests/fixtures/milestone436/native_integer_bitwise_all_ones_identity_emit_ir.cli",
        ))
        .expect("native integer bitwise all-ones identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_or_all_ones_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone499/native_integer_bitwise_or_all_ones.php");
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
            .join("tests/fixtures/milestone499/native_integer_bitwise_or_all_ones_emit_ir.cli"),
    )
    .expect("native integer bitwise OR all-ones IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_xor_all_ones_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone500/native_integer_bitwise_xor_all_ones.php");
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
            .join("tests/fixtures/milestone500/native_integer_bitwise_xor_all_ones_emit_ir.cli"),
    )
    .expect("native integer bitwise XOR all-ones IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_and_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone437/native_integer_bitwise_and_zero.php");
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
            .join("tests/fixtures/milestone437/native_integer_bitwise_and_zero_emit_ir.cli"),
    )
    .expect("native integer bitwise AND zero IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_literal_bitwise_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone454/native_integer_literal_bitwise_identity.php");
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
            "tests/fixtures/milestone454/native_integer_literal_bitwise_identity_emit_ir.cli",
        ))
        .expect("native integer literal bitwise identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_integer_shift_by_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone503/native_untracked_integer_shift_by_zero.php");
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
            .join("tests/fixtures/milestone503/native_untracked_integer_shift_by_zero_emit_ir.cli"),
    )
    .expect("native untracked integer shift-by-zero IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_integer_double_bitwise_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone510/native_untracked_integer_double_bitwise_not.php");
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
        "tests/fixtures/milestone510/native_untracked_integer_double_bitwise_not_emit_ir.cli",
    ))
    .expect("native untracked integer double bitwise-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_shift_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone278/native_integer_shift_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone278/native_integer_shift_emit_ir.cli"),
    )
    .expect("native integer shift CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_shift_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone338/native_integer_shift_result_tracking.php");
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
            .join("tests/fixtures/milestone338/native_integer_shift_result_tracking_emit_ir.cli"),
    )
    .expect("native integer shift result-tracking CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_integer_bitwise_shift_result_tracking_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone350/native_bounded_integer_bitwise_shift_result_tracking.php",
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
        "tests/fixtures/milestone350/native_bounded_integer_bitwise_shift_result_tracking_emit_ir.cli",
    ))
    .expect("native bounded integer bitwise/shift result-tracking CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_shift_by_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone431/native_integer_shift_by_zero.php");
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
        workspace_root.join("tests/fixtures/milestone431/native_integer_shift_by_zero_emit_ir.cli"),
    )
    .expect("native integer shift-by-zero IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_literal_shift_by_zero_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone455/native_integer_literal_shift_by_zero.php");
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
            .join("tests/fixtures/milestone455/native_integer_literal_shift_by_zero_emit_ir.cli"),
    )
    .expect("native integer literal shift-by-zero IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_bitwise_not_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone456/native_integer_bitwise_not_folding.php");
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
            .join("tests/fixtures/milestone456/native_integer_bitwise_not_folding_emit_ir.cli"),
    )
    .expect("native integer bitwise-not folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_integer_bitwise_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone467/native_tracked_integer_bitwise_folding.php");
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
            .join("tests/fixtures/milestone467/native_tracked_integer_bitwise_folding_emit_ir.cli"),
    )
    .expect("native tracked integer bitwise folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_expression_integer_bitwise_folding_emit_ir_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone478/native_tracked_expression_integer_bitwise_folding.php");
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
        "tests/fixtures/milestone478/native_tracked_expression_integer_bitwise_folding_emit_ir.cli",
    ))
    .expect("native tracked-expression integer bitwise folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_integer_shift_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone468/native_tracked_integer_shift_folding.php");
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
            .join("tests/fixtures/milestone468/native_tracked_integer_shift_folding_emit_ir.cli"),
    )
    .expect("native tracked integer shift folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_integer_shift_count_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone479/native_tracked_integer_shift_count.php");
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
            .join("tests/fixtures/milestone479/native_tracked_integer_shift_count_emit_ir.cli"),
    )
    .expect("native tracked integer shift-count IR CLI snapshot is readable");
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
