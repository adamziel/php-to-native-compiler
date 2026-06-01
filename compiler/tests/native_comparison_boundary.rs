use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_COMPARISON_REJECTION: &str = "LLVM comparison lowering rejects unsupported comparison operands until native PHP comparison coercions exist; same-type null, boolean, integer, finite float, known ASCII nonnumeric string comparisons, and identical string-pointer self-comparisons are lowered for the current native subset; phpc run handles current scalar comparison diagnostics";

#[test]
fn phpc_run_still_handles_current_scalar_comparisons() {
    let execution = run_source(
        r#"<?php
echo 1 == "1", "\n";
echo 1 != 2, "\n";
echo 2 < 3, "\n";
echo 3 <= 3, "\n";
echo 4 > 3, "\n";
echo 4 >= 4, "\n";
echo 1 === 1, "\n";
echo 1 !== "1", "\n";
echo null == false, "\n";
echo "10" > 2;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\n1\n1\n1\n1\n1\n1\n1\n1\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn phpc_run_handles_spaceship_and_less_greater_not_equal_alias() {
    let execution = run_source(
        r#"<?php
var_dump(3 <=> 2);
var_dump(2 <=> "3");
var_dump("a" <=> "a");
var_dump(true <=> []);
var_dump(679 <> "679");
var_dump(679 <> "679abc");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(1)\nint(-1)\nint(0)\nint(1)\nbool(false)\nbool(true)\n"
    );
}

#[test]
fn emit_ir_rejects_unsupported_comparison_operands_with_specific_boundary() {
    for source in [
        "<?php\necho \"1\" == \"1\";\n",
        "<?php\necho \"1\" != \"2\";\n",
        "<?php\necho \"1\" < \"2\";\n",
        "<?php\necho \"1\" <= \"1\";\n",
        "<?php\necho \"2\" > \"1\";\n",
        "<?php\necho \"2\" >= \"2\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_spaceship_until_native_value_ordering_result_exists() {
    let error = emit_ir_source("<?php\necho 1 <=> 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
}

#[test]
fn emit_ir_rejects_comparison_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] == [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
}

#[test]
fn emit_ir_lowers_same_type_integer_comparison_operators() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$limit = $sum + 3;

echo $sum == 3, "\n";
echo $sum != 4, "\n";
echo $sum < $limit, "\n";
echo $sum <= 3, "\n";
echo $limit > $sum, "\n";
echo $limit >= 6;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(
        !ir.contains("add i64 %tmp0, 3"),
        "tracked single-result integer arithmetic feeding comparison should fold:\n{ir}"
    );
    for redundant in [
        "icmp eq i64 %tmp0, 3",
        "icmp ne i64 %tmp0, 4",
        "icmp slt i64 %tmp0, 6",
        "icmp sle i64 %tmp0, 3",
        "icmp sgt i64 6, %tmp0",
        "icmp sge i64 6, 6",
    ] {
        assert!(
            !ir.contains(redundant),
            "tracked single-result integer comparison should fold `{redundant}`:\n{ir}"
        );
    }
    assert_eq!(ir.matches("select i1").count(), 0, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
}

#[test]
fn emit_ir_tracks_known_integer_comparison_results_for_later_boolean_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_small = $sum < 4;
$is_large = $sum > 9;
$choice = $is_three ? 2 : 4;
$ambiguous = $sum < $choice;

echo ($is_small === true) ? 1 : 0, "\n";
echo ($is_large === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("icmp slt i64 %tmp0, 4"),
        "known true integer comparison should fold before boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp sgt i64 %tmp0, 9"),
        "known false integer comparison should fold before boolean identity:\n{ir}"
    );
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 2, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp slt i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1"),
        "known true comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1"),
        "known false comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp3, i64 1, i64 0"),
        "ambiguous comparison result should feed boolean-literal identity without an extra comparison:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_tracked_single_result_integer_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$seed = 2 + 2;
$flag = $seed === 4;
$bounded = $flag ? 3 : 4;

