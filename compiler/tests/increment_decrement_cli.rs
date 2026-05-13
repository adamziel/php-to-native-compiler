use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn increment_decrement_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dirs = [
        "milestone134",
        "milestone135",
        "milestone136",
        "milestone141",
    ];
    let mut fixtures = fixture_dirs
        .iter()
        .flat_map(|dir| {
            let fixture_dir = workspace_root.join(format!("tests/fixtures/{dir}"));
            cli_snapshot_fixtures(dir, &fixture_dir)
        })
        .collect::<Vec<_>>();

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected increment/decrement CLI snapshot fixtures"
    );

    for (fixture_dir_name, fixture) in fixtures {
        let file_name = fixture
            .file_name()
            .and_then(|value| value.to_str())
            .expect("increment/decrement fixture file name is valid UTF-8");
        let fixture_arg = format!("tests/fixtures/{fixture_dir_name}/{file_name}");
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

fn cli_snapshot_fixtures(fixture_dir_name: &str, fixture_dir: &Path) -> Vec<(String, PathBuf)> {
    fs::read_dir(fixture_dir)
        .expect("increment/decrement fixture directory is readable")
        .map(|entry| {
            entry
                .expect("increment/decrement fixture entry is readable")
                .path()
        })
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("php"))
        .filter(|path| path.with_extension("cli").exists())
        .map(|path| (fixture_dir_name.to_string(), path))
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
