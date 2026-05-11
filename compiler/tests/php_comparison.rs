use std::path::Path;
use std::process::Command;

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
