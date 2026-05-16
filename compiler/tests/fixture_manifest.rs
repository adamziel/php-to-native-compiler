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
    fs::write(
        nested_dir.join("beta.phpc-only"),
        "project diagnostic has no system PHP equivalent\n",
    )
    .unwrap();
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
            "summary: php-comparison eligible=2, phpc-only=1 expectations stdout=1, stderr=1, exit=1, phpc-only=1 orphan sidecars=0 bytes source=64 stdout=6 stderr=12 exit=2 phpc-only=48\n",
            "alpha.php expectations=stdout php-comparison=eligible bytes source=20 stdout=6 stderr=- exit=- phpc-only=-\n",
            "nested/beta.php expectations=stderr,exit php-comparison=phpc-only phpc-only-reason=project diagnostic has no system PHP equivalent bytes source=19 stdout=- stderr=12 exit=2 phpc-only=48\n",
            "zeta.php expectations=none php-comparison=eligible bytes source=25 stdout=- stderr=- exit=- phpc-only=-\n",
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
            "summary: php-comparison eligible=0, phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 orphan sidecars=4 bytes source=19 stdout=5 stderr=0 exit=0 phpc-only=0\n",
            "live.php expectations=stdout php-comparison=phpc-only phpc-only-reason= bytes source=19 stdout=5 stderr=- exit=- phpc-only=0\n",
            "orphan sidecar: alpha.stdout kind=stdout expected-fixture=alpha.php bytes=6\n",
            "orphan sidecar: nested/beta.exit kind=exit expected-fixture=nested/beta.php bytes=2\n",
            "orphan sidecar: nested/beta.phpc-only kind=phpc-only expected-fixture=nested/beta.php bytes=0\n",
            "orphan sidecar: zeta.stderr kind=stderr expected-fixture=zeta.php bytes=6\n",
        )
    );
}

#[test]
fn cli_list_fixtures_json_prints_deterministic_machine_readable_manifest() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-json");
    let fixture_dir = temp.path().join("fixtures");
    let nested_dir = fixture_dir.join("nested");
    fs::create_dir_all(&nested_dir).unwrap();

    fs::write(fixture_dir.join("zeta.php"), "<?php this is not parsed\n").unwrap();
    fs::write(nested_dir.join("beta.php"), "<?php echo 'beta';\n").unwrap();
    fs::write(nested_dir.join("beta.stderr"), "beta stderr\n").unwrap();
    fs::write(nested_dir.join("beta.exit"), "7\n").unwrap();
    fs::write(
        nested_dir.join("beta.phpc-only"),
        "project diagnostic differs from system PHP\nsecond line",
    )
    .unwrap();
    fs::write(fixture_dir.join("alpha.php"), "<?php echo 'alpha';\n").unwrap();
    fs::write(fixture_dir.join("alpha.stdout"), "alpha\n").unwrap();
    fs::write(fixture_dir.join("orphan.stdout"), "stale\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--compare-php", "--list-fixtures-json"])
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
            "{\n",
            "  \"contract_version\": 4,\n",
            "  \"fixture_count\": 3,\n",
            "  \"summary\": {\n",
            "    \"total\": 3,\n",
            "    \"php_comparison_eligible\": 2,\n",
            "    \"phpc_only\": 1,\n",
            "    \"expectations\": {\n",
            "      \"stdout\": 1,\n",
            "      \"stderr\": 1,\n",
            "      \"exit\": 1,\n",
            "      \"phpc_only\": 1\n",
            "    },\n",
            "    \"orphan_sidecars\": 1,\n",
            "    \"file_bytes\": {\n",
            "      \"source\": 64,\n",
            "      \"stdout\": 6,\n",
            "      \"stderr\": 12,\n",
            "      \"exit\": 2,\n",
            "      \"phpc_only\": 54\n",
            "    }\n",
            "  },\n",
            "  \"fixtures\": [\n",
            "    {\n",
            "      \"path\": \"alpha.php\",\n",
            "      \"expectations\": [\"stdout\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 20,\n",
            "        \"stdout\": 6,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"php_comparison\": \"eligible\",\n",
            "      \"phpc_only_reason\": null\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"nested/beta.php\",\n",
            "      \"expectations\": [\"stderr\", \"exit\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 19,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": 12,\n",
            "        \"exit\": 2,\n",
            "        \"phpc_only\": 54\n",
            "      },\n",
            "      \"php_comparison\": \"phpc-only\",\n",
            "      \"phpc_only_reason\": \"project diagnostic differs from system PHP\\nsecond line\"\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"zeta.php\",\n",
            "      \"expectations\": [],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 25,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"php_comparison\": \"eligible\",\n",
            "      \"phpc_only_reason\": null\n",
            "    }\n",
            "  ],\n",
            "  \"compatibility_targets\": [\n",
            "  ],\n",
            "  \"orphan_sidecars\": [\n",
            "    {\n",
            "      \"path\": \"orphan.stdout\",\n",
            "      \"kind\": \"stdout\",\n",
            "      \"expected_fixture\": \"orphan.php\",\n",
            "      \"bytes\": 6\n",
            "    }\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
