use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn cli_runtime_error_snapshots_match_committed_outputs() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let fixture_dir = workspace_root.join("tests/fixtures/runtime_errors");
    let mut fixtures = cli_snapshot_fixtures(&fixture_dir);

    fixtures.sort();
    assert!(
        fixtures.len() >= 5,
        "expected representative runtime error CLI fixtures"
    );

    for fixture in fixtures {
        let file_name = fixture
            .file_name()
            .and_then(|value| value.to_str())
            .expect("runtime error fixture file name is valid UTF-8");
        let fixture_arg = format!("tests/fixtures/runtime_errors/{file_name}");
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

#[test]
fn cli_execution_step_budget_env_reports_source_location() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("compiler has a workspace root");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!(
        "phpc-execution-budget-{}-{unique}.php",
        std::process::id()
    ));
    fs::write(&fixture, "<?php\nwhile (true) {\n}\n").expect("write execution budget fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .current_dir(workspace_root)
        .env("PHPC_MAX_EXECUTION_STEPS", "3")
        .args(["run", fixture.to_str().expect("fixture path is UTF-8")])
        .output()
        .expect("run phpc with execution step budget");

    let _ = fs::remove_file(&fixture);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("runtime error at "));
    assert!(stderr.contains(":2:1: maximum execution step budget exceeded after 3 step(s); "));
    assert!(stderr.contains("last location "));
    assert!(stderr.contains(":2:1\n"));
}

fn cli_snapshot_fixtures(fixture_dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(fixture_dir)
        .expect("runtime error fixture directory is readable")
        .map(|entry| {
            entry
                .expect("runtime error fixture entry is readable")
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
