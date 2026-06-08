use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn array_reduce_reference_params_cli_snapshot_matches_committed_output() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture =
        workspace_root.join("tests/fixtures/milestone2323/array_reduce_reference_params.php");
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .args([
            "run",
            "tests/fixtures/milestone2323/array_reduce_reference_params.php",
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to run phpc for {:?}: {error}", fixture));

    let expected = fs::read_to_string(fixture.with_extension("cli"))
        .expect("array_reduce reference params CLI snapshot is readable");
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
