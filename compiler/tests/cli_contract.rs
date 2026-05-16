use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn compile_invalid_emit_mode_is_reported_before_input_io() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "compile",
            "tests/fixtures/milestone1101/missing-input.php",
            "--emit-object",
        ])
        .output()
        .expect("run phpc compile with invalid emit mode");

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone1101/compile_invalid_emit_mode.cli"),
    )
    .expect("invalid emit mode CLI snapshot is readable");
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
