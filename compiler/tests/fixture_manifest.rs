use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn cli_list_fixtures_prints_deterministic_manifest_without_running_fixtures() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest");
    let fixture_dir = temp.path().join("fixtures");
    let nested_dir = fixture_dir.join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    fs::write(fixture_dir.join("zeta.php"), "<?php this is not parsed\n").unwrap();
    fs::write(nested_dir.join("beta.php"), "<?php echo 'beta';\n").unwrap();
    fs::write(nested_dir.join("beta.stderr"), "beta stderr\n").unwrap();
    fs::write(nested_dir.join("beta.exit"), "7\n").unwrap();
    fs::write(nested_dir.join("beta.phpc-only"), "").unwrap();
    fs::write(fixture_dir.join("alpha.php"), "<?php echo 'alpha';\n").unwrap();
    fs::write(fixture_dir.join("alpha.stdout"), "alpha\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--compare-php", "--list-fixtures"])
        .arg(&fixture_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "{stderr}");

    assert_eq!(
        stdout,
        concat!(
            "fixture manifest: 3 fixtures\n",
            "summary: php-comparison eligible=2, phpc-only=1 expectations stdout=1, stderr=1, exit=1, phpc-only=1 orphan sidecars=0\n",
            "alpha.php expectations=stdout php-comparison=eligible\n",
            "nested/beta.php expectations=stderr,exit php-comparison=phpc-only\n",
            "zeta.php expectations=none php-comparison=eligible\n",
        )
    );
}

#[test]
fn cli_list_fixtures_reports_orphan_sidecars_deterministically() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-orphans");
    let fixture_dir = temp.path().join("fixtures");
    let nested_dir = fixture_dir.join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    fs::write(fixture_dir.join("live.php"), "<?php echo 'live';\n").unwrap();
    fs::write(fixture_dir.join("live.stdout"), "live\n").unwrap();
    fs::write(fixture_dir.join("live.phpc-only"), "").unwrap();
    fs::write(fixture_dir.join("alpha.stdout"), "stale\n").unwrap();
    fs::write(fixture_dir.join("zeta.stderr"), "stale\n").unwrap();
    fs::write(
        fixture_dir.join("ignored.out"),
        "not a recognized sidecar\n",
    )
    .unwrap();
    fs::write(nested_dir.join("beta.exit"), "1\n").unwrap();
    fs::write(nested_dir.join("beta.phpc-only"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--list-fixtures"])
        .arg(&fixture_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "{stderr}");

    assert_eq!(
        stdout,
        concat!(
            "fixture manifest: 1 fixtures\n",
            "summary: php-comparison eligible=0, phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 orphan sidecars=4\n",
            "live.php expectations=stdout php-comparison=phpc-only\n",
            "orphan sidecar: alpha.stdout kind=stdout expected-fixture=alpha.php\n",
            "orphan sidecar: nested/beta.exit kind=exit expected-fixture=nested/beta.php\n",
            "orphan sidecar: nested/beta.phpc-only kind=phpc-only expected-fixture=nested/beta.php\n",
            "orphan sidecar: zeta.stderr kind=stderr expected-fixture=zeta.php\n",
        )
    );
}

#[test]
fn cli_test_execution_ignores_orphan_sidecars() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-orphans-run");
    let fixture_dir = temp.path().join("fixtures");
    fs::create_dir_all(&fixture_dir).unwrap();

    fs::write(fixture_dir.join("runs.php"), "<?php echo 'ok';\n").unwrap();
    fs::write(fixture_dir.join("runs.stdout"), "ok\n").unwrap();
    fs::write(fixture_dir.join("dangling.exit"), "99\n").unwrap();
    fs::write(
        fixture_dir.join("dangling.stderr"),
        "should not affect execution\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test"])
        .arg(&fixture_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stdout, "fixture tests: 1 passed, 0 failed\n");
    assert!(stderr.is_empty(), "{stderr}");
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
