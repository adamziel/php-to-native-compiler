use php_compiler::error::Phase;
use php_compiler::run_source;

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
fn other_superglobals_remain_ordinary_missing_variables_for_now() {
    let error = runtime_error("<?php\necho $_GET;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$_GET'");
}
