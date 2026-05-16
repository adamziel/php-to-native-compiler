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
    fs::write(fixture_dir.join("alpha.cli"), "phpc run alpha.php\n").unwrap();

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
            "summary: php-comparison eligible=2, phpc-only=1 expectations stdout=1, stderr=1, exit=1, phpc-only=1 phpc-only-reason-gaps=0 cli-exercises=1 cli-exercise-gaps=2 missing expectation sidecars=8 orphan sidecars=0 unrecognized sidecars=0 bytes source=64 stdout=6 stderr=12 exit=2 cli=19 phpc-only=48 orphan-sidecars=0 unrecognized-sidecars=0\n",
            "alpha.php expectations=stdout cli-exercise=yes missing-expectation-sidecars=stderr,exit php-comparison=eligible bytes source=20 stdout=6 stderr=- exit=- cli=19 phpc-only=-\n",
            "nested/beta.php expectations=stderr,exit cli-exercise=no missing-expectation-sidecars=stdout,cli php-comparison=phpc-only phpc-only-reason=project diagnostic has no system PHP equivalent bytes source=19 stdout=- stderr=12 exit=2 cli=- phpc-only=48\n",
            "zeta.php expectations=none cli-exercise=no missing-expectation-sidecars=stdout,stderr,exit,cli php-comparison=eligible bytes source=25 stdout=- stderr=- exit=- cli=- phpc-only=-\n",
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
    fs::write(fixture_dir.join("live.out"), "manual note\n").unwrap();
    fs::write(fixture_dir.join("alpha.stdout"), "stale\n").unwrap();
    fs::write(fixture_dir.join("zeta.stderr"), "stale\n").unwrap();
    fs::write(
        fixture_dir.join("ignored.out"),
        "not a recognized sidecar\n",
    )
    .unwrap();
    fs::write(nested_dir.join("beta.exit"), "1\n").unwrap();
    fs::write(nested_dir.join("beta.cli"), "stale cli\n").unwrap();
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
            "summary: php-comparison eligible=0, phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 phpc-only-reason-gaps=1 cli-exercises=0 cli-exercise-gaps=1 missing expectation sidecars=3 orphan sidecars=5 unrecognized sidecars=1 bytes source=19 stdout=5 stderr=0 exit=0 cli=0 phpc-only=0 orphan-sidecars=24 unrecognized-sidecars=12\n",
            "live.php expectations=stdout cli-exercise=no missing-expectation-sidecars=stderr,exit,cli php-comparison=phpc-only phpc-only-reason= bytes source=19 stdout=5 stderr=- exit=- cli=- phpc-only=0\n",
            "orphan sidecar: alpha.stdout kind=stdout expected-fixture=alpha.php bytes=6 sha256=44ea8ede9025c26663124ceeefca2a35e40e5021cd116e436d368e2deae3355e\n",
            "orphan sidecar: nested/beta.cli kind=cli expected-fixture=nested/beta.php bytes=10 sha256=14ffec25f97b4b6f339e67a30c022ca5f17b984160fb4b71ed239a281c7745d0\n",
            "orphan sidecar: nested/beta.exit kind=exit expected-fixture=nested/beta.php bytes=2 sha256=4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865\n",
            "orphan sidecar: nested/beta.phpc-only kind=phpc-only expected-fixture=nested/beta.php bytes=0 sha256=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n",
            "orphan sidecar: zeta.stderr kind=stderr expected-fixture=zeta.php bytes=6 sha256=44ea8ede9025c26663124ceeefca2a35e40e5021cd116e436d368e2deae3355e\n",
            "unrecognized sidecar: live.out extension=out expected-fixture=live.php bytes=12 sha256=826e5a53b4ca0a63f4750ef01c1f1f1e92372518e8a8932794adaaf49950ec96\n",
        )
    );
}

