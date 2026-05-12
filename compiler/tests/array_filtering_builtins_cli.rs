use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn array_filtering_builtin_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dirs = [
        workspace_root.join("tests/fixtures/milestone20"),
        workspace_root.join("tests/fixtures/milestone21"),
    ];
    let mut fixtures = Vec::new();
    for fixture_dir in fixture_dirs {
        fixtures.extend(cli_snapshot_fixtures(&fixture_dir));
    }

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected array-filtering builtin CLI snapshot fixtures"
    );

    for fixture in fixtures {
        let file_name = fixture
            .file_name()
            .and_then(|value| value.to_str())
            .expect("array-filtering fixture file name is valid UTF-8");
        let relative_dir = fixture
            .parent()
            .and_then(|path| path.strip_prefix(workspace_root).ok())
            .expect("fixture lives under the workspace root");
        let fixture_arg = format!("{}/{file_name}", relative_dir.display());
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
        .expect("array-filtering fixture directory is readable")
        .map(|entry| {
            entry
                .expect("array-filtering fixture entry is readable")
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
