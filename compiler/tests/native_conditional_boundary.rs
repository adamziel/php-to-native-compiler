use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_CONDITIONAL_REJECTION: &str = "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior";
#[test]
fn phpc_run_still_handles_current_conditional_expressions() {
    let execution = run_source(
        r#"<?php
$missing = null;
echo true ? "yes" : fail(), "\n";
echo false ?: "fallback", "\n";
echo $missing ?? "coalesced", "\n";
$value = "present";
echo $value ?? fail();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes\nfallback\ncoalesced\npresent");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn native_conditional_boundary_emit_ir_rejection_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone168/native_conditional_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone168/native_conditional_boundary_emit_ir.cli"),
    )
    .expect("native conditional boundary IR rejection CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn emit_ir_lowers_boolean_condition_ternary_for_int_and_bool_branches() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$integer = $is_three ? 10 + 2 : 99;
$boolean = $is_four ? true : $is_three;

echo $integer, "\n";
echo $boolean ? 1 : 0, "z";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        ir.contains("%tmp4 = select i1 %tmp1, i64 %tmp3, i64 99"),
        "{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp2, i1 true, i1 %tmp1"),
        "single-result boolean ternary should fold without a select:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 %tmp4)"), "{ir}");
}

#[test]
fn emit_ir_tracks_integer_ternary_results_for_later_checked_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 10 : 20;
echo $choice + 5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp2 = select i1 %tmp1, i64 10, i64 20"),
        "{ir}"
    );
    assert!(ir.contains("%tmp3 = add i64 %tmp2, 5"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 %tmp3)"), "{ir}");
}

#[test]
fn emit_ir_folds_identical_integer_expression_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_four = $sum === 4;
$same = $is_four ? $sum : $sum;

echo $same + 4;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, i64 %tmp0, i64 %tmp0"),
        "identical integer expression branches should not emit an integer select:\n{ir}"
    );
    assert!(
        !ir.contains("add i64 %tmp0, 4"),
        "known single-result integer ternary should fold into later arithmetic:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
}

#[test]
fn emit_ir_folds_untracked_identical_integer_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$value = 4 << 62;
$seed = 1 + 2;
$flag = $seed === 3;
$same = $flag ? $value : $value;

echo $same;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 4, 62"),
        "overflow-sensitive left shift should stay emitted and untracked:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp1, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp2, i64 %tmp0, i64 %tmp0"),
        "untracked identical integer branches should not emit an integer select:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 %tmp0)"), "{ir}");
}

#[test]
fn emit_ir_folds_identical_integer_variable_full_ternary_without_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$value = 3 << 62;

echo $value ? $value : $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 3, 62"),
        "untracked integer source should stay emitted:\n{ir}"
    );
    assert!(
        !ir.contains("select i1"),
        "identical full ternary should not require truthiness lowering:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_int(i64 %tmp0)"),
        "full ternary should reuse the untracked integer expression:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_float_variable_full_ternary_without_truthiness() {
    let large = "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0";
    let source = format!("<?php\n$value = {large} * {large};\n\necho $value ? $value : $value;\n");
    let ir = emit_ir_source(&source).unwrap();

    assert!(
        ir.contains("%tmp0 = fmul double"),
        "untracked float source should stay emitted:\n{ir}"
    );
    assert!(
        !ir.contains("select i1"),
        "identical full ternary should not require float truthiness lowering:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_float(double %tmp0)"),
        "full ternary should reuse the untracked float expression:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_untracked_identical_float_ternary_branches_without_select() {
    let large = "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0";
    let source = format!(
        "<?php\n$value = {large} * {large};\n$seed = 1 + 2;\n$flag = $seed === 3;\n$same = $flag ? $value : $value;\n\necho $same;\n"
    );
    let ir = emit_ir_source(&source).unwrap();

    assert!(
        ir.contains("%tmp0 = fmul double"),
        "overflowing float multiply should stay emitted and untracked:\n{ir}"
    );
    assert!(ir.contains("%tmp1 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp1, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp2, double %tmp0, double %tmp0"),
        "untracked identical float branches should not emit a float select:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_float(double %tmp0)"), "{ir}");
}