#[test]
fn cli_list_fixtures_json_reports_unrecognized_sidecars() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-unrecognized");
    let fixture_dir = temp.path().join("fixtures");
    let php_dir = fixture_dir.join("compat").join("php");
    fs::create_dir_all(&php_dir).unwrap();

    fs::write(php_dir.join("subject.php"), "<?php echo 'ok';\n").unwrap();
    fs::write(php_dir.join("subject.note"), "manual note\n").unwrap();
    fs::write(php_dir.join("ignored.note"), "no matching fixture\n").unwrap();

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

    assert!(stdout.contains("  \"contract_version\": 13,\n"), "{stdout}");
    assert!(
        stdout.contains("    \"unrecognized_sidecars\": 1,\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("    \"unrecognized_sidecar_bytes\": 12\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("        \"unrecognized_sidecars\": 1,\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("        \"unrecognized_sidecar_bytes\": 12\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            concat!(
                "    {\n",
                "      \"path\": \"compat/php/subject.note\",\n",
                "      \"extension\": \"note\",\n",
                "      \"expected_fixture\": \"compat/php/subject.php\",\n",
                "      \"bytes\": 12,\n",
                "      \"sha256\": \"826e5a53b4ca0a63f4750ef01c1f1f1e92372518e8a8932794adaaf49950ec96\"\n",
                "    }\n"
            )
        ),
        "{stdout}"
    );
    assert!(!stdout.contains("ignored.note"), "{stdout}");
}

