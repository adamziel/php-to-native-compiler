use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn mysqli_connect_is_visible_but_connections_are_an_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_connect";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable");
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
mysqli_connect("localhost", "user", "password", "database");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset"
    );
}

#[test]
fn mysqli_report_accepts_current_wordpress_startup_modes() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_report";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo defined("MYSQLI_REPORT_OFF") ? MYSQLI_REPORT_OFF : "missing";
echo "|";
echo mysqli_report(MYSQLI_REPORT_OFF) ? "off" : "fail";
echo "|";
echo $call(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT) ? "strict" : "fail";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|off|strict");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_report_rejects_modes_outside_current_boundary() {
    let error = run_source(
        r#"<?php
mysqli_report(4);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_report(): only MYSQLI_REPORT_OFF and MYSQLI_REPORT_ERROR|MYSQLI_REPORT_STRICT are supported in the current subset"
    );

    let error = run_source(
        r#"<?php
mysqli_report("off");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_report(): report mode must be int in the current subset, got string"
    );
}

#[test]
fn mysqli_init_returns_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_init";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
echo "|", get_class($handle);
echo "|", $handle->connect_errno;
echo "|", $handle->connect_error === null ? "null" : "set";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|mysqli|0|null");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_real_connect_accepts_current_wordpress_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_real_connect";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
echo "|";
echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0) ? "connected" : "failed";
echo "|";
echo $handle->connect_errno;
echo "|";
echo $handle->connect_error === null ? "null" : "set";
echo "|";
echo $call($handle, null, null, null, null, null, null, 0) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|connected|0|null|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_get_server_info_returns_current_placeholder_version() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_get_server_info";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_get_server_info($handle);
echo "|";
echo $call($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|8.0.0-phpc-placeholder|8.0.0-phpc-placeholder"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_returns_false_for_current_wordpress_sql_mode_probe() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_query";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT @@SESSION.sql_mode");
echo "|";
echo $result === false ? "false" : "result";
echo "|";
echo empty($result) ? "empty" : "set";
echo "|";
echo $call($handle, "SELECT @@SESSION.sql_mode") === false ? "dynamic" : "result";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|false|empty|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_accepts_current_wordpress_options_empty_result_placeholders() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$autoload = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
$fallback = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$primed = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN ('siteurl','home')");
$single = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$columns = mysqli_query($handle, "SHOW FULL COLUMNS FROM `wp_options`");
$describe = mysqli_query($handle, "DESCRIBE wp_users;");
echo $autoload === false ? "autoload-empty" : "autoload-result";
echo "|";
echo $fallback === false ? "fallback-empty" : "fallback-result";
echo "|";
echo $primed === false ? "prime-empty" : "prime-result";
echo "|";
echo $single === false ? "single-empty" : "single-result";
echo "|";
echo $columns === false ? "columns-empty" : "columns-result";
echo "|";
echo $describe === false ? "describe-empty" : "describe-result";
echo "|";
echo mysqli_errno($handle);
echo "|";
echo mysqli_error($handle);
echo "|";
$errno = "mysqli_errno";
$error = "mysqli_error";
echo $errno($handle);
echo "|";
echo $error($handle) === "" ? "clean" : "dirty";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload-empty|fallback-empty|prime-empty|single-empty|columns-empty|describe-empty|0||0|clean"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_select_db_accepts_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_select_db";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_select_db($handle, "wordpress") ? "selected" : "failed";
echo "|";
echo $call($handle, null) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|selected|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_real_escape_string_escapes_current_scalar_subset() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_real_escape_string";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$data = "quote'\"\\\n\r";
echo "|";
echo mysqli_real_escape_string($handle, $data);
echo "|";
echo $call($handle, true);
echo "|";
echo mysqli_real_escape_string($handle, 42);
echo "|";
echo mysqli_real_escape_string($handle, null);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, r#"yes|callable|quote\'\"\\\n\r|1|42|"#);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_real_connect_rejects_forms_outside_current_boundary() {
    let error = run_source(
        r#"<?php
mysqli_real_connect("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_real_connect(): first argument must be mysqli object in the current subset, got string"
    );

    let error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, "3306", null, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_real_connect(): port argument must be int or null in the current subset, got string"
    );
}