echo $sum == 3, "\n";
echo $sum != 4, "\n";
echo 2 < $sum, "\n";
echo 4 <= $sum, "\n";
echo $sum > 1, "\n";
echo $sum >= 4, "\n";
echo 1 < 2, "\n";
echo $bounded == 3;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i64 %tmp0, 3"),
        "tracked single-result integer == literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i64 %tmp0, 4"),
        "tracked single-result integer != literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp slt i64 2, %tmp0"),
        "literal < tracked single-result integer should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp sle i64 4, %tmp0"),
        "literal <= tracked single-result integer should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp sgt i64 %tmp0, 1"),
        "tracked single-result integer > literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp sge i64 %tmp0, 4"),
        "tracked single-result integer >= literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp slt i64 1, 2"),
        "literal-only integer comparison should keep existing static fold:\n{ir}"
    );
    assert!(
        ir.contains("icmp eq i64 %tmp"),
        "non-single tracked integer comparison should stay emitted:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_reflexive_integer_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;

echo $value, "\n";
echo $value == $value, "\n";
echo $value != $value, "\n";
echo $value < $value, "\n";
echo $value <= $value, "\n";
echo $value > $value, "\n";
echo $value >= $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    for redundant in [
        "icmp eq i64 %tmp0, %tmp0",
        "icmp ne i64 %tmp0, %tmp0",
        "icmp slt i64 %tmp0, %tmp0",
        "icmp sle i64 %tmp0, %tmp0",
        "icmp sgt i64 %tmp0, %tmp0",
        "icmp sge i64 %tmp0, %tmp0",
    ] {
        assert!(
            !ir.contains(redundant),
            "untracked reflexive integer comparison should fold `{redundant}`:\n{ir}"
        );
    }
    assert!(ir.contains("@phpc_native_int(i64 %tmp0)"), "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.matches("@phpc_native_bool(i1").count() >= 3, "{ir}");
}

#[test]
fn emit_ir_lowers_same_type_finite_float_comparison_operators() {
    let ir = emit_ir_source(
        r#"<?php
$left = 1.25 + 2.5;
$right = $left + 1.0;

echo $left == 3.75, "\n";
echo $left != 4.25, "\n";
echo $left < $right, "\n";
echo $left <= 3.75, "\n";
echo $right > $left, "\n";
echo $right >= 4.75;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.25, 2.5"), "{ir}");
    assert!(
        !ir.contains("fcmp oeq double %tmp0, 3.75"),
        "tracked single-result float == literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp une double %tmp0, 4.25"),
        "tracked single-result float != literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fadd double %tmp0, 1.0"),
        "tracked single-result float arithmetic feeding comparison should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp olt double %tmp0, 4.75"),
        "tracked single-result float < literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp ole double %tmp0, 3.75"),
        "tracked single-result float <= literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp ogt double 4.75, %tmp0"),
        "literal > tracked single-result float should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp oge double 4.75, 4.75"),
        "literal-only final float comparison should keep existing static fold:\n{ir}"
    );
    assert_eq!(ir.matches("select i1").count(), 0, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
}

#[test]
fn emit_ir_folds_tracked_single_result_float_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$value = 1.25 + 2.5;
$seed = 2 + 2;
$flag = $seed === 4;
$bounded = $flag ? 3.75 : 4.75;

echo $value == 3.75, "\n";
echo $value != 4.25, "\n";
echo 2.5 < $value, "\n";
echo 3.5 <= $value, "\n";
echo $value > 1.25, "\n";
echo $value >= 4.0, "\n";
echo 1.25 < 2.5, "\n";
echo $bounded == 3.75;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.25, 2.5"), "{ir}");
    for redundant in [
        "fcmp oeq double %tmp0, 3.75",
        "fcmp une double %tmp0, 4.25",
        "fcmp olt double 2.5, %tmp0",
        "fcmp ole double 3.5, %tmp0",
        "fcmp ogt double %tmp0, 1.25",
        "fcmp oge double %tmp0, 4.0",
    ] {
        assert!(
            !ir.contains(redundant),
            "tracked single-result float comparison should fold `{redundant}`:\n{ir}"
        );
    }
    assert!(
        !ir.contains("fcmp olt double 1.25, 2.5"),
        "literal-only float comparison should keep existing static fold:\n{ir}"
    );
    assert!(
        ir.contains("fcmp oeq double %tmp"),
        "non-single tracked float comparison should stay emitted:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
}

#[test]
fn emit_ir_tracks_known_float_comparison_results_for_later_boolean_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 1.25 : 1.25;
$is_small = $value < 2.0;
$is_large = $value > 9.0;
$choice = $flag ? 1.0 : 2.0;
$ambiguous = $value < $choice;

echo ($is_small === true) ? 1 : 0, "\n";
echo ($is_large === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, double 1.0, double 2.0"),
        "{ir}"
    );
    assert!(ir.contains("%tmp3 = fcmp olt double 1.25, %tmp2"), "{ir}");
    assert!(
        !ir.contains("fcmp olt double 1.25, 2.0"),
        "known true float comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp ogt double 1.25, 9.0"),
        "known false float comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp3, i64 1, i64 0"),
        "ambiguous float comparison result should feed boolean-literal identity without an extra comparison:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_same_type_boolean_comparison_operators() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$other = !$flag;

echo $flag == true, "\n";
echo $flag != $other, "\n";
echo $other < $flag, "\n";
echo $other <= false, "\n";
echo $flag > $other, "\n";
echo $flag >= true;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp1, true"),
        "boolean == true should reuse the boolean expression:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1"),
        "known boolean logical-not results should fold before comparison lowering:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1"),
        "known boolean != comparison should fold without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ult i1"),
        "known boolean < comparison should fold without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ule i1 %tmp2, false"),
        "boolean <= false should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ugt i1"),
        "known boolean > comparison should fold without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp uge i1 %tmp1, true"),
        "boolean >= true should reuse the boolean expression:\n{ir}"
    );
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
}

#[test]
fn emit_ir_tracks_known_boolean_comparison_results_for_later_boolean_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$not_flag = !$flag;
$is_true = $flag == true;
$is_false = $not_flag == true;
$choice = $flag ? false : true;
$ambiguous = $flag == $choice;

echo ($is_true === true) ? 1 : 0, "\n";
echo ($is_false === false) ? 1 : 0, "\n";
echo ($ambiguous === false) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("xor i1"),
        "known boolean logical-not and ternary results should fold before later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp1, true"),
        "known true boolean comparison should reuse the boolean expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp2, true"),
        "known false boolean comparison should reuse the inverted boolean expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp1, true"),
        "known true boolean comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp2, false"),
        "known false boolean comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp4, false"),
        "ambiguous boolean comparison result compared with false should avoid an extra comparison:\n{ir}"
    );
    assert_eq!(ir.matches("@phpc_native_int(i64 1)").count(), 3, "{ir}");
}

