use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ISSET_REJECTION: &str = "LLVM isset lowering rejects array offset operands, object property operands, static property operands, complex operands, multiple operands, and unset/mutation interactions until native symbol-table storage, null-aware lookup, references/copy-on-write, and exact native error behavior exist; phpc run handles current isset behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn phpc_run_still_handles_current_direct_variable_isset_subset() {
    let execution = run_source(
        r#"<?php
$assigned = 1;
$nullable = null;
$falsey = false;
echo isset($assigned) ? "1" : "0";
echo isset($nullable) ? "1" : "0";
echo isset($missing) ? "1" : "0";
echo isset($falsey) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1001");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_direct_variable_isset_for_current_straight_line_subset() {
    let ir = emit_ir_source(
        r#"<?php
$assigned = 1;
$nullable = null;
$falsey = false;
echo isset($assigned) ? "1" : "0";
echo isset($nullable) ? "1" : "0";
echo isset($missing) ? "1" : "0";
echo isset($falsey) ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.contains("c\"1\\00\""), "{ir}");
    assert!(ir.contains("c\"0\\00\""), "{ir}");
    assert!(!ir.contains("LLVM isset lowering rejects"), "{ir}");
}

#[test]
fn emit_ir_rejects_unsupported_isset_forms_before_lowering_operands() {
    for (source, expected) in [
        (
            "<?php\n$items = 1;\necho isset($items[0]) ? 1 : 0;\n",
            LLVM_ISSET_REJECTION,
        ),
        (
            "<?php\n$box = 1;\necho isset($box->name) ? 1 : 0;\n",
            LLVM_ISSET_REJECTION,
        ),
        (
            "<?php\necho isset(Counter::$count) ? 1 : 0;\n",
            LLVM_ISSET_REJECTION,
        ),
        (
            "<?php\n$left = 1;\n$right = 2;\necho isset($left, $right) ? 1 : 0;\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
        (
            "<?php\necho isset(missing_call()) ? 1 : 0;\n",
            LLVM_FUNCTION_CALL_REJECTION,
        ),
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, expected);
    }
}

#[test]
fn native_direct_variable_isset_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone546/native_direct_variable_isset.php");
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
        workspace_root.join("tests/fixtures/milestone546/native_direct_variable_isset_emit_ir.cli"),
    )
    .expect("native direct-variable isset CLI snapshot is readable");
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