#[test]
fn emit_ir_tracks_bounded_integer_ternary_result_combinations_for_later_checked_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$left = $flag ? 1 : 2;
$right_sum = 2 + 2;
$right_flag = $right_sum === 4;
$right = $right_flag ? 10 : 20;
echo $left + $right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 1, i64 2"), "{ir}");
    assert!(ir.contains("%tmp3 = add i64 2, 2"), "{ir}");
    assert!(ir.contains("%tmp4 = icmp eq i64 %tmp3, 4"), "{ir}");
    assert!(
        ir.contains("%tmp5 = select i1 %tmp4, i64 10, i64 20"),
        "{ir}"
    );
    assert!(ir.contains("%tmp6 = add i64 %tmp2, %tmp5"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 %tmp6)"), "{ir}");
}

#[test]
fn emit_ir_folds_single_result_scalar_ternaries_without_selects() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$other_sum = 2 + 2;
$other = $other_sum === 4;
$float_sum = 1.25 + 2.5;
$int = $other ? $sum : 3;
$float = $other ? $float_sum : 3.75;
$bool = $other ? $flag : true;
$ambiguous = $other ? 10 : 20;

echo $int + 4, "\n";
echo $float + 1.25, "\n";
echo $bool ? 1 : 0, "\n";
echo $ambiguous + 1;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = add i64 2, 2"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp2, 4"), "{ir}");
    assert!(ir.contains("%tmp4 = fadd double 1.25, 2.5"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp3, i64 %tmp0, i64 3"),
        "single-result integer ternary should fold without a select:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp3, double %tmp4, double 3.75"),
        "single-result float ternary should fold without a select:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp3, i1 %tmp1, i1 true"),
        "single-result boolean ternary should fold without a select:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp3, i64 10, i64 20"),
        "ambiguous integer ternary should stay emitted:\n{ir}"
    );
    assert!(ir.contains("add i64 3, 4"), "{ir}");
    assert!(ir.contains("fadd double 3.75, 1.25"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
}

#[test]
fn emit_ir_lowers_boolean_condition_ternary_for_float_branches() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$first = $is_three ? 1.5 : 2.5;
$second = $is_four ? 9.25 : $first;

echo $first, "\n";
echo $second, "z";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = select i1 %tmp1, double 1.5, double 2.5"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp4 = select i1 %tmp2, double 9.25, double %tmp3"),
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_float(double %tmp3)"), "{ir}");
}

#[test]
fn emit_ir_folds_identical_float_expression_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1.5 + 2.5;
$count = 1 + 2;
$is_four = $count === 4;
$same = $is_four ? $sum : $sum;

echo $same + 1.5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.5"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp1, 4"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp2, double %tmp0, double %tmp0"),
        "identical float expression branches should not emit a float select:\n{ir}"
    );
    assert!(
        !ir.contains("fadd double %tmp0, 1.5"),
        "known single-result float ternary should fold into later arithmetic:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_float(double 5.5)"), "{ir}");
}

#[test]
fn emit_ir_folds_identical_numeric_literal_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$integer = $flag ? 5 : 5;
$float = $flag ? 2.5 : 2.5;

echo $integer + 7, "\n";
echo $float + 1.5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, i64 5, i64 5"),
        "identical integer literal branches should not emit an integer select:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, double 2.5, double 2.5"),
        "identical float literal branches should not emit a float select:\n{ir}"
    );
    assert!(ir.contains("add i64 5, 7"), "{ir}");
    assert!(ir.contains("fadd double 2.5, 1.5"), "{ir}");
    assert!(ir.contains("@phpc_native_int"), "{ir}");
    assert!(ir.contains("@phpc_native_float"), "{ir}");
}

#[test]
fn emit_ir_lowers_boolean_condition_ternary_for_string_branches() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$first = $is_three ? "alpha" : "beta";
$second = $is_four ? "gamma" : $first;

