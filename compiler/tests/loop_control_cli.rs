use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn loop_control_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dirs = [
        workspace_root.join("tests/fixtures/milestone6"),
        workspace_root.join("tests/fixtures/milestone725"),
    ];
    let mut fixtures = fixture_dirs
        .iter()
        .flat_map(|fixture_dir| cli_snapshot_fixtures(fixture_dir, workspace_root))
        .collect::<Vec<_>>();

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected loop-control CLI snapshot fixtures"
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

fn cli_snapshot_fixtures(fixture_dir: &Path, workspace_root: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .unwrap_or_else(|error| {
            panic!(
                "loop-control fixture directory {} is readable: {error}",
                fixture_dir
                    .strip_prefix(workspace_root)
                    .unwrap_or(fixture_dir)
                    .display()
            )
        })
        .map(|entry| {
            entry
                .expect("loop-control fixture entry is readable")
                .path()
        })
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