fn cli_list_fixtures_json_reports_compatibility_targets_as_data() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-compat-targets");
    let fixture_dir = temp.path().join("fixtures");
    let php_dir = fixture_dir.join("compat").join("php");
    let wordpress_dir = fixture_dir.join("compat").join("wordpress");
    fs::create_dir_all(&php_dir).unwrap();
    fs::create_dir_all(&wordpress_dir).unwrap();

    fs::write(php_dir.join("cross_feature.php"), "<?php echo 'ok';\n").unwrap();
    fs::write(php_dir.join("cross_feature.stdout"), "ok\n").unwrap();
    fs::write(php_dir.join("skipped.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(
        php_dir.join("skipped.phpc-only"),
        "compat target uses a project-only diagnostic\n",
    )
    .unwrap();
    fs::write(php_dir.join("stale.stderr"), "stale\n").unwrap();
    fs::write(wordpress_dir.join("source-pin.md"), "operator supplied\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--list-fixtures-json"])
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
            "{\n",
            "  \"contract_version\": 4,\n",
            "  \"fixture_count\": 2,\n",
            "  \"summary\": {\n",
            "    \"total\": 2,\n",
            "    \"php_comparison_eligible\": 1,\n",
            "    \"phpc_only\": 1,\n",
            "    \"expectations\": {\n",
            "      \"stdout\": 1,\n",
            "      \"stderr\": 0,\n",
            "      \"exit\": 0,\n",
            "      \"phpc_only\": 1\n",
            "    },\n",
            "    \"orphan_sidecars\": 1,\n",
            "    \"file_bytes\": {\n",
            "      \"source\": 36,\n",
            "      \"stdout\": 3,\n",
            "      \"stderr\": 0,\n",
            "      \"exit\": 0,\n",
            "      \"phpc_only\": 45\n",
            "    }\n",
            "  },\n",
            "  \"fixtures\": [\n",
            "    {\n",
            "      \"path\": \"compat/php/cross_feature.php\",\n",
            "      \"expectations\": [\"stdout\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 17,\n",
            "        \"stdout\": 3,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"php_comparison\": \"eligible\",\n",
            "      \"phpc_only_reason\": null\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"compat/php/skipped.php\",\n",
            "      \"expectations\": [],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 19,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"phpc_only\": 45\n",
            "      },\n",
            "      \"php_comparison\": \"phpc-only\",\n",
            "      \"phpc_only_reason\": \"compat target uses a project-only diagnostic\"\n",
            "    }\n",
            "  ],\n",
            "  \"compatibility_targets\": [\n",
            "    {\n",
            "      \"target\": \"php\",\n",
            "      \"path\": \"compat/php\",\n",
            "      \"summary\": {\n",
            "        \"total\": 2,\n",
            "        \"php_comparison_eligible\": 1,\n",
            "        \"phpc_only\": 1,\n",
            "        \"expectations\": {\n",
            "          \"stdout\": 1,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"phpc_only\": 1\n",
            "        },\n",
            "        \"orphan_sidecars\": 1,\n",
            "        \"file_bytes\": {\n",
            "          \"source\": 36,\n",
            "          \"stdout\": 3,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"phpc_only\": 45\n",
            "        }\n",
            "      }\n",
            "    },\n",
            "    {\n",
            "      \"target\": \"wordpress\",\n",
            "      \"path\": \"compat/wordpress\",\n",
            "      \"summary\": {\n",
            "        \"total\": 0,\n",
            "        \"php_comparison_eligible\": 0,\n",
            "        \"phpc_only\": 0,\n",
            "        \"expectations\": {\n",
            "          \"stdout\": 0,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"phpc_only\": 0\n",
            "        },\n",
            "        \"orphan_sidecars\": 0,\n",
            "        \"file_bytes\": {\n",
            "          \"source\": 0,\n",
            "          \"stdout\": 0,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"phpc_only\": 0\n",
            "        }\n",
            "      }\n",
            "    }\n",
            "  ],\n",
            "  \"orphan_sidecars\": [\n",
            "    {\n",
            "      \"path\": \"compat/php/stale.stderr\",\n",
            "      \"kind\": \"stderr\",\n",
            "      \"expected_fixture\": \"compat/php/stale.php\",\n",
            "      \"bytes\": 6\n",
            "    }\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