#[test]
fn emit_ir_folds_known_boolean_expression_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$always_left = $sum === 3;
$always_right = $sum !== 4;
$choice = $always_left ? 3 : 4;
$ambiguous = $sum === $choice;

echo $always_left == $always_right, "\n";
echo $always_left != $always_right, "\n";
echo $always_left == $ambiguous;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = icmp ne i64 %tmp0, 4"),
        "source boolean expression should stay emitted and tracked:\n{ir}"
    );
    assert!(ir.contains("%tmp3 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp1, %tmp2"),
        "known boolean expression equality should fold without a boolean comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp1, %tmp2"),
        "known boolean expression inequality should fold without a boolean comparison:\n{ir}"
    );
    assert!(
        ir.contains("icmp eq i64 %tmp0, %tmp3"),
        "ambiguous source comparison should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("icmp eq i1 %tmp1, %tmp4"),
        "ambiguous boolean comparison should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp"),
        "ambiguous boolean comparison result should still feed echo conversion:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_ambiguous_boolean_expression_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$ambiguous = $sum === $choice;

echo ($ambiguous == $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous != $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous < $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous <= $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous > $ambiguous) ? 1 : 0, "\n";
echo ($ambiguous >= $ambiguous) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"),
        "source ambiguous boolean expression should stay emitted:\n{ir}"
    );
    for redundant in [
        "icmp eq i1 %tmp3, %tmp3",
        "icmp ne i1 %tmp3, %tmp3",
        "icmp ult i1 %tmp3, %tmp3",
        "icmp ule i1 %tmp3, %tmp3",
        "icmp ugt i1 %tmp3, %tmp3",
        "icmp uge i1 %tmp3, %tmp3",
    ] {
        assert!(
            !ir.contains(redundant),
            "identical ambiguous boolean comparison should fold `{redundant}`:\n{ir}"
        );
    }
    assert_eq!(ir.matches("@phpc_native_int(i64 1)").count(), 3, "{ir}");
    assert_eq!(ir.matches("@phpc_native_int(i64 0)").count(), 3, "{ir}");
}

#[test]
fn emit_ir_folds_identical_untracked_string_expression_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$ambiguous = $sum === $choice;
$left = $ambiguous ? "alpha" : "bravo";
$middle = $ambiguous ? "charlie" : "delta";
$wide = $ambiguous ? $left : "echo";
$text = $ambiguous ? $wide : $middle;

echo ($text == $text) ? 1 : 0, "\n";
echo ($text != $text) ? 1 : 0, "\n";
echo ($text < $text) ? 1 : 0, "\n";
echo ($text <= $text) ? 1 : 0, "\n";
echo ($text > $text) ? 1 : 0, "\n";
echo ($text >= $text) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"),
        "source ambiguous boolean condition should stay emitted:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        8,
        "untracked source string expression should stay emitted through pointer and length selects:\n{ir}"
    );
    assert!(
        !ir.contains("@strcmp"),
        "identical untracked string expression comparisons should fold without strcmp:\n{ir}"
    );
    assert_eq!(ir.matches("@phpc_native_int(i64 1)").count(), 3, "{ir}");
    assert_eq!(ir.matches("@phpc_native_int(i64 0)").count(), 3, "{ir}");
}

#[test]
fn emit_ir_folds_bounded_integer_comparisons_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$bounded = $flag ? 7 : 8;
$other = $flag ? 2 : 4;

echo $bounded > 6, "\n";
echo $bounded < 10, "\n";
echo 1 < $other, "\n";
echo $other >= $bounded, "\n";
echo $bounded == 7;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 7, i64 8"), "{ir}");
    assert!(ir.contains("%tmp3 = select i1 %tmp1, i64 2, i64 4"), "{ir}");
    assert!(
        !ir.contains("icmp sgt i64 %tmp2, 6"),
        "bounded greater-than comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("icmp slt i64 %tmp2, 10"),
        "bounded less-than comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("icmp slt i64 1, %tmp3"),
        "literal-vs-bounded comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("icmp sge i64 %tmp3, %tmp2"),
        "bounded-vs-bounded comparison should fold to false:\n{ir}"
    );
    assert!(
        ir.contains("icmp eq i64 %tmp2, 7"),
        "ambiguous bounded comparison should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp"),
        "ambiguous bounded comparison should still feed boolean echo conversion:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_bounded_float_comparisons_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$bounded = $flag ? 7.5 : 8.5;
$other = $flag ? 2.5 : 4.5;

