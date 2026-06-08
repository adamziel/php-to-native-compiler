use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_LOGICAL_REJECTION: &str = "LLVM logical lowering rejects unsupported logical operands until native PHP truthiness and short-circuit semantics exist; phpc run handles current logical operator behavior";

#[test]
fn phpc_run_still_handles_current_logical_operators() {
    let execution = run_source(
        r#"<?php
echo true && false, "\n";
echo false || true, "\n";
echo (true and false) ? "1" : "0", "\n";
echo (false or true) ? "1" : "0", "\n";
echo (true xor false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "\n1\n0\n1\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_lowers_boolean_logical_operators() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 3 : 4;
$left = $sum === $choice;
$right = $choice === 4;
$not_right = !$right;

echo $left && $right, "\n";
echo $left || $right, "\n";
echo $left xor $right, "\n";
echo $left && $not_right, "\n";
echo (true and false), "\n";
echo (false or true), "z";
"#,
    )
    .unwrap();

    assert!(ir.contains(" and i1 "), "{ir}");
    assert!(ir.contains(" or i1 "), "{ir}");
    assert!(ir.contains(" xor i1 "), "{ir}");
    assert!(ir.contains("select i1"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_float"), "{ir}");
}

#[test]
fn emit_ir_folds_static_boolean_logical_edges_for_later_scalar_lowering() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$always = $flag || true;
$never = false && $flag;
$same = $flag && true;
$also = ($flag xor false);
$invert = ($flag xor true);

echo $always ? 10 : 20, "\n";
echo $never ? 10 : 20, "\n";
echo $same ? 1 : 0, "\n";
echo $also ? 1 : 0, "\n";
echo $invert ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(!ir.contains(" or i1 "), "{ir}");
    assert!(!ir.contains(" and i1 "), "{ir}");
    assert!(
        !ir.contains("xor i1"),
        "known xor-with-true result should fold for later scalar lowering:\n{ir}"
    );
    assert!(native_int_output_count(&ir, 10) >= 1, "{ir}");
    assert!(native_int_output_count(&ir, 20) >= 1, "{ir}");
    assert_eq!(
        ir.matches("select i1 %tmp1, i64 1, i64 0").count(),
        2,
        "{ir}"
    );
    assert!(native_int_output_count(&ir, 0) >= 1, "{ir}");
}

#[test]
fn emit_ir_folds_known_boolean_expression_logical_results() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$left = $sum === 3;
$right = $sum !== 4;
$falsey = $sum === 4;
$choice = $left ? 3 : 4;
$amb_left = $sum === $choice;
$amb_right = $choice === 4;

echo $left && $right, "\n";
echo $left || $falsey, "\n";
echo $left xor $right, "\n";
echo $falsey && $right, "\n";
echo $amb_left && $amb_right;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp ne i64 %tmp0, 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("and i1 %tmp1, %tmp2"),
        "known true && true should fold without a boolean op:\n{ir}"
    );
    assert!(
        !ir.contains("or i1 %tmp1, %tmp3"),
        "known true || false should fold without a boolean op:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1 %tmp1, %tmp2"),
        "known true xor true should fold without a boolean op:\n{ir}"
    );
    assert!(
        !ir.contains("and i1 %tmp3, %tmp2"),
        "known false && true should fold without a boolean op:\n{ir}"
    );
    assert!(
        ir.contains(" and i1 "),
        "ambiguous boolean logical expression should stay emitted:\n{ir}"
    );
    assert!(
        ir.contains("select i1"),
        "ambiguous logical result should still feed echo conversion:\n{ir}"
    );
}

#[test]
fn emit_ir_folds_known_scalar_logical_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
$int = 1 + 2;
$float = 1.25 + 2.5;

echo (1 && $int) ? 1 : 0, "\n";
echo (0 || $float) ? 1 : 0, "\n";
echo (0.0 || "0") ? 1 : 0, "\n";
echo ("php" xor 0) ? 1 : 0, "\n";
echo ($int && $float) ? 1 : 0, "\n";
echo ("" xor 0.0) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = fadd double 1.25, 2.5"), "{ir}");
    assert!(!ir.contains(" and i1 "), "{ir}");
    assert!(!ir.contains(" or i1 "), "{ir}");
    assert!(!ir.contains(" xor i1 "), "{ir}");
    assert_eq!(native_int_output_count(&ir, 1), 4, "{ir}");
    assert_eq!(native_int_output_count(&ir, 0), 2, "{ir}");
}