#[test]
fn mysqli_get_server_info_rejects_forms_outside_current_boundary() {
    let error = run_source(
        r#"<?php
mysqli_get_server_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_get_server_info(): argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_query_accepts_current_wordpress_charset_setup_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo $result === true ? "charset-ok" : "charset-result";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "charset-ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_returns_current_empty_result_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT * FROM wp_posts WHERE 1 = 0");
echo get_class($result);
echo "|";
echo mysqli_num_fields($result);
echo "|";
echo mysqli_fetch_field($result) === false ? "no-field" : "field";
echo "|";
echo mysqli_fetch_object($result) === false ? "no-row" : "row";
echo "|";
echo mysqli_free_result($result) === null ? "freed" : "value";
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "done";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mysqli_result|0|no-field|no-row|freed|done|done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_query("not-a-handle", "SELECT @@SESSION.sql_mode");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_query(): first argument must be mysqli object in the current subset, got string"
    );

    let unsupported_query = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_query($handle, "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_query.phase, Phase::Runtime);
    assert_eq!(unsupported_query.line, 3);
    assert_eq!(unsupported_query.column, 1);
    assert_eq!(
        unsupported_query.message,
        "unsupported call mysqli_query(): non-empty mysqli result sets are not implemented in the current subset; only deterministic WordPress SQL mode, charset setup, empty options, metadata, and exact empty-result placeholders are supported; got SELECT 1"
    );

    let non_empty_wordpress_select = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_query($handle, "SELECT * FROM wp_posts WHERE ID = 1");
"#,
    )
    .unwrap_err();

    assert_eq!(non_empty_wordpress_select.phase, Phase::Runtime);
    assert_eq!(non_empty_wordpress_select.line, 3);
    assert_eq!(non_empty_wordpress_select.column, 1);
    assert_eq!(
        non_empty_wordpress_select.message,
        "unsupported call mysqli_query(): non-empty mysqli result sets are not implemented in the current subset; only deterministic WordPress SQL mode, charset setup, empty options, metadata, and exact empty-result placeholders are supported; got SELECT * FROM wp_posts WHERE ID = 1"
    );

    let bad_errno_handle = run_source(
        r#"<?php
mysqli_errno("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_errno_handle.phase, Phase::Runtime);
    assert_eq!(bad_errno_handle.line, 2);
    assert_eq!(bad_errno_handle.column, 1);
    assert_eq!(
        bad_errno_handle.message,
        "unsupported call mysqli_errno(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_result_handle = run_source(
        r#"<?php
mysqli_fetch_object(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(bad_result_handle.phase, Phase::Runtime);
    assert_eq!(bad_result_handle.line, 2);
    assert_eq!(bad_result_handle.column, 1);
    assert_eq!(
        bad_result_handle.message,
        "unsupported call mysqli_fetch_object(): first argument must be mysqli_result object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_select_db_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_select_db("not-a-handle", "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_select_db(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_database = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_select_db($handle, ["wordpress"]);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_database.phase, Phase::Runtime);
    assert_eq!(bad_database.line, 3);
    assert_eq!(bad_database.column, 1);
    assert_eq!(
        bad_database.message,
        "unsupported call mysqli_select_db(): database argument must be string or null in the current subset, got array"
    );
}

#[test]
fn mysqli_real_escape_string_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_real_escape_string("not-a-handle", "value");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_real_escape_string(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_data = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_escape_string($handle, ["value"]);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_data.phase, Phase::Runtime);
    assert_eq!(bad_data.line, 3);
    assert_eq!(bad_data.column, 1);
    assert_eq!(
        bad_data.message,
        "unsupported call mysqli_real_escape_string(): data argument arrays are not implemented in the current subset"
    );
}

#[test]
fn dynamic_mysqli_connect_calls_use_the_same_database_boundary() {
    let error = run_source(
        r#"<?php
$call = "mysqli_connect";
$call("localhost");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_folds_mysqli_connect_metadata_but_rejects_direct_connection_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("mysqli_connect") ? "1" : "0";
echo is_callable("mysqli_connect") ? "1" : "0";
echo function_exists("mysqli_real_connect") ? "1" : "0";
echo is_callable("mysqli_real_connect") ? "1" : "0";
echo function_exists("mysqli_get_server_info") ? "1" : "0";
echo is_callable("mysqli_get_server_info") ? "1" : "0";
echo function_exists("mysqli_query") ? "1" : "0";
echo is_callable("mysqli_query") ? "1" : "0";
echo function_exists("mysqli_errno") ? "1" : "0";
echo is_callable("mysqli_errno") ? "1" : "0";
echo function_exists("mysqli_error") ? "1" : "0";
echo is_callable("mysqli_error") ? "1" : "0";
echo function_exists("mysqli_select_db") ? "1" : "0";
echo is_callable("mysqli_select_db") ? "1" : "0";
echo function_exists("mysqli_real_escape_string") ? "1" : "0";
echo is_callable("mysqli_real_escape_string") ? "1" : "0";
echo function_exists("mysqli_fetch_object") ? "1" : "0";
echo is_callable("mysqli_fetch_object") ? "1" : "0";
echo function_exists("mysqli_fetch_field") ? "1" : "0";
echo is_callable("mysqli_fetch_field") ? "1" : "0";
echo function_exists("mysqli_num_fields") ? "1" : "0";
echo is_callable("mysqli_num_fields") ? "1" : "0";
echo function_exists("mysqli_free_result") ? "1" : "0";
echo is_callable("mysqli_free_result") ? "1" : "0";
echo function_exists("mysqli_more_results") ? "1" : "0";
echo is_callable("mysqli_more_results") ? "1" : "0";
echo function_exists("mysqli_next_result") ? "1" : "0";
echo is_callable("mysqli_next_result") ? "1" : "0";
echo function_exists("mysqli_report") ? "1" : "0";
echo is_callable("mysqli_report") ? "1" : "0";
echo function_exists("mysqli_init") ? "1" : "0";
echo is_callable("mysqli_init") ? "1" : "0";
echo defined("MYSQLI_REPORT_OFF") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 33, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("MYSQLI_REPORT_OFF"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
mysqli_report(MYSQLI_REPORT_OFF);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_connect("localhost");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_real_connect(mysqli_init(), "localhost");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_server_info(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_query(mysqli_init(), "SELECT @@SESSION.sql_mode");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_errno(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_error(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_select_db(mysqli_init(), "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_real_escape_string(mysqli_init(), "value");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
