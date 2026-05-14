use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn array_change_key_case_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dir = workspace_root.join("tests/fixtures/milestone557");
    let mut fixtures = cli_snapshot_fixtures(&fixture_dir);
    fixtures.sort();

    assert!(
        !fixtures.is_empty(),
        "expected array_change_key_case CLI fixtures"
    );

    for fixture in fixtures {
        let file_name = fixture
            .file_name()
            .and_then(|name| name.to_str())
            .expect("fixture file name is valid UTF-8");
        let fixture_arg = format!("tests/fixtures/milestone557/{file_name}");
        let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .current_dir(workspace_root)
            .args(["run", &fixture_arg])
            .output()
            .unwrap_or_else(|error| panic!("failed to run {fixture_arg}: {error}"));

        let expected = fs::read_to_string(fixture.with_extension("cli")).unwrap_or_else(|error| {
            panic!("failed to read CLI snapshot for {fixture_arg}: {error}")
        });
        let actual = render_cli_snapshot(&output);

        assert_eq!(actual, expected, "CLI snapshot mismatch for {fixture_arg}");
    }
}

fn cli_snapshot_fixtures(fixture_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .expect("array_change_key_case fixture directory is readable")
        .filter_map(|entry| {
            let path = entry
                .expect("array_change_key_case fixture entry is readable")
                .path();
            (path.extension().and_then(|extension| extension.to_str()) == Some("php"))
                .then_some(path)
        })
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
