use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn bitwise_not_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dir = workspace_root.join("tests/fixtures/milestone156");
    let mut fixtures = cli_snapshot_fixtures(&fixture_dir);

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected bitwise-not CLI snapshot fixtures"
    );

    for fixture in fixtures {
        let relative_fixture = fixture
            .strip_prefix(workspace_root)
            .expect("fixture lives under workspace root");
        let fixture_arg = relative_fixture
            .to_str()
            .expect("fixture path is valid UTF-8")
            .to_string();
        let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .current_dir(workspace_root)
            .args(["run", &fixture_arg])
            .output()
            .unwrap_or_else(|error| panic!("failed to run phpc for {fixture_arg}: {error}"));

        let expected = fs::read_to_string(fixture.with_extension("cli")).unwrap_or_else(|error| {
            panic!("failed to read CLI snapshot for {fixture_arg}: {error}")
        });
        let actual = render_cli_snapshot(&output);

        assert_eq!(actual, expected, "CLI snapshot mismatch for {fixture_arg}");
    }
}

fn cli_snapshot_fixtures(fixture_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .expect("bitwise-not fixture directory is readable")
        .map(|entry| entry.expect("bitwise-not fixture entry is readable").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("php"))
        .filter(|path| path.with_extension("cli").exists())
        .collect()
}

fn render_cli_snapshot(output: &Output) -> String {
    let exit_code = output.status.code().unwrap_or(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!(
        "exit: {exit_code}\nstdout:\n{stdout}--- stdout end ---\nstderr:\n{stderr}--- stderr end ---\n"
    )
}
