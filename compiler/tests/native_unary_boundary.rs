use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unsupported unary operators, cast expressions, or operands until native PHP numeric coercion, truthiness conversion, scalar casts, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary and cast behavior";

#[test]
fn phpc_run_still_handles_current_unary_subset() {
    let execution = run_source(
        r#"<?php
echo -5, "\n";
echo -2.5, "\n";
echo -true, "\n";
echo !false, "\n";
echo !true, "empty\n";
echo !"0", "\n";
echo !"php", "empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "-5\n-2.5\n-1\n1\nempty\n1\nempty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_unary_minus_and_logical_not_with_specific_boundary() {
    for source in [
        "<?php\necho -true;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho !$value;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_UNARY_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_unary_forms_before_lowering_operands() {
    for source in [
        "<?php\necho -\"5\";\n",
        "<?php\necho (string) 5;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho !$value;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0.0 : 2.5;\necho !$value;\n",
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? \"\" : \"php\";\necho !$value;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_UNARY_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_unary_forms_before_backend_execution() {
    let error = emit_asm_source(
        "<?php\n$sum = 1 + 2;\n$flag = $sum === 3;\n$value = $flag ? 0 : 5;\necho !$value;\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);
}

#[test]
fn emit_ir_lowers_static_integer_unary_minus() {
    let ir = emit_ir_source(
        r#"<?php
$a = -5;
$b = 10 + 2;
$c = -$b;
echo $a, "\n", $c;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 10, 2"), "{ir}");
    assert!(!ir.contains("sub i64 0, 5"), "{ir}");
    assert!(!ir.contains("sub i64 0, %tmp0"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 -5)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 -12)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_single_known_integer_unary_minus() {
    let ir = emit_ir_source(
        r#"<?php
$literal = -5;
$expr = -(10 + 2);

echo $literal, "\n";
echo $expr;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 10, 2"), "{ir}");
    assert!(
        !ir.contains("sub i64 0, 5"),
        "integer literal unary minus should fold to the known result:\n{ir}"
    );
    assert!(
        !ir.contains("sub i64 0, %tmp0"),
        "single known integer expression unary minus should fold to the known result:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -5)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 -12)"), "{ir}");
}

#[test]
fn emit_ir_tracks_static_integer_unary_minus_for_later_arithmetic() {
    let ir = emit_ir_source(
        r#"<?php
$value = 10 + 2;
$negated = -$value;
echo $negated + 15;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 10, 2"), "{ir}");
    assert!(!ir.contains("sub i64 0, %tmp0"), "{ir}");
    assert!(ir.contains("%tmp1 = add i64 -12, 15"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp1)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_lowers_static_float_unary_minus() {
    let ir = emit_ir_source(
        r#"<?php
$a = -2.5;
$b = 1.5 + 2.25;
$c = -$b;
echo $a, "\n", $c;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(!ir.contains("fsub double 0.0, 2.5"), "{ir}");
    assert!(!ir.contains("fsub double 0.0, %tmp0"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -2.5)"),
        "{ir}"
    );
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_float, double -3.75)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_single_known_nonzero_float_unary_minus() {
    let ir = emit_ir_source(
        r#"<?php
$literal = -2.5;
$expr = -(1.5 + 2.25);

echo $literal, "\n";
echo $expr;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = fadd double 1.5, 2.25"), "{ir}");
    assert!(
        !ir.contains("fsub double 0.0, 2.5"),
        "finite nonzero float literal unary minus should fold to the known result:\n{ir}"
    );
    assert!(
        !ir.contains("fsub double 0.0, %tmp0"),
        "single known nonzero float expression unary minus should fold to the known result:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_float, double -2.5)"), "{ir}");
    assert!(
        ir.contains("@printf(ptr @.fmt_float, double -3.75)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_known_string_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$truthy = $flag ? "left" : "right";
$falsey = $flag ? "" : "0";

echo !"" ? 1 : 0, "\n";
echo !"0" ? 1 : 0, "\n";
echo !"literal" ? 1 : 0, "\n";
echo !$truthy ? 1 : 0, "\n";
echo !$falsey ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1"), "{ir}");
    assert!(ir.contains("%tmp3 = select i1 %tmp1"), "{ir}");
    assert!(
        !ir.contains("xor i1"),
        "known string logical-not should fold to static booleans:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 1)").count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 0)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_known_numeric_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$int = 1 + 2;
$float = 1.25 + 2.5;

echo !0 ? 1 : 0, "\n";
echo !1 ? 1 : 0, "\n";
echo !$int ? 1 : 0, "\n";
echo !0.0 ? 1 : 0, "\n";
echo !2.5 ? 1 : 0, "\n";
echo !$float ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = fadd double 1.25, 2.5"), "{ir}");
    assert!(
        !ir.contains("xor i1"),
        "known numeric logical-not should fold to static booleans:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 1)").count(),
        2,
        "{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 0)").count(),
        4,
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_null_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
echo !null ? 1 : 0, "\n";
echo !NULL ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(
        !ir.contains("xor i1"),
        "null logical-not should fold to true without a boolean op:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 1)").count(),
        2,
        "{ir}"
    );
}

#[test]
fn emit_ir_lowers_static_boolean_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$truth = false;
$falsey = true;
echo !$truth, "\n", !$falsey, "done";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"done\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_lowers_dynamic_boolean_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;
echo !$is_three, "\n", !$is_four, "x";
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("xor i1 %tmp1, true"),
        "single-known true boolean logical-not should fold to false:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1 %tmp2, true"),
        "single-known false boolean logical-not should fold to true:\n{ir}"
    );
    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"x\\00\""), "{ir}");
    assert!(!ir.contains("@printf(ptr @.fmt_int"), "{ir}");
}

#[test]
fn emit_ir_folds_single_known_boolean_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$is_four = $sum === 4;

echo !$is_three ? 1 : 0, "\n";
echo !$is_four ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = icmp eq i64 %tmp0, 4"), "{ir}");
    assert!(
        !ir.contains("xor i1 %tmp1, true"),
        "single-known true boolean logical-not should fold to false:\n{ir}"
    );
    assert!(
        !ir.contains("xor i1 %tmp2, true"),
        "single-known false boolean logical-not should fold to true:\n{ir}"
    );
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 0)"), "{ir}");
    assert!(ir.contains("@printf(ptr @.fmt_int, i64 1)"), "{ir}");
}

#[test]
fn emit_ir_folds_dynamic_boolean_double_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$flag = $sum === 3;
$same = !!$flag;

echo $same ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(
        !ir.contains("xor i1 %tmp1, true"),
        "double logical-not should reuse the original boolean expression:\n{ir}"
    );
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 1, i64 0"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %tmp2)"),
        "{ir}"
    );
}