#[test]
fn cli_list_fixtures_json_reports_phpc_only_reason_gaps() {
    let temp = TempFixtureDir::new("phpc-fixture-manifest-phpc-only-gap");
    let fixture_dir = temp.path().join("fixtures");
    let php_dir = fixture_dir.join("compat").join("php");
    fs::create_dir_all(&php_dir).unwrap();

    fs::write(php_dir.join("empty_reason.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(php_dir.join("empty_reason.phpc-only"), " \n").unwrap();

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
    assert!(stdout.contains("  \"contract_version\": 13,\n"), "{stdout}");
    assert_eq!(stdout.matches("\"phpc_only_reason_gaps\": 1").count(), 2);
    assert!(
        stdout.contains("      \"phpc_only_reason\": \" \"\n"),
        "{stdout}"
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
    fs::write(fixture_dir.join("alpha.cli"), "phpc run alpha.php\n").unwrap();
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
            "  \"contract_version\": 13,\n",
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
            "    \"cli_exercises\": 1,\n",
            "    \"cli_exercise_gaps\": 2,\n",
            "    \"missing_expectation_sidecars\": 8,\n",
            "    \"phpc_only_reason_gaps\": 0,\n",
            "    \"orphan_sidecars\": 1,\n",
            "    \"unrecognized_sidecars\": 0,\n",
            "    \"file_bytes\": {\n",
            "      \"source\": 64,\n",
            "      \"stdout\": 6,\n",
            "      \"stderr\": 12,\n",
            "      \"exit\": 2,\n",
            "      \"cli\": 19,\n",
            "      \"phpc_only\": 54\n",
            "    },\n",
            "    \"orphan_sidecar_bytes\": 6,\n",
            "    \"unrecognized_sidecar_bytes\": 0\n",
            "  },\n",
            "  \"fixtures\": [\n",
            "    {\n",
            "      \"path\": \"alpha.php\",\n",
            "      \"expectations\": [\"stdout\"],\n",
            "      \"missing_expectation_sidecars\": [\"stderr\", \"exit\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 20,\n",
            "        \"stdout\": 6,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": 19,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"file_sha256\": {\n",
            "        \"source\": \"736f8cf99e32d995b4b811014e31aa90592b9f833bd316904db901f7e6ed8491\",\n",
            "        \"stdout\": \"b6a98d9ce9a2d9149288fa3df42d377c3e42737afdcdaf714e33c0a100b51060\",\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": \"6610ff430f77cce31f8c81c88e4f3fc69e28a63cf51d0b8d1046300c73fbc815\",\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"php_comparison\": \"eligible\",\n",
            "      \"phpc_only_reason\": null\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"nested/beta.php\",\n",
            "      \"expectations\": [\"stderr\", \"exit\"],\n",
            "      \"missing_expectation_sidecars\": [\"stdout\", \"cli\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 19,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": 12,\n",
            "        \"exit\": 2,\n",
            "        \"cli\": null,\n",
            "        \"phpc_only\": 54\n",
            "      },\n",
            "      \"file_sha256\": {\n",
            "        \"source\": \"f7ca816521704652b33d26ad5f905019440c2c472d0b73ed90ec535a5d0f1535\",\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": \"72df5c482da8d3b6e1ecc9442ec423146fada46cec1ad169cf326c1b6a59c72b\",\n",
            "        \"exit\": \"10159baf262b43a92d95db59dae1f72c645127301661e0a3ce4e38b295a97c58\",\n",
            "        \"cli\": null,\n",
            "        \"phpc_only\": \"7bf9d39fbdf18f5972bef2d4d67bed74d3e1eebddc339a712c6a38e7d71538c7\"\n",
            "      },\n",
            "      \"php_comparison\": \"phpc-only\",\n",
            "      \"phpc_only_reason\": \"project diagnostic differs from system PHP\\nsecond line\"\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"zeta.php\",\n",
            "      \"expectations\": [],\n",
            "      \"missing_expectation_sidecars\": [\"stdout\", \"stderr\", \"exit\", \"cli\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 25,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": null,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"file_sha256\": {\n",
            "        \"source\": \"465085f544a0549b3ec91bec6a86325edffea5eb0d0a08562840a652a302b297\",\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": null,\n",
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
            "      \"bytes\": 6,\n",
            "      \"sha256\": \"44ea8ede9025c26663124ceeefca2a35e40e5021cd116e436d368e2deae3355e\"\n",
            "    }\n",
            "  ],\n",
            "  \"unrecognized_sidecars\": [\n",
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
    fs::write(
        php_dir.join("cross_feature.cli"),
        "phpc run compat/php/cross_feature.php\n",
    )
    .unwrap();
    fs::write(php_dir.join("skipped.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(
        php_dir.join("skipped.phpc-only"),
        "compat target uses a project-only diagnostic\n",
    )
    .unwrap();
    fs::write(php_dir.join("stale.stderr"), "stale\n").unwrap();
    fs::write(wordpress_dir.join("source-pin.md"), "operator supplied\n").unwrap();
    fs::write(
        wordpress_dir.join("front_controller_smoke.expected"),
        "probe expected\n",
    )
    .unwrap();

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
            "  \"contract_version\": 13,\n",
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
            "    \"cli_exercises\": 1,\n",
            "    \"cli_exercise_gaps\": 1,\n",
            "    \"missing_expectation_sidecars\": 6,\n",
            "    \"phpc_only_reason_gaps\": 0,\n",
            "    \"orphan_sidecars\": 1,\n",
            "    \"unrecognized_sidecars\": 0,\n",
            "    \"file_bytes\": {\n",
            "      \"source\": 36,\n",
            "      \"stdout\": 3,\n",
            "      \"stderr\": 0,\n",
            "      \"exit\": 0,\n",
            "      \"cli\": 38,\n",
            "      \"phpc_only\": 45\n",
            "    },\n",
            "    \"orphan_sidecar_bytes\": 6,\n",
            "    \"unrecognized_sidecar_bytes\": 0\n",
            "  },\n",
            "  \"fixtures\": [\n",
            "    {\n",
            "      \"path\": \"compat/php/cross_feature.php\",\n",
            "      \"expectations\": [\"stdout\"],\n",
            "      \"missing_expectation_sidecars\": [\"stderr\", \"exit\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 17,\n",
            "        \"stdout\": 3,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": 38,\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"file_sha256\": {\n",
            "        \"source\": \"db28e0750bdb4bf54f1458e8b5d9359db9963be65493b9cce66e0e16d57e67fd\",\n",
            "        \"stdout\": \"dc51b8c96c2d745df3bd5590d990230a482fd247123599548e0632fdbf97fc22\",\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": \"26072ce2b552de26d91fbe0f19284acebb49bbeb2aa0ab7ee5a89aa062520dc3\",\n",
            "        \"phpc_only\": null\n",
            "      },\n",
            "      \"php_comparison\": \"eligible\",\n",
            "      \"phpc_only_reason\": null\n",
            "    },\n",
            "    {\n",
            "      \"path\": \"compat/php/skipped.php\",\n",
            "      \"expectations\": [],\n",
            "      \"missing_expectation_sidecars\": [\"stdout\", \"stderr\", \"exit\", \"cli\"],\n",
            "      \"file_bytes\": {\n",
            "        \"source\": 19,\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": null,\n",
            "        \"phpc_only\": 45\n",
            "      },\n",
            "      \"file_sha256\": {\n",
            "        \"source\": \"c5b495fe803004850d43c138f28a9c97c3c8df5d31216d8f466675cfa64b11c9\",\n",
            "        \"stdout\": null,\n",
            "        \"stderr\": null,\n",
            "        \"exit\": null,\n",
            "        \"cli\": null,\n",
            "        \"phpc_only\": \"fd374f05838ba6a0f157d2754ef88c613394d70d654142bb20538d0988da4861\"\n",
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
            "        \"cli_exercises\": 1,\n",
            "        \"cli_exercise_gaps\": 1,\n",
            "        \"missing_expectation_sidecars\": 6,\n",
            "        \"phpc_only_reason_gaps\": 0,\n",
            "        \"orphan_sidecars\": 1,\n",
            "        \"unrecognized_sidecars\": 0,\n",
            "        \"file_bytes\": {\n",
            "          \"source\": 36,\n",
            "          \"stdout\": 3,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"cli\": 38,\n",
            "          \"phpc_only\": 45\n",
            "        },\n",
            "        \"orphan_sidecar_bytes\": 6,\n",
            "        \"unrecognized_sidecar_bytes\": 0\n",
            "      },\n",
            "      \"source_pin\": null,\n",
            "      \"probe_expectations\": [\n",
            "      ]\n",
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
            "        \"cli_exercises\": 0,\n",
            "        \"cli_exercise_gaps\": 0,\n",
            "        \"missing_expectation_sidecars\": 0,\n",
            "        \"phpc_only_reason_gaps\": 0,\n",
            "        \"orphan_sidecars\": 0,\n",
            "        \"unrecognized_sidecars\": 0,\n",
            "        \"file_bytes\": {\n",
            "          \"source\": 0,\n",
            "          \"stdout\": 0,\n",
            "          \"stderr\": 0,\n",
            "          \"exit\": 0,\n",
            "          \"cli\": 0,\n",
            "          \"phpc_only\": 0\n",
            "        },\n",
            "        \"orphan_sidecar_bytes\": 0,\n",
            "        \"unrecognized_sidecar_bytes\": 0\n",
            "      },\n",
            "      \"source_pin\": {\n",
            "        \"path\": \"compat/wordpress/source-pin.md\",\n",
            "        \"bytes\": 18,\n",
            "        \"sha256\": \"121ab051bd83ec3873da928f13566a2b50a494359d3a62043ee02019efc876a1\"\n",
            "      },\n",
            "      \"probe_expectations\": [\n",
            "        {\n",
            "          \"path\": \"compat/wordpress/front_controller_smoke.expected\",\n",
            "          \"bytes\": 15,\n",
            "          \"sha256\": \"f157bfc95bc502cbd4020422a86b7bf1c0b4f4d650f3ffb5390d4f97f7714fba\"\n",
            "        }\n",
            "      ]\n",
            "    }\n",
            "  ],\n",
            "  \"orphan_sidecars\": [\n",
            "    {\n",
            "      \"path\": \"compat/php/stale.stderr\",\n",
            "      \"kind\": \"stderr\",\n",
            "      \"expected_fixture\": \"compat/php/stale.php\",\n",
            "      \"bytes\": 6,\n",
            "      \"sha256\": \"44ea8ede9025c26663124ceeefca2a35e40e5021cd116e436d368e2deae3355e\"\n",
            "    }\n",
            "  ],\n",
            "  \"unrecognized_sidecars\": [\n",
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
    fs::write(
        php_dir.join("cross_feature.cli"),
        "phpc run compat/php/cross_feature.php\n",
    )
    .unwrap();
    fs::write(php_dir.join("skipped.php"), "<?php echo 'skip';\n").unwrap();
    fs::write(
        php_dir.join("skipped.phpc-only"),
        "compat target uses a project-only diagnostic\n",
    )
    .unwrap();
    fs::write(php_dir.join("stale.stderr"), "stale\n").unwrap();
    fs::write(wordpress_dir.join("source-pin.md"), "operator supplied\n").unwrap();
    fs::write(
        wordpress_dir.join("front_controller_smoke.expected"),
        "probe expected\n",
    )
    .unwrap();

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
            "summary: php-comparison eligible=1, phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 phpc-only-reason-gaps=0 cli-exercises=1 cli-exercise-gaps=1 missing expectation sidecars=6 orphan sidecars=1 unrecognized sidecars=0 bytes source=36 stdout=3 stderr=0 exit=0 cli=38 phpc-only=45 orphan-sidecars=6 unrecognized-sidecars=0\n",
            "compat/php/cross_feature.php expectations=stdout cli-exercise=yes missing-expectation-sidecars=stderr,exit php-comparison=eligible bytes source=17 stdout=3 stderr=- exit=- cli=38 phpc-only=-\n",
            "compat/php/skipped.php expectations=none cli-exercise=no missing-expectation-sidecars=stdout,stderr,exit,cli php-comparison=phpc-only phpc-only-reason=compat target uses a project-only diagnostic bytes source=19 stdout=- stderr=- exit=- cli=- phpc-only=45\n",
            "orphan sidecar: compat/php/stale.stderr kind=stderr expected-fixture=compat/php/stale.php bytes=6 sha256=44ea8ede9025c26663124ceeefca2a35e40e5021cd116e436d368e2deae3355e\n",
            "compatibility target: php path=compat/php fixtures=2 php-comparison eligible=1 phpc-only=1 expectations stdout=1, stderr=0, exit=0, phpc-only=1 phpc-only-reason-gaps=0 cli-exercises=1 cli-exercise-gaps=1 missing expectation sidecars=6 orphan sidecars=1 unrecognized sidecars=0 bytes source=36 stdout=3 stderr=0 exit=0 cli=38 phpc-only=45 orphan-sidecars=6 unrecognized-sidecars=0 probe expectations=0 bytes=0 source-pin path=- bytes=- sha256=-\n",
            "compatibility target: wordpress path=compat/wordpress fixtures=0 php-comparison eligible=0 phpc-only=0 expectations stdout=0, stderr=0, exit=0, phpc-only=0 phpc-only-reason-gaps=0 cli-exercises=0 cli-exercise-gaps=0 missing expectation sidecars=0 orphan sidecars=0 unrecognized sidecars=0 bytes source=0 stdout=0 stderr=0 exit=0 cli=0 phpc-only=0 orphan-sidecars=0 unrecognized-sidecars=0 probe expectations=1 bytes=15 source-pin path=compat/wordpress/source-pin.md bytes=18 sha256=121ab051bd83ec3873da928f13566a2b50a494359d3a62043ee02019efc876a1\n",
            "compatibility probe expectation: compat/wordpress/front_controller_smoke.expected bytes=15 sha256=f157bfc95bc502cbd4020422a86b7bf1c0b4f4d650f3ffb5390d4f97f7714fba\n",
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