echo $first, "\n";
echo $second, "!";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = select i1 %tmp1, ptr @.str.0, ptr @.str.1"),
        "{ir}"
    );
    assert!(ir.contains("%tmp4 = select i1 %tmp1, i64 5, i64 4"), "{ir}");
    assert!(
        ir.contains("%tmp5 = select i1 %tmp2, ptr @.str.2, ptr %tmp3"),
        "{ir}"
    );
    assert!(
        ir.contains("%tmp6 = select i1 %tmp2, i64 5, i64 %tmp4"),
        "{ir}"
    );
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp3, i64 %tmp4"),
        "{ir}"
    );
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp5, i64 %tmp6"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_string_ternary_branches_without_pointer_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$same = $is_three ? "same!" : "same!";

echo $same;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, ptr"),
        "identical string branches should not emit a pointer select:\n{ir}"
    );
    assert_eq!(ir.matches("c\"same!\\00\"").count(), 1, "{ir}");
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr @.str.0, i64 5"),
        "{ir}"
    );
    assert!(
        ir.contains("@phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_boolean_expression_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$same = $is_four ? $is_three : $is_three;

echo $same ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp2, i1 %tmp1, i1 %tmp1"),
        "identical boolean expression branches should not emit a bool select:\n{ir}"
    );
    assert!(
        ir.contains("select i1 %tmp1, i64 1, i64 0"),
        "the later integer ternary should still use the reused boolean expression:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int"), "{ir}");
}

#[test]
fn emit_ir_folds_boolean_literal_ternary_branches_without_select() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$same = $flag ? true : false;
$inverse = $flag ? false : true;
$always = $flag ? true : true;
$never = $flag ? false : false;

echo $same ? 1 : 0, "\n";
echo $inverse ? 1 : 0, "\n";
echo $always ? 1 : 0, "\n";
echo $never ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 false"),
        "condition ? true : false should reuse the condition:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 false, i1 true"),
        "condition ? false : true should invert the condition:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 true"),
        "condition ? true : true should fold to true:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 false, i1 false"),
        "condition ? false : false should fold to false:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1"),
        "known inverse boolean literal ternary result should fold without an explicit xor:\n{ir}"
    );
    assert!(ir.contains("select i1 %tmp1, i64 1, i64 0"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 0)"), "{ir}");
}

#[test]
fn emit_ir_lowers_boolean_short_ternary_subset() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
$choice = $is_three ? 3 : 4;
$maybe_three = $sum === $choice;
$other_choice = $is_three ? 4 : 3;
$maybe_other = $sum === $other_choice;

echo ($is_three ?: false) ? 1 : 0, "\n";
echo ($maybe_three ?: $maybe_other) ? 1 : 0, "\n";
echo ($maybe_three ?: true) ? 1 : 0, "\n";
echo (false ?: $is_four) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 false"),
        "condition ?: false should reuse the condition expression:\n{ir}"
    );
    assert!(ir.contains("select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("select i1 %tmp1, i64 4, i64 3"), "{ir}");
    assert!(ir.contains("icmp eq i64 %tmp0"), "{ir}");
    assert!(
        ir.contains("select i1 %tmp4, i1 true, i1 %tmp6"),
        "dynamic boolean short ternary with expression fallback should lower through select:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp4, i1 true, i1 true"),
        "condition ?: true should fold to true:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1 %tmp4, i1 true, i1 %tmp6").count(),
        1,
        "{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(
        ir.contains("select i1 %tmp2, i64 1, i64 0"),
        "static false short ternary should reuse the boolean fallback expression:\n{ir}"
    );
}

#[test]
fn emit_ir_rejects_non_boolean_short_ternary_operands() {
    for source in [
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? 0 : 5;\necho $maybe ?: 7;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? 0.0 : 1.25;\necho $maybe ?: 7.5;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? \"\" : \"value\";\necho $maybe ?: \"fallback\";\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\necho $flag ?: 1;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
    }
}

