use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn object_introspection_builtin_cli_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dirs = [
        workspace_root.join("tests/fixtures/milestone97"),
        workspace_root.join("tests/fixtures/milestone98"),
        workspace_root.join("tests/fixtures/milestone99"),
        workspace_root.join("tests/fixtures/milestone100"),
        workspace_root.join("tests/fixtures/milestone101"),
        workspace_root.join("tests/fixtures/milestone102"),
        workspace_root.join("tests/fixtures/milestone103"),
        workspace_root.join("tests/fixtures/milestone104"),
        workspace_root.join("tests/fixtures/milestone105"),
        workspace_root.join("tests/fixtures/milestone106"),
        workspace_root.join("tests/fixtures/milestone107"),
        workspace_root.join("tests/fixtures/milestone109"),
        workspace_root.join("tests/fixtures/milestone110"),
        workspace_root.join("tests/fixtures/milestone111"),
        workspace_root.join("tests/fixtures/milestone112"),
    ];
    let mut fixtures = fixture_dirs
        .iter()
        .flat_map(|fixture_dir| cli_snapshot_fixtures(fixture_dir))
        .collect::<Vec<_>>();

    fixtures.sort();
    assert!(
        !fixtures.is_empty(),
        "expected object introspection CLI fixtures"
    );

    for fixture in fixtures {
        let fixture_arg = fixture
            .strip_prefix(workspace_root)
            .expect("object introspection fixture is under workspace root")
            .to_string_lossy()
            .replace('\\', "/");
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
        .expect("object introspection fixture directory is readable")
        .map(|entry| {
            entry
                .expect("object introspection fixture entry is readable")
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
