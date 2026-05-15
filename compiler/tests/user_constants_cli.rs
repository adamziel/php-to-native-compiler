use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn user_constant_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let mut fixtures = Vec::new();
    for fixture_dir in [
        "tests/fixtures/milestone61",
        "tests/fixtures/milestone62",
        "tests/fixtures/milestone63",
        "tests/fixtures/milestone65",
        "tests/fixtures/milestone66",
        "tests/fixtures/milestone67",
        "tests/fixtures/milestone68",
        "tests/fixtures/milestone715",
        "tests/fixtures/milestone717",
        "tests/fixtures/milestone718",
        "tests/fixtures/milestone719",
    ] {
        fixtures.extend(cli_snapshot_fixtures(
            &workspace_root.join(fixture_dir),
            workspace_root,
        ));
    }

    fixtures.sort();
    assert!(!fixtures.is_empty(), "expected user constant CLI fixtures");

    for fixture_arg in fixtures {
        let fixture = workspace_root.join(&fixture_arg);
        let fixture_arg = fixture_arg
            .to_str()
            .expect("user constant fixture path is valid UTF-8");
        let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
            .current_dir(workspace_root)
            .args(["run", fixture_arg])
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
        .expect("user constant fixture directory is readable")
        .map(|entry| {
            entry
                .expect("user constant fixture entry is readable")
                .path()
        })
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("php"))
        .filter(|path| path.with_extension("cli").exists())
        .map(|path| {
            path.strip_prefix(workspace_root)
                .expect("user constant fixture lives under the workspace root")
                .to_path_buf()
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