echo $bounded > 6.5, "\n";
echo $bounded < 10.5, "\n";
echo 1.5 < $other, "\n";
echo $other >= $bounded, "\n";
echo $bounded == 7.5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, double 7.5, double 8.5"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp3 = select i1 %tmp1, double 2.5, double 4.5"),
        "{ir}"
    );
    assert!(
        !ir.contains("fcmp ogt double %tmp2, 6.5"),
        "bounded greater-than comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp olt double %tmp2, 10.5"),
        "bounded less-than comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp olt double 1.5, %tmp3"),
        "literal-vs-bounded comparison should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp oge double %tmp3, %tmp2"),
        "bounded-vs-bounded comparison should fold to false:\n{ir}"
    );
    assert!(
        ir.contains("fcmp oeq double %tmp2, 7.5"),
        "ambiguous bounded comparison should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp"),
        "ambiguous bounded comparison should still feed boolean echo conversion:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_known_ascii_nonnumeric_string_comparison_operators() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$left = $flag ? "alpha-2" : "beta_2";
$right = $flag ? "alpha-10" : "beta_10";

echo $left == "alpha-2", "\n";
echo $left != $right, "\n";
echo $left > $right, "\n";
echo $left <= "zeta!", "\n";
echo $right > $left, "\n";
echo $right >= "alpha-10";
"#,
    )
    .unwrap();

    assert!(ir.contains("declare i32 @strcmp(ptr, ptr)"), "{ir}");
    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp4 = select i1 %tmp1, ptr @.str.2, ptr @.str.3"),
        "{ir}"
    );
    assert!(ir.contains("%tmp3 = select i1 %tmp1, i64 7, i64 6"), "{ir}");
    assert!(ir.contains("%tmp5 = select i1 %tmp1, i64 8, i64 7"), "{ir}");
    assert_eq!(ir.matches("call i32 @strcmp").count(), 3, "{ir}");
    assert!(ir.contains("icmp eq i32"), "{ir}");
    assert!(ir.contains("icmp sgt i32"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
}

#[test]
fn emit_ir_tracks_known_string_comparison_results_for_later_boolean_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$word = $flag ? "alpha" : "beta";
$is_word = $word != "gamma";
$is_missing = $word == "gamma";
$choice = $flag ? "alpha" : "gamma";
$ambiguous = $word == $choice;

echo ($is_word === true) ? 1 : 0, "\n";
echo ($is_missing === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("declare i32 @strcmp(ptr, ptr)"), "{ir}");
    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 @strcmp(ptr %tmp2, ptr @.str.2)"),
        "known true string comparison should fold before boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("call i32 @strcmp(ptr %tmp2, ptr @.str.3)"),
        "known false string comparison should fold before boolean identity:\n{ir}"
    );
    assert!(
        ir.contains("%tmp4 = select i1 %tmp1, ptr @.str.4, ptr @.str.5"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp5 = call i32 @strcmp(ptr %tmp2, ptr %tmp4)"),
        "{ir}"
    );
    assert!(ir.contains("%tmp6 = icmp eq i32 %tmp5, 0"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1"),
        "known true string comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1"),
        "known false string comparison result should feed later boolean identity:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp6, i64 1, i64 0"),
        "ambiguous string comparison result should feed boolean-literal identity without an extra comparison:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_bounded_known_string_comparisons() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$word = $flag ? "alpha" : "beta";
$choice = $flag ? "alpha" : "gamma";

echo $word != "gamma", "\n";
echo $word < "gamma", "\n";
echo "aardvark" < $word, "\n";
echo $word >= "alpha", "\n";
echo "zeta" > $word, "\n";
echo $word == $choice;
"#,
    )
    .unwrap();

    assert!(ir.contains("declare i32 @strcmp(ptr, ptr)"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        !ir.contains("call i32 @strcmp(ptr %tmp2, ptr @.str.2)"),
        "bounded string != literal should fold:\n{ir}"
    );
    assert!(
        !ir.contains("call i32 @strcmp(ptr %tmp2, ptr @.str.4)"),
        "bounded string < literal should fold:\n{ir}"
    );
    assert_eq!(
        ir.matches("call i32 @strcmp").count(),
        1,
        "only the ambiguous bounded string comparison should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp"),
        "ambiguous comparison result should still feed echo conversion:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_same_type_null_comparison_operators() {
    let ir = emit_ir_source(
        r#"<?php
echo null == null, "\n";
echo null != null, "\n";
echo null < null, "\n";
echo null <= null, "\n";
echo null > null, "\n";
echo null >= null;
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(!ir.contains("icmp eq i64"), "{ir}");
    assert!(!ir.contains("icmp ne i64"), "{ir}");
    assert!(!ir.contains("icmp slt i64"), "{ir}");
    assert!(!ir.contains("icmp sle i64"), "{ir}");
    assert!(!ir.contains("icmp sgt i64"), "{ir}");
    assert!(!ir.contains("icmp sge i64"), "{ir}");
    assert_eq!(ir.matches("fcmp").count(), 0, "{ir}");
    assert!(!ir.contains("@strcmp"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_rejects_numeric_string_comparison_operands() {
    for source in [
        "<?php\necho \"10\" < \"2\";\n",
        "<?php\necho \" 10\" < \"2\";\n",
        "<?php\necho \"-2\" < \".5\";\n",
        "<?php\necho \".5\" < \"5.\";\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
    }
}

#[test]
fn emit_ir_lowers_string_pairs_when_shared_classifier_selects_binary_comparison() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$left = $flag ? "10" : "8foo";
$right = $flag ? "zeta" : "+foo";

echo $left < $right, "\n";
echo ".name" != "-word", "\n";
echo " 10" < "zeta";
"#,
    )
    .unwrap();

    assert!(ir.contains("declare i32 @strcmp(ptr, ptr)"), "{ir}");
    assert!(
        ir.contains("call i32 @strcmp"),
        "numeric-vs-nonnumeric, leading-numeric, and sign/dot-prefixed nonnumeric string pairs should lower through binary string comparison:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
}

#[test]
fn emit_ir_lowers_static_int_and_bool_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$one = 1;
$same = 1;
$two = 2;
$truth = true;
$falsey = false;

echo $one === $same, "\n";
echo $one !== $two, "\n";
echo $truth === true, "\n";
echo $falsey !== true;
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_lowers_static_string_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$left = "php";
$same = "php";
$right = "native";
$joined = "p" . "hp";

echo $left === $same, "\n";
echo $left === $joined, "\n";
echo $left !== $right, "\n";
echo "x", $left === $right, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_folds_bounded_string_strict_identity_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$word = $flag ? "left" : "right";

echo $word !== "none", "\n";
echo ($word === "none") ? 10 : 20, "\n";
echo ($word === "left") ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert_eq!(ir.matches("call i32 @strcmp").count(), 1, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 20)"), "{ir}");
    assert!(ir.contains("icmp eq i32"), "{ir}");
}

#[test]
fn emit_ir_lowers_static_float_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$left = 1.5;
$same = 1.5;
$right = 2.5;
$one = 1.0;

echo $left === $same, "\n";
echo $left !== $right, "\n";
echo $one === 1.0, "\n";
echo "x", $left === $right, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_folds_bounded_float_strict_identity_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? 1.5 : 2.5;

echo $value !== 9.5, "\n";
echo ($value === 9.5) ? 10 : 20, "\n";
echo ($value === 1.5) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, double 1.5, double 2.5"),
        "{ir}"
    );
    assert_eq!(ir.matches("fcmp").count(), 1, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 20)"), "{ir}");
    assert!(ir.contains("fcmp oeq double %tmp2, 1.5"), "{ir}");
}

