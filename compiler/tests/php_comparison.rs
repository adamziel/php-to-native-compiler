use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use php_compiler::test_runner::{
    run_fixture_dir_with_options, system_php_available, FixtureRunOptions,
};

#[test]
fn comparison_mode_matches_system_php_for_milestone2_fixtures() {
    if !system_php_available() {
        return;
    }

    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/milestone2");
    let summary =
        run_fixture_dir_with_options(&fixture_dir, FixtureRunOptions { compare_php: true })
            .unwrap();

    assert_eq!(summary.failed, 0, "{:#?}", summary.failures);
    assert!(summary.passed >= 2);
    assert_eq!(summary.php_compared, summary.passed);
    assert_eq!(summary.php_skipped, 0);
}

#[test]
fn cli_compare_php_accepts_fixture_dir() {
    if !system_php_available() {
        return;
    }

    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/milestone2");
    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--compare-php"])
        .arg(&fixture_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("fixture tests:"), "{stdout}");
    assert!(stdout.contains("0 failed"), "{stdout}");
    assert!(stdout.contains("system php comparison:"), "{stdout}");
    assert!(stdout.contains("0 skipped"), "{stdout}");
}

#[test]
#[cfg(unix)]
fn cli_compare_php_summary_counts_phpc_only_skips_with_fake_php() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempFixtureDir::new("phpc-compare-summary");
    let bin_dir = temp.path().join("bin");
    let fixture_dir = temp.path().join("fixtures");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&fixture_dir).unwrap();

    let fake_php = bin_dir.join("php");
    fs::write(
        &fake_php,
        "#!/bin/sh\ncase \"$1\" in\n  */matches.php) printf 'same'; exit 0 ;;\n  *) printf 'unexpected php fixture: %s\\n' \"$1\" >&2; exit 1 ;;\nesac\n",
    )
    .unwrap();
    fs::set_permissions(&fake_php, fs::Permissions::from_mode(0o755)).unwrap();

    fs::write(fixture_dir.join("matches.php"), "<?php echo 'same';\n").unwrap();
    fs::write(fixture_dir.join("matches.stdout"), "same\n").unwrap();
    fs::write(fixture_dir.join("phpc_only.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(fixture_dir.join("phpc_only.stdout"), "skip\n").unwrap();
    fs::write(fixture_dir.join("phpc_only.phpc-only"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--compare-php"])
        .arg(&fixture_dir)
        .env("PATH", &bin_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "{stderr}");
    assert!(
        stdout.contains("fixture tests: 2 passed, 0 failed"),
        "{stdout}"
    );
    assert!(
        stdout.contains("system php comparison: 1 compared, 1 skipped"),
        "{stdout}"
    );
}

struct TempFixtureDir {
    path: PathBuf,
}

impl TempFixtureDir {
    fn new(prefix: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
