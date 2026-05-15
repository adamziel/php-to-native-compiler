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
$version = "mysqli_get_server_version";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo is_callable($version) ? "version-callable" : "version-missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_get_server_info($handle);
echo "|";
echo $call($handle);
echo "|";
echo mysqli_get_server_version($handle);
echo "|";
echo $version($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|version-callable|8.0.0-phpc-placeholder|8.0.0-phpc-placeholder|80000|80000"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_get_host_info_returns_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_get_host_info";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_get_host_info($handle);
echo "|";
echo $call($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|localhost via TCP/IP (phpc-placeholder)|localhost via TCP/IP (phpc-placeholder)"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_client_and_protocol_metadata_return_current_placeholders() {
    let execution = run_source(
        r#"<?php
$client = "mysqli_get_client_info";
$version = "mysqli_get_client_version";
$proto = "mysqli_get_proto_info";
$thread = "mysqli_thread_id";
echo function_exists($client) ? "yes" : "no";
echo "|";
echo is_callable($version) ? "version-callable" : "version-missing";
echo "|";
echo is_callable($proto) ? "callable" : "missing";
echo "|";
echo is_callable($thread) ? "thread-callable" : "thread-missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_get_client_info();
echo "|";
echo mysqli_get_client_info(null);
echo "|";
echo $client($handle);
echo "|";
echo mysqli_get_client_version();
echo "|";
echo $version();
echo "|";
echo mysqli_get_proto_info($handle);
echo "|";
echo $proto($handle);
echo "|";
echo mysqli_thread_id($handle);
echo "|";
echo $thread($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|version-callable|callable|thread-callable|mysqlnd 8.0.0-phpc-placeholder|mysqlnd 8.0.0-phpc-placeholder|mysqlnd 8.0.0-phpc-placeholder|80000|80000|10|10|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_connection_stats_return_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_get_connection_stats";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$stats = mysqli_get_connection_stats($handle);
$dynamic = $call($handle);
echo "|";
echo count($stats);
echo "|";
echo $stats["bytes_sent"];
echo "|";
echo $stats["bytes_received"];
echo "|";
echo $stats["connect_success"];
echo "|";
echo $stats["active_connections"];
echo "|";
echo $dynamic["result_set_queries"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|8|0|0|1|1|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_field_count_returns_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_field_count";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo $call($handle);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_close_accepts_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_close";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_close($handle) ? "closed" : "open";
echo "|";
echo $call($handle) ? "closed" : "open";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|closed|closed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_stat_returns_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_stat";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_stat($handle);
echo "|";
echo $call($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|Uptime: 0  Threads: 0  Questions: 0  Slow queries: 0  Opens: 0  Flush tables: 0  Open tables: 0  Queries per second avg: 0.000|Uptime: 0  Threads: 0  Questions: 0  Slow queries: 0  Opens: 0  Flush tables: 0  Open tables: 0  Queries per second avg: 0.000"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_autocommit_accepts_current_placeholder_modes() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_autocommit";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_autocommit($handle, false) ? "off" : "failed";
echo "|";
echo mysqli_autocommit($handle, true) ? "on" : "failed";
echo "|";
echo $call($handle, false) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|off|on|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_begin_transaction_accepts_current_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_begin_transaction";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_begin_transaction($handle) ? "default" : "failed";
echo "|";
echo mysqli_begin_transaction($handle, 0, "wp") ? "named" : "failed";
echo "|";
echo $call($handle, 0, null) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|default|named|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_commit_and_rollback_accept_current_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$commit = "mysqli_commit";
$rollback = "mysqli_rollback";
echo function_exists($commit) ? "yes" : "no";
echo "|";
echo is_callable($rollback) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_begin_transaction($handle);
echo "|";
echo mysqli_commit($handle) ? "commit" : "failed";
mysqli_begin_transaction($handle, 0, "wp");
echo "|";
echo mysqli_rollback($handle, 0, "wp") ? "rollback" : "failed";
echo "|";
echo $commit($handle, 0, null) ? "dynamic-commit" : "failed";
echo "|";
echo $rollback($handle, 0, null) ? "dynamic-rollback" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|commit|rollback|dynamic-commit|dynamic-rollback"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_set_charset_accepts_current_utf8mb4_placeholder() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_set_charset";
$charset_call = "mysqli_get_charset";
$name_call = "mysqli_character_set_name";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo is_callable($charset_call) ? "charset-callable" : "charset-missing";
echo "|";
echo is_callable($name_call) ? "name-callable" : "name-missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_set_charset($handle, "utf8mb4") ? "set" : "failed";
echo "|";
echo $call($handle, "UTF8MB4") ? "dynamic" : "failed";
$charset = mysqli_get_charset($handle);
$dynamic = $charset_call($handle);
echo "|";
echo $charset->charset;
echo "|";
echo $charset->collation;
echo "|";
echo $charset->min_length;
echo "|";
echo $charset->max_length;
echo "|";
echo $charset->number;
echo "|";
echo $charset->state;
echo "|";
echo $dynamic->charset;
echo "|";
echo mysqli_character_set_name($handle);
echo "|";
echo $name_call($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|charset-callable|name-callable|set|dynamic|utf8mb4|utf8mb4_unicode_520_ci|1|4|246|0|utf8mb4|utf8mb4|utf8mb4"
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
fn mysqli_mutation_metadata_returns_clean_placeholder_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_error_state_metadata_returns_clean_placeholder_state() {
    let execution = run_source(
        r#"<?php
$sqlstate = "mysqli_sqlstate";
$warnings = "mysqli_warning_count";
echo function_exists($sqlstate) ? "yes" : "no";
echo "|";
echo is_callable($warnings) ? "callable" : "missing";
$handle = mysqli_init();
echo "|";
echo mysqli_errno($handle);
echo "|";
echo mysqli_error($handle);
echo "|";
echo mysqli_sqlstate($handle);
echo "|";
echo mysqli_warning_count($handle);
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo $sqlstate($handle);
echo "|";
echo $warnings($handle);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0||00000|0|00000|0");
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

    let bad_version_handle = run_source(
        r#"<?php
mysqli_get_server_version("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_version_handle.phase, Phase::Runtime);
    assert_eq!(bad_version_handle.line, 2);
    assert_eq!(bad_version_handle.column, 1);
    assert_eq!(
        bad_version_handle.message,
        "unsupported call mysqli_get_server_version(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_get_host_info_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_get_host_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_get_host_info(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_stat_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_stat("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_stat(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_client_and_protocol_metadata_reject_forms_outside_current_boundary() {
    let bad_client_arg = run_source(
        r#"<?php
mysqli_get_client_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_client_arg.phase, Phase::Runtime);
    assert_eq!(bad_client_arg.line, 2);
    assert_eq!(bad_client_arg.column, 1);
    assert_eq!(
        bad_client_arg.message,
        "unsupported call mysqli_get_client_info(): optional argument must be mysqli object or null in the current subset, got string"
    );

    let bad_client_version_arity = run_source(
        r#"<?php
mysqli_get_client_version(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(bad_client_version_arity.phase, Phase::Runtime);
    assert_eq!(bad_client_version_arity.line, 2);
    assert_eq!(bad_client_version_arity.column, 1);
    assert_eq!(
        bad_client_version_arity.message,
        "arity mismatch for mysqli_get_client_version(): expected 0 argument(s), got 1"
    );

    let bad_proto_handle = run_source(
        r#"<?php
mysqli_get_proto_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_proto_handle.phase, Phase::Runtime);
    assert_eq!(bad_proto_handle.line, 2);
    assert_eq!(bad_proto_handle.column, 1);
    assert_eq!(
        bad_proto_handle.message,
        "unsupported call mysqli_get_proto_info(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_thread_handle = run_source(
        r#"<?php
mysqli_thread_id("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_thread_handle.phase, Phase::Runtime);
    assert_eq!(bad_thread_handle.line, 2);
    assert_eq!(bad_thread_handle.column, 1);
    assert_eq!(
        bad_thread_handle.message,
        "unsupported call mysqli_thread_id(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_connection_stats_reject_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_get_connection_stats("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_get_connection_stats(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_field_count_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_field_count("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_field_count(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_close_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_close("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_close(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_autocommit_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_autocommit("not-a-handle", true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_autocommit(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_mode = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_autocommit($handle, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_mode.phase, Phase::Runtime);
    assert_eq!(bad_mode.line, 3);
    assert_eq!(bad_mode.column, 1);
    assert_eq!(
        bad_mode.message,
        "unsupported call mysqli_autocommit(): mode argument must be bool in the current subset, got int"
    );
}

#[test]
fn mysqli_begin_transaction_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_begin_transaction("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_begin_transaction(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_flags = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_begin_transaction($handle, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_flags.phase, Phase::Runtime);
    assert_eq!(bad_flags.line, 3);
    assert_eq!(bad_flags.column, 1);
    assert_eq!(
        bad_flags.message,
        "unsupported call mysqli_begin_transaction(): only flags value 0 is implemented in the current subset, got 1"
    );

    let bad_name = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_begin_transaction($handle, 0, false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_name.phase, Phase::Runtime);
    assert_eq!(bad_name.line, 3);
    assert_eq!(bad_name.column, 1);
    assert_eq!(
        bad_name.message,
        "unsupported call mysqli_begin_transaction(): name argument must be string or null in the current subset, got bool"
    );
}

#[test]
fn mysqli_commit_and_rollback_reject_forms_outside_current_boundary() {
    let bad_commit_handle = run_source(
        r#"<?php
mysqli_commit("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_commit_handle.phase, Phase::Runtime);
    assert_eq!(bad_commit_handle.line, 2);
    assert_eq!(bad_commit_handle.column, 1);
    assert_eq!(
        bad_commit_handle.message,
        "unsupported call mysqli_commit(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_rollback_handle = run_source(
        r#"<?php
mysqli_rollback("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_rollback_handle.phase, Phase::Runtime);
    assert_eq!(bad_rollback_handle.line, 2);
    assert_eq!(bad_rollback_handle.column, 1);
    assert_eq!(
        bad_rollback_handle.message,
        "unsupported call mysqli_rollback(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_commit_flags = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_commit($handle, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_commit_flags.phase, Phase::Runtime);
    assert_eq!(bad_commit_flags.line, 3);
    assert_eq!(bad_commit_flags.column, 1);
    assert_eq!(
        bad_commit_flags.message,
        "unsupported call mysqli_commit(): only flags value 0 is implemented in the current subset, got 1"
    );

    let bad_rollback_name = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_rollback($handle, 0, false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_rollback_name.phase, Phase::Runtime);
    assert_eq!(bad_rollback_name.line, 3);
    assert_eq!(bad_rollback_name.column, 1);
    assert_eq!(
        bad_rollback_name.message,
        "unsupported call mysqli_rollback(): name argument must be string or null in the current subset, got bool"
    );
}

#[test]
fn mysqli_set_charset_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_set_charset("not-a-handle", "utf8mb4");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_set_charset(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_charset = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_set_charset($handle, ["utf8mb4"]);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_charset.phase, Phase::Runtime);
    assert_eq!(bad_charset.line, 3);
    assert_eq!(bad_charset.column, 1);
    assert_eq!(
        bad_charset.message,
        "unsupported call mysqli_set_charset(): charset must be string in the current subset, got array"
    );

    let unsupported_charset = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_set_charset($handle, "latin1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_charset.phase, Phase::Runtime);
    assert_eq!(unsupported_charset.line, 3);
    assert_eq!(unsupported_charset.column, 1);
    assert_eq!(
        unsupported_charset.message,
        "unsupported call mysqli_set_charset(): only the deterministic utf8mb4 placeholder charset is implemented in the current subset, got latin1"
    );

    let bad_get_charset_handle = run_source(
        r#"<?php
mysqli_get_charset("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_get_charset_handle.phase, Phase::Runtime);
    assert_eq!(bad_get_charset_handle.line, 2);
    assert_eq!(bad_get_charset_handle.column, 1);
    assert_eq!(
        bad_get_charset_handle.message,
        "unsupported call mysqli_get_charset(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_charset_name_handle = run_source(
        r#"<?php
mysqli_character_set_name("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_charset_name_handle.phase, Phase::Runtime);
    assert_eq!(bad_charset_name_handle.line, 2);
    assert_eq!(bad_charset_name_handle.column, 1);
    assert_eq!(
        bad_charset_name_handle.message,
        "unsupported call mysqli_character_set_name(): first argument must be mysqli object in the current subset, got string"
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
fn mysqli_query_returns_current_seed_post_row_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo get_class($result);
echo "|";
echo mysqli_num_fields($result);
$field = mysqli_fetch_field($result);
echo "|", $field->name;
$field = mysqli_fetch_field($result);
echo "|", $field->name;
echo "|";
echo mysqli_fetch_field($result) === false ? "no-field" : "field";
$row = mysqli_fetch_object($result);
echo "|", get_class($row);
echo "|", $row->ID;
echo "|", $row->post_title;
echo "|";
echo mysqli_fetch_object($result) === false ? "no-row" : "row";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mysqli_result|2|ID|post_title|no-field|stdClass|1|Hello world placeholder|no-row"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_fetch_assoc_returns_current_seed_post_array_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_assoc($result);
echo $row["ID"];
echo "|";
echo $row["post_title"];
echo "|";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|Hello world placeholder|no-row");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_fetch_row_returns_current_seed_post_numeric_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_row($result);
echo $row[0];
echo "|";
echo $row[1];
echo "|";
echo isset($row["ID"]) ? "assoc" : "no-assoc";
echo "|";
echo mysqli_fetch_row($result) === false ? "no-row" : "row";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|Hello world placeholder|no-assoc|no-row"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_data_seek_resets_current_seed_post_row_cursor() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_row($result);
echo $row[1];
echo "|";
echo mysqli_data_seek($result, 0) ? "seek" : "no-seek";
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["ID"];
echo "|";
echo $row["post_title"];
echo "|";
echo mysqli_data_seek($result, 1) ? "seek" : "no-seek";
echo "|";
echo mysqli_data_seek($result, -1) ? "seek" : "no-seek";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hello world placeholder|seek|1|Hello world placeholder|no-seek|no-seek"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_num_rows_counts_placeholder_rows_without_advancing_cursor() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$empty = mysqli_query($handle, "SELECT * FROM wp_posts WHERE 1 = 0");
echo mysqli_num_rows($empty);
echo "|";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_num_rows($result);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["post_title"];
echo "|";
echo mysqli_num_rows($result);
echo "|";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|1|Hello world placeholder|1|no-row");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_fetch_array_accepts_current_seed_row_modes() {
    let execution = run_source(
        r#"<?php
echo MYSQLI_ASSOC;
echo "|";
echo MYSQLI_NUM;
echo "|";
echo MYSQLI_BOTH;
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_ASSOC);
echo "|", $row["ID"];
echo "|", $row["post_title"];
echo "|";
echo mysqli_fetch_array($result, MYSQLI_ASSOC) === false ? "no-row" : "row";
echo "|";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_NUM);
echo $row[0];
echo "|";
echo $row[1];
echo "|";
echo isset($row["ID"]) ? "assoc" : "no-assoc";
echo "|";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result);
echo $row[0];
echo "|";
echo $row["ID"];
echo "|";
echo $row[1];
echo "|";
echo $row["post_title"];
echo "|";
echo mysqli_fetch_array($result) === false ? "no-row" : "row";
echo "|";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_BOTH);
echo $row[0];
echo "|";
echo $row["ID"];
echo "|";
echo $row[1];
echo "|";
echo $row["post_title"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|2|3|1|Hello world placeholder|no-row|1|Hello world placeholder|no-assoc|1|1|Hello world placeholder|Hello world placeholder|no-row|1|1|Hello world placeholder|Hello world placeholder"
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

    let unsupported_mutation = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_query($handle, "UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_mutation.phase, Phase::Runtime);
    assert_eq!(unsupported_mutation.line, 3);
    assert_eq!(unsupported_mutation.column, 1);
    assert_eq!(
        unsupported_mutation.message,
        "unsupported call mysqli_query(): mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'"
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

    let invalid_fetch_array_mode = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_fetch_array($result, 99);
"#,
    )
    .unwrap_err();

    assert_eq!(invalid_fetch_array_mode.phase, Phase::Runtime);
    assert_eq!(invalid_fetch_array_mode.line, 5);
    assert_eq!(invalid_fetch_array_mode.column, 1);
    assert_eq!(
        invalid_fetch_array_mode.message,
        "unsupported call mysqli_fetch_array(): mode must be MYSQLI_ASSOC, MYSQLI_NUM, or MYSQLI_BOTH in the current subset, got int"
    );

    let invalid_data_seek_offset = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_data_seek($result, "0");
"#,
    )
    .unwrap_err();

    assert_eq!(invalid_data_seek_offset.phase, Phase::Runtime);
    assert_eq!(invalid_data_seek_offset.line, 5);
    assert_eq!(invalid_data_seek_offset.column, 1);
    assert_eq!(
        invalid_data_seek_offset.message,
        "unsupported call mysqli_data_seek(): offset must be int in the current subset, got string"
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
fn mysqli_mutation_metadata_rejects_forms_outside_current_boundary() {
    let bad_affected_rows_handle = run_source(
        r#"<?php
mysqli_affected_rows("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_affected_rows_handle.phase, Phase::Runtime);
    assert_eq!(bad_affected_rows_handle.line, 2);
    assert_eq!(bad_affected_rows_handle.column, 1);
    assert_eq!(
        bad_affected_rows_handle.message,
        "unsupported call mysqli_affected_rows(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_insert_id_handle = run_source(
        r#"<?php
mysqli_insert_id("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_insert_id_handle.phase, Phase::Runtime);
    assert_eq!(bad_insert_id_handle.line, 2);
    assert_eq!(bad_insert_id_handle.column, 1);
    assert_eq!(
        bad_insert_id_handle.message,
        "unsupported call mysqli_insert_id(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_error_state_metadata_rejects_forms_outside_current_boundary() {
    let bad_sqlstate_handle = run_source(
        r#"<?php
mysqli_sqlstate("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_sqlstate_handle.phase, Phase::Runtime);
    assert_eq!(bad_sqlstate_handle.line, 2);
    assert_eq!(bad_sqlstate_handle.column, 1);
    assert_eq!(
        bad_sqlstate_handle.message,
        "unsupported call mysqli_sqlstate(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_warning_count_handle = run_source(
        r#"<?php
mysqli_warning_count("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_warning_count_handle.phase, Phase::Runtime);
    assert_eq!(bad_warning_count_handle.line, 2);
    assert_eq!(bad_warning_count_handle.column, 1);
    assert_eq!(
        bad_warning_count_handle.message,
        "unsupported call mysqli_warning_count(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_ping_accepts_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_ping";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_ping($handle) ? "alive" : "down";
echo "|";
echo $call($handle) ? "dynamic" : "down";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|alive|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_ping_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_ping("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_ping(): first argument must be mysqli object in the current subset, got string"
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
echo function_exists("mysqli_get_server_version") ? "1" : "0";
echo is_callable("mysqli_get_server_version") ? "1" : "0";
echo function_exists("mysqli_get_host_info") ? "1" : "0";
echo is_callable("mysqli_get_host_info") ? "1" : "0";
echo function_exists("mysqli_get_client_info") ? "1" : "0";
echo is_callable("mysqli_get_client_info") ? "1" : "0";
echo function_exists("mysqli_get_client_version") ? "1" : "0";
echo is_callable("mysqli_get_client_version") ? "1" : "0";
echo function_exists("mysqli_get_proto_info") ? "1" : "0";
echo is_callable("mysqli_get_proto_info") ? "1" : "0";
echo function_exists("mysqli_thread_id") ? "1" : "0";
echo is_callable("mysqli_thread_id") ? "1" : "0";
echo function_exists("mysqli_get_charset") ? "1" : "0";
echo is_callable("mysqli_get_charset") ? "1" : "0";
echo function_exists("mysqli_character_set_name") ? "1" : "0";
echo is_callable("mysqli_character_set_name") ? "1" : "0";
echo function_exists("mysqli_field_count") ? "1" : "0";
echo is_callable("mysqli_field_count") ? "1" : "0";
echo function_exists("mysqli_close") ? "1" : "0";
echo is_callable("mysqli_close") ? "1" : "0";
echo function_exists("mysqli_get_connection_stats") ? "1" : "0";
echo is_callable("mysqli_get_connection_stats") ? "1" : "0";
echo function_exists("mysqli_stat") ? "1" : "0";
echo is_callable("mysqli_stat") ? "1" : "0";
echo function_exists("mysqli_autocommit") ? "1" : "0";
echo is_callable("mysqli_autocommit") ? "1" : "0";
echo function_exists("mysqli_begin_transaction") ? "1" : "0";
echo is_callable("mysqli_begin_transaction") ? "1" : "0";
echo function_exists("mysqli_commit") ? "1" : "0";
echo is_callable("mysqli_commit") ? "1" : "0";
echo function_exists("mysqli_rollback") ? "1" : "0";
echo is_callable("mysqli_rollback") ? "1" : "0";
echo function_exists("mysqli_set_charset") ? "1" : "0";
echo is_callable("mysqli_set_charset") ? "1" : "0";
echo function_exists("mysqli_query") ? "1" : "0";
echo is_callable("mysqli_query") ? "1" : "0";
echo function_exists("mysqli_errno") ? "1" : "0";
echo is_callable("mysqli_errno") ? "1" : "0";
echo function_exists("mysqli_error") ? "1" : "0";
echo is_callable("mysqli_error") ? "1" : "0";
echo function_exists("mysqli_sqlstate") ? "1" : "0";
echo is_callable("mysqli_sqlstate") ? "1" : "0";
echo function_exists("mysqli_warning_count") ? "1" : "0";
echo is_callable("mysqli_warning_count") ? "1" : "0";
echo function_exists("mysqli_affected_rows") ? "1" : "0";
echo is_callable("mysqli_affected_rows") ? "1" : "0";
echo function_exists("mysqli_insert_id") ? "1" : "0";
echo is_callable("mysqli_insert_id") ? "1" : "0";
echo function_exists("mysqli_ping") ? "1" : "0";
echo is_callable("mysqli_ping") ? "1" : "0";
echo function_exists("mysqli_select_db") ? "1" : "0";
echo is_callable("mysqli_select_db") ? "1" : "0";
echo function_exists("mysqli_real_escape_string") ? "1" : "0";
echo is_callable("mysqli_real_escape_string") ? "1" : "0";
echo function_exists("mysqli_fetch_object") ? "1" : "0";
echo is_callable("mysqli_fetch_object") ? "1" : "0";
echo function_exists("mysqli_fetch_assoc") ? "1" : "0";
echo is_callable("mysqli_fetch_assoc") ? "1" : "0";
echo function_exists("mysqli_fetch_row") ? "1" : "0";
echo is_callable("mysqli_fetch_row") ? "1" : "0";
echo function_exists("mysqli_fetch_array") ? "1" : "0";
echo is_callable("mysqli_fetch_array") ? "1" : "0";
echo function_exists("mysqli_fetch_field") ? "1" : "0";
echo is_callable("mysqli_fetch_field") ? "1" : "0";
echo function_exists("mysqli_num_fields") ? "1" : "0";
echo is_callable("mysqli_num_fields") ? "1" : "0";
echo function_exists("mysqli_num_rows") ? "1" : "0";
echo is_callable("mysqli_num_rows") ? "1" : "0";
echo function_exists("mysqli_data_seek") ? "1" : "0";
echo is_callable("mysqli_data_seek") ? "1" : "0";
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
echo defined("MYSQLI_ASSOC") ? "1" : "0";
echo defined("MYSQLI_NUM") ? "1" : "0";
echo defined("MYSQLI_BOTH") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 90, "{ir}");
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
mysqli_get_server_version(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_host_info(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_charset(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_character_set_name(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_field_count(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_close(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_connection_stats(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_thread_id(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stat(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_client_info();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_client_version();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_proto_info(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_autocommit(mysqli_init(), false);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_begin_transaction(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_commit(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_rollback(mysqli_init());
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
mysqli_sqlstate(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_warning_count(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_ping(mysqli_init());
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