#[test]
fn emit_ir_lowers_static_null_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$nil = null;
$also = null;

echo $nil === $also, "\n";
echo null === null, "\n";
echo "x", $nil !== null, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_lowers_mixed_scalar_strict_identity_from_type_alone() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$text = "3";
$nil = null;
$flag = true;
$float = 3.0;

echo $sum !== $float, "\n";
echo $text !== $sum, "\n";
echo $nil !== false, "\n";
echo $flag !== 1, "\n";
echo "x", $sum === $float, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_lowers_dynamic_integer_strict_identity_with_boolean_echo() {
    let ir = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$y = 3 * 2;
echo $x === 3, "\n";
echo $y !== 6, "x";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = mul i64 3, 2"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("icmp ne i64 %tmp1, 6"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_bounded_integer_strict_identity_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;

echo $value !== 7, "\n";
echo ($value === 7) ? 10 : 20;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 5, i64 6"), "{ir}");
    assert_eq!(ir.matches("icmp eq i64").count(), 1, "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 20)"), "{ir}");
}

#[test]
fn emit_ir_keeps_ambiguous_bounded_integer_strict_identity_dynamic() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$value = $flag ? 5 : 6;

echo $value === 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 5, i64 6"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp2, 5"), "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 %tmp3)"), "{ir}");
}

#[test]
fn emit_ir_folds_reflexive_dynamic_scalar_strict_identity_for_safe_operands() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$int = $flag ? 5 : 6;
$float = $flag ? 1.5 : 2.5;
$word = $flag ? "left" : "right";
$bool = $flag ? true : false;

echo ($int === $int) ? 1 : 0, "\n";
echo ($float !== $float) ? 1 : 0, "\n";
echo ($word === $word) ? 1 : 0, "\n";
echo ($bool !== $bool) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 5, i64 6"), "{ir}");
    assert!(
        ir.contains("%tmp3 = select i1 %tmp1, double 1.5, double 2.5"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp4 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 false"),
        "boolean literal ternary should fold to the condition:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i64 %tmp2, %tmp2"),
        "reflexive int identity should fold:\n{ir}"
    );
    assert!(
        !ir.contains("fcmp"),
        "reflexive finite float identity should fold:\n{ir}"
    );
    assert!(
        !ir.contains("@strcmp"),
        "reflexive string pointer identity should fold:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp1, %tmp1"),
        "reflexive bool identity should fold:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 0)"), "{ir}");
}

#[test]
fn emit_ir_tracks_known_integer_strict_identity_results_for_later_boolean_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$choice = $is_three ? 3 : 4;
$ambiguous = $sum === $choice;