#[test]
fn emit_ir_folds_identical_string_variable_short_ternary_without_truthiness() {
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

echo $text ?: $text;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(
        ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"),
        "source ambiguous condition should stay emitted:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        8,
        "untracked source string expression should stay emitted through pointer and length selects:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp9, i64 %tmp10"),
        "identical short ternary should reuse the string pointer and length expression:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp9"),
        "short ternary should not invent pointer truthiness lowering:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_string_variable_full_ternary_without_truthiness() {
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

echo $text ? $text : $text;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"),
        "source ambiguous condition should stay emitted:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        8,
        "untracked source string expression should stay emitted through pointer and length selects only:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp9, i64 %tmp10"),
        "identical full ternary should reuse the string pointer and length expression:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp9"),
        "full ternary should not invent pointer truthiness lowering:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_untracked_integer_short_ternary_without_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$value = 3 << 62;

echo $value ?: $value;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 3, 62"),
        "overflow-sensitive integer source should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("select i1"),
        "identical integer short ternary should not require truthiness lowering:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_int(i64 %tmp0)"),
        "short ternary should reuse the untracked integer expression:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_untracked_float_short_ternary_without_truthiness() {
    let large = "100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000.0";
    let source = format!("<?php\n$value = {large} * {large};\n\necho $value ?: $value;\n");
    let ir = emit_ir_source(&source).unwrap();

    assert!(
        ir.contains("%tmp0 = fmul double"),
        "overflowing float source should stay emitted and untracked:\n{ir}"
    );
    assert!(
        !ir.contains("select i1"),
        "identical float short ternary should not require truthiness lowering:\n{ir}"
    );
    assert!(
        ir.contains("@phpc_native_float(double %tmp0)"),
        "short ternary should reuse the untracked float expression:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_boolean_variable_short_ternary_without_redundant_select() {
    let ir = emit_ir_source(
        r#"<?php
$value = 3 << 62;
$flag = $value !== 0;

echo $flag ?: $flag;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 3, 62"),
        "source integer expression should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("%tmp1 = icmp ne i64 %tmp0, 0"),
        "source boolean expression should stay emitted:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 true, i1 %tmp1"),
        "identical boolean short ternary should not emit a redundant boolean select:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        0,
        "boxed boolean stdout should not require an extra select:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_boolean_variable_full_ternary_without_redundant_select() {
    let ir = emit_ir_source(
        r#"<?php
$value = 3 << 62;
$flag = $value !== 0;

echo $flag ? $flag : $flag;
"#,
    )
    .unwrap();

    assert!(
        ir.contains("%tmp0 = shl i64 3, 62"),
        "source integer expression should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("%tmp1 = icmp ne i64 %tmp0, 0"),
        "source boolean expression should stay emitted:\n{ir}"
    );
    assert!(
        !ir.contains("select i1 %tmp1, i1 %tmp1, i1 %tmp1"),
        "identical boolean full ternary should not emit a redundant boolean select:\n{ir}"
    );
    assert_eq!(
        ir.matches("select i1").count(),
        0,
        "boxed boolean stdout should not require an extra select:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_identical_null_variable_full_ternary_without_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$value = null;

echo "a", $value ? $value : $value, "b";
"#,
    )
    .unwrap();

    assert!(
        !ir.contains("select i1"),
        "identical null full ternary should not require truthiness lowering:\n{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeScalarValue @phpc_native_int("),
        "null full ternary should not emit numeric output:\n{ir}"
    );
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_direct_null_variable_full_ternary_false_branch_without_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$value = null;

