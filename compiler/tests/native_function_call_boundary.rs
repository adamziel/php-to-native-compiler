use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";

#[test]
fn phpc_run_still_handles_current_function_call_subset() {
    let execution = run_source(
        r#"<?php
echo strlen("abc"), "\n";
function label($value) {
    return $value . "!";
}
echo label("user"), "\n";
$call = "label";
echo $call("dynamic"), "\n";
$builtin = "strlen";
echo $builtin("callable");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\nuser!\ndynamic!\n8");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_direct_builtin_and_user_calls_with_specific_boundary() {
    for source in [
        "<?php\necho label(\"user\");\nfunction label($value) { return $value; }\n",
        "<?php\necho dirname(\"/a/b.php\");\n",
        "<?php\nassert(true);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_folds_direct_strlen_known_strings() {
    let ir = emit_ir_source(
        r#"<?php
$known = "native";
echo strlen("abc"), "\n";
echo strlen($known), "\n";
echo strlen(true ? "same" : "size"), "\n";
"#,
    )
    .unwrap();

    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 3)"));
    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 6)"));
    assert!(ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 4)"));
    assert!(!ir.contains("strlen"));
}

#[test]
fn emit_ir_rejects_direct_strlen_unsupported_operands() {
    for source in [
        "<?php\necho strlen(123);\n",
        "<?php\necho strlen(\"abc\", \"extra\");\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_direct_calls_before_lowering_arguments() {
    for source in [
        "<?php\necho label([]);\nfunction label($value) { return $value; }\n",
        "<?php\nassert([]);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_rejects_dynamic_calls_before_lowering_callee_or_arguments() {
    for source in [
        "<?php\n$call = \"strlen\";\necho $call([]);\n",
        "<?php\n$call = \"assert\";\necho $call([]);\n",
        "<?php\n$call = \"strlen\";\necho $call(\"abc\");\n",
        "<?php\n$call = \"strlen\";\necho $call(\"abc\",);\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_ir_routes_text_membership_consumers_through_native_value_boundary() {
    let source = r#"<?php
$function = true ? "strlen" : "trim";
$ctype = true ? "ctype_alpha" : "ctype_xdigit";
$callable = true ? "substr" : "dirname";
$extension = true ? "json" : "PDO";
echo function_exists($function) ? "1" : "0";
echo function_exists($ctype) ? "1" : "0";
echo is_callable($callable) ? "1" : "0";
echo extension_loaded($extension) ? "1" : "0";
"#;
    let ir = emit_ir_source(source).unwrap();

    assert!(ir.contains("@phpc_text_membership_candidates_"));
    assert!(
        ir.matches("call i1 @phpc_native_value_text_membership_with_diagnostic")
            .count()
            >= 4
    );
    assert!(ir.contains("ctype_alpha"));
    assert!(ir.contains("ctype_xdigit"));
    assert!(ir.contains("i8 4"));
    assert!(ir.contains("i8 6"));
    assert!(ir.contains("@phpc_native_symbol_table_write"));
    assert!(ir.contains("@phpc_native_symbol_table_read"));
    assert!(ir.contains("@phpc_native_diagnostic_message_stderr"));
    assert!(ir.contains("@phpc_native_diagnostic_free"));

    emit_asm_source(source).unwrap();
}

#[test]
fn emit_asm_rejects_function_calls_before_backend_execution() {
    for source in [
        "<?php\necho label(\"abc\");\nfunction label($value) { return $value; }\n",
        "<?php\necho dirname(\"/a/b.php\");\n",
        "<?php\nassert(true);\n",
    ] {
        let error = emit_asm_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
    }
}

#[test]
fn emit_asm_rejects_dynamic_calls_before_backend_execution() {
    let error = emit_asm_source("<?php\n$call = \"strlen\";\necho $call(\"abc\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
}

#[test]
fn native_function_call_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone169/native_function_call_boundary.php");
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
            .join("tests/fixtures/milestone169/native_function_call_boundary_emit_ir.cli"),
    )
    .expect("native function-call CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_function_call_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary.phpc-source");
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
            .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary_emit_ir.cli"),
    )
    .expect("native dynamic function-call IR CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_dynamic_function_call_emit_asm_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root
        .join("tests/fixtures/milestone1128/native_dynamic_function_call_boundary.phpc-source");
    let relative_fixture = fixture
        .strip_prefix(workspace_root)
        .expect("fixture lives under workspace root")
        .to_str()
        .expect("fixture path is valid UTF-8")
        .to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args(["compile", &relative_fixture, "--emit-asm"])
        .output()
        .unwrap_or_else(|error| panic!("failed to compile {relative_fixture}: {error}"));

    let expected =
        fs::read_to_string(workspace_root.join(
            "tests/fixtures/milestone1128/native_dynamic_function_call_boundary_emit_asm.cli",
        ))
        .expect("native dynamic function-call assembly CLI snapshot is readable");
    let actual = render_cli_snapshot(&output);

    assert_eq!(actual, expected);
}

#[test]
fn native_strlen_emit_ir_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture = workspace_root.join("tests/fixtures/milestone562/native_strlen.php");
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
        workspace_root.join("tests/fixtures/milestone562/native_strlen_emit_ir.cli"),
    )
    .expect("native strlen IR CLI snapshot is readable");
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
