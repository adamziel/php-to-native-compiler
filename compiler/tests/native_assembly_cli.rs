use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn native_scalar_echo_emit_asm_cli_summary_matches_committed_output() {
    if !has_assembly_backend() {
        return;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone182/native_scalar_echo_assembly.php");
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

    let expected = fs::read_to_string(
        workspace_root.join("tests/fixtures/milestone182/native_scalar_echo_assembly_emit_asm.cli"),
    )
    .expect("native scalar echo assembly CLI snapshot is readable");
    let actual = render_asm_cli_summary(&output);

    assert_eq!(actual, expected);
}

fn has_assembly_backend() -> bool {
    ["clang", "llc", "cc"]
        .iter()
        .any(|command| Command::new(command).arg("--version").output().is_ok())
}

fn render_asm_cli_summary(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\nnonempty: {}\ncontains_main: {}\ncontains_printf: {}\n--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n",
        !stdout.is_empty(),
        stdout.contains("main"),
        stdout.contains("printf"),
    )
}