fn cli_list_fixtures_reports_compatibility_target_byte_counts() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-text-compat-targets");
    let fixture_dir = temp.path().join("fixtures");
    let php_dir = fixture_dir.join("compat").join("php");
    let wordpress_dir = fixture_dir.join("compat").join("wordpress");
    fs::create_dir_all(&php_dir).unwrap();
    fs::create_dir_all(&wordpress_dir).unwrap();

    fs::write(php_dir.join("cross_feature.php"), "<?php echo 'ok';\n").unwrap();
    fs::write(php_dir.join("cross_feature.stdout"), "ok\n").unwrap();
    fs::write(php_dir.join("skipped.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(
        php_dir.join("skipped.phpc-only"),
        "compat target uses a project-only diagnostic\n",
    )
    .unwrap();
    fs::write(php_dir.join("stale.stderr"), "stale\n").unwrap();
    fs::write(wordpress_dir.join("source-pin.md"), "operator supplied\n").unwrap();

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
            "fixture manifest: 2 fixtures\n",
            "summary: php-comparison eligible=1, phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 orphan sidecars=1 bytes source=36 stdout=3 stderr=0 exit=0 phpc-only=45\n",
            "compat/php/cross_feature.php expectations=stdout php-comparison=eligible bytes source=17 stdout=3 stderr=- exit=- phpc-only=-\n",
            "compat/php/skipped.php expectations=none php-comparison=phpc-only phpc-only-reason=compat target uses a project-only diagnostic bytes source=19 stdout=- stderr=- exit=- phpc-only=45\n",
            "orphan sidecar: compat/php/stale.stderr kind=stderr expected-fixture=compat/php/stale.php bytes=6\n",
            "compatibility target: php path=compat/php fixtures=2 php-comparison eligible=1 phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 orphan sidecars=1 bytes source=36 stdout=3 stderr=0 exit=0 phpc-only=45\n",
            "compatibility target: wordpress path=compat/wordpress fixtures=0 php-comparison eligible=0 phpc-only=0 expectations stdout=0, stderr=0, exit=0, phpc-only=0 orphan sidecars=0 bytes source=0 stdout=0 stderr=0 exit=0 phpc-only=0\n",
        )
    );
}

#[test]
fn cli_list_fixtures_rejects_multiple_manifest_output_modes() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-modes");
    let fixture_dir = temp.path().join("fixtures");
    fs::create_dir_all(&fixture_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--list-fixtures", "--list-fixtures-json"])
        .arg(&fixture_dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "stdout:\n{stdout}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("cli error at 0:0: expected at most one fixture manifest output mode"),
        "{stderr}"
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
