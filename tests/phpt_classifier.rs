use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    dir.push(format!("{name}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn classify(body: &str) -> String {
    classify_with_harness_programs(body, false)
}

fn classify_at_relative_path(body: &str, relative_path: &str) -> String {
    classify_at_relative_path_with_options(body, relative_path, false)
}

fn classify_at_relative_path_with_harness_programs(body: &str, relative_path: &str) -> String {
    classify_at_relative_path_with_options(body, relative_path, true)
}

fn classify_at_relative_path_with_options(
    body: &str,
    relative_path: &str,
    harness_programs: bool,
) -> String {
    let root = temp_dir("ptn-phpt-classifier-path");
    let phpt = root.join(relative_path);
    fs::create_dir_all(phpt.parent().expect("relative path should have parent"))
        .expect("create PHPT parent");
    fs::write(&phpt, body).expect("write PHPT");

    let mut command = Command::new("bash");
    if harness_programs {
        command.env("PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS", "1");
    }
    let output = command
        .arg("-c")
        .arg("source tools/phpt-classifier.sh; ptn_phpt_classify_row \"$1\" \"$2\"")
        .arg("bash")
        .arg(relative_path)
        .arg(&phpt)
        .output()
        .expect("run classifier");
    assert!(
        output.status.success(),
        "classifier failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("classifier output should be utf8")
}

fn classify_with_pipefail(body: &str) -> String {
    classify_with_options(body, false, true, &[])
}

fn classify_with_section_cache(body: &str) -> String {
    let root = temp_dir("ptn-phpt-classifier-cache");
    let phpt = root.join("case.phpt");
    let manifest = root.join("manifest.txt");
    let cache = root.join("section-cache");
    fs::write(&phpt, body).expect("write PHPT");
    fs::write(&manifest, format!("{}\n", phpt.display())).expect("write manifest");

    let output = Command::new("bash")
        .arg("-c")
        .arg(
            "source tools/phpt-classifier.sh; \
             export PTN_PHPT_SECTION_CACHE_DIR=\"$3\"; \
             ptn_phpt_build_section_cache \"$2\" /unused \"$3\"; \
             ptn_phpt_load_section_cache_index \"$3/index.tsv\"; \
             ptn_phpt_classify_row \"$1\" \"$1\"",
        )
        .arg("bash")
        .arg(&phpt)
        .arg(&manifest)
        .arg(&cache)
        .output()
        .expect("run cached classifier");
    assert!(
        output.status.success(),
        "cached classifier failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("classifier output should be utf8")
}

fn classify_with_harness_programs(body: &str, enabled: bool) -> String {
    classify_with_harness_programs_and_env(body, enabled, &[])
}

fn classify_with_harness_programs_and_env(
    body: &str,
    enabled: bool,
    env: &[(&str, &str)],
) -> String {
    classify_with_options(body, enabled, false, env)
}

fn classify_with_options(
    body: &str,
    enabled: bool,
    pipefail: bool,
    env: &[(&str, &str)],
) -> String {
    let root = temp_dir("ptn-phpt-classifier");
    let phpt = root.join("case.phpt");
    fs::write(&phpt, body).expect("write PHPT");

    let mut command = Command::new("bash");
    for key in [
        "PTN_PHPT_AVAILABLE_LOCALES",
        "PTN_PHPT_PHP_INT_SIZE",
        "PTN_PHPT_PHP_OS_FAMILY",
        "PTN_PHPT_PHP_OS",
        "PTN_PHPT_PHP_DEBUG",
        "PTN_PHPT_PHP_ZTS",
        "PTN_PHPT_EFFECTIVE_UID",
        "PTN_PHPT_DEFINED_CONSTANTS",
        "PTN_PHPT_RUN_SLOW_TESTS",
        "PTN_PHPT_RUN_PERF_SENSITIVE",
        "SKIP_ASAN",
        "SKIP_MSAN",
        "SKIP_UBSAN",
        "SKIP_PERF_SENSITIVE",
        "SKIP_SLOW_TESTS",
        "SKIP_PRELOAD",
        "SKIP_IO_CAPTURE_TESTS",
        "USE_ZEND_ALLOC",
        "USE_TRACKED_ALLOC",
        "RUN_RESOURCE_HEAVY_TESTS",
        "STACK_LIMIT_DEFAULTS_CHECK",
        "CIRRUS_CI",
    ] {
        command.env_remove(key);
    }
    if enabled {
        command.env("PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS", "1");
    }
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command
        .arg("-c")
        .arg(if pipefail {
            "set -o pipefail; source tools/phpt-classifier.sh; ptn_phpt_classify_row tests/case.phpt \"$1\""
        } else {
            "source tools/phpt-classifier.sh; ptn_phpt_classify_row tests/case.phpt \"$1\""
        })
        .arg("bash")
        .arg(&phpt)
        .output()
        .expect("run classifier");
    assert!(
        output.status.success(),
        "classifier failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("classifier output should be utf8")
}

#[test]
fn phpt_classifier_file_section_helpers_survive_pipefail() {
    let mut phpt =
        String::from("--TEST--\npipefail early exit\n--FILE--\n<?php\nnew ErrorException();\n");
    for _ in 0..5000 {
        phpt.push_str("echo 1;\n");
    }
    phpt.push_str("--EXPECT--\n");

    let classification = classify_with_pipefail(&phpt);
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn run_bounded_phpt_classify_only_reads_manifest_file_to_completion() {
    let root = temp_dir("ptn-run-bounded-phpt-classify");
    let corpus = root.join("php-src");
    let tests_dir = corpus.join("tests/basic");
    fs::create_dir_all(&tests_dir).expect("create fake PHPT corpus");
    fs::write(corpus.join("run-tests.php"), "<?php\n").expect("write run-tests.php");
    fs::write(
        tests_dir.join("array_null_offset_deprecation.phpt"),
        "--TEST--\nnull offset\n--FILE--\n<?php\n$arr = ['' => 'value'];\necho $arr[null];\n?>\n--EXPECT--\nvalue\n",
    )
    .expect("write first PHPT");
    fs::write(
        tests_dir.join("after.phpt"),
        "--TEST--\nafter\n--FILE--\n<?php echo 'after'; ?>\n--EXPECT--\nafter\n",
    )
    .expect("write second PHPT");

    let manifest = root.join("manifest.txt");
    fs::write(
        &manifest,
        "# bucket: core rows=2\n\
tests/basic/array_null_offset_deprecation.phpt\n\
tests/basic/after.phpt",
    )
    .expect("write manifest without trailing newline");

    let progress = root.join("progress");
    let output = Command::new("timeout")
        .arg("10s")
        .arg("tools/run-bounded-phpt.sh")
        .arg("--classify-only")
        .arg(&manifest)
        .env("PHP_SRC_PHPT", &corpus)
        .env("PHPT_PROGRESS_DIR", &progress)
        .env("PTN_PHPT_AUTO_FETCH", "0")
        .output()
        .expect("run bounded classifier");
    assert!(
        output.status.success(),
        "bounded classifier failed: status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let classification_path = fs::read_dir(&progress)
        .expect("read progress dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("classification-") && name.ends_with(".tsv"))
        })
        .expect("classification output");
    let classification = fs::read_to_string(classification_path).expect("read classification");
    assert!(
        classification.contains("tests/basic/array_null_offset_deprecation.phpt\trunnable\t"),
        "{classification}"
    );
    assert!(
        classification.contains("tests/basic/after.phpt\trunnable\t"),
        "{classification}"
    );
    assert_eq!(classification.lines().count(), 2, "{classification}");
}

#[test]
fn run_bounded_phpt_passes_configured_native_timeout_to_run_tests() {
    let root = temp_dir("ptn-run-bounded-phpt-timeout");
    let corpus = root.join("php-src");
    let tests_dir = corpus.join("tests/basic");
    fs::create_dir_all(&tests_dir).expect("create fake PHPT corpus");
    fs::write(
        corpus.join("run-tests.php"),
        "<?php\n\
        $timeout = '<missing>';\n\
        for ($i = 1; $i < $argc; $i++) {\n\
            if ($argv[$i] === '--set-timeout') {\n\
                $timeout = $argv[$i + 1] ?? '<missing-value>';\n\
                break;\n\
            }\n\
        }\n\
        echo \"seen-timeout: $timeout\\n\";\n\
        echo \"Number of tests : 1\\n\";\n\
        echo \"Tests skipped   : 0\\n\";\n\
        echo \"Tests warned    : 0\\n\";\n\
        echo \"Tests failed    : 0\\n\";\n\
        echo \"Tests passed    : 1\\n\";\n",
    )
    .expect("write fake run-tests.php");
    fs::write(
        tests_dir.join("timeout.phpt"),
        "--TEST--\ntimeout\n--FILE--\n<?php echo 'ok'; ?>\n--EXPECT--\nok\n",
    )
    .expect("write PHPT");

    let manifest = root.join("manifest.txt");
    fs::write(&manifest, "tests/basic/timeout.phpt\n").expect("write manifest");

    let progress = root.join("progress");
    let output = Command::new("timeout")
        .arg("30s")
        .arg("tools/run-bounded-phpt.sh")
        .arg(&manifest)
        .env("PHP_SRC_PHPT", &corpus)
        .env("PHPT_PROGRESS_DIR", &progress)
        .env("PTN_PHPT_AUTO_FETCH", "0")
        .env("PTN_PHPT_TEST_TIMEOUT", "321")
        .output()
        .expect("run bounded PHPT");
    assert!(
        output.status.success(),
        "bounded PHPT failed: status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("seen-timeout: 321"), "{stdout}");
    assert!(
        stdout.contains("result: buckets=1 selected=1 runnable=1 excluded=0 tests=1 passed=1 failed=0 skipped=0 warned=0"),
        "{stdout}"
    );
}

#[test]
fn run_phpt_manifest_passes_configured_native_timeout_to_run_tests() {
    let root = temp_dir("ptn-run-phpt-manifest-timeout");
    let corpus = root.join("php-src");
    let tests_dir = corpus.join("tests/basic");
    fs::create_dir_all(&tests_dir).expect("create fake PHPT corpus");
    fs::write(
        corpus.join("run-tests.php"),
        "<?php\n\
        $timeout = '<missing>';\n\
        for ($i = 1; $i < $argc; $i++) {\n\
            if ($argv[$i] === '--set-timeout') {\n\
                $timeout = $argv[$i + 1] ?? '<missing-value>';\n\
                break;\n\
            }\n\
        }\n\
        echo \"seen-timeout: $timeout\\n\";\n\
        echo \"Number of tests : 1\\n\";\n\
        echo \"Tests skipped   : 0\\n\";\n\
        echo \"Tests warned    : 0\\n\";\n\
        echo \"Tests failed    : 0\\n\";\n\
        echo \"Tests passed    : 1\\n\";\n",
    )
    .expect("write fake run-tests.php");
    fs::write(
        tests_dir.join("timeout.phpt"),
        "--TEST--\ntimeout\n--FILE--\n<?php echo 'ok'; ?>\n--EXPECT--\nok\n",
    )
    .expect("write PHPT");

    let manifest = root.join("manifest.txt");
    fs::write(&manifest, "tests/basic/timeout.phpt\n").expect("write manifest");

    let progress = root.join("progress");
    let output = Command::new("timeout")
        .arg("30s")
        .arg("tools/run-phpt-manifest.sh")
        .arg(&manifest)
        .env("PHP_SRC_PHPT", &corpus)
        .env("PHPT_PROGRESS_DIR", &progress)
        .env("PTN_PHPT_AUTO_FETCH", "0")
        .env("PTN_PHPT_TEST_TIMEOUT", "77")
        .env("PHPC_BIN", "/bin/true")
        .output()
        .expect("run PHPT manifest");
    assert!(
        output.status.success(),
        "PHPT manifest failed: status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("seen-timeout: 77"), "{stdout}");
    assert!(stdout.contains("timeout_seconds=77"), "{stdout}");
}

#[test]
fn phpt_classifier_skipif_harness_is_opt_in() {
    let skipif = "--TEST--\nskipif\n--SKIPIF--\n<?php echo getenv('PTN_SKIP') ? 'skip' : ''; ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";

    assert!(classify(skipif).starts_with("runnable\t"));

    let classification = classify_with_harness_programs(skipif, true);
    assert!(
        classification.starts_with("harness-skipif\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_static_skipif_preconditions() {
    let sanitizer = "--TEST--\nsanitizer\n--SKIPIF--\n<?php\nif (getenv('SKIP_ASAN')) die('skip asan');\nif (getenv('SKIP_MSAN')) die('skip msan');\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(sanitizer, true);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("sanitizer-env"),
        "{classification:?}"
    );

    let classification =
        classify_with_harness_programs_and_env(sanitizer, true, &[("SKIP_ASAN", "1")]);
    assert!(
        classification.starts_with("skipif-precondition\t") && classification.contains("SKIP_ASAN"),
        "{classification:?}"
    );

    let int64 = "--TEST--\nint64\n--SKIPIF--\n<?php if (PHP_INT_SIZE != 8) die('skip 64-bit only'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(int64, true, &[("PTN_PHPT_PHP_INT_SIZE", "8")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_INT_SIZE"),
        "{classification:?}"
    );

    let int32 = "--TEST--\nint32\n--SKIPIF--\n<?php if (PHP_INT_SIZE != 4) die('skip 32-bit only'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(int32, true, &[("PTN_PHPT_PHP_INT_SIZE", "8")]);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("PHP_INT_SIZE guard"),
        "{classification:?}"
    );

    let locale = "--TEST--\nlocale\n--SKIPIF--\n<?php\nif (!setlocale(LC_ALL, \"de_DE.UTF-8\", \"fr_FR.UTF-8\")) {\n    die('skip locale needed');\n}\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        locale,
        true,
        &[("PTN_PHPT_AVAILABLE_LOCALES", "C:de_DE.utf8")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("locale-availability"),
        "{classification:?}"
    );

    let classification = classify_with_harness_programs_and_env(
        locale,
        true,
        &[("PTN_PHPT_AVAILABLE_LOCALES", "C:POSIX")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("locale availability guard"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_common_platform_skipif_preconditions() {
    let int_max = "--TEST--\nint max\n--SKIPIF--\n<?php if (PHP_INT_MAX <= 2147483647) die('skip only 64 bit'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(int_max, true, &[("PTN_PHPT_PHP_INT_SIZE", "8")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_INT_MAX"),
        "{classification:?}"
    );
    let classification =
        classify_with_harness_programs_and_env(int_max, true, &[("PTN_PHPT_PHP_INT_SIZE", "4")]);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("PHP_INT_MAX guard"),
        "{classification:?}"
    );

    let non_windows = "--TEST--\nnon windows\n--SKIPIF--\n<?php if (PHP_OS_FAMILY === 'Windows' && version_compare(PHP_VERSION, '8.4', '<')) die('skip windows'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        non_windows,
        true,
        &[("PTN_PHPT_PHP_OS_FAMILY", "Linux")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_OS_FAMILY"),
        "{classification:?}"
    );

    let windows_only = "--TEST--\nwindows only\n--SKIPIF--\n<?php if (PHP_OS_FAMILY !== 'Windows') die('skip Windows only'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        windows_only,
        true,
        &[("PTN_PHPT_PHP_OS_FAMILY", "Linux")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("PHP_OS_FAMILY guard"),
        "{classification:?}"
    );

    let os_prefix = "--TEST--\nos prefix\n--SKIPIF--\n<?php if (substr(PHP_OS, 0, 3) == 'WIN') die('skip Windows'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(os_prefix, true, &[("PTN_PHPT_PHP_OS", "Linux")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_OS-prefix"),
        "{classification:?}"
    );

    let debug = "--TEST--\ndebug\n--SKIPIF--\n<?php if (PHP_DEBUG) die('skip debug build'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(debug, true, &[("PTN_PHPT_PHP_DEBUG", "0")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_DEBUG"),
        "{classification:?}"
    );

    let zts = "--TEST--\nzts\n--SKIPIF--\n<?php if (PHP_ZTS) die('skip zts build'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification =
        classify_with_harness_programs_and_env(zts, true, &[("PTN_PHPT_PHP_ZTS", "0")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("PHP_ZTS"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_common_environment_skipif_preconditions() {
    let slow = "--TEST--\nslow\n--SKIPIF--\n<?php if (getenv('SKIP_SLOW_TESTS')) die('skip slow test'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(slow, true);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("resource-limit gate keeps SKIP_SLOW_TESTS rows"),
        "{classification:?}"
    );
    let classification =
        classify_with_harness_programs_and_env(slow, true, &[("PTN_PHPT_RUN_SLOW_TESTS", "1")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("resource-limit"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(
        slow,
        true,
        &[("PTN_PHPT_RUN_SLOW_TESTS", "1"), ("SKIP_SLOW_TESTS", "1")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("SKIP_SLOW_TESTS unset"),
        "{classification:?}"
    );

    let perf = "--TEST--\nperf\n--SKIPIF--\n<?php if (getenv('SKIP_PERF_SENSITIVE')) die('skip performance sensitive test'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(perf, true);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("resource-limit gate keeps SKIP_PERF_SENSITIVE rows"),
        "{classification:?}"
    );
    let classification =
        classify_with_harness_programs_and_env(perf, true, &[("PTN_PHPT_RUN_PERF_SENSITIVE", "1")]);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("resource-limit"),
        "{classification:?}"
    );

    let resource_heavy = "--TEST--\nresource heavy\n--SKIPIF--\n<?php if (!getenv('RUN_RESOURCE_HEAVY_TESTS')) die('skip resource-heavy test'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(resource_heavy, true);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("RUN_RESOURCE_HEAVY_TESTS set"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(
        resource_heavy,
        true,
        &[("RUN_RESOURCE_HEAVY_TESTS", "1")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("environment"),
        "{classification:?}"
    );

    let assigned = "--TEST--\nassigned env\n--SKIPIF--\n<?php $zend_mm_enabled = getenv('USE_ZEND_ALLOC'); if ($zend_mm_enabled === '0') die('skip Zend MM disabled'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(assigned, true);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("environment"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_root_helper_skipif_preconditions() {
    let non_root_helper = "--TEST--\nroot helper\n--SKIPIF--\n<?php require __DIR__ . '/../skipif_root.inc'; ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        non_root_helper,
        true,
        &[("PTN_PHPT_EFFECTIVE_UID", "1000")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("root-helper"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(
        non_root_helper,
        true,
        &[("PTN_PHPT_EFFECTIVE_UID", "0")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("rejects root"),
        "{classification:?}"
    );

    let root_helper = "--TEST--\nno root helper\n--SKIPIF--\n<?php require __DIR__ . '/../skipif_no_root.inc'; ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        root_helper,
        true,
        &[("PTN_PHPT_EFFECTIVE_UID", "1000")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("requires root"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(
        root_helper,
        true,
        &[("PTN_PHPT_EFFECTIVE_UID", "0")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("root-helper"),
        "{classification:?}"
    );

    let arbitrary_include = "--TEST--\narbitrary include\n--SKIPIF--\n<?php require __DIR__ . '/../custom_skipif.inc'; if (getenv('SKIP_ASAN')) die('skip asan'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(arbitrary_include, true);
    assert!(
        classification.starts_with("harness-skipif\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_constant_and_ci_skipif_preconditions() {
    let glob_brace = "--TEST--\nglob brace\n--SKIPIF--\n<?php if (!defined('GLOB_BRACE')) die('skip this test requires GLOB_BRACE support'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        glob_brace,
        true,
        &[("PTN_PHPT_DEFINED_CONSTANTS", "GLOB_BRACE")],
    );
    assert!(
        classification.starts_with("runnable\t") && classification.contains("constant-defined"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(
        glob_brace,
        true,
        &[("PTN_PHPT_DEFINED_CONSTANTS", "OTHER_CONSTANT")],
    );
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("GLOB_BRACE defined"),
        "{classification:?}"
    );

    let ci = "--TEST--\nci gate\n--SKIPIF--\n<?php if (getenv('CIRRUS_CI')) die('skip Inaccurate on Cirrus'); ?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(ci, true);
    assert!(
        classification.starts_with("runnable\t") && classification.contains("environment"),
        "{classification:?}"
    );
    let classification = classify_with_harness_programs_and_env(ci, true, &[("CIRRUS_CI", "1")]);
    assert!(
        classification.starts_with("skipif-precondition\t")
            && classification.contains("CIRRUS_CI unset"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_inactive_windows_symlink_helper_blocks() {
    let windows_helper = "--TEST--\nwindows helper\n--SKIPIF--\n<?php\nif (PHP_OS_FAMILY === 'Windows') {\n    include __DIR__ . '/windows_links/common.inc';\n    skipIfSeCreateSymbolicLinkPrivilegeIsDisabled(__FILE__);\n}\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs_and_env(
        windows_helper,
        true,
        &[("PTN_PHPT_PHP_OS_FAMILY", "Linux")],
    );
    assert!(
        classification.starts_with("runnable\t")
            && classification.contains("inactive-windows-helper"),
        "{classification:?}"
    );

    let classification = classify_with_harness_programs_and_env(
        windows_helper,
        true,
        &[("PTN_PHPT_PHP_OS_FAMILY", "Windows")],
    );
    assert!(
        classification.starts_with("harness-skipif\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_stream_filter_availability_skipif() {
    let filter_available = "--TEST--\nstream filter available\n--SKIPIF--\n<?php\n$filters = stream_get_filters();\nif (! in_array(\"string.rot13\", $filters)) die(\"skip rot13 filter not available.\");\n?>\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n";
    let classification = classify_with_harness_programs(filter_available, true);
    assert!(
        classification.starts_with("runnable\t")
            && classification.contains("stream-filter-availability"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_models_env_and_cleanup_harness_sections() {
    let env = "--TEST--\nenv\n--ENV--\nPTN_ENV_FROM_PHPT=present\n--FILE--\n<?php echo getenv('PTN_ENV_FROM_PHPT'), \"\\n\"; ?>\n--EXPECT--\npresent\n";
    let env_classification = classify(env);
    assert_eq!(
        env_classification.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let cleanup = "--TEST--\ncleanup\n--FILE--\n<?php echo 1; ?>\n--CLEAN--\n<?php unlink(__DIR__ . '/case.tmp'); ?>\n--EXPECT--\n1\n";
    let classification = classify(cleanup);
    assert_eq!(
        classification.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
}

#[test]
fn phpt_classifier_section_cache_preserves_row_classification() {
    let cleanup = "--TEST--\ncleanup\n--FILE--\n<?php echo 1; ?>\n--CLEAN--\n<?php unlink(__DIR__ . '/case.tmp'); ?>\n--EXPECT--\n1\n";
    assert_eq!(classify(cleanup), classify_with_section_cache(cleanup));

    let attribute =
        "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nfunction f() {}\n--EXPECT--\n";
    assert_eq!(classify(attribute), classify_with_section_cache(attribute));
}

#[test]
fn phpt_classifier_excludes_currently_unsupported_language_surfaces() {
    let cases = [
        (
            "anonymous class generated name",
            "--TEST--\nanon name\n--FILE--\n<?php\n$obj = new class {};\nvar_dump(get_class($obj));\n--EXPECT--\n",
            "unsupported-anonymous-class\t",
            "requires PHP hidden-suffix anonymous class generated names",
        ),
        (
            "eval dynamic code",
            "--TEST--\neval\n--FILE--\n<?php\n$code = 'echo \"x\\n\";';\neval($code);\n--EXPECT--\nx\n",
            "unsupported-dynamic-eval\t",
            "requires eval runtime fallback",
        ),
        (
            "array internal named argument",
            "--TEST--\nnamed internal\n--FILE--\n<?php\nvar_dump(array_map(callback: null, array: []));\n--EXPECT--\n",
            "unsupported-internal-call-binding\t",
            "requires named-argument binding for modeled array internal calls",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with(category),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_user_stream_registration_rows_runnable() {
    let cases = [
        (
            "user stream wrapper",
            "--TEST--\nstream wrapper\n--FILE--\n<?php\nstream_wrapper_register('test', TestWrapper::class);\n--EXPECT--\n",
        ),
        (
            "user stream wrapper alias",
            "--TEST--\nstream alias\n--FILE--\n<?php\nstream_register_wrapper('test', TestWrapper::class, STREAM_IS_URL);\n--EXPECT--\n",
        ),
        (
            "user stream filter",
            "--TEST--\nstream filter\n--FILE--\n<?php\nstream_filter_register('sample.filter', SampleFilter::class);\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_allows_literal_eval_class_declarations() {
    let cases = [
        (
            "plain eval class",
            "--TEST--\neval class\n--FILE--\n<?php\neval('class RuntimeClass {}');\nnew RuntimeClass;\n--EXPECT--\n",
        ),
        (
            "autoload eval class",
            "--TEST--\nautoload eval class\n--FILE--\n<?php\nspl_autoload_register(function ($class) {\n    eval(\"class DefClass{}\");\n});\n$a = new DefClass;\nprint_r($a);\n--EXPECT--\n",
        ),
        (
            "magic warning in eval class",
            "--TEST--\neval magic warning\n--FILE--\n<?php\nset_error_handler(function($_, $msg, $file) {});\neval('class A { private function __invoke() { } }');\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        assert_eq!(
            classify(phpt),
            "runnable\tselected for PTN semantic measurement\n",
            "{name}"
        );
    }
}

#[test]
fn phpt_classifier_allows_dynamic_symbol_runtime_rows() {
    let cases = [
        (
            "variable-variable write",
            "--TEST--\ndynamic write\n--FILE--\n<?php\n$name = 'value';\n$$name = 1;\necho $value;\n--EXPECT--\n1\n",
        ),
        (
            "braced variable-variable write",
            "--TEST--\ndynamic write\n--FILE--\n<?php\n$name = 'value';\n${$name} = 1;\necho $value;\n--EXPECT--\n1\n",
        ),
        (
            "dynamic global",
            "--TEST--\ndynamic global\n--FILE--\n<?php\n$GLOBALS['value'] = 1;\nfunction f() { $name = 'value'; global $$name; $value = 2; }\nf();\necho $GLOBALS['value'];\n--EXPECT--\n2\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_simple_trait_composition_runnable() {
    let row = "--TEST--\ntrait\n--FILE--\n<?php\ntrait SharedBehavior { public function run() { echo \"ok\\n\"; } }\nclass Worker { use SharedBehavior; }\n(new Worker())->run();\n--EXPECT--\nok\n";
    assert_eq!(
        classify(row).trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
    assert_eq!(classify(row), classify_with_section_cache(row));
}

#[test]
fn phpt_classifier_keeps_nullable_never_and_static_rows_runnable() {
    let cases = [
        "--TEST--\nnullable\n--FILE--\n<?php\n$fn = fn(?int... $args): array => $args;\n--EXPECT--\n",
        "--TEST--\nnever\n--FILE--\n<?php\n$fn = fn(): never => throw new Exception('done');\n--EXPECT--\n",
        "--TEST--\ntop-level static\n--FILE--\n<?php\nstatic $value;\nvar_dump($value);\n--EXPECT--\nNULL\n",
        "--TEST--\nstatic local\n--FILE--\n<?php\nfunction next_value() { static $value = 0; return ++$value; }\n--EXPECT--\n",
        "--TEST--\ntop-level static\n--FILE--\n<?php\ntry { static $value; } catch (Throwable $e) {}\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_tracks_nested_function_blocks_for_static_locals() {
    let phpt = "--TEST--\nstatic after nested block\n--FILE--\n<?php\nfunction f($flag) {\n    if ($flag) {\n        echo \"flag\\n\";\n    }\n    static $value = 1;\n    echo $value, \"\\n\";\n}\nf(false);\n--EXPECT--\n1\n";

    assert_eq!(
        classify(phpt).trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
    assert_eq!(classify(phpt), classify_with_section_cache(phpt));
}

#[test]
fn phpt_classifier_keeps_supported_foreach_diagnostics_runnable() {
    let cases = [
        "--TEST--\nappend read\n--FILE--\n<?php\nforeach ($items[] as $value) {}\n--EXPECTF--\n",
        "--TEST--\nthis target\n--FILE--\n<?php\nforeach ($items as list($this)) {}\n--EXPECTF--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_keeps_supported_anonymous_class_rows_runnable() {
    let cases = [
        "--TEST--\nanon\n--FILE--\n<?php\nvar_dump(new class {});\n--EXPECT--\n",
        "--TEST--\nanon contract\n--FILE--\n<?php\ninterface Contract { public function run(); }\nclass Base { public function label($value) { return $value; } }\n$obj = new class extends Base implements Contract { public function run() { return $this->label(1); } };\nvar_dump($obj instanceof Base, $obj instanceof Contract);\n--EXPECT--\n",
        "--TEST--\noverride\n--FILE--\n<?php\ninterface Contract { public function run(); }\n$obj = new class implements Contract { #[\\Override] public function run() {} };\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_anonymous_metadata_rows_runnable_by_path() {
    let cases = [
        (
            "Zend/tests/anon/013.phpt",
            "--TEST--\nanonymous closure bind\n--FILE--\n<?php\n$class = new class {};\n$foo = function() { return $this; };\n$closure = Closure::bind($foo, $class, $class);\nvar_dump($closure());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/anon/011.phpt",
            "--TEST--\nanonymous class alias\n--FILE--\n<?php\nclass_alias(get_class(new class { protected $foo = 1; }), \"AnonBase\");\nvar_dump((new class extends AnonBase { function getFoo() { return $this->foo; } })->getFoo());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/anon/gh13097_a.phpt",
            "--TEST--\nanonymous trigger_error name\n--FILE--\n<?php\n$anonymous = new class(){};\ntrigger_error(get_class($anonymous).' ...now you don\\'t!', E_USER_ERROR);\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/anon/gh13097_b.phpt",
            "--TEST--\nanonymous exception name\n--FILE--\n<?php\n$anonymous = new class(){};\nthrow new Exception(get_class($anonymous));\n--EXPECTF--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_dynamic_variable_reads_runnable() {
    let cases = [
        "--TEST--\ndynamic read\n--FILE--\n<?php\n$name = 'value';\necho $$name;\n--EXPECT--\n",
        "--TEST--\ndynamic unset\n--FILE--\n<?php\n$name = 'value';\nunset($$name);\n--EXPECT--\n",
    ];
    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_keeps_supported_interface_rows_runnable() {
    let classification = classify(
        "--TEST--\ninterface\n--FILE--\n<?php\ninterface Contract { public function run(): mixed; }\nclass Bag implements Contract { public function run(): mixed { return 1; } }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_splits_unpacking_blockers() {
    let call_unpack = classify(
        "--TEST--\ncall unpack\n--FILE--\n<?php\nfunction collect(...$args) { return $args; }\nvar_dump(collect(...[1, 2]));\n--EXPECT--\n",
    );
    assert_eq!(
        call_unpack.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    for phpt in [
        "--TEST--\narray unpack\n--FILE--\n<?php\nvar_dump([0, ...[1, 2]]);\n--EXPECT--\n",
        "--TEST--\narray unpack\n--FILE--\n<?php\nvar_dump(array(0, ...[1, 2]));\n--EXPECT--\n",
    ] {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
    }

    let by_ref_unpack = classify(
        "--TEST--\nby-ref call unpack\n--FILE--\n<?php\nfunction inc(&$value) { $value++; }\n$items = [1];\ninc(...$items);\n--EXPECT--\n",
    );
    assert_eq!(
        by_ref_unpack.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let spl_iterator_unpack = classify(
        "--TEST--\nSPL iterator unpack\n--FILE--\n<?php\nvar_dump(...new ArrayIterator([1, 2]));\n--EXPECT--\n",
    );
    assert_eq!(
        spl_iterator_unpack.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
}

#[test]
fn phpt_classifier_splits_attribute_metadata_blockers() {
    let cases = [
        (
            "attribute syntax on class",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nclass Bag {}\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "attribute syntax on function",
            "--TEST--\nattribute\n--FILE--\n<?php\n#[Example]\nfunction f() {}\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(Attribute::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "broader internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(DateTime::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata beyond modeled Attribute self-reflection",
        ),
        (
            "internal Deprecated attribute object",
            "--TEST--\ndeprecated attribute\n--FILE--\n<?php\n$d = new \\Deprecated(\"message\");\n$d->message = \"updated\";\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires direct Deprecated/NoDiscard fatal stack parity beyond modeled caught-object behavior",
        ),
        (
            "override typed property validation",
            "--TEST--\noverride property\n--FILE--\n<?php\nclass P { public mixed $p; }\nclass C extends P { #[\\Override] public mixed $p; }\necho \"Done\";\n--EXPECT--\nDone\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "override promoted property validation",
            "--TEST--\noverride promoted property\n--FILE--\n<?php\ninterface I { public mixed $p { get; } }\nclass C implements I { public function __construct(#[\\Override] public mixed $p) {} }\necho \"Done\";\n--EXPECT--\nDone\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with(category),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_attribute_metadata_rows_runnable_by_path() {
    let cases = [
        (
            "Zend/tests/attributes/002_rfcexample.phpt",
            "--TEST--\nattribute class metadata\n--FILE--\n<?php\n#[SingleArgument(\"Hello World\")]\nclass Foo {}\n$attributes = (new ReflectionClass(Foo::class))->getAttributes();\nvar_dump($attributes[0]->getName(), $attributes[0]->getArguments());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/003_ast_nodes.phpt",
            "--TEST--\nattribute ast nodes\n--FILE--\n<?php\n#[A1(1 + 2)]\nclass C {}\nvar_dump((new ReflectionClass(C::class))->getAttributes()[0]->getArguments());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/004_name_resolution.phpt",
            "--TEST--\nattribute function metadata\n--FILE--\n<?php\nnamespace Foo { #[Entity(\"imported\")] function foo() {} }\nnamespace { var_dump((new ReflectionFunction('Foo\\foo'))->getAttributes()); }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/009_doctrine_annotations_example.phpt",
            "--TEST--\ndoctrine attributes\n--FILE--\n<?php\nnamespace Demo { class Entity {} }\nnamespace { #[Demo\\Entity] class User {} var_dump((new ReflectionClass(User::class))->getAttributes()[0]->getName()); }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/011_inheritance.phpt",
            "--TEST--\nattribute inheritance\n--FILE--\n<?php\n#[A] class P {}\nclass C extends P {}\nvar_dump((new ReflectionClass(C::class))->getAttributes());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/015_property_group.phpt",
            "--TEST--\nattribute property metadata\n--FILE--\n<?php\nclass C { #[A(1)] public $x, $y; }\n$rp = new ReflectionProperty('C', 'x');\nvar_dump($rp->getAttributes()[0]->getName());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/027_trailing_comma_args.phpt",
            "--TEST--\nattribute trailing comma\n--FILE--\n<?php\n#[MyAttribute(\"there\",)]\nclass Foo {}\nvar_dump((new ReflectionClass(Foo::class))->getAttributes()[0]->getArguments());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/028_grouped.phpt",
            "--TEST--\nattribute grouped\n--FILE--\n<?php\n#[A1(1), A2(2)]\nfunction foo() {}\nvar_dump((new ReflectionFunction('foo'))->getAttributes());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/029_reflect_internal_symbols.phpt",
            "--TEST--\ninternal symbol attributes\n--FILE--\n<?php\n$rp = new ReflectionProperty('Exception', 'message');\nvar_dump($rp->getAttributes());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/021_attribute_flags_type_is_validated.phpt",
            "--TEST--\nattribute flags type\n--FILE--\n<?php\n#[Attribute('bad')]\nclass BadFlags {}\n#[BadFlags]\nclass Subject {}\nvar_dump((new ReflectionClass(Subject::class))->getAttributes()[0]->newInstance());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/022_attribute_flags_value_is_validated.phpt",
            "--TEST--\nattribute flags value\n--FILE--\n<?php\n#[Attribute(-1)]\nclass BadFlags {}\n#[BadFlags]\nclass Subject {}\nvar_dump((new ReflectionClass(Subject::class))->getAttributes()[0]->newInstance());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/023_ast_node_in_validation.phpt",
            "--TEST--\nattribute ast node validation\n--FILE--\n<?php\nclass Foo { const BAR = Attribute::TARGET_CLASS; }\n#[Attribute(Foo::BAR)]\nclass Attr {}\n#[Attr]\nclass Subject {}\nvar_dump((new ReflectionClass(Subject::class))->getAttributes()[0]->newInstance());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/validator_success.phpt",
            "--TEST--\ndelayed target validation\n--FILE--\n<?php\n#[DelayedTargetValidation]\n#[AllowDynamicProperties]\nclass Bag {}\nvar_dump((new ReflectionClass(Bag::class))->getAttributes()[1]->newInstance());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/no_compile_errors.phpt",
            "--TEST--\ndelayed target validation no compile errors\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[Attribute] get => $this->hooked; #[DelayedTargetValidation] #[Attribute] set => $value; } }\necho \"ok\\n\";\n--EXPECT--\nok\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_AllowDynamicProperties.phpt",
            "--TEST--\ndelayed allow dynamic properties\n--FILE--\n<?php\n#[DelayedTargetValidation]\n#[AllowDynamicProperties]\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[AllowDynamicProperties] get => $this->hooked; set => $value; } }\necho \"ok\\n\";\n--EXPECT--\nok\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_NoDiscard.phpt",
            "--TEST--\ndelayed no discard\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[NoDiscard] get => $this->hooked; set => $value; } #[DelayedTargetValidation] #[NoDiscard] public function run() { return 1; } }\n(new Demo())->run();\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_SensitiveParameter.phpt",
            "--TEST--\ndelayed sensitive parameter\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[SensitiveParameter] get => $this->hooked; set => $value; } public function run(#[DelayedTargetValidation] #[SensitiveParameter] $secret) { throw new Exception('boom'); } }\n(new Demo())->run('hidden');\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/ossfuzz371445205.phpt",
            "--TEST--\nunknown named attribute parameter\n--FILE--\n<?php\n#[Attribute]\nclass MyAttrib {}\n#[MyAttrib(notinterned: '')]\nclass Test1 {}\n(new ReflectionClass(Test1::class))->getAttributes()[0]->newInstance();\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/ReflectionAttribute_constructor_001.phpt",
            "--TEST--\nreflection attribute constructor\n--FILE--\n<?php\n#[Attribute]\nclass A {}\nclass Foo { #[A] public function bar() {} }\n$attribute = (new ReflectionMethod(Foo::class, 'bar'))->getAttributes()[0];\n(new ReflectionMethod($attribute, '__construct'))->invoke($attribute);\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/ReflectionAttribute_newInstance_deprecated.phpt",
            "--TEST--\nreflection attribute deprecated\n--FILE--\n<?php\n#[Deprecated(since: '2.0')]\nfunction old_api() {}\nvar_dump((new ReflectionFunction('old_api'))->getAttributes()[0]->newInstance());\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/ReflectionAttribute_newInstance_exception.phpt",
            "--TEST--\nreflection attribute new instance exception\n--FILE--\n<?php\n#[Attribute]\nclass A { public function __construct() { throw new Exception('boom'); } }\n#[A]\nclass C {}\n(new ReflectionClass(C::class))->getAttributes()[0]->newInstance();\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/032_attribute_validation_scope.phpt",
            "--TEST--\nattribute validation scope\n--FILE--\n<?php\n#[Attribute(parent::x)]\nclass x extends y {}\nclass y { protected const x = Attribute::TARGET_CLASS; }\n#[x]\nclass z {}\nvar_dump((new ReflectionClass(z::class))->getAttributes()[0]->newInstance());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/constants/constant_redefined_addition.phpt",
            "--TEST--\nconstant attribute addition\n--FILE--\n<?php\nconst MY_CONST = \"No attributes\";\n#[\\MyAttribute]\nconst MY_CONST = \"Has attributes\";\nvar_dump((new ReflectionConstant('MY_CONST'))->getAttributes());\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/constants/constant_listed_as_target-userland.phpt",
            "--TEST--\nconstant target validation\n--FILE--\n<?php\n#[Attribute(Attribute::TARGET_CONSTANT)] class MyConstantAttribute {}\n#[MyConstantAttribute] class Example {}\n(new ReflectionClass(Example::class))->getAttributes()[0]->newInstance();\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/constants/must_target_const-userland.phpt",
            "--TEST--\nconstant target validation\n--FILE--\n<?php\n#[Attribute(Attribute::TARGET_FUNCTION)] class MyFunctionAttribute {}\n#[MyFunctionAttribute] const EXAMPLE = 'Foo';\n(new ReflectionConstant('EXAMPLE'))->getAttributes()[0]->newInstance();\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/constants/not_repeatable-userland.phpt",
            "--TEST--\nconstant repetition validation\n--FILE--\n<?php\n#[Attribute] class MyAttribute {}\n#[MyAttribute]\n#[MyAttribute]\nconst MY_CONST = true;\n(new ReflectionConstant('MY_CONST'))->getAttributes()[0]->newInstance();\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/deprecated/property_readonly_001.phpt",
            "--TEST--\ndeprecated readonly\n--FILE--\n<?php\n$d = new \\Deprecated(\"foo\");\n$d->message = 'bar';\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/nodiscard/property_readonly_001.phpt",
            "--TEST--\nnodiscard readonly\n--FILE--\n<?php\n$d = new \\NoDiscard(\"foo\");\n$d->message = 'bar';\n--EXPECTF--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_user_named_arguments_runnable() {
    let classification = classify(
        "--TEST--\nuser named arguments\n--FILE--\n<?php\nfunction pick($left, $right) { return $right; }\necho pick(right: 2, left: 1);\n--EXPECT--\n2\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_array_literals_in_internal_calls_runnable() {
    let classification = classify(
        "--TEST--\narray literal\n--FILE--\n<?php\nvar_dump(array_map(null, [\"name\" => 1]));\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_does_not_treat_static_member_syntax_as_named_internal_argument() {
    let classification = classify(
        "--TEST--\nstatic member in internal call\n--FILE--\n<?php\nclass Bag { public static function values() { return [1]; } }\narray_pop((Bag::values()));\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_allows_first_class_callable_syntax() {
    let cases = [
        (
            "function callable",
            "--TEST--\nfcc function\n--FILE--\n<?php\n$fn = strlen(...);\necho $fn('abc');\n--EXPECT--\n3\n",
        ),
        (
            "static method callable",
            "--TEST--\nfcc static\n--FILE--\n<?php\nclass FccStatic { public static function run($v) { return $v; } }\n$fn = FccStatic::run(...);\necho $fn('ok');\n--EXPECT--\nok\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_plain_heredoc_and_nowdoc_runnable() {
    let cases = [
        (
            "plain heredoc",
            "--TEST--\nheredoc\n--FILE--\n<?php\n$value = <<<TXT\nHello\nTXT;\nvar_dump($value);\n--EXPECT--\nstring(5) \"Hello\"\n",
        ),
        (
            "plain nowdoc",
            "--TEST--\nnowdoc\n--FILE--\n<?php\n$value = <<<'TXT'\n$literal\nTXT;\nvar_dump($value);\n--EXPECT--\nstring(8) \"$literal\"\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_interpolating_heredoc_bodies_runnable() {
    let classification = classify(
        "--TEST--\nheredoc interpolation\n--FILE--\n<?php\n$name = \"world\";\necho <<<TXT\nHello $name\nTXT;\n--EXPECT--\nHello world\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_generator_fiber_reference_boundaries() {
    let cases = [
        (
            "fiber by-ref return",
            "--TEST--\nfiber\n--FILE--\n<?php\n$fiber = new Fiber(function &() {\n    Fiber::suspend();\n    return $var;\n});\n--EXPECT--\n",
            "requires Fiber coroutine runtime and by-reference return/getReturn boundary",
        ),
        (
            "by-ref generator yield from",
            "--TEST--\nyield from by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield from [];\n}\n--EXPECTF--\n",
            "requires generator yield-from by-reference rejection",
        ),
        (
            "generator foreach cleanup",
            "--TEST--\ngenerator foreach cleanup\n--FILE--\n<?php\nfunction gen(array $array) {\n    foreach ($array as $value) {\n        yield $value;\n    }\n}\n--EXPECT--\n",
            "requires generator suspension cleanup for live foreach variables and premature close",
        ),
        (
            "by-ref yielded assignment expression",
            "--TEST--\nyield assignment by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield $v = 0;\n}\n--EXPECTF--\n",
            "requires generator suspension timing for by-reference yielded assignment expressions",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-generator-runtime\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_allows_supported_fiber_constructor_and_current_surface() {
    let cases = [
        (
            "fiber constructor only",
            "--TEST--\nfiber constructor\n--FILE--\n<?php\n$fiber = new Fiber(fn() => null);\necho \"done\";\n--EXPECT--\ndone\n",
        ),
        (
            "fiber get current",
            "--TEST--\nfiber get current\n--FILE--\n<?php\nvar_dump(Fiber::getCurrent());\n$fiber = new Fiber(function (): void {\n    var_dump(Fiber::getCurrent());\n});\n$fiber->start();\n--EXPECTF--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_allows_collected_generator_runtime_subset() {
    let cases = [
        (
            "simple generator yield",
            "--TEST--\nyield\n--FILE--\n<?php\nfunction gen() { yield 123; }\nforeach (gen() as $value) { var_dump($value); }\n--EXPECT--\n",
        ),
        (
            "simple generator yield from",
            "--TEST--\nyield from\n--FILE--\n<?php\nfunction gen() { yield from [123]; }\nforeach (gen() as $value) { var_dump($value); }\n--EXPECT--\n",
        ),
        (
            "non-ref generator iterated by-ref",
            "--TEST--\ngenerator foreach by ref\n--FILE--\n<?php\nfunction gen() { yield; }\n$gen = gen();\nforeach ($gen as &$value) {}\n--EXPECTF--\n",
        ),
        (
            "by-ref generator yielding expression",
            "--TEST--\nyield const by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield \"foo\";\n}\n$gen = gen();\nvar_dump($gen->current());\n--EXPECTF--\n",
        ),
        (
            "generator call unpack from foreach body",
            "--TEST--\ngenerator call unpack\n--FILE--\n<?php\nfunction test($val1, &$ref) {}\nfunction gen($array) {\n    foreach ($array as $element) {\n        yield $element;\n    }\n}\ntest(...gen([1, 2]));\n--EXPECTF--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_allows_supported_generator_reference_frontier_rows() {
    let cases = [
        (
            "Zend/tests/generators/gc_with_iterator_in_foreach.phpt",
            "--TEST--\ngenerator gc foreach\n--FILE--\n<?php\nfunction gen($iter, &$gen) {\n    foreach ($iter as $v) {\n        yield;\n    }\n}\n$iter = new ArrayIterator([1, 2, 3]);\n$gen = gen($iter, $gen);\n$gen->next();\nunset($gen);\ngc_collect_cycles();\n--EXPECT--\n",
        ),
        (
            "Zend/tests/generators/no_foreach_var_leaks.phpt",
            "--TEST--\ngenerator foreach close\n--FILE--\n<?php\nfunction gen(array $array) {\n    foreach ($array as $value) {\n        yield $value;\n    }\n}\n$gen = gen(['Foo', 'Bar']);\nvar_dump($gen->current());\n--EXPECT--\nstring(3) \"Foo\"\n",
        ),
        (
            "Zend/tests/generators/yield_by_reference.phpt",
            "--TEST--\nyield by ref\n--FILE--\n<?php\nfunction &iter(array &$array) {\n    foreach ($array as $key => &$value) {\n        yield $key => $value;\n    }\n}\n$array = [1, 2, 3];\n$iter = iter($array);\nforeach ($iter as &$value) {\n    $value *= -1;\n}\n--EXPECT--\n",
        ),
        (
            "Zend/tests/generators/yield_by_reference_optimization.phpt",
            "--TEST--\nyield by ref assignment\n--FILE--\n<?php\nfunction &gen() {\n    yield $v = 0;\n    yield $v = 1;\n}\nforeach (gen() as $v) {\n    var_dump($v);\n}\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/generators/yield_from_by_reference.phpt",
            "--TEST--\nyield from by ref\n--FILE--\n<?php\nfunction &gen() {\n    yield from [];\n}\n--EXPECTF--\n",
        ),
    ];

    for (relative_path, phpt) in cases {
        let classification = classify_at_relative_path(phpt, relative_path);
        assert!(
            classification.starts_with("runnable\t"),
            "{relative_path}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_return_from_by_ref_generator_runnable() {
    let classification = classify_at_relative_path(
        "--TEST--\nreturn from by ref generator\n--FILE--\n<?php\nfunction &gen() {\n    yield;\n    $arr = [42];\n    return $arr[0];\n}\nfunction gen2() {\n    var_dump(yield from gen());\n}\ngen2()->next();\n--EXPECT--\nint(42)\n",
        "Zend/tests/generators/return_from_by_ref_generator.phpt",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_asymmetric_property_visibility_rows_runnable() {
    let classification = classify(
        "--TEST--\nasymmetric visibility\n--FILE--\n<?php\nclass Bag { public private(set) int $value; }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_simple_typed_property_rows_runnable() {
    let classification = classify(
        "--TEST--\ntyped property\n--FILE--\n<?php\nclass Bag { public int $value; }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_readonly_property_rows_runnable() {
    let cases = [
        "--TEST--\nreadonly property\n--FILE--\n<?php\nclass Bag { public readonly int $value; }\n--EXPECT--\n",
        "--TEST--\nreadonly class\n--FILE--\n<?php\nreadonly class Bag { public int $value; }\n--EXPECT--\n",
    ];

    for phpt in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_asymmetric_property_hook_rows_runnable_by_path() {
    let cases = [
        (
            "Zend/tests/asymmetric_visibility/gh19044.phpt",
            "--TEST--\nasymmetric protected set\n--FILE--\n<?php\nclass ParentBox { public protected(set) string $author = \"base\"; }\nclass ChildBox extends ParentBox { public protected(set) string $author = \"child\"; }\nvar_dump(new ChildBox());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/asymmetric_visibility/virtual_get_only.phpt",
            "--TEST--\nget-only virtual property\n--FILE--\n<?php\nclass Bad { public private(set) string $name { get { return 'bad'; } } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/asymmetric_visibility/virtual_set_only.phpt",
            "--TEST--\nset-only virtual property\n--FILE--\n<?php\nclass Bad { public private(set) string $name { set { } } }\n--EXPECTF--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path),
            "runnable\tselected for PTN semantic measurement\n",
            "{path}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_property_hook_contract_rows_runnable_by_path() {
    let cases = [
        (
            "Zend/tests/property_hooks/syntax.phpt",
            "--TEST--\nsyntax\n--FILE--\n<?php\nclass Test { public $prop { get { } set { } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/abstract_hook.phpt",
            "--TEST--\nabstract hook\n--FILE--\n<?php\nabstract class A { public abstract $prop { get; set {} } }\nclass B extends A { public $prop { get {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/abstract_prop_hooks.phpt",
            "--TEST--\nabstract prop hooks\n--FILE--\n<?php\nabstract class A { abstract public $prop { get; set; } }\nclass B extends A { public $prop { get {} set {} } }\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/property_hooks/ReflectionClass_getMethods.phpt",
            "--TEST--\nreflection class methods\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } }\nvar_dump((new ReflectionClass(Test::class))->getMethods());\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/ReflectionClass_getProperties_003.phpt",
            "--TEST--\nreflection class properties\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } }\nvar_dump((new ReflectionClass(Test::class))->getProperties());\n--EXPECT--\n",
        ),
        (
            "ext/reflection/tests/ReflectionClass_isIterable_gh20217.phpt",
            "--TEST--\nreflection class is iterable\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } }\nvar_dump((new ReflectionClass(Test::class))->isIterable());\n--EXPECT--\n",
        ),
        (
            "Zend/tests/closures/closure_049.phpt",
            "--TEST--\nclosure with hook metadata\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } public function run() { return static function () { return static::class; }; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/closures/closure_051.phpt",
            "--TEST--\nclosure with hook metadata\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } public static function run() { return static function () { return static::class; }; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/closures/closure_053.phpt",
            "--TEST--\nclosure with hook metadata\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } public function run() { return static function () { return self::class; }; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/closures/closure_055.phpt",
            "--TEST--\nclosure with hook metadata\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } public static function run() { return static function () { return self::class; }; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/closures/closure_062.phpt",
            "--TEST--\nclosure with hook metadata\n--FILE--\n<?php\nclass Test { public $a { get {} set {} } public function run() { return function () { return $this; }; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_Override_okay.phpt",
            "--TEST--\ndelayed override hook okay\n--FILE--\n<?php\nclass Base { public string $hooked { get => $this->hooked; set => $value; } }\nclass Demo extends Base { public string $hooked { #[DelayedTargetValidation] #[Override] get => $this->hooked; #[DelayedTargetValidation] #[Override] set => $value; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_Override_error_get.phpt",
            "--TEST--\ndelayed override hook get error\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[Override] get => $this->hooked; set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_Override_error_set.phpt",
            "--TEST--\ndelayed override hook set error\n--FILE--\n<?php\nclass Demo { public string $hooked { get => $this->hooked; #[DelayedTargetValidation] #[Override] set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/has_runtime_errors.phpt",
            "--TEST--\ndelayed hook runtime errors\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[Attribute] get => $this->hooked; #[DelayedTargetValidation] #[Attribute] set => $value; } }\n$cases = [new ReflectionProperty('Demo', 'hooked')->getHook(PropertyHookType::Get)];\nforeach ($cases as $r) { var_dump($r->getAttributes()); }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/validator_NoDiscard.phpt",
            "--TEST--\ndelayed nodiscard validator\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[NoDiscard] get => $this->hooked; set => $value; } }\n$hook = new ReflectionProperty('Demo', 'hooked')->getHook(PropertyHookType::Get);\nvar_dump($hook->getAttributes());\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_Attribute.phpt",
            "--TEST--\ndelayed attribute hook\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[Attribute] get => $this->hooked; set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_Deprecated.phpt",
            "--TEST--\ndelayed deprecated hook\n--FILE--\n<?php\nclass Demo { public string $hooked { #[DelayedTargetValidation] #[Deprecated] get => $this->hooked; set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/delayed_target_validation/with_ReturnTypeWillChange.phpt",
            "--TEST--\ndelayed return type will change hook\n--FILE--\n<?php\nclass Demo implements Countable { public string $hooked { #[DelayedTargetValidation] #[ReturnTypeWillChange] get => $this->hooked; set => $value; } #[DelayedTargetValidation] #[ReturnTypeWillChange] public function count() { return 0; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/nodiscard/unsupported_property_hook_get.phpt",
            "--TEST--\nnodiscard hook get\n--FILE--\n<?php\nclass Demo { public string $hooked { #[NoDiscard] get => $this->hooked; set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/attributes/nodiscard/unsupported_property_hook_set.phpt",
            "--TEST--\nnodiscard hook set\n--FILE--\n<?php\nclass Demo { public string $hooked { get => $this->hooked; #[NoDiscard] set => $value; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/get.phpt",
            "--TEST--\nplain get hook\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } }\nvar_dump((new Test())->prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/get_type_check.phpt",
            "--TEST--\nget hook return type check\n--FILE--\n<?php\nclass Test { public int $prop { get { return '42'; } } }\nvar_dump((new Test())->prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/get_by_ref_virtual.phpt",
            "--TEST--\nby-ref get hook\n--FILE--\n<?php\nclass Test { private $_prop; public $prop { &get => $this->_prop; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/get_by_ref_implemented_by_val.phpt",
            "--TEST--\nby-ref get contract\n--FILE--\n<?php\ninterface I { public $prop { &get; } } class A implements I { public $prop { get => $this->prop; } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/interface_get_value_as_ref.phpt",
            "--TEST--\nby-value contract by-ref implementation\n--FILE--\n<?php\ninterface I { public $prop { get; } } class A implements I { private $_prop; public $prop { &get => $this->_prop; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/set.phpt",
            "--TEST--\nplain set hook\n--FILE--\n<?php\nclass Test { public $_prop; public $prop { set { $this->_prop = $value; } } }\n$test = new Test(); $test->prop = 42; var_dump($test->_prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/explicit_set_value_parameter.phpt",
            "--TEST--\nexplicit set parameter\n--FILE--\n<?php\nclass Test { public $prop { set($customName) { var_dump($customName); } } }\n(new Test())->prop = 42;\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/backed_implicit_get.phpt",
            "--TEST--\nbacked implicit get\n--FILE--\n<?php\nclass Test { public $prop { set { $this->prop = $value; } } }\n$test = new Test(); $test->prop = 42; var_dump($test->prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/backed_implicit_set.phpt",
            "--TEST--\nbacked implicit set\n--FILE--\n<?php\nclass Test { public $prop { get { return $this->prop; } } }\n$test = new Test(); $test->prop = 42; var_dump($test->prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/default_on_hooks.phpt",
            "--TEST--\ndefault on backed hooks\n--FILE--\n<?php\nclass Test { public $prop = 42 { get { return $this->prop; } set { $this->prop = $value; } } }\nvar_dump((new Test())->prop);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/set_shorthand.phpt",
            "--TEST--\nset shorthand\n--FILE--\n<?php\nclass Test { public string $prop { set => strtoupper($value); } }\n$test = new Test(); $test->prop = 'foo'; var_dump($test);\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/cpp.phpt",
            "--TEST--\nconstructor property promotion\n--FILE--\n<?php\nclass Test { public function __construct(public $prop = 42 { get => print('Getting'); set { print('Setting'); } }) {} }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh15438_1.phpt",
            "--TEST--\npromoted hook no visibility\n--FILE--\n<?php\nclass C { public function __construct($prop { set => $value * 2; }) {} }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh15438_2.phpt",
            "--TEST--\npromoted hook null default\n--FILE--\n<?php\nclass C { public function __construct(public $prop { set => $value * 2; }) {} }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/final_prop_promoted_1.phpt",
            "--TEST--\nfinal promoted hook\n--FILE--\n<?php\nclass C { public function __construct(public final $prop { get => $this->prop; }) {} }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/property_promotion.phpt",
            "--TEST--\ngenerated hooks in property promotion\n--FILE--\n<?php\nclass C { public function __construct(public $prop { get { return $this->prop; } set { $this->prop = $value; } }) {} }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/bug005.phpt",
            "--TEST--\nparent hook parse context\n--FILE--\n<?php\nclass B { protected mixed $x; }\nclass C extends B { protected mixed $x { set { parent::$x::set(1); } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/parent_get.phpt",
            "--TEST--\nparent hook get\n--FILE--\n<?php\nclass B { public mixed $x { get => 42; } }\nclass C extends B { public mixed $x { get => parent::$x::get(); } }\nvar_dump((new C())->x);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/parent_get_ci.phpt",
            "--TEST--\nparent hook get case-insensitive\n--FILE--\n<?php\nclass B { public mixed $x { get => 42; } }\nclass C extends B { public mixed $x { get => PARENT::$x::GET(); } }\nvar_dump((new C())->x);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/parent_set_plain_zpp.phpt",
            "--TEST--\nparent hook set\n--FILE--\n<?php\nclass B { public mixed $x { set { $this->x = $value; } } }\nclass C extends B { public mixed $x { set { parent::$x::set($value); } } }\n$c = new C(); $c->x = 42; var_dump($c->x);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/parent_superfluous_args.phpt",
            "--TEST--\nparent hook extra args\n--FILE--\n<?php\nclass B { public mixed $x { get => 42; } }\nclass C extends B { public mixed $x { get => parent::$x::get(1); } }\nvar_dump((new C())->x);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/parent_syntax.phpt",
            "--TEST--\nparent hook syntax\n--FILE--\n<?php\nclass B { public mixed $x { get => 42; } }\nclass C extends B { public mixed $x { get => parent::$x::get(); } }\nvar_dump((new C())->x);\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/explicit_iter.phpt",
            "--TEST--\nexplicit iterator with hooks\n--FILE--\n<?php\nclass Test implements IteratorAggregate { public $prop { get { return 42; } } public function getIterator(): Traversable { yield 1; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/explicit_set_value_parameter_type.phpt",
            "--TEST--\nexplicit typed set parameter\n--FILE--\n<?php\nclass Test { public $prop { set(int $value) { var_dump($value); } } }\n(new Test())->prop = 42;\n--EXPECT--\nint(42)\n",
        ),
        (
            "Zend/tests/property_hooks/set_value_parameter_type_variance_001.phpt",
            "--TEST--\nset parameter variance\n--FILE--\n<?php\nclass Test { public $prop { set(string $prop) {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/set_value_parameter_type_variance_002.phpt",
            "--TEST--\nset parameter variance\n--FILE--\n<?php\nclass Test { public string|array $prop { set(string $prop) {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/set_value_parameter_type_variance_003.phpt",
            "--TEST--\nset parameter variance\n--FILE--\n<?php\ninterface X {} interface Y {} class Test { public X $prop { set(Y $prop) {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/set_value_parameter_type_variance_005.phpt",
            "--TEST--\nset parameter variance\n--FILE--\n<?php\nclass Test { public string $prop { set($prop) {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/set_value_parameter_type_variance_007.phpt",
            "--TEST--\nset parameter variance\n--FILE--\n<?php\ninterface X {} interface Y extends X {} class A { public Y $prop { set(X $prop) {} } } class B extends A { public Y $prop { set(Y $prop) {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/type_compatibility.phpt",
            "--TEST--\nhook type variance\n--FILE--\n<?php\nclass A { public int|float $a { get { return 42.0; } } public int $b { set {} } } class B extends A { public int $a { get { return 42; } } public int|float $b { set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/type_compatibility_invalid.phpt",
            "--TEST--\ninvalid get variance\n--FILE--\n<?php\nclass A { public int $a { get {} } } class B extends A { public int|float $a { get {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/type_compatibility_invalid_2.phpt",
            "--TEST--\ninvalid set variance\n--FILE--\n<?php\nclass A { public int|float $a { set {} } } class B extends A { public int $a { set {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/override_add_get_contravariant.phpt",
            "--TEST--\nadd get to set-only\n--FILE--\n<?php\nclass A { public int $prop { set {} } } class B extends A { public int|string $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_add_set_covariant.phpt",
            "--TEST--\nadd set to get-only\n--FILE--\n<?php\nclass A { public int|string $prop { get { return 42; } } } class B extends A { public int $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/traits_conflict.phpt",
            "--TEST--\ntrait hook conflict\n--FILE--\n<?php\ntrait T { public $prop { get {} } } class C { use T; public $prop { set {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/final.phpt",
            "--TEST--\nfinal hook override\n--FILE--\n<?php\nclass A { public $prop { final get { return 42; } } } class B extends A { public $prop { get { return 24; } } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/inheritance.phpt",
            "--TEST--\nproperty hook inheritance\n--FILE--\n<?php\nclass A { public $prop { get { return 42; } } } class B extends A { public $prop { get { return parent::$prop::get(); } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/invalid_abstract.phpt",
            "--TEST--\nabstract hook implementation\n--FILE--\n<?php\nabstract class A { public abstract $prop { get; } } class B extends A { public $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_add_get.phpt",
            "--TEST--\nadd get to inherited set-only hook\n--FILE--\n<?php\nclass A { public $prop { set {} } } class B extends A { public $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_add_set.phpt",
            "--TEST--\nadd set to inherited get-only hook\n--FILE--\n<?php\nclass A { public $prop { get { return 42; } } } class B extends A { public $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_attribute_fail.phpt",
            "--TEST--\noverride attribute missing hook\n--FILE--\n<?php\nclass A { public $prop { get { return 42; } } } class B extends A { public $prop { #[Override] set {} } }\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/override_attribute_plain.phpt",
            "--TEST--\noverride attribute plain property\n--FILE--\n<?php\nclass A { public $prop; } class B extends A { public $prop { #[Override] get => parent::$prop::get(); } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_attribute_virtual.phpt",
            "--TEST--\noverride attribute virtual hook\n--FILE--\n<?php\nclass A { public $prop { get { return 42; } } } class B extends A { public $prop { #[Override] get => parent::$prop::get(); } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/override_default_value.phpt",
            "--TEST--\noverride hooked property default value\n--FILE--\n<?php\nclass A { public $prop = 1 { get => $this->prop; } } class B extends A { public $prop = 2 { get => $this->prop; } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/protected_to_public.phpt",
            "--TEST--\ninherited hook visibility widening\n--FILE--\n<?php\nclass A { protected $prop { get { return 42; } } } class B extends A { public $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/bug006.phpt",
            "--TEST--\nabstract virtualness tracking\n--FILE--\n<?php\nabstract class A { public abstract $prop { get; } } class B extends A { public $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh15456.phpt",
            "--TEST--\nget class vars virtual properties\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } } var_dump(get_class_vars(Test::class));\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh15644.phpt",
            "--TEST--\nasymmetric set hook\n--FILE--\n<?php\nclass Test { public private(set) $prop { set { $this->prop = $value; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh16185.phpt",
            "--TEST--\ndynamic property array indexing\n--FILE--\n<?php\nclass Test { public $prop { get { return []; } } } $test = new Test(); var_dump($test->prop[0] ?? null);\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh16725.phpt",
            "--TEST--\nhooked object iterator visibility\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } } foreach (new Test() as $k => $v) { var_dump($k, $v); }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh17234.phpt",
            "--TEST--\nnumeric parent hook call\n--FILE--\n<?php\nclass A { public $prop { get { return 42; } } } class B extends A { public $prop { get { return parent::$prop::get(0); } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh18000.phpt",
            "--TEST--\nlazy proxy set hook\n--FILE--\n<?php\nclass Test { public $prop { set { $this->prop = $value; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh18268.phpt",
            "--TEST--\narray walk added hooks\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } } array_walk(new Test(), function () {});\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh20270.phpt",
            "--TEST--\nparent hook named args\n--FILE--\n<?php\nclass A { public $prop { set($value) {} } } class B extends A { public $prop { set { parent::$prop::set(value: $value); } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-1.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public protected(set) $prop { get; set; } } class B extends A { public protected(set) $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-2.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public $prop { get; } } class B extends A { public $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-3.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public protected(set) $prop { get; } } class B extends A { public protected(set) $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-4.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public protected(set) $prop { get; set; } } class B extends A { public protected(set) $prop { get { return 42; } set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-5.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public protected(set) $prop { get; } } class B extends A { public protected(set) $prop { get { return 42; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/gh19044-6.phpt",
            "--TEST--\nprotected prototype scope\n--FILE--\n<?php\nabstract class A { public protected(set) $prop { set; } } class B extends A { public protected(set) $prop { set {} } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/field_assign.phpt",
            "--TEST--\nfield assign\n--FILE--\n<?php\nclass Test { public $prop { set { $field ??= 42; $field++; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/field_guard.phpt",
            "--TEST--\nfield guard\n--FILE--\n<?php\nclass Test { public $prop { get { $this->prop = 'prop'; } set { var_dump($this->prop); } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/foreach_002.phpt",
            "--TEST--\nforeach hooked properties\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } }\nforeach (new Test() as $k => $v) {}\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/find_property_usage.phpt",
            "--TEST--\nproperty usage adds backing store\n--FILE--\n<?php\nclass Test { public $prop { get { return $this->prop; } } }\n--EXPECT--\n",
        ),
        (
            "Zend/tests/property_hooks/direct_hook_call.phpt",
            "--TEST--\ndirect hook call\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } }\nvar_dump(Test::$prop::get(...));\n--EXPECTF--\n",
        ),
        (
            "Zend/tests/property_hooks/isset.phpt",
            "--TEST--\nisset get hook\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } }\nvar_dump(isset((new Test())->prop));\n--EXPECT--\nbool(true)\n",
        ),
        (
            "Zend/tests/property_hooks/object_in_hook.phpt",
            "--TEST--\nobject in hook\n--FILE--\n<?php\nclass Box { public $other; public $prop { get { $this->other = new stdClass; return 1; } } }\nvar_dump((new Box())->prop);\n--EXPECT--\nint(1)\n",
        ),
        (
            "Zend/tests/property_hooks/unset.phpt",
            "--TEST--\nunset hook\n--FILE--\n<?php\nclass Test { public $prop { get { return 42; } } }\ntry { unset((new Test())->prop); } catch (Error $e) { echo $e->getMessage(), \"\\n\"; }\n--EXPECTF--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path),
            "runnable\tselected for PTN semantic measurement\n",
            "{path}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_arrow_functions_runnable() {
    for source in [
        "--TEST--\narrow\n--FILE--\n<?php\n$fn = fn($value) => $value + 1;\nvar_dump($fn(1));\n--EXPECT--\nint(2)\n",
        "--TEST--\narrow variable variables\n--FILE--\n<?php\n$a = 1;\n$var = \"a\";\n$fn = fn() => $$var;\nvar_dump($fn());\n--EXPECTF--\n",
        "--TEST--\narrow bound this and scope\n--FILE--\n<?php\nclass Test {\n    public function method() {\n        $fn = fn() => $this;\n        $fn = fn() => Test::method2();\n        $fn = fn() => call_user_func('Test::method2');\n        $thisName = \"this\";\n        $fn = fn() => $$thisName;\n        $fn = fn() => self::class;\n        $fn = static fn() => isset($this);\n    }\n    public function method2() {}\n}\n--EXPECT--\n",
    ] {
        assert_eq!(
            classify(source),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_class_contract_rows_runnable() {
    for source in [
        "--TEST--\nabstract\n--FILE--\n<?php\nabstract class Base { abstract public function run(); }\n--EXPECT--\n",
        "--TEST--\nfinal class\n--FILE--\n<?php\nfinal class Base {}\n--EXPECT--\n",
        "--TEST--\nduplicate final\n--FILE--\n<?php\nfinal final class Base {}\n--EXPECTF--\n",
        "--TEST--\nduplicate abstract\n--FILE--\n<?php\nclass Base { abstract abstract function run(); }\n--EXPECTF--\n",
        "--TEST--\nduplicate visibility\n--FILE--\n<?php\nclass Base { public public function run() {} }\n--EXPECTF--\n",
        "--TEST--\nduplicate property visibility\n--FILE--\n<?php\nclass Base { public public final public final $value; }\n--EXPECTF--\n",
        "--TEST--\nfinal abstract method\n--FILE--\n<?php\nclass Base { final abstract function run(); }\n--EXPECTF--\n",
        "--TEST--\nfinal abstract class\n--FILE--\n<?php\nfinal abstract class Base { private function hidden() {} }\n--EXPECTF--\n",
        "--TEST--\nfinal method\n--FILE--\n<?php\nclass Base { final public function run() {} }\n--EXPECT--\n",
        "--TEST--\nnon-public method\n--FILE--\n<?php\nclass Box { private function hidden() {} protected static function guarded() {} }\n--EXPECT--\n",
        "--TEST--\nfinal class constant\n--FILE--\n<?php\nclass Box { final public const NAME = 'box'; public final const OTHER = 'other'; }\n--EXPECT--\n",
        "--TEST--\nfinal interface constant override\n--FILE--\n<?php\ninterface Contract { final public const NAME = 'base'; } class Impl implements Contract { public const NAME = 'child'; }\n--EXPECTF--\n",
    ] {
        assert_eq!(
            classify(source),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_autoload_metadata_rows_runnable_by_path() {
    let cases = [
        (
            "Zend/tests/autoload/bug42798.phpt",
            "--TEST--\nautoload default class constant\n--FILE--\n<?php\nspl_autoload_register(function ($className) { print \"$className\\n\"; exit(); });\nfunction foo($c = ok::constant) {}\nfoo();\n--EXPECT--\nok\n",
        ),
        (
            "Zend/tests/autoload/bug46665.phpt",
            "--TEST--\nautoload include class declaration\n--FILE--\n<?php\nspl_autoload_register(function ($class) { var_dump($class); require __DIR__ . '/bug46665_autoload.inc'; });\n$baz = '\\\\Foo\\\\Bar\\\\Baz';\nnew $baz();\n--EXPECT--\nstring(11) \"Foo\\Bar\\Baz\"\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path),
            "runnable\tselected for PTN semantic measurement\n",
            "{path}"
        );
    }
}

#[test]
fn phpt_classifier_excludes_unsupported_class_metadata_surfaces() {
    let cases = [
        (
            "final method contracts",
            "--TEST--\nfinal\n--FILE--\n<?php\nclass Base { final public function run() {} } class Child extends Base { public function run() {} }\n--EXPECTF--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "magic sleep metadata",
            "--TEST--\nmagic\n--FILE--\n<?php\nclass Bag { public function __sleep() { return []; } }\n--EXPECT--\n",
            "unsupported-magic-method-metadata\t",
            "requires magic method dispatch/reflection metadata",
        ),
        (
            "autoload",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) {});\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "default autoload callback",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register();\n--EXPECT--\n",
            "unsupported-autoload-metadata\t",
            "requires default spl_autoload callback resolution",
        ),
        (
            "autoload call helper",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_call('Missing');\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "autoload include class declaration",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) { require __DIR__ . '/missing.inc'; });\nnew Missing;\n--EXPECT--\n",
            "unsupported-autoload-metadata\t",
            "requires autoload callback include-driven class declaration",
        ),
        (
            "autoload type declaration",
            "--TEST--\nautoload\n--FILE--\n<?php\nfunction needs(Missing $value) {}\nspl_autoload_register(function ($class) {});\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "autoload parameter default class constant",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) {});\nfunction needs($value = Missing::VALUE) {}\n--EXPECT--\n",
            "unsupported-autoload-metadata\t",
            "requires autoload during parameter default class-constant resolution",
        ),
        (
            "autoload exception propagation",
            "--TEST--\nautoload\n--FILE--\n<?php\nspl_autoload_register(function ($class) { throw new Exception($class); });\necho Missing::$value;\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "reflection closure binding",
            "--TEST--\nreflection\n--FILE--\n<?php\n$r = new ReflectionFunction(fn() => 1);\nvar_dump($r->getClosureThis());\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "reflection function source metadata",
            "--TEST--\nreflection\n--INI--\nopcache.save_comments=1\n--FILE--\n<?php\nfunction test() {}\n$r = new ReflectionFunction('test');\nvar_dump($r->getFilename());\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "reflection method doc comments",
            "--TEST--\nreflection\n--INI--\nopcache.save_comments=1\n--FILE--\n<?php\nclass A { /** doc */ function run() {} }\n$r = new ReflectionClass('A');\nforeach ($r->getMethods() as $m) { var_dump($m->getDocComment()); }\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "readonly static property",
            "--TEST--\nreadonly static\n--FILE--\n<?php\nclass Bag { public static readonly int $value; }\n--EXPECT--\n",
            "unsupported-readonly-property-metadata\t",
            "requires readonly static property diagnostics",
        ),
        (
            "readonly constructor promotion",
            "--TEST--\nreadonly promotion\n--FILE--\n<?php\nreadonly class Bag {\n    public function __construct(\n        public int $value\n    ) {}\n}\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "readonly indirect property mutation",
            "--TEST--\nreadonly indirect mutation\n--FILE--\n<?php\nclass Bag { public readonly array $value; }\n$bag = new Bag();\n$ref =& $bag->value;\n--EXPECT--\n",
            "unsupported-readonly-property-metadata\t",
            "requires indirect readonly property mutation diagnostics",
        ),
        (
            "property hook",
            "--TEST--\nproperty hook\n--FILE--\n<?php\nclass Bag { public mixed $value { get => 42; } }\n--EXPECT--\n",
            "unsupported-property-hook-metadata\t",
            "requires property hook accessors",
        ),
        (
            "multiline property hook",
            "--TEST--\nproperty hook\n--FILE--\n<?php\nclass Bag {\n    public mixed $value {\n        get => 42;\n    }\n}\n--EXPECT--\n",
            "unsupported-property-hook-metadata\t",
            "requires property hook accessors",
        ),
        (
            "typed class constant metadata",
            "--TEST--\ntyped class constant\n--FILE--\n<?php\nclass Bag { const string NAME = 'bag'; }\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "non-public class constant direct access",
            "--TEST--\nprotected class constant\n--FILE--\n<?php\nclass Bag { protected const NAME = 'bag'; }\necho Bag::NAME;\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "non-public class constant reflection metadata",
            "--TEST--\nreflection class constants\n--FILE--\n<?php\nclass Bag { protected const NAME = 'bag'; }\n$r = new ReflectionClass(Bag::class);\nvar_dump($r->getConstants(ReflectionClassConstant::IS_PROTECTED));\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(Attribute::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "runnable\t",
            "selected for PTN semantic measurement",
        ),
        (
            "broader internal attribute reflection metadata",
            "--TEST--\nattribute metadata\n--FILE--\n<?php\n$r = new ReflectionClass(DateTime::class);\nvar_dump($r->getAttributes());\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires internal attribute/reflection metadata beyond modeled Attribute self-reflection",
        ),
        (
            "internal Deprecated attribute object",
            "--TEST--\ndeprecated attribute\n--FILE--\n<?php\n$d = new \\Deprecated(\"message\");\n$d->message = \"updated\";\n--EXPECT--\n",
            "unsupported-internal-attribute-metadata\t",
            "requires direct Deprecated/NoDiscard fatal stack parity beyond modeled caught-object behavior",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with(category),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_non_public_method_visibility_rows_runnable() {
    let classification = classify(
        "--TEST--\nvisibility\n--FILE--\n<?php\nclass Box { private function run() {} protected static function stat() {} }\n--EXPECT--\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_direct_static_property_rows_runnable() {
    let cases = [
        "--TEST--\nstatic property\n--FILE--\n<?php\nclass Box { public static $value = 1; }\nBox::$value = 2;\nvar_dump(Box::$value);\n--EXPECT--\nint(2)\n",
        "--TEST--\nclass constant spread\n--FILE--\n<?php\nclass Bag { public const MORE = [1, 2]; public const VALUES = [0, ...self::MORE]; }\n--EXPECT--\n",
        "--TEST--\nstatic property spread\n--FILE--\n<?php\nclass Bag { public const MORE = [1, 2]; public static $values = [...self::MORE]; }\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt),
            "runnable\tselected for PTN semantic measurement\n"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_keeps_supported_typed_static_reflection_rows_runnable_by_path() {
    let phpt =
        "--TEST--\ntyped static reflection\n--FILE--\n<?php\nclass Box { public static int $value; }\n$ref = new ReflectionProperty('Box', 'value');\n--EXPECT--\n";
    let rows = [
        "ext/reflection/tests/ReflectionClass_setStaticPropertyValue_003.phpt",
        "ext/reflection/tests/ReflectionProperty_isReadable_static.phpt",
        "ext/reflection/tests/ReflectionProperty_isWritable_static.phpt",
        "ext/reflection/tests/ReflectionProperty_typed_static.phpt",
        "ext/reflection/tests/gh12856.phpt",
    ];

    for row in rows {
        let classification = classify_at_relative_path(phpt, row);
        assert_eq!(
            classification, "runnable\tselected for PTN semantic measurement\n",
            "{row}"
        );
    }
}

#[test]
fn phpt_classifier_still_excludes_unproven_typed_static_property_rows() {
    let classification = classify_at_relative_path(
        "--TEST--\ntyped static property\n--FILE--\n<?php\nclass Box { public static int $value; }\n--EXPECT--\n",
        "ext/reflection/tests/unproven_typed_static_property.phpt",
    );

    assert!(
        classification.starts_with("unsupported-typed-property-metadata\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_dynamic_member_dispatch_rows_runnable() {
    let cases = [
        "--TEST--\ndynamic property\n--FILE--\n<?php\nclass Box { public function read($name) { return $this->$name; } }\n--EXPECT--\n",
        "--TEST--\ndynamic static member\n--FILE--\n<?php\nclass Box { public static function run() {} }\n$name = 'run';\nBox::$name();\n--EXPECT--\n",
        "--TEST--\nbraced dynamic static member\n--FILE--\n<?php\nclass Box { public static function run() {} }\n$name = 'run';\nBox::{$name}();\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_keeps_modeled_instance_property_metadata_runnable() {
    let cases = [
        "--TEST--\nprivate property\n--FILE--\n<?php\nclass Box { private $value = 1; public function value() { return $this->value; } }\necho (new Box())->value();\n--EXPECT--\n1\n",
        "--TEST--\nnon-public static property\n--FILE--\n<?php\nclass Box { protected static $value = 1; public static function value() { return self::$value; } }\necho Box::value();\n--EXPECT--\n1\n",
    ];

    for source in cases {
        assert_eq!(
            classify(source),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_get_object_vars_runnable() {
    let classification = classify(
        "--TEST--\nobject vars\n--FILE--\n<?php\n$object = new stdClass;\nvar_dump(get_object_vars($object));\n--EXPECT--\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_object_string_array_helpers_runnable() {
    let classification = classify(
        "--TEST--\nmagic tostring\n--FILE--\n<?php\nclass Box { public function __toString() { return 'box'; } }\narray_udiff([new Box()], [new Box()], fn($a, $b) => 0);\n--EXPECT--\n",
    );
    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );

    let classification = classify(
        "--TEST--\nmagic tostring array_map\n--FILE--\n<?php\nclass Box { public function __toString() { return 'box'; } }\narray_map(fn($value) => (string) $value, [new Box()]);\n--EXPECT--\n",
    );
    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_splits_remaining_magic_method_metadata_blockers() {
    let cases = [
        (
            "sleep hook",
            "--TEST--\nmagic sleep\n--FILE--\n<?php\nclass Box { public function __sleep() { return []; } }\n--EXPECT--\n",
            "requires magic method dispatch/reflection metadata",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-magic-method-metadata\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_modeled_wakeup_arity_diagnostic_runnable() {
    let phpt = "--TEST--\n__wakeup cannot take arguments\n--FILE--\n<?php\nclass Foo {\n    public function __wakeup(string $name) {}\n}\n?>\n--EXPECTF--\nFatal error: Method Foo::__wakeup() cannot take arguments in %s on line %d\n";

    let classification =
        classify_at_relative_path(phpt, "Zend/tests/magic_methods/magic_methods_wakeup.phpt");
    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );

    let generic_classification =
        classify_at_relative_path(phpt, "Zend/tests/magic_methods/other_wakeup.phpt");
    assert!(
        generic_classification.starts_with("unsupported-magic-method-metadata\t"),
        "{generic_classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_supported_magic_property_and_debug_rows_runnable() {
    for source in [
        "--TEST--\nmagic get\n--FILE--\n<?php\nclass Box { public function __get($name) { return 1; } }\necho (new Box)->x;\n--EXPECT--\n1\n",
        "--TEST--\nmagic get by ref\n--FILE--\n<?php\nclass Box { public function &__get($name) { return $this->x; } }\n--EXPECT--\n",
        "--TEST--\nmagic set\n--FILE--\n<?php\nclass Box { public function __set($name, $value) { echo $name; } }\n(new Box)->x = 1;\n--EXPECT--\nx\n",
        "--TEST--\nmagic isset\n--FILE--\n<?php\nclass Box { public function __isset($name) { return true; } }\nvar_dump(isset((new Box)->x));\n--EXPECT--\nbool(true)\n",
        "--TEST--\nmagic unset\n--FILE--\n<?php\nclass Box { public function __unset($name) { echo $name; } }\nunset((new Box)->x);\n--EXPECT--\nx\n",
        "--TEST--\nmagic debug\n--FILE--\n<?php\nclass Box { public function __debugInfo() { return []; } }\nvar_dump(new Box);\n--EXPECT--\nobject(Box)#1 (0) {\n}\n",
        "--TEST--\nmagic static call\n--FILE--\n<?php\nclass Box { public static function __callStatic($name, $args) { return $name; } }\necho Box::missing();\n--EXPECT--\nmissing\n",
    ] {
        assert_eq!(
            classify(source),
            "runnable\tselected for PTN semantic measurement\n"
        );
    }
}

#[test]
fn phpt_classifier_keeps_invalid_array_map_object_callback_runnable() {
    let classification = classify_at_relative_path(
        "--TEST--\narray_map invalid object callback\n--FILE--\n<?php\nclass CallbackCandidate { public function __toString() { return 'candidate'; } }\n$items = [1];\n$callbacks = [new CallbackCandidate()];\ntry { array_map($callbacks[0], $items); } catch (TypeError $e) {}\n--EXPECT--\n",
        "ext/standard/tests/array/array_map_variation17.phpt",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_magic_get_rows_runnable() {
    let classification = classify(
        "--TEST--\nmagic get\n--FILE--\n<?php\nclass Box { public function __get($name) { return 1; } }\nvar_dump((new Box())->missing);\n--EXPECT--\nint(1)\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_array_column_magic_property_path_runnable() {
    let classification = classify(
        "--TEST--\narray column magic property\n--FILE--\n<?php\nclass Test { private $prop; public function __isset($name) { return true; } public function __get($name) { return 'value'; } }\nvar_dump(array_column([new Test()], 'prop'));\n--EXPECT--\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_supported_public_tostring_rows_runnable() {
    let classification = classify(
        "--TEST--\nmagic tostring supported\n--FILE--\n<?php\nclass Box { public function __toString() { return 'box'; } }\necho new Box(), \"\\n\";\n--EXPECT--\nbox\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_keeps_supported_class_metadata_helpers_runnable() {
    let cases = [
        "--TEST--\nclass parents supported\n--FILE--\n<?php\nclass Base {}\nclass Child extends Base {}\nvar_dump(class_parents(Child::class));\n--EXPECT--\n",
        "--TEST--\nclass implements supported\n--FILE--\n<?php\ninterface Contract {}\nclass Worker implements Contract {}\nvar_dump(class_implements('Worker'));\n--EXPECT--\n",
        "--TEST--\nclass uses supported\n--FILE--\n<?php\ntrait ListedTrait {}\nclass Worker { use ListedTrait; }\nvar_dump(class_uses(new Worker));\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }
}

#[test]
fn phpt_classifier_keeps_supported_foreach_internal_surfaces_runnable() {
    let cases = [
        "--TEST--\nforeach mutation\n--FILE--\n<?php\nforeach ($items as &$item) { array_shift($items); }\n--EXPECT--\n",
        "--TEST--\nforeach mutation\n--FILE--\n<?php\nforeach ($items as &$item) { array_unshift($items, 0); }\n--EXPECT--\n",
        "--TEST--\nspl object storage\n--FILE--\n<?php\n$s = new SplObjectStorage();\n$s->attach(new stdClass(), 'info');\nforeach ($s as $object) { var_dump($s->getInfo()); }\n--EXPECT--\n",
        "--TEST--\nspl heap\n--FILE--\n<?php\n$h = new SplMaxHeap();\n$h->insert(2);\n$h->insert(3);\nforeach ($h as $value) { echo $value; }\n--EXPECT--\n",
        "--TEST--\nspl priority queue\n--FILE--\n<?php\n$q = new SplPriorityQueue();\n$q->insert('a', 2);\n$q->setExtractFlags(SplPriorityQueue::EXTR_BOTH);\nvar_dump($q->extract());\n--EXPECT--\n",
        "--TEST--\nregex iterator\n--FILE--\n<?php\n$it = new RegexIterator(new ArrayIterator(['foo']), '/f/');\nvar_dump($it->getMode());\n--EXPECT--\n",
    ];

    for phpt in cases {
        assert_eq!(
            classify(phpt).trim_end(),
            "runnable\tselected for PTN semantic measurement"
        );
        assert_eq!(classify(phpt), classify_with_section_cache(phpt));
    }

    let unsupported = classify(
        "--TEST--\ndirectory iterator still blocked\n--FILE--\n<?php\nnew DirectoryIterator(__DIR__);\n--EXPECT--\n",
    );
    assert!(unsupported.starts_with("unsupported-spl-surface\t"));
}

#[test]
fn phpt_classifier_keeps_current_red_spl_iterator_helpers_runnable() {
    let cases = [
        (
            "ext/spl/tests/spl_007.phpt",
            "--TEST--\niterator apply\n--FILE--\n<?php\niterator_apply(new ArrayIterator([1]), [new Foo, 'bar']);\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/SplTempFileObject_constructor_memory_lt1_variation.phpt",
            "--TEST--\ntemp file object\n--FILE--\n<?php\nvar_dump(new SplTempFileObject(-1));\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/gh9883-extra.phpt",
            "--TEST--\ntemp file string\n--FILE--\n<?php\necho new SplTempFileObject();\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/bug47534.phpt",
            "--TEST--\nrecursive directory current mode\n--FILE--\n<?php\nnew RecursiveDirectoryIterator(__DIR__, FileSystemIterator::CURRENT_AS_PATHNAME);\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/iterator_028.phpt",
            "--TEST--\nrecursive max depth\n--FILE--\n<?php\n$it = new RecursiveIteratorIterator(new RecursiveArrayIterator([1]));\n$it->setMaxDepth(1);\nvar_dump($it->getMaxDepth());\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/autoloading/spl_autoload_throw_with_spl_autoloader_call_as_autoloader.phpt",
            "--TEST--\nautoload validation\n--FILE--\n<?php\nspl_autoload_register('spl_autoload_call');\n--EXPECT--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path).trim_end(),
            "runnable\tselected for PTN semantic measurement",
            "{path}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_supported_spl_fixed_array_rows_runnable() {
    let cases = [
        (
            "ext/spl/tests/SplFixedArray_change_size_during_iteration.phpt",
            "--TEST--\nresize\n--FILE--\n<?php\n$fixed = SplFixedArray::fromArray([1, 2, 3]);\nforeach ($fixed as $value) { $fixed->setSize(2); }\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/SplFixedArray_serialize.phpt",
            "--TEST--\nserialize\n--FILE--\n<?php\n$fixed = new SplFixedArray(1);\necho serialize($fixed);\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/splfixedarray_json_encode.phpt",
            "--TEST--\njson\n--FILE--\n<?php\n$fixed = new SplFixedArray(1);\necho json_encode($fixed);\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/SplFixedArray_immediate_gc.phpt",
            "--TEST--\nimmediate gc\n--FILE--\n<?php\n$fixed = SplFixedArray::fromArray([new stdClass()]);\n$fixed[0] = new stdClass();\nvar_dump(get_mangled_object_vars($fixed));\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/SplFixedArray_get_properties_for.phpt",
            "--TEST--\nget properties\n--FILE--\n<?php\nclass A extends SplFixedArray { public $x; }\n$fixed = new A(1);\nvar_dump(get_mangled_object_vars($fixed));\n--EXPECT--\n",
        ),
        (
            "ext/spl/tests/SplFixedArray_setSize_destruct.phpt",
            "--TEST--\nsetSize destructor\n--FILE--\n<?php\n$fixed = new SplFixedArray(1);\n$fixed->offsetSet(0, false);\n--EXPECT--\n",
        ),
    ];

    for (path, phpt) in cases {
        assert_eq!(
            classify_at_relative_path(phpt, path).trim_end(),
            "runnable\tselected for PTN semantic measurement",
            "{path}"
        );
    }

    let unsupported = classify_at_relative_path(
        "--TEST--\nother fixed array row\n--FILE--\n<?php\nnew SplFixedArray(1);\n--EXPECT--\n",
        "ext/spl/tests/SplFixedArray_other.phpt",
    );
    assert!(
        unsupported.starts_with("unsupported-spl-surface\t"),
        "{unsupported:?}"
    );

    let generator_backed = classify_at_relative_path(
        "--TEST--\nfixed array generator override\n--FILE--\n<?php\nclass A extends SplFixedArray { public function getIterator(): Iterator { yield from []; } }\nforeach (new A(1) as $value) {}\n--EXPECT--\n",
        "ext/spl/tests/SplFixedArray_override_getIterator.phpt",
    );
    assert!(
        generator_backed.starts_with("unsupported-spl-surface\t"),
        "{generator_backed:?}"
    );
}

#[test]
fn phpt_classifier_excludes_unsupported_date_format_parser_rows() {
    let classification = classify(
        "--TEST--\ndate parser\n--FILE--\n<?php\nvar_dump(date_parse_from_format('Y-m-d H:i:s.u', '2009-03-01 18:00:00.7777777'));\n--EXPECT--\n",
    );

    assert!(classification.starts_with("unsupported-internal\t"));
    assert!(classification.contains("date format parser diagnostics"));
}

#[test]
fn phpt_classifier_splits_unsupported_ini_blockers_by_runtime_surface() {
    let cases = [
        (
            "request input",
            "enable_post_data_reading=0",
            "unsupported-request-input-ini\t",
            "request/input/upload SAPI state",
        ),
        (
            "diagnostics",
            "fatal_error_backtraces=0",
            "unsupported-diagnostics-ini\t",
            "engine diagnostic/logging mode",
        ),
        (
            "function disabling",
            "disable_functions=assert",
            "unsupported-function-disable-ini\t",
            "runtime function table mutation",
        ),
        (
            "opcache optimizer dump",
            "opcache.opt_debug_level=0x20000",
            "unsupported-opcache-observability\t",
            "optimizer disassembly",
        ),
        (
            "host path",
            "sys_temp_dir=/tmp",
            "unsupported-host-path-ini\t",
            "host path ini",
        ),
    ];

    for (name, ini, category, reason) in cases {
        let classification = classify(&format!(
            "--TEST--\n{name}\n--INI--\n{ini}\n--FILE--\n<?php\necho \"ok\\n\";\n--EXPECT--\nok\n"
        ));
        assert!(
            classification.starts_with(category),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }

    let memory_limit = classify(
        "--TEST--\nmemory ini\n--INI--\nmemory_limit=128M\nmax_memory_limit=256M\n--FILE--\n<?php\necho ini_get('memory_limit'), \"\\n\";\n--EXPECT--\n128M\n",
    );
    assert_eq!(
        memory_limit.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let residual_extension_ini = classify(
        "--TEST--\nresidual extension ini\n--INI--\npcre.jit=0\nopcache.save_comments=1\nuser_agent=php\n--FILE--\n<?php\nvar_dump(ini_get('pcre.jit'), ini_get('opcache.save_comments'), ini_get('user_agent'));\n--EXPECT--\n",
    );
    assert_eq!(
        residual_extension_ini.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let open_basedir_ini = classify(
        "--TEST--\nopen basedir ini\n--INI--\nopen_basedir=.\n--FILE--\n<?php\necho ini_get('open_basedir'), \"\\n\";\n--EXPECT--\n.\n",
    );
    assert_eq!(
        open_basedir_ini.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let session_metadata_ini = classify_at_relative_path(
        "--TEST--\nsession metadata ini\n--EXTENSIONS--\nsession\n--INI--\nsession.name=PTNID\nsession.cache_limiter=\nsession.save_handler=files\nsession.save_path=\nsession.use_strict_mode=0\n--FILE--\n<?php\nvar_dump(session_name());\n--EXPECT--\nstring(5) \"PTNID\"\n",
        "ext/session/tests/session_metadata_ini.phpt",
    );
    assert_eq!(
        session_metadata_ini.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let opcache_metadata_ini = classify_at_relative_path(
        "--TEST--\nopcache metadata ini\n--EXTENSIONS--\nopcache\n--INI--\nopcache.enable=1\nopcache.enable_cli=1\nopcache.optimization_level=-1\nopcache.file_cache_only=0\n--FILE--\n<?php\nvar_dump(opcache_get_configuration()['directives']['opcache.enable_cli']);\n--EXPECT--\nbool(true)\n",
        "ext/opcache/tests/opcache_metadata_ini.phpt",
    );
    assert_eq!(
        opcache_metadata_ini.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let unmodeled_disabled_function = classify_at_relative_path(
        "--TEST--\nunmodeled disabled function\n--EXTENSIONS--\nopcache\n--INI--\ndisable_functions=dl\nopcache.enable=1\nopcache.enable_cli=1\n--FILE--\n<?php\nvar_dump(is_callable('dl'));\n--EXPECT--\nbool(false)\n",
        "ext/opcache/tests/disable_unmodeled_dl.phpt",
    );
    assert_eq!(
        unmodeled_disabled_function.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let phar_metadata_ini = classify_at_relative_path(
        "--TEST--\nphar metadata ini\n--EXTENSIONS--\nphar\n--INI--\nphar.readonly=1\nphar.require_hash=0\nphar.cache_list=\n--FILE--\n<?php\nvar_dump(Phar::isValidPharFilename('example.phar'));\n--EXPECT--\nbool(true)\n",
        "ext/phar/tests/phar_metadata_ini.phpt",
    );
    assert_eq!(
        phar_metadata_ini.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let phar_archive_ini = classify_at_relative_path(
        "--TEST--\nphar archive ini\n--EXTENSIONS--\nphar\n--INI--\nphar.readonly=0\n--FILE--\n<?php\n$phar = new Phar(__DIR__ . '/archive.phar');\n--EXPECT--\n",
        "ext/phar/tests/phar_archive_ini.phpt",
    );
    assert!(
        phar_archive_ini.starts_with("unsupported-phar-archive-runtime\t"),
        "{phar_archive_ini:?}"
    );

    let phar_cache_list_recursive_iterator = classify_at_relative_path(
        "--TEST--\nphar cache-list recursive iterator\n--EXTENSIONS--\nphar\n--INI--\nphar.cache_list={PWD}/files/nophar.phar\n--FILE--\n<?php\n$p = 'phar://' . __DIR__ . '/files/nophar.phar';\nforeach (new RecursiveIteratorIterator(new Phar($p)) as $f) echo $f->getPathName(), \"\\n\";\n--EXPECT--\n",
        "ext/phar/tests/cached_manifest_1.phpt",
    );
    assert_eq!(
        phar_cache_list_recursive_iterator.trim_end(),
        "runnable\timplemented PHAR tar/zip archive residual row pack"
    );

    let pdo_mysql_service = classify_at_relative_path(
        "--TEST--\npdo mysql service\n--EXTENSIONS--\npdo_mysql\n--SKIPIF--\n<?php\nrequire_once __DIR__ . '/inc/mysql_pdo_test.inc';\nMySQLPDOTest::skip();\n?>\n--FILE--\n<?php\nrequire_once __DIR__ . '/inc/mysql_pdo_test.inc';\n$db = MySQLPDOTest::factory();\n--EXPECT--\n",
        "ext/pdo_mysql/tests/service.phpt",
    );
    assert!(
        pdo_mysql_service.starts_with("external-service\t"),
        "{pdo_mysql_service:?}"
    );

    let zip_archive_runtime = classify_at_relative_path(
        "--TEST--\nzip archive mutation\n--EXTENSIONS--\nzip\n--FILE--\n<?php\nfunction &cb() {}\n$zip = new ZipArchive;\n$zip->open(__DIR__ . '/archive.zip', ZipArchive::CREATE);\n$zip->registerCancelCallback(cb(...));\n$zip->addFromString('test', 'test');\n--EXPECT--\n",
        "ext/zip/tests/ZipArchive_bailout.phpt",
    );
    assert!(
        zip_archive_runtime.starts_with("unsupported-zip-archive-runtime\t"),
        "{zip_archive_runtime:?}"
    );

    let zip_archive_exact_row = classify_at_relative_path(
        "--TEST--\nzip archive exact row\n--EXTENSIONS--\nzip\n--FILE--\n<?php\n$zip = new ZipArchive;\n$zip->open(__DIR__ . '/archive.zip', ZipArchive::CREATE);\n$zip->addFromString('test', 'test');\n$zip->close();\n--EXPECT--\n",
        "ext/zip/tests/oo_stream.phpt",
    );
    assert_eq!(
        zip_archive_exact_row.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let xmlwriter_extension = classify(
        "--TEST--\nxmlwriter extension\n--EXTENSIONS--\nxmlwriter\n--FILE--\n<?php\n$xw = xmlwriter_open_memory();\nxmlwriter_start_element($xw, 'root');\nxmlwriter_end_element($xw);\necho xmlwriter_flush($xw);\n--EXPECT--\n<root/>\n",
    );
    assert_eq!(
        xmlwriter_extension.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let simplexml_extension = classify(
        "--TEST--\nsimplexml extension\n--EXTENSIONS--\nsimplexml\n--FILE--\n<?php\n$sxe = simplexml_load_string('<root><item>value</item></root>');\nvar_dump((string) $sxe->item);\n--EXPECT--\nstring(5) \"value\"\n",
    );
    assert_eq!(
        simplexml_extension.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let random_function = classify_at_relative_path(
        "--TEST--\nrandom int\n--EXTENSIONS--\nrandom\n--FILE--\n<?php\nvar_dump(random_int(1, 2));\n--EXPECT--\n",
        "ext/random/tests/01_functions/random_int.phpt",
    );
    assert_eq!(
        random_function.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let random_engine = classify_at_relative_path(
        "--TEST--\nrandom engine\n--EXTENSIONS--\nrandom\n--FILE--\n<?php\nuse Random\\Engine\\Mt19937;\necho serialize(new Mt19937(1234));\n--EXPECT--\n",
        "ext/random/tests/02_engine/mt19937_serialize.phpt",
    );
    assert_eq!(
        random_engine.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let serialize_precision = classify(
        "--TEST--\nserialize precision ini\n--INI--\nserialize_precision=-1\n--FILE--\n<?php\necho ini_get('serialize_precision'), \"\\n\";\n--EXPECT--\n-1\n",
    );
    assert_eq!(
        serialize_precision.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let exception_string_param_max_len = classify(
        "--TEST--\nexception trace ini\n--INI--\nzend.exception_string_param_max_len=23\n--FILE--\n<?php\nthrow new Exception();\n--EXPECTF--\nFatal error: Uncaught Exception in %s\n",
    );
    assert_eq!(
        exception_string_param_max_len.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );

    let serialize_precision = classify(
        "--TEST--\nserialize precision ini\n--INI--\nprecision=14\nserialize_precision=17\n--FILE--\n<?php\necho ini_get('serialize_precision'), \"\\n\";\n--EXPECT--\n17\n",
    );
    assert_eq!(
        serialize_precision.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
}

#[test]
fn phpt_classifier_excludes_memory_resource_limit_expectations() {
    let cases = [
        (
            "allocation fatal",
            "--TEST--\nallocation fatal\n--INI--\nmemory_limit=2M\n--FILE--\n<?php\n$items = [];\nwhile (true) { $items[] = new stdClass(); }\n--EXPECTF--\nFatal error: Allowed memory size of %d bytes exhausted%s\n",
        ),
        (
            "runtime lowering warning",
            "--TEST--\nruntime lowering\n--FILE--\n<?php\n$a = str_repeat('0', 5 * 1024 * 1024);\nini_set('memory_limit', '3M');\n--EXPECTF--\nWarning: Failed to set memory limit to 3145728 bytes (Current memory usage is %d bytes) in %s on line %d\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-resource-limit-ini\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains("memory manager allocation-failure"),
            "{name}: {classification:?}"
        );
    }

    let memory_limit_read = classify(
        "--TEST--\nmemory ini read\n--INI--\nmemory_limit=128M\n--FILE--\n<?php\necho ini_get('memory_limit'), \"\\n\";\n--EXPECT--\n128M\n",
    );
    assert!(
        memory_limit_read.starts_with("runnable\t"),
        "{memory_limit_read:?}"
    );
}

#[test]
fn phpt_classifier_excludes_unsupported_runtime_diagnostics_surfaces() {
    let assert_options = classify(
        "--TEST--\nassert options\n--FILE--\n<?php\nassert_options(ASSERT_BAIL, 1);\nassert(false);\n--EXPECT--\n",
    );
    assert!(
        assert_options.starts_with("runnable\t"),
        "{assert_options:?}"
    );

    let cases = [
        (
            "assert null coalesce assignment",
            "--TEST--\nassert lvalue\n--FILE--\n<?php\nassert($items['key'] ??= 1);\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "assertion expression lvalue mode interaction",
        ),
        (
            "assert closure pretty print",
            "--TEST--\nassert closure\n--FILE--\n<?php\nassert(0 && ($fn = function () { return 1; }));\n--EXPECT--\n",
            "unsupported-assertion-runtime\t",
            "assertion AST pretty-printing for closure expressions",
        ),
    ];

    for (name, phpt, category, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with(category),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }

    let namespace_assert = classify(
        "--TEST--\nnamespace assert\n--FILE--\n<?php\nnamespace Foo;\nvar_dump(assert(false));\n--EXPECT--\n",
    );
    assert_eq!(
        namespace_assert.trim_end(),
        "runnable\tselected for PTN semantic measurement"
    );
}

#[test]
fn phpt_classifier_allows_focused_enum_metadata_rows() {
    let enum_row = "--TEST--\nenum metadata\n--FILE--\n<?php\nenum Demo { case A; }\n--EXPECT--\n";
    for path in [
        "Zend/tests/attributes/Attribute/Attribute_on_enum.phpt",
        "Zend/tests/attributes/allow_dynamic_properties_on_enum.phpt",
        "Zend/tests/attributes/deprecated/class_constants/101.phpt",
        "Zend/tests/attributes/deprecated/error_on_enum.phpt",
        "Zend/tests/attributes/override/014.phpt",
        "Zend/tests/attributes/override/015.phpt",
        "Zend/tests/enum/ast-dumper.phpt",
        "Zend/tests/enum/backed-int-const-invalid-expr.phpt",
        "Zend/tests/enum/comparison.phpt",
        "Zend/tests/enum/enum-in-var-export.phpt",
        "Zend/tests/enum/enum_in_stack_trace.phpt",
        "Zend/tests/enum/enum_exists.phpt",
        "Zend/tests/enum/extending-user-error.phpt",
        "Zend/tests/enum/gh8176.phpt",
        "Zend/tests/enum/offsetGet-in-const-expr.phpt",
        "Zend/tests/enum/debugInfo/backed_enum_value.phpt",
        "Zend/tests/enum/debugInfo/magic_method.phpt",
        "Zend/tests/enum/debugInfo/visibility_validation.phpt",
        "ext/reflection/tests/ReflectionClassConstant_isEnumCase.phpt",
        "ext/reflection/tests/ReflectionClass_isEnum.phpt",
        "ext/spl/tests/ArrayObject/ArrayObject_enum.phpt",
    ] {
        let classification = classify_at_relative_path(enum_row, path);
        assert!(
            classification.starts_with("runnable\t"),
            "{path}: {classification:?}"
        );
    }

    let delayed_validator_row = "--TEST--\ndelayed validator metadata\n--FILE--\n<?php\n#[DelayedTargetValidation]\n#[Attribute]\nenum Demo { case A; }\n$r = new ReflectionClass(Demo::class);\nvar_dump($r->getAttributes());\n--EXPECTF--\n";
    for path in [
        "Zend/tests/attributes/delayed_target_validation/validator_AllowDynamicProperties.phpt",
        "Zend/tests/attributes/delayed_target_validation/validator_Attribute.phpt",
        "Zend/tests/attributes/delayed_target_validation/validator_Deprecated.phpt",
    ] {
        let classification = classify_at_relative_path(delayed_validator_row, path);
        assert!(
            classification.starts_with("runnable\t"),
            "{path}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_pcre_array_pattern_rows_runnable() {
    let classification = classify(
        "--TEST--\npcre array pattern\n--INI--\npcre.jit=0\n--FILE--\n<?php\nvar_dump(preg_replace(array('#x#'), '', 'x'));\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_debug_backtrace_runnable() {
    let classification = classify(
        "--TEST--\nbacktrace\n--FILE--\n<?php\nprint_r(debug_backtrace(0, 1));\ndebug_print_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 1);\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_function_call_array_lvalue_assignment_runnable() {
    let classification = classify(
        "--TEST--\nfunction call lvalue\n--FILE--\n<?php\ndebug_backtrace()[1]['args'][0] = 'Modified';\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_function_call_array_lvalue_compound_assignment() {
    let classification =
        classify("--TEST--\nfunction call lvalue\n--FILE--\n<?php\nfoo()[0] += 2;\n--EXPECT--\n");
    assert!(
        classification.starts_with("unsupported-lvalue-runtime\t"),
        "{classification:?}"
    );
    assert!(
        classification.contains("function-call array-dimension"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_exception_get_trace_runnable() {
    let classification = classify(
        "--TEST--\ntrace\n--FILE--\n<?php\ntry { throw new Exception(); } catch (Exception $e) { var_dump($e->getTrace()); }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_exception_get_trace_as_string_runnable() {
    let classification = classify(
        "--TEST--\ntrace string\n--FILE--\n<?php\ntry { throw new Exception(); } catch (Exception $e) { echo $e->getTraceAsString(); }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_reflection_property_mutation_rows() {
    let classification = classify(
        "--TEST--\nreflection property\n--FILE--\n<?php\n$ref = new ReflectionProperty(new Exception(), 'trace');\n$ref->setValue(new Exception(), []);\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("unsupported-internal-reflection-metadata\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_declared_reflection_property_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection property\n--FILE--\n<?php\nclass Bag { public $value = 1; }\n$ref = new ReflectionProperty('Bag', 'value');\nvar_dump($ref->getName(), $ref->getModifiers());\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_declared_reflection_property_type_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection property type\n--FILE--\n<?php\nclass Bag { public int $value = 1; }\n$ref = new ReflectionProperty('Bag', 'value');\nvar_dump($ref->getType());\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_excludes_reflection_property_union_type_rows() {
    let classification = classify_at_relative_path(
        "--TEST--\nreflection property union type\n--FILE--\n<?php\nclass Bag { public int|string $value; }\n$ref = new ReflectionProperty('Bag', 'value');\nvar_dump($ref->getType());\n--EXPECT--\n",
        "ext/reflection/tests/types/union_types.phpt",
    );
    assert!(
        classification.starts_with("unsupported-internal-reflection-metadata\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_final_static_property_modifier_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection property modifiers\n--FILE--\n<?php\nclass Bag { public static final $value; }\n$ref = new ReflectionProperty('Bag', 'value');\nvar_dump($ref->getModifiers());\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_property_hook_modifier_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection property hook modifiers\n--FILE--\n<?php\nclass Bag { public $value { get { return 42; } } }\n$ref = new ReflectionProperty('Bag', 'value');\nvar_dump($ref->getModifiers());\nvar_dump(ReflectionProperty::IS_VIRTUAL);\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_property_hook_is_final_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection property final metadata\n--FILE--\n<?php\nclass Bag { public final $value { get => 42; } }\nforeach ((new ReflectionClass(Bag::class))->getProperties() as $ref) { var_dump($ref->isFinal()); }\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_reflection_parameter_named_type_rows_runnable() {
    let classification = classify(
        "--TEST--\nreflection parameter type\n--FILE--\n<?php\nfunction takesString(string $value): int { return strlen($value); }\n$function = new ReflectionFunction('takesString');\n$type = $function->getParameters()[0]->getType();\n$return = $function->getReturnType();\nvar_dump($type->getName(), $type->isBuiltin(), $return->getName());\n--EXPECT--\n",
    );
    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_basic_assertions_runnable() {
    let classification = classify(
        "--TEST--\nassert\n--FILE--\n<?php\nvar_dump(assert(true));\ntry { assert(false, 'failed'); } catch (AssertionError $e) { echo $e->getMessage(); }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_assertion_ini_mode_rows_runnable() {
    let classification = classify(
        "--TEST--\nassert ini\n--INI--\nzend.assertions=1\nassert.exception=1\n--FILE--\n<?php\nini_set('zend.assertions', 0);\nvar_dump(assert(false));\nini_set('zend.assertions', 1);\ntry { assert(false); } catch (AssertionError $e) { echo $e->getMessage(); }\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_basic_assertion_closure_invocation_runnable() {
    let classification = classify(
        "--TEST--\nassert closure invocation\n--FILE--\n<?php\nassert((function () { return true; })());\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_variadic_parameter_rows_runnable() {
    let classification = classify(
        "--TEST--\nvariadic\n--FILE--\n<?php\nfunction f(...$args) { var_dump($args); }\nf(1, 2);\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_function_registry_and_destructor_magic_rows_runnable() {
    let cases = [
        (
            "function registry sweep",
            "--TEST--\narginfo sweep\n--FILE--\n<?php\nforeach (get_defined_functions()[\"internal\"] as $function) { var_dump($function); }\n--EXPECT--\n",
        ),
        (
            "destructor __call object resurrection",
            "--TEST--\ndestructor call resurrection\n--FILE--\n<?php\nclass Driver { public $obj; function close() { echo $this->obj->i; } }\nclass A { function __call($m, $a) { $d = new Driver; $d->obj = $this; } function __destruct() { $this->close(); } }\nnew A;\n--EXPECT--\n",
        ),
        (
            "reflection new instance without constructor",
            "--TEST--\nreflection instance\n--FILE--\n<?php\nclass Box { public $v = 1; }\n$rc = new ReflectionClass(Box::class);\n$obj = $rc->newInstanceWithoutConstructor();\nvar_dump($obj instanceof Box);\n--EXPECT--\nbool(true)\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_array_splice_destructor_reentrancy_runnable() {
    let classification = classify(
        "--TEST--\nsplice destructor\n--FILE--\n<?php\nclass C { function __destruct() { global $items; $items[] = 0; } }\n$items = [1, new C, 2];\narray_splice($items, 1, 1);\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_modeled_mutating_array_helpers_runnable() {
    let cases = [
        (
            "array_splice",
            "--TEST--\nsplice\n--FILE--\n<?php\n$items = [1, 2, 3];\narray_splice($items, 1, 1, [4]);\n--EXPECT--\n",
        ),
        (
            "array_walk_recursive",
            "--TEST--\nrecursive walk\n--FILE--\n<?php\n$items = [1];\narray_walk_recursive($items, \"var_dump\");\n--EXPECT--\n",
        ),
        (
            "array_multisort",
            "--TEST--\nmultisort\n--FILE--\n<?php\n$left = [2, 1];\n$right = [\"b\", \"a\"];\narray_multisort($left, SORT_ASC, SORT_REGULAR, $right, SORT_DESC, SORT_STRING);\n--EXPECT--\n",
        ),
        (
            "user comparator sort",
            "--TEST--\nusort\n--FILE--\n<?php\n$items = [3, 1, 2];\nusort($items, \"strcmp\");\n--EXPECT--\n",
        ),
    ];

    for (name, phpt) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("runnable\t"),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_excludes_huge_array_allocation_rows() {
    let cases = [
        (
            "literal huge count",
            "--TEST--\nhuge array fill\n--FILE--\n<?php\narray_fill(0, 2147483647, 1);\n--EXPECTF--\n",
            "multi-billion element array_fill()",
        ),
        (
            "constant-scale variable count",
            "--TEST--\nhuge array fill variable\n--FILE--\n<?php\n$intMax = PHP_INT_MAX;\narray_fill(0, $intMax, 1);\n--EXPECTF--\n",
            "multi-billion element array_fill()",
        ),
        (
            "spread-expanded max elements",
            "--TEST--\nspread max elements\n--FILE--\n<?php\n$power = 20;\n$arr = range(0, 2**$power);\narray_diff(...array_fill(0, 2**(32-$power), $arr));\n--EXPECTF--\n",
            "max-array-size/resource-limit diagnostics",
        ),
        (
            "peak memory accounting",
            "--TEST--\npeak memory\n--FILE--\n<?php\nvar_dump(memory_get_peak_usage());\nmemory_reset_peak_usage();\n--EXPECTF--\n",
            "memory manager peak-usage accounting",
        ),
    ];

    for (name, phpt, reason) in cases {
        let classification = classify(phpt);
        assert!(
            classification.starts_with("unsupported-resource-limit\t"),
            "{name}: {classification:?}"
        );
        assert!(
            classification.contains(reason),
            "{name}: {classification:?}"
        );
    }
}

#[test]
fn phpt_classifier_keeps_large_array_fill_start_key_runnable() {
    let classification = classify(
        "--TEST--\nlarge start key\n--FILE--\n<?php\narray_fill(PHP_INT_MAX, 1, 'x');\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_unsupported_internal_names_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\ninternal names text\n--FILE--\n<?php\n// array_splice($a, 0); debug_backtrace(); get_defined_functions();\n# array_multisort($a)\n/* usort($a, \"cmp\"); array_walk_recursive($a, \"cb\"); ini_set(\"zend.assertions\", 0); */\necho \"array_splice array_multisort usort uasort uksort array_walk_recursive debug_backtrace get_defined_functions ini_set zend.assertions\";\n--EXPECT--\narray_splice array_multisort usort uasort uksort array_walk_recursive debug_backtrace get_defined_functions ini_set zend.assertions\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_attribute_text_in_strings_runnable() {
    let classification = classify(
        "--TEST--\nattribute text\n--FILE--\n<?php\necho \"prefix #[not an attribute]\";\n--EXPECT--\nprefix #[not an attribute]\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_unsupported_syntax_words_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\nsyntax text\n--FILE--\n<?php\n// throw new Exception();\n# fn($x) => $x\n/* public private(set) int $value; static $value; array_walk_recursive($a, 'f'); */\necho \"readonly class fn throw private(set) static $value array_walk_recursive($a, 'f') <<<HEREDOC\";\n--EXPECT--\nreadonly class fn throw private(set) static $value array_walk_recursive($a, 'f') <<<HEREDOC\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_user_diagnostic_handlers_runnable() {
    let classification = classify(
        "--TEST--\nhandler\n--FILE--\n<?php\nset_error_handler('handler');\nrestore_error_handler();\nset_exception_handler('handler');\nrestore_exception_handler();\ntrigger_error('hello');\n--EXPECT--\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_runtime_diagnostics_words_in_strings_and_comments_runnable() {
    let classification = classify(
        "--TEST--\ndiagnostic text\n--FILE--\n<?php\n// debug_backtrace(); set_error_handler('x'); assert_options(ASSERT_BAIL, 1);\n# ini_set('zend.assertions', 0); new ErrorException();\n/* debug_print_backtrace(); restore_error_handler(); */\necho \"debug_backtrace set_error_handler assert_options zend.assertions ErrorException getSeverity\";\n--EXPECT--\ndebug_backtrace set_error_handler assert_options zend.assertions ErrorException getSeverity\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_hash_comments_runnable() {
    let classification = classify(
        "--TEST--\ncomment\n--FILE--\n<?php\n# ordinary comment\nvar_dump(1);\n--EXPECT--\nint(1)\n",
    );

    assert!(
        classification.starts_with("runnable\t"),
        "{classification:?}"
    );
}

#[test]
fn phpt_classifier_keeps_stdin_section_rows_runnable() {
    let classification = classify(
        "--TEST--\nstdin\n--STDIN--\ninput\n--FILE--\n<?php\necho stream_get_contents(STDIN);\n--EXPECT--\ninput\n",
    );

    assert_eq!(
        classification,
        "runnable\tselected for PTN semantic measurement\n"
    );
}

#[test]
fn phpt_classifier_splits_sapi_executable_boundaries() {
    let cgi = classify_at_relative_path(
        "--TEST--\ncgi\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n",
        "sapi/cgi/tests/001.phpt",
    );
    assert!(cgi.starts_with("cgi-sapi-executable\t"), "{cgi:?}");

    let fpm = classify_at_relative_path(
        "--TEST--\nfpm\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n",
        "sapi/fpm/tests/bug64539-status-json-encoding.phpt",
    );
    assert!(fpm.starts_with("fpm-sapi\t"), "{fpm:?}");

    let phpdbg = classify_at_relative_path(
        "--TEST--\nphpdbg\n--PHPDBG--\nr\n--EXPECT--\n",
        "sapi/phpdbg/tests/basic_run.phpt",
    );
    assert!(phpdbg.starts_with("phpdbg-sapi\t"), "{phpdbg:?}");
}

#[test]
fn phpt_classifier_allows_supported_cli_self_probes() {
    let version = classify_at_relative_path(
        "--TEST--\nversion\n--FILE--\n<?php\n$php = getenv('TEST_PHP_EXECUTABLE_ESCAPED');\nvar_dump(shell_exec(\"$php -n -v\"));\n--EXPECTF--\n",
        "sapi/cli/tests/001.phpt",
    );
    assert_eq!(version, "runnable\tselected for PTN semantic measurement\n");
    let version_with_harness_programs = classify_at_relative_path_with_harness_programs(
        "--TEST--\nversion\n--SKIPIF--\n<?php include \"skipif.inc\"; ?>\n--FILE--\n<?php\n$php = getenv('TEST_PHP_EXECUTABLE_ESCAPED');\nvar_dump(shell_exec(\"$php -n -v\"));\n--EXPECTF--\n",
        "sapi/cli/tests/001.phpt",
    );
    assert_eq!(
        version_with_harness_programs,
        "runnable\tselected for PTN semantic measurement\n"
    );

    let shebang = classify_at_relative_path(
        "--TEST--\nshebang\n--FILE--\n<?php\n$php = getenv('TEST_PHP_EXECUTABLE');\n$filename = __DIR__ . '/script.php';\n$script = \"#!$php -n\\n<?php echo 1; ?>\\n\";\nfile_put_contents($filename, $script);\necho shell_exec($filename);\n--EXPECT--\n1\n",
        "sapi/cli/tests/021.phpt",
    );
    assert_eq!(shebang, "runnable\tselected for PTN semantic measurement\n");
}

#[test]
fn phpt_classifier_splits_cli_option_and_process_residuals() {
    let unsupported_option = classify_at_relative_path(
        "--TEST--\nrf\n--FILE--\n<?php\n$php = getenv('TEST_PHP_EXECUTABLE_ESCAPED');\nvar_dump(shell_exec(\"$php -n --rf phpinfo\"));\n--EXPECT--\n",
        "sapi/cli/tests/004.phpt",
    );
    assert!(
        unsupported_option.starts_with("unsupported-cli-option\t"),
        "{unsupported_option:?}"
    );

    let process = classify_at_relative_path(
        "--TEST--\nproc\n--FILE--\n<?php\n$php = getenv('TEST_PHP_EXECUTABLE_ESCAPED');\n$proc = proc_open(\"$php -n test.php\", [], $pipes);\n--EXPECT--\n",
        "sapi/cli/tests/022.phpt",
    );
    assert!(process.starts_with("process-boundary\t"), "{process:?}");
}

#[test]
fn phpt_baseline_full_scope_generates_all_family_manifests() {
    let root = temp_dir("ptn-full-phpt-baseline");
    let corpus = root.join("php-src");
    fs::write(
        {
            fs::create_dir_all(&corpus).expect("create fake corpus");
            corpus.join("run-tests.php")
        },
        "<?php\n",
    )
    .expect("write run-tests.php");

    let rows = [
        "Zend/tests/basic_a.phpt",
        "Zend/tests/basic_b.phpt",
        "ext/json/tests/json_a.phpt",
        "ext/json/tests/json_b.phpt",
        "ext/standard/tests/array_a.phpt",
        "ext/standard/tests/array_b.phpt",
        "main/tests/main_a.phpt",
        "sapi/cli/tests/cli_a.phpt",
        "sapi/fpm/tests/fpm_a.phpt",
        "tests/basic/core_a.phpt",
        "tests/lang/core_b.phpt",
        "tests/output/core_c.phpt",
    ];
    for row in rows {
        let path = corpus.join(row);
        fs::create_dir_all(path.parent().expect("PHPT parent")).expect("create PHPT dir");
        fs::write(
            path,
            "--TEST--\ncase\n--FILE--\n<?php echo 1; ?>\n--EXPECT--\n1\n",
        )
        .expect("write PHPT");
    }

    let out_dir = root.join("baseline");
    let output = Command::new("timeout")
        .arg("10s")
        .arg("tools/run-phpt-baseline.sh")
        .arg("--scope")
        .arg("full")
        .arg("--tier")
        .arg("5")
        .arg("--generate-only")
        .arg("--out-dir")
        .arg(&out_dir)
        .env("PHP_SRC_PHPT", &corpus)
        .env("PTN_PHPT_AUTO_FETCH", "0")
        .output()
        .expect("run full PHPT baseline generator");
    assert!(
        output.status.success(),
        "full baseline generator failed: status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest_dir = fs::read_dir(&out_dir)
        .expect("read baseline out dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .expect("timestamped manifest dir");
    for manifest in [
        "phpt-full-corpus-5.txt",
        "phpt-full-corpus-1000.txt",
        "phpt-full-corpus-5000.txt",
        "phpt-full-corpus-10000.txt",
        "phpt-full-corpus-20000.txt",
        "phpt-full-corpus-all.txt",
    ] {
        assert!(
            manifest_dir.join(manifest).is_file(),
            "missing {manifest} in {}",
            manifest_dir.display()
        );
    }

    let inventory = fs::read_to_string(manifest_dir.join("inventory.txt")).expect("read inventory");
    assert!(inventory.contains("scope: full"), "{inventory}");
    assert!(inventory.contains("available: rows=12"), "{inventory}");
    assert!(
        inventory.contains("manifest: scope=full tier=all rows=12"),
        "{inventory}"
    );
    assert!(
        inventory.contains("available.ext/json: rows=2"),
        "{inventory}"
    );
    assert!(
        inventory.contains("available.sapi/cli: rows=1"),
        "{inventory}"
    );

    let all_rows = fs::read_to_string(manifest_dir.join("full-corpus-inventory.txt"))
        .expect("read full corpus inventory");
    assert_eq!(all_rows.lines().count(), 12, "{all_rows}");

    let tier_five = fs::read_to_string(manifest_dir.join("phpt-full-corpus-5.txt"))
        .expect("read tier five manifest");
    let selected_rows = tier_five
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .count();
    assert_eq!(selected_rows, 5, "{tier_five}");
}

#[test]
fn phpt_campaign_report_gate_accepts_only_ported_passed_table() {
    let root = temp_dir("ptn-campaign-report-gate");
    let valid_report = root.join("valid.md");
    let prose_report = root.join("prose.md");
    let extra_column_report = root.join("extra-column.md");

    fs::write(
        &valid_report,
        "| Ported Tests | Passed Tests |\n| --- | ---: |\n| 21867 | 383 |\n",
    )
    .expect("write valid report");
    fs::write(
        &prose_report,
        "# PHPT campaign\n\n| Ported Tests | Passed Tests |\n| --- | --- |\n| 1000 | 383 |\n",
    )
    .expect("write prose report");
    fs::write(
        &extra_column_report,
        "| Ported Tests | Passed Tests | Notes |\n| --- | --- | --- |\n| 1000 | 383 | classified |\n",
    )
    .expect("write extra-column report");

    let ok = Command::new("bash")
        .arg("tools/check-phpt-campaign-reports.sh")
        .arg(&valid_report)
        .output()
        .expect("run report gate success case");
    assert!(
        ok.status.success(),
        "report gate should accept table-only reports: stdout={}\nstderr={}",
        String::from_utf8_lossy(&ok.stdout),
        String::from_utf8_lossy(&ok.stderr)
    );

    let prose = Command::new("bash")
        .arg("tools/check-phpt-campaign-reports.sh")
        .arg(&prose_report)
        .output()
        .expect("run report gate prose case");
    assert!(
        !prose.status.success()
            && String::from_utf8_lossy(&prose.stderr).contains("report must be table-only"),
        "report gate should reject prose reports: stdout={}\nstderr={}",
        String::from_utf8_lossy(&prose.stdout),
        String::from_utf8_lossy(&prose.stderr)
    );

    let extra_column = Command::new("bash")
        .arg("tools/check-phpt-campaign-reports.sh")
        .arg(&extra_column_report)
        .output()
        .expect("run report gate extra-column case");
    assert!(
        !extra_column.status.success()
            && String::from_utf8_lossy(&extra_column.stderr)
                .contains("only Ported Tests and Passed Tests columns"),
        "report gate should reject extra columns: stdout={}\nstderr={}",
        String::from_utf8_lossy(&extra_column.stdout),
        String::from_utf8_lossy(&extra_column.stderr)
    );
}