echo ($is_three === true) ? 1 : 0, "\n";
echo ($is_four === false) ? 1 : 0, "\n";
echo ($ambiguous === true) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp1, true"),
        "known true integer identity result should feed later boolean identity:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp2, false"),
        "known false integer identity result should feed later boolean identity:\n{ir}"
    );
    assert!(ir.contains("%tmp3 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp4 = icmp eq i64 %tmp0, %tmp3"), "{ir}");
    assert!(
        ir.contains("select i1 %tmp4, i64 1, i64 0"),
        "ambiguous integer identity result should feed boolean-literal identity without an extra comparison:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_dynamic_float_strict_identity_with_boolean_echo() {
    let ir = emit_ir_source(
        r#"<?php
$seed = 1 + 2;
$flag = $seed === 3;
$sum = $flag ? 3.75 : 4.25;
echo $sum === 3.75, "\n";
echo $sum !== 4.25, "x";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, double 3.75, double 4.25"),
        "{ir}"
    );
    assert!(ir.contains("%tmp3 = fcmp oeq double %tmp2, 3.75"), "{ir}");
    assert!(ir.contains("fcmp une double %tmp2, 4.25"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
}

#[test]
fn emit_ir_lowers_dynamic_boolean_strict_identity_with_boolean_echo() {
    let ir = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$is_three = $x === 3;
$choice = $is_three ? 3 : 4;
$maybe = $x === $choice;
echo $maybe === true, "\n";
echo $maybe !== false, "x";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp3, true"),
        "dynamic boolean === true should reuse the boolean expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp3, false"),
        "dynamic boolean !== false should reuse the boolean expression:\n{ir}"
    );
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_dynamic_boolean_strict_identity_with_bool_literals() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum === $choice;

echo $maybe === true, "\n";
echo true === $maybe, "\n";
echo $maybe !== false, "\n";
echo false !== $maybe;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp3, true"),
        "dynamic boolean expression === true should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp3, false"),
        "dynamic boolean expression !== false should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 true, %tmp3"),
        "true === dynamic boolean expression should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 false, %tmp3"),
        "false !== dynamic boolean expression should reuse the expression:\n{ir}"
    );
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_inverse_dynamic_boolean_strict_identity_with_bool_literals() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum === $choice;

echo $maybe === false, "\n";
echo false === $maybe, "\n";
echo $maybe !== true, "\n";
echo true !== $maybe;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp3, false"),
        "dynamic boolean expression === false should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp3, true"),
        "dynamic boolean expression !== true should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 false, %tmp3"),
        "false === dynamic boolean expression should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 true, %tmp3"),
        "true !== dynamic boolean expression should invert without an extra comparison:\n{ir}"
    );
    assert_eq!(ir.matches("xor i1 %tmp3, true").count(), 4, "{ir}");
}

#[test]
fn emit_ir_folds_dynamic_boolean_loose_comparison_with_bool_literals() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum == $choice;

echo $maybe == true, "\n";
echo true == $maybe, "\n";
echo $maybe != false, "\n";
echo false != $maybe, "\n";
echo $maybe == false, "\n";
echo false == $maybe, "\n";
echo $maybe != true, "\n";
echo true != $maybe;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp eq i1 %tmp3, true"),
        "dynamic boolean expression == true should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp3, false"),
        "dynamic boolean expression != false should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp eq i1 %tmp3, false"),
        "dynamic boolean expression == false should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ne i1 %tmp3, true"),
        "dynamic boolean expression != true should invert without an extra comparison:\n{ir}"
    );
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert_eq!(ir.matches("xor i1 %tmp3, true").count(), 4, "{ir}");
}

#[test]
fn emit_ir_folds_dynamic_boolean_ordering_comparison_with_bool_literals() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$choice = $flag ? 3 : 4;
$maybe = $sum == $choice;

echo $maybe < true, "\n";
echo $maybe > false, "\n";
echo $maybe <= true, "\n";
echo $maybe >= false, "\n";
echo false < $maybe, "\n";
echo true > $maybe, "\n";
echo false <= $maybe, "\n";
echo true >= $maybe;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(
        !ir.contains("icmp ult i1 %tmp3, true"),
        "dynamic boolean expression < true should invert without an extra comparison:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ugt i1 %tmp3, false"),
        "dynamic boolean expression > false should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ule i1 %tmp3, true"),
        "dynamic boolean expression <= true should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("icmp uge i1 %tmp3, false"),
        "dynamic boolean expression >= false should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ult i1 false, %tmp3"),
        "false < dynamic boolean expression should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("icmp ugt i1 true, %tmp3"),
        "true > dynamic boolean expression should invert without an extra comparison:\n{ir}"
    );
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert_eq!(ir.matches("xor i1 %tmp3, true").count(), 2, "{ir}");
}

#[test]
fn emit_ir_folds_bounded_boolean_strict_identity_when_all_outcomes_match() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$value = $flag ? true : true;
$inverse = !$value;
$ambiguous = $flag ? true : false;

echo $value === true, "\n";
echo ($inverse === true) ? 10 : 20, "\n";
echo ($ambiguous === true) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 true"),
        "condition ? true : true should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 false"),
        "condition ? true : false should fold to the condition:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1"),
        "inverting a statically true ternary should fold to false:\n{ir}"
    );
    assert!(!ir.contains("icmp eq i1 %tmp2, true"), "{ir}");
    assert!(!ir.contains("icmp eq i1 %tmp3, true"), "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1 true"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 20)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
}

