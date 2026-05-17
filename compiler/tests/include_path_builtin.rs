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
fn include_uses_configured_include_path_after_source_relative_lookup() {
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
