use std::path::Path;
use std::process::{Command, Output};

use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source, run_source};

const ASSEMBLY_VARIABLE_READ_REJECTION: &str = "assembly variable-read lowering rejects reads that are not statically assigned earlier in the same straight-line native subset until native symbol-table storage, undefined-variable diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current variable-read behavior";

#[test]
fn phpc_run_reports_current_undefined_variable_runtime_error() {
    let error = run_source("<?php\necho $missing;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn emit_ir_lowers_undefined_variable_reads_through_native_symbol_table() {
    for source in [
        "<?php\necho $missing;\n",
        "<?php\n$value = 'defined';\necho $value, $missing;\n",
        "<?php\n$value = $missing;\necho $value;\n",
        "<?php\nprint $missing;\n",
    ] {
        let ir = emit_ir_source(source).unwrap();

        assert!(ir.contains("phpc_native_symbol_table_read"), "{ir}");
        assert!(ir.contains("phpc_native_value_echo_stdout"), "{ir}");
        assert!(ir.contains("c\"missing\\00\""), "{ir}");
    }
}

#[test]
fn emit_ir_still_allows_reads_after_straight_line_static_assignment() {
    let ir = emit_ir_source("<?php\n$value = 'defined';\necho $value;\n").unwrap();

    assert!(ir.contains("defined\\00"), "{ir}");
}

#[test]
fn emit_asm_rejects_undefined_variable_reads_before_backend_execution() {
    let error = emit_asm_source("<?php\necho $missing;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, ASSEMBLY_VARIABLE_READ_REJECTION);
}

#[test]
fn native_variable_read_emit_ir_cli_uses_symbol_table_helper() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone181/native_variable_read_boundary.php");
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

    let actual = render_cli_snapshot(&output);

    assert!(output.status.success(), "{actual}");
    assert!(actual.contains("phpc_native_symbol_table_read"), "{actual}");
    assert!(actual.contains("phpc_native_value_echo_stdout"), "{actual}");
    assert!(actual.contains("c\"missing\\00\""), "{actual}");
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