#[test]
fn emit_ir_lowers_dynamic_string_strict_identity_with_boolean_echo() {
    let ir = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$is_three = $x === 3;
$word = $is_three ? "alpha" : "beta";

echo $word === "alpha", "\n";
echo $word !== "gamma", "x";
"#,
    )
    .unwrap();

    assert!(ir.contains("declare i32 @strcmp(ptr, ptr)"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp4 = call i32 @strcmp(ptr %tmp2, ptr @.str.2)"),
        "{ir}"
    );
    assert!(ir.contains("%tmp5 = icmp eq i32 %tmp4, 0"), "{ir}");
    assert_eq!(ir.matches("call i32 @strcmp").count(), 1, "{ir}");
    assert!(!ir.contains("gamma"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_mixed_dynamic_boolean_strict_identity_from_type_alone() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;

echo $is_three, "\n";
echo $is_three !== 1, "\n";
echo $is_three !== "1", "\n";
echo $is_three !== null, "\n";
echo $is_three !== 1.0, "\n";
echo "x", $is_three === 1, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert_eq!(ir.matches("icmp eq i1").count(), 0, "{ir}");
    assert_eq!(ir.matches("icmp ne i1").count(), 0, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_folds_mixed_dynamic_float_strict_identity_from_type_alone() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1.5 + 2.25;

echo $sum !== 1, "\n";
echo $sum !== "3.75", "\n";
echo $sum !== null, "\n";
echo $sum !== true, "\n";
echo "x", $sum === 1, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert_eq!(ir.matches("fcmp").count(), 0, "{ir}");
    assert_eq!(ir.matches("select i1").count(), 0, "{ir}");
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_folds_mixed_dynamic_string_strict_identity_from_type_alone() {
    let ir = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$is_three = $x === 3;
$word = $is_three ? "alpha" : "beta";

echo $word !== 1, "\n";
echo $word !== 1.0, "\n";
echo $word !== null, "\n";
echo $word !== true, "\n";
echo "x", $word === 1, "y";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(!ir.contains("@strcmp"), "{ir}");
    assert!(
        ir.contains("select i1") || ir.contains("@phpc_native_bool(i1"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_bool(i1"), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(ir.contains("c\"y\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_asm_rejects_comparisons_before_backend_execution() {
    let error = emit_asm_source("<?php\necho 1 == \"1\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_COMPARISON_REJECTION);
}

#[test]
fn emit_asm_lowers_same_type_scalar_comparisons_when_backend_is_available() {
    if !has_assembly_backend() {
        return;
    }

    let asm = emit_asm_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$not_flag = !$flag;
$left = 1.25 + 2.5;
$right = $left + 1.0;
$word = $flag ? "alpha" : "beta";
echo $sum < 4, "\n";
echo $sum >= 3, "\n";
echo $not_flag < $flag, "\n";
echo $left < $right, "\n";
echo $word < "zeta", "\n";
echo null <= null;
"#,
    )
    .unwrap();

    assert!(asm.contains("main"), "{asm}");
}

#[test]
fn native_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone165/native_comparison_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone165/native_comparison_boundary_emit_ir.cli"),
    )
    .expect("native comparison CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone242/native_static_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone242/native_static_strict_identity_emit_ir.cli"),
    )
    .expect("native static strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_string_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone248/native_static_string_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone248/native_static_string_strict_identity_emit_ir.cli"),
    )
    .expect("native static string strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_float_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone251/native_static_float_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone251/native_static_float_strict_identity_emit_ir.cli"),
    )
    .expect("native static float strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_null_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone254/native_static_null_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone254/native_static_null_strict_identity_emit_ir.cli"),
    )
    .expect("native static null strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mixed_scalar_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone257/native_mixed_scalar_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone257/native_mixed_scalar_strict_identity_emit_ir.cli"),
    )
    .expect("native mixed scalar strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_integer_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone260/native_dynamic_integer_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone260/native_dynamic_integer_strict_identity_emit_ir.cli"),
    )
    .expect("native dynamic integer strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_integer_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone353/native_bounded_integer_strict_identity.php");
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
            .join("tests/fixtures/milestone353/native_bounded_integer_strict_identity_emit_ir.cli"),
    )
    .expect("native bounded integer strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_boolean_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone263/native_dynamic_boolean_strict_identity_emit_ir.cli"),
    )
    .expect("native dynamic boolean strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_literal_strict_identity_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone441/native_boolean_literal_strict_identity_folding.php");
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
        "tests/fixtures/milestone441/native_boolean_literal_strict_identity_folding_emit_ir.cli",
    ))
    .expect("native boolean literal strict-identity folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_inverse_boolean_literal_strict_identity_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone442/native_inverse_boolean_literal_strict_identity_folding.php",
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
        "tests/fixtures/milestone442/native_inverse_boolean_literal_strict_identity_folding_emit_ir.cli",
    ))
    .expect("native inverse boolean literal strict-identity folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_literal_loose_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone445/native_boolean_literal_loose_comparison_folding.php");
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
        "tests/fixtures/milestone445/native_boolean_literal_loose_comparison_folding_emit_ir.cli",
    ))
    .expect("native boolean literal loose-comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_literal_ordering_comparison_folding_emit_ir_cli_snapshot_matches_committed_output(
) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone446/native_boolean_literal_ordering_comparison_folding.php");
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
        "tests/fixtures/milestone446/native_boolean_literal_ordering_comparison_folding_emit_ir.cli",
    ))
    .expect("native boolean literal ordering-comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_float_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone299/native_dynamic_float_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone299/native_dynamic_float_strict_identity_emit_ir.cli"),
    )
    .expect("native dynamic float strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_string_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone305/native_dynamic_string_strict_identity_emit_ir.php");
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
            .join("tests/fixtures/milestone305/native_dynamic_string_strict_identity_emit_ir.cli"),
    )
    .expect("native dynamic string strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mixed_dynamic_boolean_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_emit_ir.php",
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
        "tests/fixtures/milestone266/native_mixed_dynamic_boolean_strict_identity_emit_ir.cli",
    ))
    .expect("native mixed dynamic boolean strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mixed_dynamic_float_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_emit_ir.php");
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
        "tests/fixtures/milestone302/native_mixed_dynamic_float_strict_identity_emit_ir.cli",
    ))
    .expect("native mixed dynamic float strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_mixed_dynamic_string_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join(
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_emit_ir.php",
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
        "tests/fixtures/milestone308/native_mixed_dynamic_string_strict_identity_emit_ir.cli",
    ))
    .expect("native mixed dynamic string strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_string_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone359/native_bounded_string_strict_identity.php");
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
            .join("tests/fixtures/milestone359/native_bounded_string_strict_identity_emit_ir.cli"),
    )
    .expect("native bounded string strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_float_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone362/native_bounded_float_strict_identity.php");
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
            .join("tests/fixtures/milestone362/native_bounded_float_strict_identity_emit_ir.cli"),
    )
    .expect("native bounded float strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_boolean_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone365/native_bounded_boolean_strict_identity.php");
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
            .join("tests/fixtures/milestone365/native_bounded_boolean_strict_identity_emit_ir.cli"),
    )
    .expect("native bounded boolean strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_reflexive_scalar_strict_identity_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone371/native_reflexive_scalar_strict_identity.php");
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
            "tests/fixtures/milestone371/native_reflexive_scalar_strict_identity_emit_ir.cli",
        ))
        .expect("native reflexive scalar strict-identity IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_strict_identity_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone374/native_integer_strict_identity_result_tracking.php");
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
        "tests/fixtures/milestone374/native_integer_strict_identity_result_tracking_emit_ir.cli",
    ))
    .expect("native integer strict-identity result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_comparison_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone377/native_integer_comparison_result_tracking.php");
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
            "tests/fixtures/milestone377/native_integer_comparison_result_tracking_emit_ir.cli",
        ))
        .expect("native integer comparison result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_reflexive_integer_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone504/native_untracked_reflexive_integer_comparison.php");
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
        "tests/fixtures/milestone504/native_untracked_reflexive_integer_comparison_emit_ir.cli",
    ))
    .expect("native untracked reflexive integer comparison IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_integer_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone469/native_tracked_integer_comparison_folding.php");
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
            "tests/fixtures/milestone469/native_tracked_integer_comparison_folding_emit_ir.cli",
        ))
        .expect("native tracked integer comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_tracked_float_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone470/native_tracked_float_comparison_folding.php");
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
            "tests/fixtures/milestone470/native_tracked_float_comparison_folding_emit_ir.cli",
        ))
        .expect("native tracked float comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_string_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone471/native_bounded_string_comparison_folding.php");
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
            "tests/fixtures/milestone471/native_bounded_string_comparison_folding_emit_ir.cli",
        ))
        .expect("native bounded string comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_expression_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone473/native_boolean_expression_comparison_folding.php");
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
        "tests/fixtures/milestone473/native_boolean_expression_comparison_folding_emit_ir.cli",
    ))
    .expect("native boolean expression comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_expression_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone512/native_identical_boolean_expression_comparison.php");
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
        "tests/fixtures/milestone512/native_identical_boolean_expression_comparison_emit_ir.cli",
    ))
    .expect("native identical boolean expression comparison IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_string_expression_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone513/native_identical_string_expression_comparison.php");
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
        "tests/fixtures/milestone513/native_identical_string_expression_comparison_emit_ir.cli",
    ))
    .expect("native identical string expression comparison IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_integer_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone474/native_bounded_integer_comparison_folding.php");
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
            "tests/fixtures/milestone474/native_bounded_integer_comparison_folding_emit_ir.cli",
        ))
        .expect("native bounded integer comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_float_comparison_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone475/native_bounded_float_comparison_folding.php");
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
            "tests/fixtures/milestone475/native_bounded_float_comparison_folding_emit_ir.cli",
        ))
        .expect("native bounded float comparison folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_comparison_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone380/native_float_comparison_result_tracking.php");
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
            "tests/fixtures/milestone380/native_float_comparison_result_tracking_emit_ir.cli",
        ))
        .expect("native float comparison result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_comparison_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone383/native_boolean_comparison_result_tracking.php");
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
            "tests/fixtures/milestone383/native_boolean_comparison_result_tracking_emit_ir.cli",
        ))
        .expect("native boolean comparison result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_string_comparison_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone386/native_string_comparison_result_tracking.php");
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
            "tests/fixtures/milestone386/native_string_comparison_result_tracking_emit_ir.cli",
        ))
        .expect("native string comparison result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_comparison_boundary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone389/native_null_comparison_boundary.php");
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
            .join("tests/fixtures/milestone389/native_null_comparison_boundary_emit_ir.cli"),
    )
    .expect("native null comparison IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_broader_ascii_string_comparison_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone392/native_broader_ascii_string_comparison.php");
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
            .join("tests/fixtures/milestone392/native_broader_ascii_string_comparison_emit_ir.cli"),
    )
    .expect("native broader ASCII string comparison IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

fn has_assembly_backend() -> bool {
    ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok())
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
