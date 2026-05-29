use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source, run_source_with_source_file};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn fixture_source_file() -> String {
    "tests/fixtures/milestone1303/include_path_resolution.php".to_string()
}

#[test]
fn include_path_builtins_mutate_current_runtime_include_path() {
    let execution = run_source(
        r#"<?php
echo get_include_path();
echo "|";
$old = set_include_path("alpha:beta");
echo $old;
echo "|";
echo get_include_path();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, ".|.|alpha:beta");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_uses_configured_include_path_with_source_relative_fallback() {
    let execution = run_source_with_source_file(
        r#"<?php
$old = set_include_path(__DIR__ . "/include_path_lib");
$result = include "wp_loader.inc";
echo "result=" . $result;
echo "|old=" . $old;
echo "|loaded=" . $loaded;
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inc:tests/fixtures/milestone1303/include_path_lib|dir:tests/fixtures/milestone1303/include_path_lib|result=loader-return|old=.|loaded=from-include-path"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_included_files_reports_main_file_before_included_files() {
    let execution = run_source_with_source_file(
        r#"<?php
include __DIR__ . "/include_path_lib/wp_loader.inc";
$files = get_included_files();
echo "|files=" . basename($files[0]) . ":" . basename($files[1]);
echo "|same=" . (get_required_files() == $files ? "yes" : "no");
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inc:.|dir:./tests/fixtures/milestone1303/include_path_lib||files=include_path_resolution.php:wp_loader.inc|same=yes"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_path_lookup_precedes_source_relative_fallback_for_matching_names() {
    let execution = run_source_with_source_file(
        r#"<?php
$old = set_include_path(__DIR__ . "/include_path_lib");
$first = include "same_name.inc";
echo "|first=" . $first;
set_include_path(__DIR__ . "/missing_path");
$second = include "source_fallback.inc";
echo "|second=" . $second;
"#,
        "tests/fixtures/milestone1619/include_path_ordering.php".to_string(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "loaded=include-path|first=lib-returnloaded=source-relative|second=source-return"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn missing_include_emits_warnings_returns_false_and_continues() {
    let execution = run_source_with_source_file(
        r#"<?php
function capture_include_warning($errno, $errstr) {
    echo "|warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : (str_contains($errstr, "Failed opening") ? "opening" : "other"));
    return true;
}

set_error_handler("capture_include_warning", E_WARNING);
$result = include __DIR__ . "/missing-wordpress-optional.php";
echo "|include=" . ($result === false ? "false" : "value");
$once = include_once __DIR__ . "/missing-wordpress-once.php";
echo "|once=" . ($once === false ? "false" : "value");
echo "|continued";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "|warning:2:path:open|warning:2:path:opening|include=false|warning:2:path:open|warning:2:path:opening|once=false|continued"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn missing_require_emits_warnings_then_fatal_exit() {
    let execution = run_source_with_source_file(
        r#"<?php
function capture_require_warning($errno, $errstr) {
    echo "|warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-required") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : (str_contains($errstr, "Failed opening") ? "opening" : "other"));
    return true;
}

set_error_handler("capture_require_warning", E_WARNING);
require "missing-wordpress-required.php";
echo "|not-reached";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "|warning:2:path:open|warning:2:path:opening"
    );
    assert_eq!(execution.exit_code, 255);
    assert!(execution.stderr.contains(
        "PHP Fatal error:  require(): Failed opening required 'tests/fixtures/milestone1303/missing-wordpress-required.php' (include_path='.') in tests/fixtures/milestone1303/include_path_resolution.php on line 10"
    ), "{}", execution.stderr);
}

#[test]
fn missing_require_once_expression_emits_warnings_then_fatal_exit() {
    let execution = run_source_with_source_file(
        r#"<?php
function capture_require_once_warning($errno, $errstr) {
    echo "|warning:" . $errno;
    echo ":" . (str_contains($errstr, "missing-wordpress-required-once") ? "path" : "missing");
    echo ":" . (str_contains($errstr, "Failed to open stream") ? "open" : (str_contains($errstr, "Failed opening") ? "opening" : "other"));
    return true;
}

set_error_handler("capture_require_once_warning", E_WARNING);
$result = require_once "missing-wordpress-required-once.php";
echo "|not-reached";
"#,
        fixture_source_file(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "|warning:2:path:open|warning:2:path:opening"
    );
    assert_eq!(execution.exit_code, 255);
    assert!(execution.stderr.contains(
        "PHP Fatal error:  require_once(): Failed opening required 'tests/fixtures/milestone1303/missing-wordpress-required-once.php' (include_path='.') in tests/fixtures/milestone1303/include_path_resolution.php on line 10"
    ), "{}", execution.stderr);
}

#[test]
fn include_and_require_once_accept_bounded_local_file_urls() {
    let execution = run_source_with_source_file(
        r#"<?php
$include_url = "file://" . realpath(__DIR__ . "/file_url_include.inc");
$include_result = include $include_url;
echo "include=" . $include_result . ":" . $included_from_url;
$require_url = "file://" . realpath(__DIR__ . "/file_url_required.inc");
$require_result = require_once $require_url;
echo "|require=" . $require_result . ":" . $required_from_url;
$again = require_once $require_url;
echo "|again=" . ($again === true ? "true" : "other");
"#,
        "tests/fixtures/milestone1631/file_url_wrapper_reads.php".to_string(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "include=include-return:file-url-include|require=require-return:file-url-require|again=true"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_and_require_once_percent_decode_bounded_local_file_urls() {
    let execution = run_source_with_source_file(
        r#"<?php
$base = realpath(__DIR__);
$include_result = include "file://" . $base . "/file%20url%20include%20%23encoded.inc";
echo "include=" . $include_result . ":" . $included_from_percent_url;
$require_result = require_once "file://" . $base . "/file%20url%20required%20%2Bencoded.inc";
echo "|require=" . $require_result . ":" . $required_from_percent_url;
"#,
        "tests/fixtures/milestone1637/file_url_percent_decoding.php".to_string(),
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "include=percent-include-return:percent-include|require=percent-require-return:percent-require"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn include_path_builtins_reject_forms_outside_current_subset() {
    let get_too_many = runtime_error("<?php\necho get_include_path('extra');\n");
    assert_eq!(get_too_many.line, 2);
    assert_eq!(get_too_many.column, 6);
    assert_eq!(
        get_too_many.message,
        "arity mismatch for get_include_path(): expected 0 argument(s), got 1"
    );

    let set_non_string = runtime_error("<?php\necho set_include_path(42);\n");
    assert_eq!(set_non_string.line, 2);
    assert_eq!(set_non_string.column, 6);
    assert_eq!(
        set_non_string.message,
        "unsupported call set_include_path(): path argument must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_folds_include_path_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("get_include_path") ? "1" : "0";
echo is_callable("set_include_path") ? "1" : "0";
echo defined("PATH_SEPARATOR") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");

    let error = emit_ir_source("<?php\necho get_include_path();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