#[test]
fn emit_ir_folds_known_scalar_double_logical_not() {
    let ir = emit_ir_source(
        r#"<?php
$int = 1 + 2;
$float = 1.25 + 2.5;

echo !!0 ? 1 : 0, "\n";
echo !!$int ? 1 : 0, "\n";
echo !!0.0 ? 1 : 0, "\n";
echo !!$float ? 1 : 0, "\n";
echo !!"" ? 1 : 0, "\n";
echo !!"0" ? 1 : 0, "\n";
echo !!"php" ? 1 : 0, "\n";
echo !!null ? 1 : 0;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = fadd double 1.25, 2.5"), "{ir}");
    assert!(
        !ir.contains("xor i1"),
        "known scalar double logical-not should fold without boolean ops:\n{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 1)").count(),
        3,
        "{ir}"
    );
    assert_eq!(
        ir.matches("@printf(ptr @.fmt_int, i64 0)").count(),
        5,
        "{ir}"
    );
}

#[test]
fn native_unary_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone177/native_unary_boundary.php");
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
        workspace_root.join("tests/fixtures/milestone177/native_unary_boundary_emit_ir.cli"),
    )
    .expect("native unary CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_unary_minus_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone236/native_integer_unary_minus_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone236/native_integer_unary_minus_emit_ir.cli"),
    )
    .expect("native integer unary-minus IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_unary_minus_result_tracking_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone332/native_integer_unary_minus_result_tracking.php");
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
        "tests/fixtures/milestone332/native_integer_unary_minus_result_tracking_emit_ir.cli",
    ))
    .expect("native integer unary-minus result-tracking IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_integer_unary_minus_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone457/native_integer_unary_minus_folding.php");
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
            .join("tests/fixtures/milestone457/native_integer_unary_minus_folding_emit_ir.cli"),
    )
    .expect("native integer unary-minus folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_unary_minus_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone296/native_float_unary_minus_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone296/native_float_unary_minus_emit_ir.cli"),
    )
    .expect("native float unary-minus IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_float_unary_minus_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone458/native_float_unary_minus_folding.php");
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
            .join("tests/fixtures/milestone458/native_float_unary_minus_folding_emit_ir.cli"),
    )
    .expect("native float unary-minus folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone239/native_boolean_logical_not_emit_ir.php");
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
        workspace_root.join("tests/fixtures/milestone239/native_boolean_logical_not_emit_ir.cli"),
    )
    .expect("native boolean logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_boolean_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone269/native_dynamic_boolean_logical_not_emit_ir.php");
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
            .join("tests/fixtures/milestone269/native_dynamic_boolean_logical_not_emit_ir.cli"),
    )
    .expect("native dynamic boolean logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_logical_not_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone459/native_boolean_logical_not_folding.php");
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
            .join("tests/fixtures/milestone459/native_boolean_logical_not_folding_emit_ir.cli"),
    )
    .expect("native boolean logical-not folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_logical_not_c_fallback_folding_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone460/native_boolean_logical_not_c_fallback_folding.php");
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
        "tests/fixtures/milestone460/native_boolean_logical_not_c_fallback_folding_emit_ir.cli",
    ))
    .expect("native boolean logical-not C fallback folding IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_string_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone487/native_known_string_logical_not.php");
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
            .join("tests/fixtures/milestone487/native_known_string_logical_not_emit_ir.cli"),
    )
    .expect("native known string logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_numeric_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone488/native_known_numeric_logical_not.php");
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
            .join("tests/fixtures/milestone488/native_known_numeric_logical_not_emit_ir.cli"),
    )
    .expect("native known numeric logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_null_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone490/native_null_logical_not.php");
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
        workspace_root.join("tests/fixtures/milestone490/native_null_logical_not_emit_ir.cli"),
    )
    .expect("native null logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_boolean_double_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone440/native_boolean_double_logical_not.php");
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
            .join("tests/fixtures/milestone440/native_boolean_double_logical_not_emit_ir.cli"),
    )
    .expect("native boolean double logical-not IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_known_scalar_double_logical_not_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone511/native_known_scalar_double_logical_not.php");
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
            .join("tests/fixtures/milestone511/native_known_scalar_double_logical_not_emit_ir.cli"),
    )
    .expect("native known scalar double logical-not IR CLI snapshot is readable");
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
