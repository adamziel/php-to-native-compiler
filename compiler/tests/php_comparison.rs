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
    assert!(
        stdout
            .contains("system php comparison: 1 compared, 1 skipped (0 missing php, 1 phpc-only)"),
        "{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn cli_compare_php_summary_counts_missing_php_skips_with_empty_path() {
    let temp = TempFixtureDir::new("phpc-compare-missing-php-summary");
    let bin_dir = temp.path().join("bin");
    let fixture_dir = temp.path().join("fixtures");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&fixture_dir).unwrap();

    fs::write(fixture_dir.join("runs.php"), "<?php echo 'same';\n").unwrap();
    fs::write(fixture_dir.join("runs.stdout"), "same\n").unwrap();

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
        stdout.contains("fixture tests: 1 passed, 0 failed"),
        "{stdout}"
    );
    assert!(
        stdout
            .contains("system php comparison: 0 compared, 1 skipped (1 missing php, 0 phpc-only)"),
        "{stdout}"
    );
}

#[test]
#[cfg(unix)]
fn cli_compare_php_json_reports_deterministic_summary_with_fake_php() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempFixtureDir::new("phpc-compare-json-summary");
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
        .args(["test", "--compare-php-json"])
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

    assert_eq!(
        stdout,
        concat!(
            "{\n",
            "  \"contract_version\": 1,\n",
            "  \"summary\": {\n",
            "    \"fixtures\": { \"passed\": 2, \"failed\": 0, \"total\": 2 },\n",
            "    \"php_comparison\": {\n",
            "      \"compared\": 1,\n",
            "      \"skipped\": 1,\n",
            "      \"missing_system_php\": 0,\n",
            "      \"phpc_only\": 1\n",
            "    }\n",
            "  },\n",
            "  \"failures\": [\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
#[cfg(unix)]
fn cli_compare_php_json_reports_missing_php_skips_with_empty_path() {
    let temp = TempFixtureDir::new("phpc-compare-json-missing-php-summary");
    let bin_dir = temp.path().join("bin");
    let fixture_dir = temp.path().join("fixtures");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::create_dir_all(&fixture_dir).unwrap();

    fs::write(fixture_dir.join("runs.php"), "<?php echo 'same';\n").unwrap();
    fs::write(fixture_dir.join("runs.stdout"), "same\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--compare-php-json"])
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

    assert_eq!(
        stdout,
        concat!(
            "{\n",
            "  \"contract_version\": 1,\n",
            "  \"summary\": {\n",
            "    \"fixtures\": { \"passed\": 1, \"failed\": 0, \"total\": 1 },\n",
            "    \"php_comparison\": {\n",
            "      \"compared\": 0,\n",
            "      \"skipped\": 1,\n",
            "      \"missing_system_php\": 1,\n",
            "      \"phpc_only\": 0\n",
            "    }\n",
            "  },\n",
            "  \"failures\": [\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
#[cfg(unix)]
fn cli_php_versions_json_reports_configured_php_binary_matrix() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempFixtureDir::new("phpc-php-version-manifest");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let php82 = bin_dir.join("php8.2");
    fs::write(&php82, "#!/bin/sh\nprintf '8.2.99'\n").unwrap();
    fs::set_permissions(&php82, fs::Permissions::from_mode(0o755)).unwrap();

    let php84 = bin_dir.join("php8.4");
    fs::write(&php84, "#!/bin/sh\nprintf '8.4.1'\n").unwrap();
    fs::set_permissions(&php84, fs::Permissions::from_mode(0o755)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--php-versions-json"])
        .env("PATH", &bin_dir)
        .env("PHPC_PHP_BINARIES", "php8.4, missing-php, php8.2, php8.4")
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
            "  \"contract_version\": 1,\n",
            "  \"tracked_php_branches\": [\"8.2\", \"8.3\", \"8.4\", \"8.5\"],\n",
            "  \"summary\": {\n",
            "    \"requested\": 3,\n",
            "    \"available\": 2,\n",
            "    \"tracked_available\": 2,\n",
            "    \"missing_tracked_branches\": [\"8.3\", \"8.5\"]\n",
            "  },\n",
            "  \"php_binaries\": [\n",
            "    {\n",
            "      \"command\": \"missing-php\",\n",
            "      \"available\": false,\n",
            "      \"version\": null,\n",
            "      \"branch\": null,\n",
            "      \"tracked\": false\n",
            "    },\n",
            "    {\n",
            "      \"command\": \"php8.2\",\n",
            "      \"available\": true,\n",
            "      \"version\": \"8.2.99\",\n",
            "      \"branch\": \"8.2\",\n",
            "      \"tracked\": true\n",
            "    },\n",
            "    {\n",
            "      \"command\": \"php8.4\",\n",
            "      \"available\": true,\n",
            "      \"version\": \"8.4.1\",\n",
            "      \"branch\": \"8.4\",\n",
            "      \"tracked\": true\n",
            "    }\n",
            "  ]\n",
            "}\n",
        )
    );
}

#[test]
#[cfg(unix)]
fn cli_php_versions_json_uses_default_php_command_when_env_is_unset() {
    let temp = TempFixtureDir::new("phpc-php-version-manifest-default");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_phpc"))
        .args(["test", "--php-versions-json"])
        .env("PATH", &bin_dir)
        .env_remove("PHPC_PHP_BINARIES")
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
            "  \"contract_version\": 1,\n",
            "  \"tracked_php_branches\": [\"8.2\", \"8.3\", \"8.4\", \"8.5\"],\n",
            "  \"summary\": {\n",
            "    \"requested\": 1,\n",
            "    \"available\": 0,\n",
            "    \"tracked_available\": 0,\n",
            "    \"missing_tracked_branches\": [\"8.2\", \"8.3\", \"8.4\", \"8.5\"]\n",
            "  },\n",
            "  \"php_binaries\": [\n",
            "    {\n",
            "      \"command\": \"php\",\n",
            "      \"available\": false,\n",
            "      \"version\": null,\n",
            "      \"branch\": null,\n",
            "      \"tracked\": false\n",
            "    }\n",
            "  ]\n",
            "}\n",
        )
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