echo "a", $value ? fail() : "fallback", "b";
"#,
    )
    .unwrap();

    assert!(
        !ir.contains("select i1"),
        "direct null full ternary should not emit a conditional select:\n{ir}"
    );
    assert!(
        !ir.contains("fail"),
        "unselected full ternary branch should not be lowered:\n{ir}"
    );
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"fallback\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_known_string_short_ternary_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo "literal" ?: [], "\n";
echo "" ?: "empty", "\n";
echo "0" ?: "zero", "\n";
echo $truthy ?: "fallback", "\n";
echo $falsey ?: "falsey";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp3 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp4 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp5 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("c\"literal\\00\""), "{ir}");
    assert!(ir.contains("c\"empty\\00\""), "{ir}");
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(ir.contains("c\"falsey\\00\""), "{ir}");
    assert!(
        ir.contains("@phpc_native_value_from_string_bytes_with_diagnostic(ptr %tmp2, i64 %tmp3"),
        "known-truthy string expression should be reused as the short ternary result:\n{ir}"
    );
    assert!(
        !ir.contains("fallback"),
        "known-truthy string short ternary should not lower the fallback:\n{ir}"
    );
    assert!(
        !ir.contains("LLVM array lowering rejects arrays"),
        "truthy string literal short ternary should not lower the unsupported array fallback:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_single_known_float_short_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1.25 + 2.5;

echo 1.5 ?: [], "\n";
echo 0.0 ?: "zero", "\n";
echo $sum ?: "fallback";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.25, 2.5"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 1.5)"), "{ir}");
    assert!(
        ir.contains("@phpc_native_float(double %tmp0)"),
        "single-known nonzero float expression should be reused as the short ternary result:\n{ir}"
    );
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(
        !ir.contains("fallback"),
        "single-known nonzero float short ternary should not lower the fallback:\n{ir}"
    );
    assert!(!ir.contains("select i1"), "{ir}");
}

#[test]
fn emit_ir_folds_single_known_integer_short_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;

echo 1 ?: [], "\n";
echo 0 ?: "zero", "\n";
echo $sum ?: "fallback";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(
        ir.contains("@phpc_native_int(i64 %tmp0)"),
        "single-known nonzero integer expression should be reused as the short ternary result:\n{ir}"
    );
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(
        !ir.contains("fallback"),
        "single-known nonzero integer short ternary should not lower the fallback:\n{ir}"
    );
    assert!(!ir.contains("select i1"), "{ir}");
}

#[test]
fn emit_ir_folds_single_known_integer_full_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;