#[test]
fn emit_ir_folds_null_logical_truthiness() {
    let ir = emit_ir_source(
        r#"<?php
echo (null && true) ? 1 : 0, "\n";
echo (null || true) ? 1 : 0, "\n";
echo (null xor true) ? 1 : 0, "\n";
echo (false || null) ? 1 : 0, "\n";
echo (true && null) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(!ir.contains(" and i1 "), "{ir}");
    assert!(!ir.contains(" or i1 "), "{ir}");
    assert!(!ir.contains(" xor i1 "), "{ir}");
    assert_eq!(native_int_output_count(&ir, 1), 2, "{ir}");
    assert_eq!(native_int_output_count(&ir, 0), 3, "{ir}");

    let error = emit_ir_source("<?php\necho null || [];\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("LLVM array lowering rejects arrays"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_short_circuits_static_logical_selected_operands() {
    let ir = emit_ir_source(
        r#"<?php
echo (false && []) ? 1 : 0, "\n";
echo (true || []) ? 1 : 0, "\n";
echo (0 && []) ? 1 : 0, "\n";
echo ("php" || []) ? 1 : 0, "\n";
echo (null && []) ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(!ir.contains(" and i1 "), "{ir}");
    assert!(!ir.contains(" or i1 "), "{ir}");
    assert!(!ir.contains(" xor i1 "), "{ir}");
    assert!(
        !ir.contains("@phpc_native_array_new") && !ir.contains("@phpc_native_array_append"),
        "unselected array operands should not be lowered:\n{ir}"
    );
    assert_eq!(native_int_output_count(&ir, 1), 2, "{ir}");
    assert_eq!(native_int_output_count(&ir, 0), 3, "{ir}");

    for source in ["<?php\necho true && [];\n", "<?php\necho false || [];\n"] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("LLVM array lowering rejects arrays"),
            "{}",
            error.message
        );
    }

    let error = emit_ir_source("<?php\necho false xor [];\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("LLVM array lowering rejects arrays"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_folds_identical_boolean_expression_logical_and_or_without_ops() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$both = $is_three && $is_three;
$either = $is_three || $is_three;

echo $both ? 1 : 0, "\n", $either ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("and i1 %tmp1, %tmp1"),
        "identical boolean expression && should reuse the expression:\n{ir}"
    );
    assert!(
        !ir.contains("or i1 %tmp1, %tmp1"),
        "identical boolean expression || should reuse the expression:\n{ir}"
    );
    assert!(ir.contains("select i1 %tmp1, i64 1, i64 0"), "{ir}");
    assert!(uses_native_diagnostic_result_output(&ir), "{ir}");
}

#[test]
fn emit_ir_folds_identical_boolean_expression_logical_xor_to_false() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$different = ($is_three xor $is_three);

echo $different ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("xor i1 %tmp1, %tmp1"),
        "identical boolean expression xor should fold to false:\n{ir}"
    );
    assert!(!ir.contains("select i1 %tmp1"), "{ir}");
    assert!(native_int_output_count(&ir, 0) >= 1, "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_logical_operands() {
    for source in [
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho $value && true;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? \"\" : \"php\";\necho $value || false;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_LOGICAL_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_unsupported_logical_operands_before_backend_execution() {
    let error = emit_asm_source(
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho $value && true;\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_LOGICAL_REJECTION);
}

#[test]
fn native_boolean_logical_operator_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone272/native_boolean_logical_operator_emit_ir.php");
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
            .join("tests/fixtures/milestone272/native_boolean_logical_operator_emit_ir.cli"),
    )
    .expect("native boolean logical operator CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_logical_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone356/native_boolean_logical_folding.php");
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
            .join("tests/fixtures/milestone356/native_boolean_logical_folding_emit_ir.cli"),
    )
    .expect("native boolean logical folding CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_logical_known_result_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone476/native_boolean_logical_known_result_folding.php");
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
        "tests/fixtures/milestone476/native_boolean_logical_known_result_folding_emit_ir.cli",
    ))
    .expect("native boolean logical known-result folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_scalar_logical_truthiness_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone489/native_known_scalar_logical_truthiness.php");
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
            .join("tests/fixtures/milestone489/native_known_scalar_logical_truthiness_emit_ir.cli"),
    )
    .expect("native known scalar logical truthiness IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_logical_truthiness_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone496/native_null_logical_truthiness.php");
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
            .join("tests/fixtures/milestone496/native_null_logical_truthiness_emit_ir.cli"),
    )
    .expect("native null logical truthiness IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_static_logical_short_circuit_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone498/native_static_logical_short_circuit.php");
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
            .join("tests/fixtures/milestone498/native_static_logical_short_circuit_emit_ir.cli"),
    )
    .expect("native static logical short-circuit IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_expr_logical_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone407/native_identical_boolean_expr_logical.php");
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
            .join("tests/fixtures/milestone407/native_identical_boolean_expr_logical_emit_ir.cli"),
    )
    .expect("native identical boolean expression logical IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_identical_boolean_expr_xor_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone410/native_identical_boolean_expr_xor.php");
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
            .join("tests/fixtures/milestone410/native_identical_boolean_expr_xor_emit_ir.cli"),
    )
    .expect("native identical boolean expression xor IR CLI snapshot is readable");
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

fn native_int_output_count(ir: &str, value: i64) -> usize {
    let direct_printf = format!("@printf(ptr @.fmt_int, i64 {value})");
    let boxed_native_int = format!("@phpc_native_int(i64 {value})");

    ir.matches(&direct_printf).count() + ir.matches(&boxed_native_int).count()
}

fn uses_native_diagnostic_result_output(ir: &str) -> bool {
    ir.contains("@phpc_native_diagnostic_result_from_value")
        && ir.contains("@phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
}
