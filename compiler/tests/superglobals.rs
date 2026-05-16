use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn server_superglobal_is_materialized_with_current_request_defaults() {
    let execution = run_source(
        r#"<?php
echo is_array($_SERVER) ? "array" : "missing";
echo "|";
echo $_SERVER["REQUEST_URI"];
echo "|";
echo $_SERVER["HTTP_HOST"];
echo "|";
echo $_SERVER["PHP_SELF"];
echo "|";
echo $_SERVER["SCRIPT_FILENAME"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|/|localhost|/index.php|/index.php");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_is_available_as_current_cli_runtime_constant() {
    let execution = run_source(
        r#"<?php
echo PHP_SAPI;
echo "|";
echo defined("PHP_SAPI") ? "defined" : "missing";
echo "|";
echo constant("PHP_SAPI");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cli|defined|cli");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_name_returns_current_cli_runtime_sapi() {
    let execution = run_source(
        r#"<?php
echo php_sapi_name();
echo "|";
$call = "php_sapi_name";
echo function_exists($call) ? "exists" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cli|exists|callable|cli");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_sapi_name_rejects_arguments() {
    let error = runtime_error(
        r#"<?php
echo php_sapi_name("cgi-fcgi");
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for php_sapi_name(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_php_sapi_name_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("php_sapi_name") ? "1" : "0";
echo is_callable("php_sapi_name") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho php_sapi_name();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn server_superglobal_routes_function_scope_reads_and_writes_to_root() {
    let execution = run_source(
        r#"<?php
function fix_server_vars() {
    $_SERVER = array_merge(["SERVER_SOFTWARE" => "", "REQUEST_URI" => ""], $_SERVER);
    if (empty($_SERVER["REQUEST_URI"])) {
        $_SERVER["REQUEST_URI"] = "/";
    }
    $_SERVER["HTTP_HOST"] = "example.test";
}

fix_server_vars();
echo $_SERVER["SERVER_SOFTWARE"];
echo "|";
echo $_SERVER["REQUEST_URI"];
echo "|";
echo $_SERVER["HTTP_HOST"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "phpc|/|example.test");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn cookie_superglobal_is_materialized_as_empty_auto_global_array() {
    let execution = run_source(
        r#"<?php
echo is_array($_COOKIE) ? "array" : "missing";
echo "|";
echo isset($_COOKIE["wordpress_test_cookie"]) ? "cookie" : "empty";

function seed_cookie() {
    $_COOKIE["wordpress_test_cookie"] = "WP Cookie check";
}

seed_cookie();
echo "|";
echo $_COOKIE["wordpress_test_cookie"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|empty|WP Cookie check");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_direct_string_offsets_route_function_scope_writes_to_root() {
    let execution = run_source(
        r#"<?php
function boot_cache() {
    $GLOBALS["wp_object_cache"] = "ready";
}

function use_cache() {
    global $wp_object_cache;
    echo $wp_object_cache;
}

boot_cache();
use_cache();
echo "|", $GLOBALS["wp_object_cache"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ready|ready");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_direct_string_offsets_bind_reference_targets_to_direct_sources() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["target"] =& $value;
echo $target, "|", $GLOBALS["target"], "|";
$value = "second";
echo $target, "|", $GLOBALS["target"], "|";
$GLOBALS["target"] = "third";
echo $value, "|", $target;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first|second|second|third|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_reference_targets_can_bind_function_local_sources_to_root() {
    let execution = run_source(
        r#"<?php
function bind_target() {
    $value = "local";
    $GLOBALS["target"] =& $value;
    $value = "changed";
    echo $GLOBALS["target"], "|";
}

bind_target();
echo $target, "|";
$target = "global-write";
echo $GLOBALS["target"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed|global-write");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_reference_targets_detach_when_source_name_is_unset() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["target"] =& $value;
unset($value);
$value = "new";
echo $GLOBALS["target"], "|", $value, "|";
$GLOBALS["target"] = "global";
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|new|new");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_reference_targets_bind_direct_sources_to_root_arrays() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["bag"]["slot"] =& $value;
echo $bag["slot"], "|", $GLOBALS["bag"]["slot"], "|";
$value = "second";
echo $bag["slot"], "|";
$GLOBALS["bag"]["slot"] = "third";
echo $value, "|", $bag["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|first|second|third|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_reference_targets_can_bind_function_local_sources() {
    let execution = run_source(
        r#"<?php
function bind_nested() {
    $value = "local";
    $GLOBALS["bag"]["slot"] =& $value;
    $value = "changed";
    echo $GLOBALS["bag"]["slot"], "|";
}

bind_nested();
echo $bag["slot"], "|";
$bag["slot"] = "global-write";
echo $GLOBALS["bag"]["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "changed|changed|global-write");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_append_reference_targets_bind_selected_auto_key() {
    let execution = run_source(
        r#"<?php
$value = "first";
$GLOBALS["bag"][] =& $value;
echo $bag[0], "|";
$value = "second";
echo $GLOBALS["bag"][0], "|";
$bag[0] = "third";
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|second|third");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_append_reference_sources_bind_alias_to_root_array_slot() {
    let execution = run_source(
        r#"<?php
$GLOBALS["bag"] = [];
$alias =& $GLOBALS["bag"][];
$alias = "from-alias";
echo $bag[0], "|";
$GLOBALS["bag"][0] = "from-slot";
echo $alias;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn globals_nested_append_reference_sources_bind_function_local_alias() {
    let execution = run_source(
        r#"<?php
function bind_nested() {
    $alias =& $GLOBALS["bag"]["outer"][];
    $alias = "from-alias";
    echo $GLOBALS["bag"]["outer"][0], "|";
    $GLOBALS["bag"]["outer"][0] = "from-slot";
    echo $alias;
}

bind_nested();
echo "|", $bag["outer"][0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "from-alias|from-slot|from-slot");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn other_superglobals_remain_ordinary_missing_variables_for_now() {
    let error = runtime_error("<?php\necho $_GET;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$_GET'");
}