echo 1 ? "one" : "bad", "\n";
echo 0 ? "bad" : "zero", "\n";
echo $sum ? 7 : 9;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("c\"one\\00\""), "{ir}");
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
    assert!(!ir.contains("select i1"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$maybe = $flag ? 0 : 5;
echo $maybe ? 1 : 2;
"#,
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_ir_folds_single_known_float_full_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1.25 + 2.5;

echo 1.5 ? "one" : "bad", "\n";
echo 0.0 ? "bad" : "zero", "\n";
echo $sum ? 7.5 : 9.5;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.25, 2.5"), "{ir}");
    assert!(ir.contains("c\"one\\00\""), "{ir}");
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 7.5)"), "{ir}");
    assert!(!ir.contains("select i1"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$maybe = $flag ? 0.0 : 1.25;
echo $maybe ? 1 : 2;
"#,
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_ir_folds_known_string_full_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo "literal" ? "one" : "bad", "\n";
echo "" ? "bad" : "empty", "\n";
echo "0" ? "bad" : "zero", "\n";
echo $truthy ? 7 : 9, "\n";
echo $falsey ? "bad" : "falsey";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp3 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp4 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp5 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("c\"one\\00\""), "{ir}");
    assert!(ir.contains("c\"empty\\00\""), "{ir}");
    assert!(ir.contains("c\"zero\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
    assert!(ir.contains("c\"falsey\\00\""), "{ir}");
    assert_eq!(ir.matches("select i1").count(), 4, "{ir}");

    let error = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$maybe = $flag ? "" : "value";
echo $maybe ? 1 : 2;
"#,
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_ir_folds_null_full_ternary_conditions() {
    let ir = emit_ir_source(
        r#"<?php
echo null ? "bad" : "fallback", "\n";
echo NULL ? 1 : 7, "\n";
echo "a", null ? 1 : null, "b";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"fallback\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
    assert!(!ir.contains("select i1"), "{ir}");

    let error = emit_ir_source("<?php\necho null ? 1 : [];\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("LLVM array lowering rejects arrays"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_lazily_lowers_static_full_ternary_selected_branches() {
    let ir = emit_ir_source(
        r#"<?php
echo true ? "truthy" : [], "\n";
echo false ? [] : "falsey", "\n";
echo 1 ? 7 : [], "\n";
echo 0 ? [] : 9, "\n";
echo null ? [] : "null", "\n";
echo "php" ? "string" : [];
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"truthy\\00\""), "{ir}");
    assert!(ir.contains("c\"falsey\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 9)"), "{ir}");
    assert!(ir.contains("c\"null\\00\""), "{ir}");
    assert!(ir.contains("c\"string\\00\""), "{ir}");
    assert!(!ir.contains("select i1"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
echo $flag ? 1 : [];
"#,
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("LLVM array lowering rejects arrays"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_lowers_static_false_short_ternary_scalar_fallbacks() {
    let ir = emit_ir_source(
        r#"<?php
echo false ?: 42, "\n";
echo false ?: 2.5, "\n";
echo false ?: "fallback", "\n";
echo "a", false ?: null, "b", "\n";
echo true ?: [];
"#,
    )
    .unwrap();

    assert!(ir.contains("@phpc_native_int(i64 42)"), "{ir}");
    assert!(ir.contains("@phpc_native_float(double 2.5)"), "{ir}");
    assert!(ir.contains("c\"fallback\\00\""), "{ir}");
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
    assert!(
        ir.contains("@phpc_native_bool(i1 true)"),
        "static true short ternary should print true without lowering the unsupported array fallback:\n{ir}"
    );
    assert!(!ir.contains("select i1"), "{ir}");
}

#[test]
fn emit_ir_folds_null_short_ternary_fallbacks() {
    let ir = emit_ir_source(
        r#"<?php
echo null ?: "fallback", "\n";
echo NULL ?: 7, "\n";
echo "a", null ?: null, "b";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"fallback\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
    assert!(!ir.contains("select i1"), "{ir}");
}

#[test]
fn emit_ir_folds_direct_null_variable_short_ternary_fallback() {
    let ir = emit_ir_source(
        r#"<?php
$value = null;

echo "a", $value ?: $value, "b";
"#,
    )
    .unwrap();

    assert!(
        !ir.contains("select i1"),
        "direct null-variable short ternary should not require truthiness lowering:\n{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeScalarValue @phpc_native_int("),
        "null short ternary should not emit numeric output:\n{ir}"
    );
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_boolean_condition_ternary_for_null_branches() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$nil = $is_three ? null : null;

echo "a", $nil, "b";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(!ir.contains("select i1 %tmp1"), "{ir}");
    assert!(
        !ir.contains("call %phpc.NativeScalarValue @phpc_native_int("),
        "{ir}"
    );
    assert!(
        !ir.contains("call %phpc.NativeScalarValue @phpc_native_float("),
        "{ir}"
    );
    assert!(ir.contains("c\"a\\00\""), "{ir}");
    assert!(ir.contains("c\"b\\00\""), "{ir}");
}

#[test]
fn emit_ir_folds_static_boolean_ternary_for_mixed_scalar_branches() {
    let ir = emit_ir_source(
        r#"<?php
echo true ? 1 : "no", "\n";
echo false ? 9.5 : "picked", "\n";
echo (1 === "1") ? false : 7;
"#,
    )
    .unwrap();

    assert!(
        !ir.contains(" select "),
        "static boolean ternary should fold without select:\n{ir}"
    );
    assert!(ir.contains("@phpc_native_int(i64 1)"), "{ir}");
    assert!(ir.contains("c\"picked\\00\""), "{ir}");
    assert!(ir.contains("@phpc_native_int(i64 7)"), "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_conditional_expressions_with_specific_boundary() {
    for source in [
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? 0 : 5;\necho $maybe ? 1 : 2;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\necho $flag ?: 2;\n",
        "<?php\n$value = null;\necho $value ?? 2;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_ternary_with_unsupported_branch_types() {
    let error =
        emit_ir_source("<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\necho $flag ? 1 : \"no\";\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_ir_rejects_null_coalescing_before_lowering_operands() {
    let error = emit_ir_source("<?php\necho [] ?? [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn emit_asm_rejects_conditional_expressions_before_backend_execution() {
    let error = emit_asm_source(
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$maybe = $flag ? 0 : 5;\necho $maybe ? 1 : 2;\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_CONDITIONAL_REJECTION);
}

#[test]
fn native_boolean_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone281/native_boolean_ternary_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone281/native_boolean_ternary_emit_ir.cli"),
    )
    .expect("native boolean ternary CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone482/native_boolean_short_ternary.php");
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
        workspace_root.join("tests/fixtures/milestone482/native_boolean_short_ternary_emit_ir.cli"),
    )
    .expect("native boolean short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_string_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone515/native_identical_string_short_ternary.php");
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
            .join("tests/fixtures/milestone515/native_identical_string_short_ternary_emit_ir.cli"),
    )
    .expect("native identical string short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_integer_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone516/native_identical_integer_short_ternary.php");
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
            .join("tests/fixtures/milestone516/native_identical_integer_short_ternary_emit_ir.cli"),
    )
    .expect("native identical integer short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_float_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone517/native_identical_float_short_ternary.php");
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
            .join("tests/fixtures/milestone517/native_identical_float_short_ternary_emit_ir.cli"),
    )
    .expect("native identical float short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone518/native_identical_boolean_short_ternary.php");
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
            .join("tests/fixtures/milestone518/native_identical_boolean_short_ternary_emit_ir.cli"),
    )
    .expect("native identical boolean short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone522/native_identical_boolean_full_ternary.php");
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
            .join("tests/fixtures/milestone522/native_identical_boolean_full_ternary_emit_ir.cli"),
    )
    .expect("native identical boolean full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_null_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone523/native_identical_null_full_ternary.php");
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
            .join("tests/fixtures/milestone523/native_identical_null_full_ternary_emit_ir.cli"),
    )
    .expect("native identical null full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_direct_null_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone526/native_direct_null_full_ternary.php");
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
            .join("tests/fixtures/milestone526/native_direct_null_full_ternary_emit_ir.cli"),
    )
    .expect("native direct null full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_integer_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone519/native_identical_integer_full_ternary.php");
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
            .join("tests/fixtures/milestone519/native_identical_integer_full_ternary_emit_ir.cli"),
    )
    .expect("native identical integer full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_string_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone521/native_identical_string_full_ternary.php");
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
            .join("tests/fixtures/milestone521/native_identical_string_full_ternary_emit_ir.cli"),
    )
    .expect("native identical string full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_float_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone520/native_identical_float_full_ternary.php");
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
            .join("tests/fixtures/milestone520/native_identical_float_full_ternary_emit_ir.cli"),
    )
    .expect("native identical float full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_false_short_ternary_scalar_fallback_emit_ir_cli_snapshot_matches_committed_output()
{
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone483/native_static_false_short_ternary_scalar_fallback.php");
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
        "tests/fixtures/milestone483/native_static_false_short_ternary_scalar_fallback_emit_ir.cli",
    ))
    .expect("native static false short ternary scalar fallback IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_known_integer_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone484/native_single_known_integer_short_ternary.php");
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
            "tests/fixtures/milestone484/native_single_known_integer_short_ternary_emit_ir.cli",
        ))
        .expect("native single-known integer short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_known_integer_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone492/native_single_known_integer_ternary.php");
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
            .join("tests/fixtures/milestone492/native_single_known_integer_ternary_emit_ir.cli"),
    )
    .expect("native single-known integer full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_known_float_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone493/native_single_known_float_ternary.php");
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
            .join("tests/fixtures/milestone493/native_single_known_float_ternary_emit_ir.cli"),
    )
    .expect("native single-known float full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_string_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone494/native_known_string_ternary.php");
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
        workspace_root.join("tests/fixtures/milestone494/native_known_string_ternary_emit_ir.cli"),
    )
    .expect("native known string full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_full_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone495/native_null_ternary.php");
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
        workspace_root.join("tests/fixtures/milestone495/native_null_ternary_emit_ir.cli"),
    )
    .expect("native null full ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_full_ternary_selected_branch_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone497/native_static_ternary_selected_branch.php");
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
            .join("tests/fixtures/milestone497/native_static_ternary_selected_branch_emit_ir.cli"),
    )
    .expect("native static full ternary selected branch IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_known_float_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone485/native_single_known_float_short_ternary.php");
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
            "tests/fixtures/milestone485/native_single_known_float_short_ternary_emit_ir.cli",
        ))
        .expect("native single-known float short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_ternary_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone341/native_integer_ternary_result_tracking.php");
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
            .join("tests/fixtures/milestone341/native_integer_ternary_result_tracking_emit_ir.cli"),
    )
    .expect("native integer ternary result-tracking CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_bounded_integer_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone347/native_bounded_integer_result_tracking.php");
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
            .join("tests/fixtures/milestone347/native_bounded_integer_result_tracking_emit_ir.cli"),
    )
    .expect("native bounded integer result-tracking CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone284/native_float_ternary_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone284/native_float_ternary_emit_ir.cli"),
    )
    .expect("native float ternary CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_string_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone287/native_string_ternary_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone287/native_string_ternary_emit_ir.cli"),
    )
    .expect("native string ternary CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_mixed_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone290/native_static_mixed_ternary_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone290/native_static_mixed_ternary_emit_ir.cli"),
    )
    .expect("native static mixed ternary CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_string_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone395/native_identical_string_ternary.php");
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
            .join("tests/fixtures/milestone395/native_identical_string_ternary_emit_ir.cli"),
    )
    .expect("native identical string ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_expr_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone398/native_identical_boolean_expr_ternary.php");
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
            .join("tests/fixtures/milestone398/native_identical_boolean_expr_ternary_emit_ir.cli"),
    )
    .expect("native identical boolean expression ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_literal_ternary_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone447/native_boolean_literal_ternary_folding.php");
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
            .join("tests/fixtures/milestone447/native_boolean_literal_ternary_folding_emit_ir.cli"),
    )
    .expect("native boolean literal ternary folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_numeric_literal_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone450/native_identical_numeric_literal_ternary.php");
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
            "tests/fixtures/milestone450/native_identical_numeric_literal_ternary_emit_ir.cli",
        ))
        .expect("native identical numeric literal ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_integer_expr_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone401/native_identical_integer_expr_ternary.php");
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
            .join("tests/fixtures/milestone401/native_identical_integer_expr_ternary_emit_ir.cli"),
    )
    .expect("native identical integer expression ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_identical_integer_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone508/native_untracked_identical_integer_ternary.php");
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
        "tests/fixtures/milestone508/native_untracked_identical_integer_ternary_emit_ir.cli",
    ))
    .expect("native untracked identical integer ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_untracked_identical_float_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone509/native_untracked_identical_float_ternary.php");
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
            "tests/fixtures/milestone509/native_untracked_identical_float_ternary_emit_ir.cli",
        ))
        .expect("native untracked identical float ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_float_expr_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone404/native_identical_float_expr_ternary.php");
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
            .join("tests/fixtures/milestone404/native_identical_float_expr_ternary_emit_ir.cli"),
    )
    .expect("native identical float expression ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_single_result_scalar_ternary_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone472/native_single_result_scalar_ternary.php");
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
            .join("tests/fixtures/milestone472/native_single_result_scalar_ternary_emit_ir.cli"),
    )
    .expect("native single-result scalar ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_string_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone486/native_known_string_short_ternary.php");
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
            .join("tests/fixtures/milestone486/native_known_string_short_ternary_emit_ir.cli"),
    )
    .expect("native known string short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone491/native_null_short_ternary.php");
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
        workspace_root.join("tests/fixtures/milestone491/native_null_short_ternary_emit_ir.cli"),
    )
    .expect("native null short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_direct_null_short_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone524/native_direct_null_short_ternary.php");
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
            .join("tests/fixtures/milestone524/native_direct_null_short_ternary_emit_ir.cli"),
    )
    .expect("native direct null short ternary IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_ternary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone311/native_null_ternary_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone311/native_null_ternary_emit_ir.cli"),
    )
    .expect("native null ternary CLI snapshot is readable");
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
