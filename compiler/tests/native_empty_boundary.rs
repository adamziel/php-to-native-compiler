use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_EMPTY_REJECTION: &str = "LLVM empty lowering rejects array offset operands, object property operands, complex operands, arrays, unset/mutation interactions, and ambiguous truthiness until native symbol-table storage, PHP truthiness, references/copy-on-write, and exact native error behavior exist; phpc run handles current empty behavior";

#[test]
fn phpc_run_still_handles_current_direct_variable_empty_subset() {
    let execution = run_source(
        r#"<?php
$null = null;
$false = false;
$zero = 0;
$one = 1;
$empty_string = "";
$zero_string = "0";
$text = "value";
echo empty($missing) ? "1" : "0";
echo empty($null) ? "1" : "0";
echo empty($false) ? "1" : "0";
echo empty($zero) ? "1" : "0";
echo empty($one) ? "1" : "0";
echo empty($empty_string) ? "1" : "0";
echo empty($zero_string) ? "1" : "0";
echo empty($text) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11110110");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_direct_variable_empty_for_current_straight_line_subset() {
    let ir = emit_ir_source(
        r#"<?php
$null = null;
$false = false;
$zero = 0;
$one = 1;
$empty_string = "";
$zero_string = "0";
$text = "value";
echo empty($missing) ? "1" : "0";
echo empty($null) ? "1" : "0";
echo empty($false) ? "1" : "0";
echo empty($zero) ? "1" : "0";
echo empty($one) ? "1" : "0";
echo empty($empty_string) ? "1" : "0";
echo empty($zero_string) ? "1" : "0";
echo empty($text) ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"0\\00\""), "{ir}");
    assert!(!ir.contains("LLVM empty lowering rejects"), "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_empty_forms_before_lowering_operands() {
    for source in [
        "<?php\n$items = 1;\necho empty($items[0]) ? 1 : 0;\n",
        "<?php\n$box = 1;\necho empty($box->name) ? 1 : 0;\n",
        "<?php\n$left = 1;\n$right = 2;\necho empty($left, $right) ? 1 : 0;\n",
        "<?php\necho empty(missing_call()) ? 1 : 0;\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_EMPTY_REJECTION);
    }
}

#[test]
fn native_direct_variable_empty_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone556/native_direct_variable_empty.php");
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
        workspace_root.join("tests/fixtures/milestone556/native_direct_variable_empty_emit_ir.cli"),
    )
    .expect("native direct-variable empty CLI snapshot is readable");
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
