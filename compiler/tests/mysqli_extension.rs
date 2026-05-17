use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn mysqli_connect_returns_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_connect";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_connect("localhost", "user", "password", "database", 3306, null);
echo "|";
echo get_class($handle);
echo "|";
echo mysqli_get_server_info($handle);
$implicit = $call();
echo "|";
echo get_class($implicit);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|mysqli|8.0.0-phpc-placeholder|mysqli"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_execute_query_runs_current_placeholder_shapes() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_execute_query";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_execute_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array(1));
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$option = $call($handle, "SELECT option_value FROM wp_options WHERE option_name = ?", array("siteurl"));
echo mysqli_num_rows($option);
echo ":";
echo mysqli_num_fields($option);
echo ":";
$option_row = mysqli_fetch_assoc($option);
echo $option_row["option_value"];
echo "|";
echo mysqli_execute_query($handle, "SET SESSION sql_mode=''") ? "no-result" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|1:Hello world placeholder|1:1:https://example.test|no-result"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_execute_query_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_execute_query("not-a-handle", "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array(1));
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_execute_query(): first argument must be mysqli object in the current subset, got string"
    );

    let named_params = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_execute_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array("id" => 1));
"#,
    )
    .unwrap_err();

    assert_eq!(named_params.phase, Phase::Runtime);
    assert_eq!(named_params.line, 3);
    assert_eq!(named_params.column, 1);
    assert_eq!(
        named_params.message,
        "unsupported call mysqli_execute_query(): params array must be a list in the current subset"
    );

    let param_count = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_execute_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array());
"#,
    )
    .unwrap_err();

    assert_eq!(param_count.phase, Phase::Runtime);
    assert_eq!(param_count.line, 3);
    assert_eq!(param_count.column, 1);
    assert_eq!(
        param_count.message,
        "unsupported call mysqli_execute_query(): params array length must match query placeholder count 1, got 0"
    );

    let unsupported_select = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_execute_query($handle, "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_select.phase, Phase::Runtime);
    assert_eq!(unsupported_select.line, 3);
    assert_eq!(unsupported_select.column, 1);
    assert_eq!(
        unsupported_select.message,
        "unsupported call mysqli_execute_query(): statement result metadata is implemented only for current WordPress placeholder SELECT shapes"
    );
}

#[test]
fn mysqli_connect_rejects_forms_outside_current_placeholder_boundary() {
    let username_error = run_source(
        r#"<?php
mysqli_connect("localhost", array(), "password", "database");
"#,
    )
    .unwrap_err();

    assert_eq!(username_error.phase, Phase::Runtime);
    assert_eq!(username_error.line, 2);
    assert_eq!(username_error.column, 1);
    assert_eq!(
        username_error.message,
        "unsupported call mysqli_connect(): username argument must be string or null in the current subset, got array"
    );

    let port_error = run_source(
        r#"<?php
mysqli_connect("localhost", "user", "password", "database", "3306");
"#,
    )
    .unwrap_err();

    assert_eq!(port_error.phase, Phase::Runtime);
    assert_eq!(port_error.line, 2);
    assert_eq!(port_error.column, 1);
    assert_eq!(
        port_error.message,
        "unsupported call mysqli_connect(): port argument must be int or null in the current subset, got string"
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
mysqli_ssl_set($handle, null, null, null, null, null);
mysqli_options($handle, MYSQLI_OPT_SSL_VERIFY_SERVER_CERT, true);
echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, MYSQLI_CLIENT_SSL | MYSQLI_CLIENT_FOUND_ROWS | MYSQLI_CLIENT_IGNORE_SPACE) ? "ssl-flags" : "failed";
echo "|";
echo MYSQLI_CLIENT_SSL, ":", MYSQLI_CLIENT_COMPRESS, ":", MYSQLI_CLIENT_INTERACTIVE, ":", MYSQLI_CLIENT_IGNORE_SPACE, ":", MYSQLI_CLIENT_NO_SCHEMA, ":", MYSQLI_CLIENT_FOUND_ROWS, ":", MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT, ":", MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT, ":", MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS;
echo "|";
echo $call($handle, null, null, null, null, null, null, 0) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|connected|0|null|ssl-flags|2048:32:1024:256:16:2:1073741824:64:4194304|dynamic"
    );
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
$kill = "mysqli_kill";
echo function_exists($client) ? "yes" : "no";
echo "|";
echo is_callable($version) ? "version-callable" : "version-missing";
echo "|";
echo is_callable($proto) ? "callable" : "missing";
echo "|";
echo is_callable($thread) ? "thread-callable" : "thread-missing";
echo "|";
echo is_callable($kill) ? "kill-callable" : "kill-missing";
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
echo "|";
echo mysqli_kill($handle, mysqli_thread_id($handle)) ? "killed" : "missing-thread";
echo "|";
echo $kill($handle, 99) ? "unexpected-thread" : "no-thread";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|version-callable|callable|thread-callable|kill-callable|mysqlnd 8.0.0-phpc-placeholder|mysqlnd 8.0.0-phpc-placeholder|mysqlnd 8.0.0-phpc-placeholder|80000|80000|10|10|1|1|killed|no-thread"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_change_user_accepts_current_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_change_user";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_change_user($handle, "user", "pass", "wordpress") ? "changed" : "failed";
echo "|";
echo $call($handle, "user", "pass", null) ? "changed-null-db" : "failed";
echo "|";
echo mysqli_ping($handle) ? "still-open" : "closed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|changed|changed-null-db|still-open"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_refresh_accepts_current_placeholder_flags() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_refresh";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo MYSQLI_REFRESH_GRANT;
echo "|";
echo MYSQLI_REFRESH_REPLICA === MYSQLI_REFRESH_SLAVE ? "alias" : "different";
echo "|";
echo mysqli_refresh($handle, MYSQLI_REFRESH_LOG | MYSQLI_REFRESH_TABLES) ? "refreshed" : "failed";
echo "|";
echo $call($handle, MYSQLI_REFRESH_STATUS | MYSQLI_REFRESH_THREADS | MYSQLI_REFRESH_BACKUP_LOG) ? "dynamic" : "failed";
echo "|";
echo mysqli_ping($handle) ? "still-open" : "closed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|1|alias|refreshed|dynamic|still-open"
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
fn mysqli_links_stats_return_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_get_links_stats";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$stats = mysqli_get_links_stats();
echo "|", $stats["total"];
echo "|", $stats["active_plinks"];
echo "|", $stats["cached_plinks"];
echo "|";
$dynamic = $call();
echo $dynamic["total"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|0|0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_client_stats_return_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_get_client_stats";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$stats = mysqli_get_client_stats();
echo "|", $stats["bytes_sent"];
echo "|", $stats["bytes_received"];
echo "|", $stats["packets_sent"];
echo "|", $stats["packets_received"];
echo "|", $stats["protocol_overhead_in"];
echo "|", $stats["protocol_overhead_out"];
echo "|", $stats["connect_success"];
echo "|", $stats["active_connections"];
echo "|";
$dynamic = $call();
echo $dynamic["bytes_sent"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|0|0|0|0|0|0|0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_thread_safe_returns_current_placeholder_metadata() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_thread_safe";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo mysqli_thread_safe() ? "thread-safe" : "not-safe";
echo "|";
echo $call() ? "dynamic" : "not-safe";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|thread-safe|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_lifecycle_is_visible_but_an_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$stmt_init = "mysqli_stmt_init";
$prepare = "mysqli_prepare";
echo function_exists($stmt_init) ? "yes" : "no";
echo "|";
echo is_callable($stmt_init) ? "stmt-callable" : "stmt-missing";
echo "|";
echo function_exists($prepare) ? "prepare-exists" : "prepare-missing";
echo "|";
echo is_callable($prepare) ? "prepare-callable" : "prepare-missing";
$handle = mysqli_init();
$stmt = mysqli_stmt_init($handle);
echo "|";
echo get_class($stmt);
echo "|";
$prepared = mysqli_prepare($handle, "SELECT option_value FROM wp_options WHERE option_name = ?");
echo get_class($prepared);
echo "|";
echo mysqli_stmt_param_count($prepared);
echo "|";
echo mysqli_stmt_close($stmt) ? "closed" : "open";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|stmt-callable|prepare-exists|prepare-callable|mysqli_stmt|mysqli_stmt|1|closed"
    );
    assert_eq!(execution.exit_code, 0);

    let stmt_error = run_source(
        r#"<?php
mysqli_stmt_init("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(stmt_error.phase, Phase::Runtime);
    assert_eq!(stmt_error.line, 2);
    assert_eq!(stmt_error.column, 1);
    assert_eq!(
        stmt_error.message,
        "unsupported call mysqli_stmt_init(): first argument must be mysqli object in the current subset, got string"
    );

    let prepare_error = run_source(
        r#"<?php
mysqli_prepare("not-a-handle", "SELECT option_value FROM wp_options WHERE option_name = ?");
"#,
    )
    .unwrap_err();

    assert_eq!(prepare_error.phase, Phase::Runtime);
    assert_eq!(prepare_error.line, 2);
    assert_eq!(prepare_error.column, 1);
    assert_eq!(
        prepare_error.message,
        "unsupported call mysqli_prepare(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_statement_prepare_param_and_diagnostic_lists_are_visible_but_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
$prepare = "mysqli_stmt_prepare";
$param_count = "mysqli_stmt_param_count";
$get_warnings = "mysqli_stmt_get_warnings";
$error_list = "mysqli_stmt_error_list";
echo function_exists($prepare) ? "yes" : "no";
echo "|";
echo is_callable($prepare) ? "prepare-callable" : "prepare-missing";
echo "|";
echo function_exists($param_count) ? "param-count-exists" : "param-count-missing";
echo "|";
echo is_callable($param_count) ? "param-count-callable" : "param-count-missing";
echo "|";
echo function_exists($get_warnings) ? "warnings-exists" : "warnings-missing";
echo "|";
echo is_callable($get_warnings) ? "warnings-callable" : "warnings-missing";
echo "|";
echo function_exists($error_list) ? "error-list-exists" : "error-list-missing";
echo "|";
echo is_callable($error_list) ? "error-list-callable" : "error-list-missing";
$handle = mysqli_init();
$stmt = mysqli_stmt_init($handle);
echo "|";
echo mysqli_stmt_prepare($stmt, "SELECT option_value FROM wp_options WHERE option_name = ?") ? "prepared" : "failed";
echo "|";
echo mysqli_stmt_param_count($stmt);
echo "|";
mysqli_stmt_prepare($stmt, "SELECT option_name FROM wp_options");
echo mysqli_stmt_param_count($stmt);
echo "|";
echo mysqli_stmt_get_warnings($stmt) === false ? "no-warnings" : "warnings";
echo "|";
echo count(mysqli_stmt_error_list($stmt));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|prepare-callable|param-count-exists|param-count-callable|warnings-exists|warnings-callable|error-list-exists|error-list-callable|prepared|1|0|no-warnings|0"
    );
    assert_eq!(execution.exit_code, 0);

    let prepare_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_prepare($stmt, "SELECT option_value FROM wp_options WHERE option_name = ?");
"#,
    )
    .unwrap_err();

    assert_eq!(prepare_error.phase, Phase::Runtime);
    assert_eq!(prepare_error.line, 3);
    assert_eq!(prepare_error.column, 1);
    assert_eq!(
        prepare_error.message,
        "unsupported call mysqli_stmt_prepare(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let param_count_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_param_count($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(param_count_error.phase, Phase::Runtime);
    assert_eq!(param_count_error.line, 3);
    assert_eq!(param_count_error.column, 1);
    assert_eq!(
        param_count_error.message,
        "unsupported call mysqli_stmt_param_count(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let warnings_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_get_warnings($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(warnings_error.phase, Phase::Runtime);
    assert_eq!(warnings_error.line, 3);
    assert_eq!(warnings_error.column, 1);
    assert_eq!(
        warnings_error.message,
        "unsupported call mysqli_stmt_get_warnings(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let error_list_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_error_list($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(error_list_error.phase, Phase::Runtime);
    assert_eq!(error_list_error.line, 3);
    assert_eq!(error_list_error.column, 1);
    assert_eq!(
        error_list_error.message,
        "unsupported call mysqli_stmt_error_list(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_statement_bind_param_and_execute_have_placeholder_state() {
    let execution = run_source(
        r#"<?php
$bind = "mysqli_stmt_bind_param";
$execute = "mysqli_stmt_execute";
$execute_alias = "mysqli_execute";
echo function_exists($bind) ? "yes" : "no";
echo "|";
echo is_callable($bind) ? "bind-callable" : "bind-missing";
echo "|";
echo function_exists($execute) ? "execute-exists" : "execute-missing";
echo "|";
echo is_callable($execute) ? "execute-callable" : "execute-missing";
echo "|";
echo function_exists($execute_alias) ? "alias-exists" : "alias-missing";
echo "|";
echo is_callable($execute_alias) ? "alias-callable" : "alias-missing";
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id = 2;
echo "|";
echo mysqli_stmt_bind_param($stmt, "i", $id) ? "bound" : "not-bound";
$id = 1;
echo "|";
echo mysqli_stmt_execute($stmt) ? "executed" : "not-executed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
$stmt2 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id2 = 2;
mysqli_stmt_bind_param($stmt2, "i", $id2);
$id2 = 1;
echo "|";
echo call_user_func("mysqli_stmt_execute", $stmt2) ? "call-user-func" : "not-executed";
$result2 = mysqli_stmt_get_result($stmt2);
$row2 = mysqli_fetch_assoc($result2);
echo "|";
echo $row2["ID"], ":", $row2["post_title"];
$stmt3 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id3 = 2;
mysqli_stmt_bind_param($stmt3, "i", $id3);
$id3 = 1;
echo "|";
echo call_user_func_array("mysqli_stmt_execute", array($stmt3)) ? "call-user-func-array" : "not-executed";
$result3 = mysqli_stmt_get_result($stmt3);
$row3 = mysqli_fetch_assoc($result3);
echo "|";
echo $row3["ID"], ":", $row3["post_title"];
$stmt4 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo "|";
echo mysqli_stmt_execute($stmt4, array(1)) ? "array-executed" : "not-executed";
$result4 = mysqli_stmt_get_result($stmt4);
$row4 = mysqli_fetch_assoc($result4);
echo "|";
echo $row4["ID"], ":", $row4["post_title"];
$stmt6 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo "|";
echo mysqli_execute($stmt6, array(1)) ? "alias-executed" : "not-executed";
$result6 = mysqli_stmt_get_result($stmt6);
$row6 = mysqli_fetch_assoc($result6);
echo "|";
echo $row6["ID"], ":", $row6["post_title"];
$stmt7 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id7 = 2;
mysqli_stmt_bind_param($stmt7, "i", $id7);
$id7 = 1;
echo "|";
echo call_user_func("mysqli_execute", $stmt7) ? "alias-call-user-func" : "not-executed";
$result7 = mysqli_stmt_get_result($stmt7);
$row7 = mysqli_fetch_assoc($result7);
echo "|";
echo $row7["ID"], ":", $row7["post_title"];
$stmt8 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id8 = 2;
mysqli_stmt_bind_param($stmt8, "i", $id8);
$id8 = 1;
echo "|";
echo call_user_func_array("mysqli_execute", array($stmt8)) ? "alias-call-user-func-array" : "not-executed";
$result8 = mysqli_stmt_get_result($stmt8);
$row8 = mysqli_fetch_assoc($result8);
echo "|";
echo $row8["ID"], ":", $row8["post_title"];
$stmt5 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$blob = "unused";
echo "|";
echo mysqli_stmt_bind_param($stmt5, "b", $blob) ? "blob-bound" : "not-bound";
mysqli_stmt_send_long_data($stmt5, 0, "1");
echo "|";
echo mysqli_stmt_execute($stmt5) ? "blob-executed" : "not-executed";
$result5 = mysqli_stmt_get_result($stmt5);
$row5 = mysqli_fetch_assoc($result5);
echo "|";
echo $row5["ID"], ":", $row5["post_title"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|bind-callable|execute-exists|execute-callable|alias-exists|alias-callable|bound|executed|1:Hello world placeholder|call-user-func|1:Hello world placeholder|call-user-func-array|1:Hello world placeholder|array-executed|1:Hello world placeholder|alias-executed|1:Hello world placeholder|alias-call-user-func|1:Hello world placeholder|alias-call-user-func-array|1:Hello world placeholder|blob-bound|blob-executed|1:Hello world placeholder"
    );
    assert_eq!(execution.exit_code, 0);

    let bind_error = run_source(
        r#"<?php
$stmt = mysqli_init();
$value = "home";
mysqli_stmt_bind_param($stmt, "s", $value);
"#,
    )
    .unwrap_err();

    assert_eq!(bind_error.phase, Phase::Runtime);
    assert_eq!(bind_error.line, 4);
    assert_eq!(bind_error.column, 1);
    assert_eq!(
        bind_error.message,
        "unsupported call mysqli_stmt_bind_param(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let execute_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_execute($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(execute_error.phase, Phase::Runtime);
    assert_eq!(execute_error.line, 3);
    assert_eq!(execute_error.column, 1);
    assert_eq!(
        execute_error.message,
        "unsupported call mysqli_stmt_execute(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let execute_alias_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_execute($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(execute_alias_error.phase, Phase::Runtime);
    assert_eq!(execute_alias_error.line, 3);
    assert_eq!(execute_alias_error.column, 1);
    assert_eq!(
        execute_alias_error.message,
        "unsupported call mysqli_execute(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let parameter_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT option_value FROM wp_options WHERE option_name = ?");
mysqli_stmt_execute($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(parameter_error.phase, Phase::Runtime);
    assert_eq!(parameter_error.line, 3);
    assert_eq!(parameter_error.column, 1);
    assert_eq!(
        parameter_error.message,
        "unsupported call mysqli_stmt_execute(): bound parameter values are not available for the current placeholder statement"
    );

    let type_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id = 1;
mysqli_stmt_bind_param($stmt, "x", $id);
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 4);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_bind_param(): only s, i, d, and b parameter type markers are implemented in the current subset, got x"
    );

    let variable_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_bind_param($stmt, "i", 1);
"#,
    )
    .unwrap_err();

    assert_eq!(variable_error.phase, Phase::Runtime);
    assert_eq!(variable_error.line, 3);
    assert_eq!(variable_error.column, 36);
    assert_eq!(
        variable_error.message,
        "unsupported call mysqli_stmt_bind_param(): parameter bindings must be direct variables in the current subset"
    );

    let params_array_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array("id" => 1));
"#,
    )
    .unwrap_err();

    assert_eq!(params_array_error.phase, Phase::Runtime);
    assert_eq!(params_array_error.line, 3);
    assert_eq!(params_array_error.column, 1);
    assert_eq!(
        params_array_error.message,
        "unsupported call mysqli_stmt_execute(): params array must be a list in the current subset"
    );

    let sparse_params_array_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array(1 => 1));
"#,
    )
    .unwrap_err();

    assert_eq!(sparse_params_array_error.phase, Phase::Runtime);
    assert_eq!(sparse_params_array_error.line, 3);
    assert_eq!(sparse_params_array_error.column, 1);
    assert_eq!(
        sparse_params_array_error.message,
        "unsupported call mysqli_stmt_execute(): params array must be a list in the current subset"
    );

    let alias_params_array_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_execute($stmt, array("id" => 1));
"#,
    )
    .unwrap_err();

    assert_eq!(alias_params_array_error.phase, Phase::Runtime);
    assert_eq!(alias_params_array_error.line, 3);
    assert_eq!(alias_params_array_error.column, 1);
    assert_eq!(
        alias_params_array_error.message,
        "unsupported call mysqli_execute(): params array must be a list in the current subset"
    );
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_value_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "SELECT option_value FROM wp_options WHERE option_name = ?");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
$result = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($result);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
$name = "home";
echo "|";
echo mysqli_stmt_execute($stmt) ? "missing-executed" : "failed";
$missing = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|1|https://example.test|missing-executed|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_row_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name = ?");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
$result = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($result);
echo ":";
echo mysqli_num_fields($result);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["option_name"], "=", $row["option_value"];
$name = "home";
echo "|";
echo mysqli_stmt_execute($stmt) ? "missing-executed" : "failed";
$missing = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($missing), ":", mysqli_num_fields($missing);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name = ?", array("siteurl"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_name"], "=", $direct_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|1:2|siteurl=https://example.test|missing-executed|0:0|siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_name_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN (?, ?, ?)");
$first = "theme_mods";
$second = "missing";
$third = "siteurl";
mysqli_stmt_bind_param($stmt, "sss", $first, $second, $third);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
$one = mysqli_fetch_assoc($result);
$two = mysqli_fetch_assoc($result);
echo $one["option_name"], "=", $one["option_value"];
echo "|";
echo $two["option_name"], "=", $two["option_value"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` IN (?, ?, ?)", array("home", "missing", "theme_mods"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
$direct_one = mysqli_fetch_assoc($direct);
$direct_two = mysqli_fetch_assoc($direct);
echo $direct_one["option_name"], ":", $direct_one["autoload"];
echo "|";
echo $direct_two["option_name"], ":", $direct_two["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:2|theme_mods=theme-db|siteurl=https://example.test|2:3|home:no|theme_mods:on"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_full_row_name_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name IN (?, ?, ?)");
$first = "theme_mods";
$second = "missing";
$third = "siteurl";
mysqli_stmt_bind_param($stmt, "sss", $first, $second, $third);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
$one = mysqli_fetch_assoc($result);
$two = mysqli_fetch_assoc($result);
echo $one["option_id"], ":", $one["option_name"], ":", $one["autoload"];
echo "|";
echo $two["option_id"], ":", $two["option_name"], ":", $two["autoload"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` IN (?, ?, ?)", array("home", "missing", "theme_mods"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
$direct_one = mysqli_fetch_assoc($direct);
$direct_two = mysqli_fetch_assoc($direct);
echo $direct_one["option_id"], ":", $direct_one["option_name"], ":", $direct_one["autoload"];
echo "|";
echo $direct_two["option_id"], ":", $direct_two["option_name"], ":", $direct_two["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:4|3:theme_mods:on|1:siteurl:yes|2:4|2:home:no|3:theme_mods:on"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_autoload_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload IN (?, ?)");
$first = "yes";
$second = "on";
mysqli_stmt_bind_param($stmt, "ss", $first, $second);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
$one = mysqli_fetch_assoc($result);
$two = mysqli_fetch_assoc($result);
echo $one["option_name"], "=", $one["option_value"];
echo "|";
echo $two["option_name"], "=", $two["option_value"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `autoload` IN (?, ?)", array("on", "yes"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
$direct_one = mysqli_fetch_assoc($direct);
$direct_two = mysqli_fetch_assoc($direct);
echo $direct_one["option_id"], ":", $direct_one["option_name"], ":", $direct_one["autoload"];
echo "|";
echo $direct_two["option_id"], ":", $direct_two["option_name"], ":", $direct_two["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:2|siteurl=https://example.test|theme_mods=theme-db|2:4|1:siteurl:yes|3:theme_mods:on"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_name_autoload_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_name, autoload FROM wp_options WHERE option_name IN (?, ?, ?)");
$first = "theme_mods";
$second = "missing";
$third = "siteurl";
mysqli_stmt_bind_param($stmt, "sss", $first, $second, $third);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
$one = mysqli_fetch_assoc($result);
$two = mysqli_fetch_assoc($result);
echo $one["option_name"], ":", $one["autoload"];
echo "|";
echo $two["option_name"], ":", $two["autoload"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_name`, `autoload` FROM `wp_options` WHERE `autoload` IN (?, ?)", array("on", "yes"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
$direct_one = mysqli_fetch_assoc($direct);
$direct_two = mysqli_fetch_assoc($direct);
echo $direct_one["option_name"], ":", $direct_one["autoload"];
echo "|";
echo $direct_two["option_name"], ":", $direct_two["autoload"];
echo "|";
$literal = mysqli_query($handle, "SELECT option_name, autoload FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
echo mysqli_num_rows($literal), ":", mysqli_num_fields($literal);
echo "|";
$literal_one = mysqli_fetch_assoc($literal);
$literal_two = mysqli_fetch_assoc($literal);
echo $literal_one["option_name"], ":", $literal_one["autoload"];
echo "|";
echo $literal_two["option_name"], ":", $literal_two["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:2|theme_mods:on|siteurl:yes|2:2|siteurl:yes|theme_mods:on|2:2|siteurl:yes|theme_mods:on"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_name_only_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_name FROM wp_options WHERE option_name IN (?, ?, ?)");
$first = "theme_mods";
$second = "missing";
$third = "siteurl";
mysqli_stmt_bind_param($stmt, "sss", $first, $second, $third);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
echo mysqli_fetch_column($result);
echo ",";
echo mysqli_fetch_column($result);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_name` FROM `wp_options` WHERE `autoload` IN (?, ?)", array("on", "yes"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
echo mysqli_fetch_column($direct);
echo ",";
echo mysqli_fetch_column($direct);
echo "|";
$literal = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
echo mysqli_num_rows($literal), ":", mysqli_num_fields($literal);
echo "|";
echo mysqli_fetch_column($literal);
echo ",";
echo mysqli_fetch_column($literal);
echo "|";
$all_stmt = mysqli_prepare($handle, "SELECT option_name FROM wp_options");
mysqli_stmt_execute($all_stmt);
$all = mysqli_stmt_get_result($all_stmt);
echo mysqli_num_rows($all), ":", mysqli_num_fields($all);
echo "|";
echo mysqli_fetch_column($all);
echo ",";
echo mysqli_fetch_column($all);
echo ",";
echo mysqli_fetch_column($all);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:1|theme_mods,siteurl|2:1|siteurl,theme_mods|2:1|siteurl,theme_mods|3:1|home,siteurl,theme_mods"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_value_only_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_value FROM wp_options WHERE option_name IN (?, ?, ?)");
$first = "theme_mods";
$second = "missing";
$third = "siteurl";
mysqli_stmt_bind_param($stmt, "sss", $first, $second, $third);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo mysqli_num_rows($result), ":", mysqli_num_fields($result);
echo "|";
echo mysqli_fetch_column($result);
echo ",";
echo mysqli_fetch_column($result);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_value` FROM `wp_options` WHERE `autoload` IN (?, ?)", array("on", "yes"));
echo mysqli_num_rows($direct), ":", mysqli_num_fields($direct);
echo "|";
echo mysqli_fetch_column($direct);
echo ",";
echo mysqli_fetch_column($direct);
echo "|";
$literal = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
echo mysqli_num_rows($literal), ":", mysqli_num_fields($literal);
echo "|";
echo mysqli_fetch_column($literal);
echo ",";
echo mysqli_fetch_column($literal);
echo "|";
$all_stmt = mysqli_prepare($handle, "SELECT option_value FROM wp_options");
mysqli_stmt_execute($all_stmt);
$all = mysqli_stmt_get_result($all_stmt);
echo mysqli_num_rows($all), ":", mysqli_num_fields($all);
echo "|";
echo mysqli_fetch_column($all);
echo ",";
echo mysqli_fetch_column($all);
echo ",";
echo mysqli_fetch_column($all);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|2:1|theme-db,https://example.test|2:1|https://example.test,theme-db|2:1|https://example.test,theme-db|3:1|https://home.test,https://example.test,theme-db"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_rejects_non_string_wordpress_option_name_list_parameters() {
    let error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN (?, ?)");
$one = "siteurl";
$two = 123;
mysqli_stmt_bind_param($stmt, "ss", $one, $two);
mysqli_stmt_execute($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 8);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options option-name-list select requires string option name parameters in the current subset"
    );
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_name_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "SELECT option_name FROM wp_options WHERE option_name = ? LIMIT 1");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
$result = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($result);
echo ":";
echo mysqli_num_fields($result);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["option_name"];
$name = "home";
echo "|";
echo mysqli_stmt_execute($stmt) ? "missing-executed" : "failed";
$missing = mysqli_stmt_get_result($stmt);
echo "|";
echo mysqli_num_rows($missing), ":", mysqli_num_fields($missing);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `option_name` FROM `wp_options` WHERE `option_name` = ? LIMIT 1", array("siteurl"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|1:1|siteurl|missing-executed|0:0|siteurl"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_inserts_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$stmt = mysqli_prepare($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)");
$name = "siteurl";
$value = "https://example.test";
$autoload = "yes";
mysqli_stmt_bind_param($stmt, "sss", $name, $value, $autoload);
echo mysqli_stmt_execute($stmt) ? "inserted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
$name = "home";
$value = "https://home.test";
$autoload = "no";
echo "|";
echo mysqli_stmt_execute($stmt) ? "inserted-again" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inserted|1|1|1|https://example.test|inserted-again|1|2|https://home.test"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)");
mysqli_stmt_execute($stmt, array("blog_public", 1, "yes"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 3);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options insert requires string option name, option value, and autoload parameters in the current subset"
    );
}

#[test]
fn mysqli_statement_replaces_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)");
$name = "siteurl";
$value = "https://replaced.test";
$autoload = "no";
mysqli_stmt_bind_param($stmt, "sss", $name, $value, $autoload);
echo mysqli_stmt_execute($stmt) ? "replaced" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
$name = "home";
$value = "https://home.test";
$autoload = "yes";
echo "|";
echo mysqli_stmt_execute($stmt) ? "inserted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "replaced|2|2|2|https://replaced.test|inserted|1|3|https://home.test"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)");
mysqli_stmt_execute($stmt, array("blog_public", 1, "yes"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 3);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options replace requires string option name, option value, and autoload parameters in the current subset"
    );
}

#[test]
fn mysqli_statement_upserts_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE option_name = VALUES(option_name), option_value = VALUES(option_value), autoload = VALUES(autoload)");
$name = "siteurl";
$value = "https://upserted.test";
$autoload = "no";
mysqli_stmt_bind_param($stmt, "sss", $name, $value, $autoload);
echo mysqli_stmt_execute($stmt) ? "updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
$name = "home";
$value = "https://home.test";
$autoload = "yes";
echo "|";
echo mysqli_stmt_execute($stmt) ? "inserted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|2|2|2|https://upserted.test|inserted|1|3|https://home.test"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload)");
mysqli_stmt_execute($stmt, array("blog_public", 1, "yes"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 3);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options insert-on-duplicate requires string option name, option value, and autoload parameters in the current subset"
    );
}

#[test]
fn mysqli_execute_query_upserts_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$query = "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE `option_value` = VALUES(`option_value`), `autoload` = VALUES(`autoload`)";
echo mysqli_execute_query($handle, $query, array("siteurl", "https://execute-query.test", "no")) ? "updated" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$siteurl = mysqli_execute_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1", array("siteurl"));
$siteurl_row = mysqli_fetch_assoc($siteurl);
echo $siteurl_row["option_value"], ":", $siteurl_row["autoload"];
echo "|";
echo mysqli_execute_query($handle, $query, array("home", "https://home.test", "yes")) ? "inserted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = 'home'");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_id"], ":", $home_row["option_name"], ":", $home_row["option_value"], ":", $home_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|2|2|https://execute-query.test:no|inserted|1|3|3:home:https://home.test:yes"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_execute_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload)", array("blog_public", 1, "yes"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 3);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_execute_query(): prepared wp_options insert-on-duplicate requires string option name, option value, and autoload parameters in the current subset"
    );
}

#[test]
fn mysqli_execute_query_mutates_current_wordpress_prepared_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_execute_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)", array("blogname", "Before", "yes")) ? "inserted" : "insert-failed";
echo ":";
echo mysqli_affected_rows($handle), ":", mysqli_insert_id($handle);
echo "|";
echo mysqli_execute_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)", array("blogname", "Duplicate", "no")) ? "duplicate" : "duplicate-rejected";
echo ":";
echo mysqli_affected_rows($handle), ":", mysqli_insert_id($handle);
echo "|";
echo mysqli_execute_query($handle, "UPDATE `wp_options` SET `option_value` = ?, `autoload` = ? WHERE `option_name` = ?", array("After", "auto-on", "blogname")) ? "updated" : "update-failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "UPDATE wp_options SET option_value = ? WHERE option_name = ?", array("ValueOnly", "blogname")) ? "value-only" : "value-failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "UPDATE wp_options SET autoload = ? WHERE option_name = ?", array("no", "missing")) ? "missing-autoload" : "autoload-failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$row_result = mysqli_execute_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1", array("blogname"));
$row = mysqli_fetch_assoc($row_result);
echo $row["option_value"], ":", $row["autoload"];
echo "|";
echo mysqli_execute_query($handle, "REPLACE INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES (?, ?, ?)", array("blogname", "Replaced", "yes")) ? "replaced" : "replace-failed";
echo ":";
echo mysqli_affected_rows($handle), ":", mysqli_insert_id($handle);
echo "|";
echo mysqli_execute_query($handle, "DELETE FROM wp_options WHERE option_name = ?", array("blogname")) ? "deleted" : "delete-failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$missing = mysqli_execute_query($handle, "SELECT option_value FROM wp_options WHERE option_name = ? LIMIT 1", array("blogname"));
echo mysqli_fetch_assoc($missing) ? "still-present" : "gone";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inserted:1:1|duplicate-rejected:0:1|updated:1|value-only:1|missing-autoload:0|ValueOnly:auto-on|replaced:2:2|deleted:1|gone"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_execute_query($handle, "UPDATE wp_options SET option_value = ? WHERE option_name = ?", array(1, "blogname"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 3);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_execute_query(): prepared wp_options update requires string option value and option name parameters in the current subset"
    );
}

#[test]
fn mysqli_statement_updates_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "UPDATE wp_options SET option_value = ? WHERE option_name = ?");
$value = "https://updated.test";
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "ss", $value, $name);
echo mysqli_stmt_execute($stmt) ? "updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
$name = "home";
echo "|";
echo mysqli_stmt_execute($stmt) ? "missing-updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|1|1|https://updated.test|missing-updated|0"
    );
    assert_eq!(execution.exit_code, 0);

    let no_state = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "UPDATE wp_options SET option_value = ? WHERE option_name = ?");
mysqli_stmt_execute($stmt, array("1", "blog_public"));
"#,
    )
    .unwrap_err();

    assert_eq!(no_state.phase, Phase::Runtime);
    assert_eq!(no_state.line, 3);
    assert_eq!(no_state.column, 1);
    assert_eq!(
        no_state.message,
        "unsupported call mysqli_stmt_execute(): statement mutation execution and host database state are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_updates_current_wordpress_option_value_and_autoload_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'before', 'no')");
$stmt = mysqli_prepare($handle, "UPDATE `wp_options` SET `option_value` = ?, `autoload` = ? WHERE `option_name` = ?");
$value = "after";
$autoload = "auto-on";
$name = "blogdescription";
mysqli_stmt_bind_param($stmt, "sss", $value, $autoload, $name);
echo mysqli_stmt_execute($stmt) ? "updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"], ":", $row["autoload"];
echo "|";
$name = "missing";
echo mysqli_stmt_execute($stmt) ? "missing-updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|1|1|after:auto-on|missing-updated|0"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'before', 'no')");
$stmt = mysqli_prepare($handle, "UPDATE wp_options SET option_value = ?, autoload = ? WHERE option_name = ?");
mysqli_stmt_execute($stmt, array("after", 1, "blogdescription"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 6);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options value/autoload update requires string option value, autoload, and option name parameters in the current subset"
    );
}

#[test]
fn mysqli_statement_updates_current_wordpress_option_autoload_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'value-kept', 'no')");
$stmt = mysqli_prepare($handle, "UPDATE `wp_options` SET `autoload` = ? WHERE `option_name` = ?");
$autoload = "auto-on";
$name = "blogdescription";
mysqli_stmt_bind_param($stmt, "ss", $autoload, $name);
echo mysqli_stmt_execute($stmt) ? "updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"], ":", $row["autoload"];
echo "|";
$name = "missing";
echo mysqli_stmt_execute($stmt) ? "missing-updated" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|1|1|value-kept:auto-on|missing-updated|0"
    );
    assert_eq!(execution.exit_code, 0);

    let type_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'before', 'no')");
$stmt = mysqli_prepare($handle, "UPDATE wp_options SET autoload = ? WHERE option_name = ?");
mysqli_stmt_execute($stmt, array(1, "blogdescription"));
"#,
    )
    .unwrap_err();

    assert_eq!(type_error.phase, Phase::Runtime);
    assert_eq!(type_error.line, 6);
    assert_eq!(type_error.column, 1);
    assert_eq!(
        type_error.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options autoload update requires string autoload and option name parameters in the current subset"
    );

    let no_state = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "UPDATE wp_options SET autoload = ? WHERE option_name = ?");
mysqli_stmt_execute($stmt, array("yes", "blog_public"));
"#,
    )
    .unwrap_err();

    assert_eq!(no_state.phase, Phase::Runtime);
    assert_eq!(no_state.line, 3);
    assert_eq!(no_state.column, 1);
    assert_eq!(
        no_state.message,
        "unsupported call mysqli_stmt_execute(): statement mutation execution and host database state are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_deletes_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name = ?");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "deleted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
echo mysqli_num_rows($result);
$name = "home";
echo "|";
echo mysqli_stmt_execute($stmt) ? "missing-deleted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "deleted|1|1|0|missing-deleted|0");
    assert_eq!(execution.exit_code, 0);

    let no_state = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "DELETE FROM wp_options WHERE option_name = ?");
mysqli_stmt_execute($stmt, array("blog_public"));
"#,
    )
    .unwrap_err();

    assert_eq!(no_state.phase, Phase::Runtime);
    assert_eq!(no_state.line, 3);
    assert_eq!(no_state.column, 1);
    assert_eq!(
        no_state.message,
        "unsupported call mysqli_stmt_execute(): statement mutation execution and host database state are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_bind_result_has_direct_variable_placeholder_state() {
    let execution = run_source(
        r#"<?php
$bind_result = "mysqli_stmt_bind_result";
echo function_exists($bind_result) ? "yes" : "no";
echo "|";
echo is_callable($bind_result) ? "callable" : "missing";
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$id = null;
$title = null;
echo "|";
echo mysqli_stmt_bind_result($stmt, $id, $title) ? "bound" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|bound");
    assert_eq!(execution.exit_code, 0);

    let arity_error = run_source(
        r#"<?php
mysqli_stmt_bind_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(arity_error.phase, Phase::Runtime);
    assert_eq!(arity_error.line, 2);
    assert_eq!(arity_error.column, 1);
    assert_eq!(
        arity_error.message,
        "arity mismatch for mysqli_stmt_bind_result(): expected at least 2 argument(s), got 1"
    );

    let bind_error = run_source(
        r#"<?php
$stmt = mysqli_init();
$title = null;
mysqli_stmt_bind_result($stmt, $title);
"#,
    )
    .unwrap_err();

    assert_eq!(bind_error.phase, Phase::Runtime);
    assert_eq!(bind_error.line, 4);
    assert_eq!(bind_error.column, 1);
    assert_eq!(
        bind_error.message,
        "unsupported call mysqli_stmt_bind_result(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let variable_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_bind_result($stmt, $id, "title");
"#,
    )
    .unwrap_err();

    assert_eq!(variable_error.phase, Phase::Runtime);
    assert_eq!(variable_error.line, 3);
    assert_eq!(variable_error.column, 37);
    assert_eq!(
        variable_error.message,
        "unsupported call mysqli_stmt_bind_result(): result bindings must be direct variables, direct variable array offsets, or direct object-property targets in the current subset"
    );

    let count_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_bind_result($stmt, $id);
"#,
    )
    .unwrap_err();

    assert_eq!(count_error.phase, Phase::Runtime);
    assert_eq!(count_error.line, 3);
    assert_eq!(count_error.column, 1);
    assert_eq!(
        count_error.message,
        "unsupported call mysqli_stmt_bind_result(): bound result variable count must match current placeholder field count 2, got 1"
    );
}

#[test]
fn mysqli_statement_bind_result_writes_direct_array_offset_targets() {
    let execution = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array(1));
mysqli_stmt_store_result($stmt);
$row = array();
$id_key = "ID";
echo mysqli_stmt_bind_result($stmt, $row[$id_key], $row["post_title"]) ? "bound" : "failed";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $row["ID"] . ":" . $row["post_title"] : "no-row";
echo "|";
$id_key = "changed";
echo mysqli_stmt_fetch($stmt) === null ? "done" : "again";
echo "|";
echo array_key_exists("changed", $row) ? "changed" : "stable";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bound|1:Hello world placeholder|done|stable"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_bind_result_writes_direct_object_property_targets() {
    let execution = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array(1));
mysqli_stmt_store_result($stmt);
class Row {
    public $ID;
    public $fields;
}
$row = new Row();
$row->fields = array();
$title_key = "post_title";
echo mysqli_stmt_bind_result($stmt, $row->ID, $row->fields[$title_key]) ? "bound" : "failed";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $row->ID . ":" . $row->fields["post_title"] : "no-row";
echo "|";
$title_key = "changed";
echo mysqli_stmt_fetch($stmt) === null ? "done" : "again";
echo "|";
echo array_key_exists("changed", $row->fields) ? "changed" : "stable";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bound|1:Hello world placeholder|done|stable"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_bind_result_fetches_unbuffered_executed_rows() {
    let execution = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array(1));
$id = null;
$title = null;
echo mysqli_stmt_num_rows($stmt);
echo "|";
echo mysqli_stmt_bind_result($stmt, $id, $title) ? "bound" : "failed";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $id . ":" . $title : "no-row";
echo "|";
echo mysqli_stmt_fetch($stmt) === null ? "done" : "again";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|bound|1:Hello world placeholder|done");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_result_and_close_are_visible_but_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
$get_result = "mysqli_stmt_get_result";
$close = "mysqli_stmt_close";
echo function_exists($get_result) ? "yes" : "no";
echo "|";
echo is_callable($get_result) ? "result-callable" : "result-missing";
echo "|";
echo function_exists($close) ? "close-exists" : "close-missing";
echo "|";
echo is_callable($close) ? "close-callable" : "close-missing";
$prepared = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($prepared);
$result = mysqli_stmt_get_result($prepared);
echo "|";
echo get_class($result);
echo "|";
echo mysqli_fetch_assoc($result)["post_title"];
$stmt = mysqli_stmt_init(mysqli_init());
echo "|";
echo mysqli_stmt_close($stmt) ? "closed" : "open";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|result-callable|close-exists|close-callable|mysqli_result|Hello world placeholder|closed"
    );
    assert_eq!(execution.exit_code, 0);

    let result_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_get_result($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(result_error.phase, Phase::Runtime);
    assert_eq!(result_error.line, 3);
    assert_eq!(result_error.column, 1);
    assert_eq!(
        result_error.message,
        "unsupported call mysqli_stmt_get_result(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let close_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_close($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(close_error.phase, Phase::Runtime);
    assert_eq!(close_error.line, 3);
    assert_eq!(close_error.column, 1);
    assert_eq!(
        close_error.message,
        "unsupported call mysqli_stmt_close(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_statement_error_metadata_is_visible_but_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$errno = "mysqli_stmt_errno";
$error = "mysqli_stmt_error";
$affected = "mysqli_stmt_affected_rows";
echo function_exists($errno) ? "yes" : "no";
echo "|";
echo is_callable($errno) ? "errno-callable" : "errno-missing";
echo "|";
echo function_exists($error) ? "error-exists" : "error-missing";
echo "|";
echo is_callable($error) ? "error-callable" : "error-missing";
echo "|";
echo function_exists($affected) ? "affected-exists" : "affected-missing";
echo "|";
echo is_callable($affected) ? "affected-callable" : "affected-missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|errno-callable|error-exists|error-callable|affected-exists|affected-callable"
    );
    assert_eq!(execution.exit_code, 0);

    let diagnostics = run_source(
        r#"<?php
$stmt = mysqli_stmt_init(mysqli_init());
echo mysqli_stmt_errno($stmt);
echo "|";
echo mysqli_stmt_error($stmt) === "" ? "empty" : "non-empty";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(diagnostics.stdout, "0|empty|0");
    assert_eq!(diagnostics.exit_code, 0);

    let errno_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_errno($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(errno_error.phase, Phase::Runtime);
    assert_eq!(errno_error.line, 3);
    assert_eq!(errno_error.column, 1);
    assert_eq!(
        errno_error.message,
        "unsupported call mysqli_stmt_errno(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let error_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_error($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(error_error.phase, Phase::Runtime);
    assert_eq!(error_error.line, 3);
    assert_eq!(error_error.column, 1);
    assert_eq!(
        error_error.message,
        "unsupported call mysqli_stmt_error(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let affected_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_affected_rows($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(affected_error.phase, Phase::Runtime);
    assert_eq!(affected_error.line, 3);
    assert_eq!(affected_error.column, 1);
    assert_eq!(
        affected_error.message,
        "unsupported call mysqli_stmt_affected_rows(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_statement_result_cursor_is_visible_but_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$store = "mysqli_stmt_store_result";
$num_rows = "mysqli_stmt_num_rows";
$fetch = "mysqli_stmt_fetch";
echo function_exists($store) ? "yes" : "no";
echo "|";
echo is_callable($store) ? "store-callable" : "store-missing";
echo "|";
echo function_exists($num_rows) ? "num-rows-exists" : "num-rows-missing";
echo "|";
echo is_callable($num_rows) ? "num-rows-callable" : "num-rows-missing";
echo "|";
echo function_exists($fetch) ? "fetch-exists" : "fetch-missing";
echo "|";
echo is_callable($fetch) ? "fetch-callable" : "fetch-missing";
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_stmt_num_rows($stmt);
echo "|";
mysqli_stmt_execute($stmt);
echo mysqli_stmt_store_result($stmt) ? "stored" : "not-stored";
echo "|";
echo mysqli_stmt_num_rows($stmt);
echo "|";
$id = null;
$title = null;
echo mysqli_stmt_bind_result($stmt, $id, $title) ? "bound" : "not-bound";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $id . ":" . $title : "no-row";
echo "|";
echo mysqli_stmt_fetch($stmt) ? "again" : "done";
echo "|";
mysqli_stmt_free_result($stmt);
echo mysqli_stmt_num_rows($stmt);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|store-callable|num-rows-exists|num-rows-callable|fetch-exists|fetch-callable|0|stored|1|bound|1:Hello world placeholder|done|0"
    );
    assert_eq!(execution.exit_code, 0);

    let store_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_store_result($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(store_error.phase, Phase::Runtime);
    assert_eq!(store_error.line, 3);
    assert_eq!(store_error.column, 1);
    assert_eq!(
        store_error.message,
        "unsupported call mysqli_stmt_store_result(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let num_rows_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_num_rows($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(num_rows_error.phase, Phase::Runtime);
    assert_eq!(num_rows_error.line, 3);
    assert_eq!(num_rows_error.column, 1);
    assert_eq!(
        num_rows_error.message,
        "unsupported call mysqli_stmt_num_rows(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let fetch_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_fetch($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(fetch_error.phase, Phase::Runtime);
    assert_eq!(fetch_error.line, 3);
    assert_eq!(fetch_error.column, 1);
    assert_eq!(
        fetch_error.message,
        "unsupported call mysqli_stmt_fetch(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let fetch_unbound_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
mysqli_stmt_store_result($stmt);
mysqli_stmt_fetch($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(fetch_unbound_error.phase, Phase::Runtime);
    assert_eq!(fetch_unbound_error.line, 5);
    assert_eq!(fetch_unbound_error.column, 1);
    assert_eq!(
        fetch_unbound_error.message,
        "unsupported call mysqli_stmt_fetch(): bound result variables are not available in the current subset"
    );
}

#[test]
fn mysqli_statement_result_metadata_is_visible_but_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$metadata = "mysqli_stmt_result_metadata";
$field_count = "mysqli_stmt_field_count";
$free_result = "mysqli_stmt_free_result";
echo function_exists($metadata) ? "yes" : "no";
echo "|";
echo is_callable($metadata) ? "metadata-callable" : "metadata-missing";
echo "|";
echo function_exists($field_count) ? "field-count-exists" : "field-count-missing";
echo "|";
echo is_callable($field_count) ? "field-count-callable" : "field-count-missing";
echo "|";
echo function_exists($free_result) ? "free-result-exists" : "free-result-missing";
echo "|";
echo is_callable($free_result) ? "free-result-callable" : "free-result-missing";
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_stmt_field_count($stmt);
echo "|";
$result = mysqli_stmt_result_metadata($stmt);
echo get_class($result);
echo "|";
echo mysqli_fetch_field_direct($result, 1)->name;
echo "|";
mysqli_stmt_free_result($stmt);
echo "freed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|metadata-callable|field-count-exists|field-count-callable|free-result-exists|free-result-callable|2|mysqli_result|post_title|freed"
    );
    assert_eq!(execution.exit_code, 0);

    let metadata_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_result_metadata($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(metadata_error.phase, Phase::Runtime);
    assert_eq!(metadata_error.line, 3);
    assert_eq!(metadata_error.column, 1);
    assert_eq!(
        metadata_error.message,
        "unsupported call mysqli_stmt_result_metadata(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let field_count_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_field_count($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(field_count_error.phase, Phase::Runtime);
    assert_eq!(field_count_error.line, 3);
    assert_eq!(field_count_error.column, 1);
    assert_eq!(
        field_count_error.message,
        "unsupported call mysqli_stmt_field_count(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let free_result_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_free_result($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(free_result_error.phase, Phase::Runtime);
    assert_eq!(free_result_error.line, 3);
    assert_eq!(free_result_error.column, 1);
    assert_eq!(
        free_result_error.message,
        "unsupported call mysqli_stmt_free_result(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let unknown_select_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID FROM wp_posts");
mysqli_stmt_field_count($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(unknown_select_error.phase, Phase::Runtime);
    assert_eq!(unknown_select_error.line, 3);
    assert_eq!(unknown_select_error.column, 1);
    assert_eq!(
        unknown_select_error.message,
        "unsupported call mysqli_stmt_field_count(): statement result metadata is implemented only for current WordPress placeholder SELECT shapes"
    );
}

#[test]
fn mysqli_statement_positioning_and_attributes_have_placeholder_state() {
    let execution = run_source(
        r#"<?php
$data_seek = "mysqli_stmt_data_seek";
$attr_get = "mysqli_stmt_attr_get";
$attr_set = "mysqli_stmt_attr_set";
echo function_exists($data_seek) ? "yes" : "no";
echo "|";
echo is_callable($data_seek) ? "data-seek-callable" : "data-seek-missing";
echo "|";
echo function_exists($attr_get) ? "attr-get-exists" : "attr-get-missing";
echo "|";
echo is_callable($attr_get) ? "attr-get-callable" : "attr-get-missing";
echo "|";
echo function_exists($attr_set) ? "attr-set-exists" : "attr-set-missing";
echo "|";
echo is_callable($attr_set) ? "attr-set-callable" : "attr-set-missing";
echo "|";
echo MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH;
echo ":";
echo MYSQLI_STMT_ATTR_CURSOR_TYPE;
echo ":";
echo MYSQLI_STMT_ATTR_PREFETCH_ROWS;
echo ":";
echo MYSQLI_CURSOR_TYPE_NO_CURSOR;
echo ":";
echo MYSQLI_CURSOR_TYPE_READ_ONLY;
echo ":";
echo MYSQLI_CURSOR_TYPE_FOR_UPDATE;
echo ":";
echo MYSQLI_CURSOR_TYPE_SCROLLABLE;
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH);
echo ":";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE);
echo ":";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS);
echo "|";
echo mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH, true) ? "set-update" : "failed";
echo ":";
echo mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE, MYSQLI_CURSOR_TYPE_READ_ONLY) ? "set-cursor" : "failed";
echo ":";
echo mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS, 8) ? "set-prefetch" : "failed";
echo "|";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH);
echo ":";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE);
echo ":";
echo mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS);
$seek = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($seek);
mysqli_stmt_store_result($seek);
echo "|";
echo mysqli_stmt_num_rows($seek);
mysqli_stmt_data_seek($seek, 0);
echo ":seeked";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|data-seek-callable|attr-get-exists|attr-get-callable|attr-set-exists|attr-set-callable|0:1:2:0:1:2:4|0:0:0|set-update:set-cursor:set-prefetch|1:1:8|1:seeked"
    );
    assert_eq!(execution.exit_code, 0);

    let data_seek_handle_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_data_seek($stmt, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(data_seek_handle_error.phase, Phase::Runtime);
    assert_eq!(data_seek_handle_error.line, 3);
    assert_eq!(data_seek_handle_error.column, 1);
    assert_eq!(
        data_seek_handle_error.message,
        "unsupported call mysqli_stmt_data_seek(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let data_seek_unbuffered_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
mysqli_stmt_data_seek($stmt, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(data_seek_unbuffered_error.phase, Phase::Runtime);
    assert_eq!(data_seek_unbuffered_error.line, 4);
    assert_eq!(data_seek_unbuffered_error.column, 1);
    assert_eq!(
        data_seek_unbuffered_error.message,
        "unsupported call mysqli_stmt_data_seek(): buffered statement result state is not available in the current subset"
    );

    let data_seek_range_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
mysqli_stmt_store_result($stmt);
mysqli_stmt_data_seek($stmt, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(data_seek_range_error.phase, Phase::Runtime);
    assert_eq!(data_seek_range_error.line, 5);
    assert_eq!(data_seek_range_error.column, 1);
    assert_eq!(
        data_seek_range_error.message,
        "unsupported call mysqli_stmt_data_seek(): offset 1 is outside the current buffered statement row range 1"
    );

    let attr_get_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_stmt_attr_get($stmt, 1);
"#,
    )
    .unwrap();

    assert_eq!(attr_get_error.stdout, "0");

    let attr_get_handle_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_attr_get($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE);
"#,
    )
    .unwrap_err();

    assert_eq!(attr_get_handle_error.phase, Phase::Runtime);
    assert_eq!(attr_get_handle_error.line, 3);
    assert_eq!(attr_get_handle_error.column, 1);
    assert_eq!(
        attr_get_handle_error.message,
        "unsupported call mysqli_stmt_attr_get(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let attr_get_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_attr_get($stmt, 999);
"#,
    )
    .unwrap_err();

    assert_eq!(attr_get_error.phase, Phase::Runtime);
    assert_eq!(attr_get_error.line, 3);
    assert_eq!(attr_get_error.column, 1);
    assert_eq!(
        attr_get_error.message,
        "unsupported call mysqli_stmt_attr_get(): only MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH, MYSQLI_STMT_ATTR_CURSOR_TYPE, and MYSQLI_STMT_ATTR_PREFETCH_ROWS are implemented in the current subset, got 999"
    );

    let attr_set_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_CURSOR_TYPE, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(attr_set_error.phase, Phase::Runtime);
    assert_eq!(attr_set_error.line, 3);
    assert_eq!(attr_set_error.column, 1);
    assert_eq!(
        attr_set_error.message,
        "unsupported call mysqli_stmt_attr_set(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let attr_set_value_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_attr_set($stmt, MYSQLI_STMT_ATTR_PREFETCH_ROWS, "8");
"#,
    )
    .unwrap_err();

    assert_eq!(attr_set_value_error.phase, Phase::Runtime);
    assert_eq!(attr_set_value_error.line, 3);
    assert_eq!(attr_set_value_error.column, 1);
    assert_eq!(
        attr_set_value_error.message,
        "unsupported call mysqli_stmt_attr_set(): value argument must be int or bool in the current subset, got string"
    );
}

#[test]
fn mysqli_statement_streaming_reset_and_multi_results_are_visible_but_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
$send_long_data = "mysqli_stmt_send_long_data";
$reset = "mysqli_stmt_reset";
$more_results = "mysqli_stmt_more_results";
$next_result = "mysqli_stmt_next_result";
echo function_exists($send_long_data) ? "yes" : "no";
echo "|";
echo is_callable($send_long_data) ? "send-long-callable" : "send-long-missing";
echo "|";
echo function_exists($reset) ? "reset-exists" : "reset-missing";
echo "|";
echo is_callable($reset) ? "reset-callable" : "reset-missing";
echo "|";
echo function_exists($more_results) ? "more-results-exists" : "more-results-missing";
echo "|";
echo is_callable($more_results) ? "more-results-callable" : "more-results-missing";
echo "|";
echo function_exists($next_result) ? "next-result-exists" : "next-result-missing";
echo "|";
echo is_callable($next_result) ? "next-result-callable" : "next-result-missing";
$stmt = mysqli_prepare(mysqli_init(), "SELECT option_value FROM wp_options WHERE option_name = ?");
echo "|";
echo mysqli_stmt_param_count($stmt);
echo "|";
echo mysqli_stmt_send_long_data($stmt, 0, "blob") ? "sent-long" : "send-failed";
echo "|";
echo $send_long_data($stmt, 0, "-chunk") ? "sent-long-dynamic" : "send-failed";
echo "|";
echo mysqli_stmt_reset($stmt) ? "reset" : "failed";
echo "|";
echo mysqli_stmt_param_count($stmt);
$multi = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($multi);
echo "|";
echo mysqli_stmt_more_results($multi) ? "more" : "no-more";
echo "|";
echo mysqli_stmt_next_result($multi) ? "next" : "no-next";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|send-long-callable|reset-exists|reset-callable|more-results-exists|more-results-callable|next-result-exists|next-result-callable|1|sent-long|sent-long-dynamic|reset|0|no-more|no-next"
    );
    assert_eq!(execution.exit_code, 0);

    let send_long_data_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_send_long_data($stmt, 0, "blob");
"#,
    )
    .unwrap_err();

    assert_eq!(send_long_data_error.phase, Phase::Runtime);
    assert_eq!(send_long_data_error.line, 3);
    assert_eq!(send_long_data_error.column, 1);
    assert_eq!(
        send_long_data_error.message,
        "unsupported call mysqli_stmt_send_long_data(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let send_long_param_error = run_source(
        r#"<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_send_long_data($stmt, 1, "blob");
"#,
    )
    .unwrap_err();

    assert_eq!(send_long_param_error.phase, Phase::Runtime);
    assert_eq!(send_long_param_error.line, 3);
    assert_eq!(send_long_param_error.column, 1);
    assert_eq!(
        send_long_param_error.message,
        "unsupported call mysqli_stmt_send_long_data(): param_num argument must reference one of the current 1 placeholder parameter(s), got 1"
    );

    let reset_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_reset($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(reset_error.phase, Phase::Runtime);
    assert_eq!(reset_error.line, 3);
    assert_eq!(reset_error.column, 1);
    assert_eq!(
        reset_error.message,
        "unsupported call mysqli_stmt_reset(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let more_results_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_more_results($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(more_results_error.phase, Phase::Runtime);
    assert_eq!(more_results_error.line, 3);
    assert_eq!(more_results_error.column, 1);
    assert_eq!(
        more_results_error.message,
        "unsupported call mysqli_stmt_more_results(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let next_result_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_next_result($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(next_result_error.phase, Phase::Runtime);
    assert_eq!(next_result_error.line, 3);
    assert_eq!(next_result_error.column, 1);
    assert_eq!(
        next_result_error.message,
        "unsupported call mysqli_stmt_next_result(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_statement_diagnostics_and_insert_metadata_are_visible_but_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
$sqlstate = "mysqli_stmt_sqlstate";
$warning_count = "mysqli_stmt_warning_count";
$insert_id = "mysqli_stmt_insert_id";
echo function_exists($sqlstate) ? "yes" : "no";
echo "|";
echo is_callable($sqlstate) ? "sqlstate-callable" : "sqlstate-missing";
echo "|";
echo function_exists($warning_count) ? "warning-count-exists" : "warning-count-missing";
echo "|";
echo is_callable($warning_count) ? "warning-count-callable" : "warning-count-missing";
echo "|";
echo function_exists($insert_id) ? "insert-id-exists" : "insert-id-missing";
echo "|";
echo is_callable($insert_id) ? "insert-id-callable" : "insert-id-missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|sqlstate-callable|warning-count-exists|warning-count-callable|insert-id-exists|insert-id-callable"
    );
    assert_eq!(execution.exit_code, 0);

    let diagnostics = run_source(
        r#"<?php
$stmt = mysqli_stmt_init(mysqli_init());
echo mysqli_stmt_sqlstate($stmt);
echo "|";
echo mysqli_stmt_warning_count($stmt);
echo "|";
echo mysqli_stmt_insert_id($stmt);
"#,
    )
    .unwrap();

    assert_eq!(diagnostics.stdout, "00000|0|0");
    assert_eq!(diagnostics.exit_code, 0);

    let sqlstate_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_sqlstate($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(sqlstate_error.phase, Phase::Runtime);
    assert_eq!(sqlstate_error.line, 3);
    assert_eq!(sqlstate_error.column, 1);
    assert_eq!(
        sqlstate_error.message,
        "unsupported call mysqli_stmt_sqlstate(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let warning_count_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_warning_count($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(warning_count_error.phase, Phase::Runtime);
    assert_eq!(warning_count_error.line, 3);
    assert_eq!(warning_count_error.column, 1);
    assert_eq!(
        warning_count_error.message,
        "unsupported call mysqli_stmt_warning_count(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );

    let insert_id_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_insert_id($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(insert_id_error.phase, Phase::Runtime);
    assert_eq!(insert_id_error.line, 3);
    assert_eq!(insert_id_error.column, 1);
    assert_eq!(
        insert_id_error.message,
        "unsupported call mysqli_stmt_insert_id(): first argument must be mysqli_stmt object in the current subset, got mysqli object"
    );
}

#[test]
fn mysqli_result_field_metadata_helpers_execute_current_placeholder_subset() {
    let execution = run_source(
        r#"<?php
$fetch_fields = "mysqli_fetch_fields";
$fetch_direct = "mysqli_fetch_field_direct";
$field_seek = "mysqli_field_seek";
$field_tell = "mysqli_field_tell";
echo function_exists($fetch_fields) ? "yes" : "no";
echo "|";
echo is_callable($fetch_fields) ? "fetch-fields-callable" : "fetch-fields-missing";
echo "|";
echo function_exists($fetch_direct) ? "fetch-direct-exists" : "fetch-direct-missing";
echo "|";
echo is_callable($fetch_direct) ? "fetch-direct-callable" : "fetch-direct-missing";
echo "|";
echo function_exists($field_seek) ? "field-seek-exists" : "field-seek-missing";
echo "|";
echo is_callable($field_seek) ? "field-seek-callable" : "field-seek-missing";
echo "|";
echo function_exists($field_tell) ? "field-tell-exists" : "field-tell-missing";
echo "|";
echo is_callable($field_tell) ? "field-tell-callable" : "field-tell-missing";
echo "|";
echo function_exists("mysqli_stmt_fetch_fields") ? "stmt-fields-exists" : "stmt-fields-missing";
echo "|";
echo function_exists("mysqli_stmt_fetch_field") ? "stmt-field-exists" : "stmt-field-missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$fields = mysqli_fetch_fields($result);
echo "|";
echo $fields[0]->name;
echo ",";
echo $fields[1]->name;
echo "|";
echo $fields[0]->orgname, ":", $fields[0]->table, ":", $fields[0]->type, ":", $fields[0]->length, ":", $fields[0]->charsetnr;
echo "|";
$direct = mysqli_fetch_field_direct($result, 1);
echo $direct->name;
echo ":", $direct->orgname, ":", $direct->orgtable, ":", $direct->db, ":", $direct->catalog, ":", $direct->type, ":", $direct->max_length;
echo "|";
echo mysqli_field_tell($result);
echo "|";
echo mysqli_field_seek($result, 1) ? "seek" : "no-seek";
echo "|";
echo mysqli_field_tell($result);
echo "|";
$field = mysqli_fetch_field($result);
echo $field->name;
echo "|";
echo mysqli_field_tell($result);
echo "|";
echo mysqli_fetch_field_direct($result, 99) === false ? "no-direct" : "direct";
echo "|";
echo mysqli_field_seek($result, 99) ? "seek" : "no-seek";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|fetch-fields-callable|fetch-direct-exists|fetch-direct-callable|field-seek-exists|field-seek-callable|field-tell-exists|field-tell-callable|stmt-fields-missing|stmt-field-missing|ID,post_title|ID:wp_posts:3:20:63|post_title:post_title:wp_posts:wordpress:def:253:23|0|seek|1|post_title|2|no-direct|no-seek"
    );
    assert_eq!(execution.exit_code, 0);

    let fetch_direct_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_fetch_field_direct($result, "1");
"#,
    )
    .unwrap_err();

    assert_eq!(fetch_direct_error.phase, Phase::Runtime);
    assert_eq!(fetch_direct_error.line, 5);
    assert_eq!(fetch_direct_error.column, 1);
    assert_eq!(
        fetch_direct_error.message,
        "unsupported call mysqli_fetch_field_direct(): field index must be int in the current subset, got string"
    );

    let field_seek_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_field_seek($result, "1");
"#,
    )
    .unwrap_err();

    assert_eq!(field_seek_error.phase, Phase::Runtime);
    assert_eq!(field_seek_error.line, 5);
    assert_eq!(field_seek_error.column, 1);
    assert_eq!(
        field_seek_error.message,
        "unsupported call mysqli_field_seek(): field offset must be int in the current subset, got string"
    );
}

#[test]
fn mysqli_dump_debug_info_accepts_current_placeholder_handle() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_dump_debug_info";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_dump_debug_info($handle) ? "dumped" : "failed";
echo "|";
echo $call($handle) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|dumped|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_debug_accepts_current_placeholder_options() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_debug";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo mysqli_debug("d:t:o,/tmp/phpc-mysqli-debug.trace") ? "debug" : "failed";
echo "|";
echo $call(null) ? "dynamic" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|debug|dynamic");
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
fn mysqli_options_accepts_current_int_and_float_native_placeholder_option() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_options";
$alias = "mysqli_set_opt";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo function_exists($alias) ? "alias-exists" : "alias-missing";
echo "|";
echo is_callable($alias) ? "alias-callable" : "alias-missing";
$handle = mysqli_init();
echo "|";
echo defined("MYSQLI_OPT_INT_AND_FLOAT_NATIVE") ? MYSQLI_OPT_INT_AND_FLOAT_NATIVE : "missing";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true) ? "set" : "failed";
echo "|";
echo $call($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, 1) ? "set" : "failed";
echo "|";
echo mysqli_set_opt($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, false) ? "alias-set" : "failed";
echo "|";
echo $alias($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, 0) ? "dynamic-alias" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_CONNECT_TIMEOUT, 5) ? "connect-timeout" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_READ_TIMEOUT, 7) ? "read-timeout" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_INIT_COMMAND, "SET NAMES utf8mb4") ? "init-command" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_LOCAL_INFILE, true) ? "local-infile" : "failed";
echo "|";
echo mysqli_options($handle, MYSQLI_OPT_SSL_VERIFY_SERVER_CERT, false) ? "ssl-verify" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|alias-exists|alias-callable|201|set|set|alias-set|dynamic-alias|connect-timeout|read-timeout|init-command|local-infile|ssl-verify"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_options_local_infile_state_changes_load_data_boundary() {
    let disabled = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_query($handle, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");
"#,
    )
    .unwrap_err();

    assert_eq!(disabled.phase, Phase::Runtime);
    assert_eq!(disabled.line, 3);
    assert_eq!(disabled.column, 1);
    assert_eq!(
        disabled.message,
        "unsupported call mysqli_query(): LOAD DATA LOCAL INFILE is disabled by MYSQLI_OPT_LOCAL_INFILE in the current placeholder connection; real local infile loading is not implemented; got LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts"
    );

    let enabled = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_options($handle, MYSQLI_OPT_LOCAL_INFILE, true);
mysqli_query($handle, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");
"#,
    )
    .unwrap_err();

    assert_eq!(enabled.phase, Phase::Runtime);
    assert_eq!(enabled.line, 4);
    assert_eq!(enabled.column, 1);
    assert_eq!(
        enabled.message,
        "unsupported call mysqli_query(): LOAD DATA LOCAL INFILE execution is not implemented in the current subset; MYSQLI_OPT_LOCAL_INFILE placeholder state is recorded but host file loading and mutation SQL remain unsupported; got LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts"
    );

    let real_query_enabled = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_options($handle, MYSQLI_OPT_LOCAL_INFILE, 1);
mysqli_real_query($handle, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");
"#,
    )
    .unwrap_err();

    assert_eq!(real_query_enabled.phase, Phase::Runtime);
    assert_eq!(real_query_enabled.line, 4);
    assert_eq!(real_query_enabled.column, 1);
    assert_eq!(
        real_query_enabled.message,
        "unsupported call mysqli_real_query(): LOAD DATA LOCAL INFILE execution is not implemented in the current subset; MYSQLI_OPT_LOCAL_INFILE placeholder state is recorded but host file loading and mutation SQL remain unsupported; got LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts"
    );

    let multi_query_disabled = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_set_opt($handle, MYSQLI_OPT_LOCAL_INFILE, 0);
mysqli_multi_query($handle, "LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts");
"#,
    )
    .unwrap_err();

    assert_eq!(multi_query_disabled.phase, Phase::Runtime);
    assert_eq!(multi_query_disabled.line, 4);
    assert_eq!(multi_query_disabled.column, 1);
    assert_eq!(
        multi_query_disabled.message,
        "unsupported call mysqli_multi_query(): LOAD DATA LOCAL INFILE is disabled by MYSQLI_OPT_LOCAL_INFILE in the current placeholder connection; real local infile loading is not implemented; got LOAD DATA LOCAL INFILE '/tmp/posts.csv' INTO TABLE wp_posts"
    );
}

#[test]
fn mysqli_options_init_command_affects_real_connect_boundary() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
echo mysqli_options($handle, MYSQLI_INIT_COMMAND, "SET NAMES utf8mb4") ? "init-set" : "failed";
echo "|";
echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0) ? "connected" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-pending" : "pending";

$handle2 = mysqli_init();
mysqli_set_opt($handle2, MYSQLI_INIT_COMMAND, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo "|";
echo mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0) ? "alias-connected" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "init-set|connected|0|no-pending|alias-connected"
    );
    assert_eq!(execution.exit_code, 0);

    let unsupported = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_options($handle, MYSQLI_INIT_COMMAND, "SELECT 1");
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported.phase, Phase::Runtime);
    assert_eq!(unsupported.line, 4);
    assert_eq!(unsupported.column, 1);
    assert_eq!(
        unsupported.message,
        "unsupported call mysqli_real_connect(): MYSQLI_INIT_COMMAND execution is not implemented for arbitrary SQL; only deterministic no-result init commands are supported in the current subset, got SELECT 1"
    );
}

#[test]
fn mysqli_ssl_set_accepts_current_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_ssl_set";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
echo "|";
echo mysqli_ssl_set($handle, null, null, null, null, null) ? "nulls" : "failed";
echo "|";
echo $call($handle, "key.pem", "cert.pem", "ca.pem", "capath", "cipher") ? "strings" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|nulls|strings");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_connect_error_state_returns_current_clean_placeholders() {
    let execution = run_source(
        r#"<?php
$errno = "mysqli_connect_errno";
$error = "mysqli_connect_error";
echo function_exists($errno) ? "yes" : "no";
echo "|";
echo is_callable($error) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_options($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_connect_errno();
echo "|";
echo mysqli_connect_error() === null ? "null" : mysqli_connect_error();
echo "|";
echo $errno();
echo "|";
echo $error() === null ? "null" : $error();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|0|null|0|null");
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
fn mysqli_transactions_restore_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_begin_transaction($handle);
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://rolled-back.test' WHERE option_name = 'siteurl'");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'yes')");
echo mysqli_rollback($handle) ? "rollback" : "failed";
echo "|";
$site = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$site_row = mysqli_fetch_assoc($site);
echo $site_row["option_value"];
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($home);
mysqli_begin_transaction($handle);
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://committed.test' WHERE option_name = 'siteurl'");
echo "|";
echo mysqli_commit($handle) ? "commit" : "failed";
echo "|";
$committed = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$committed_row = mysqli_fetch_assoc($committed);
echo $committed_row["option_value"];
mysqli_autocommit($handle, false);
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://autocommit.test' WHERE option_name = 'siteurl'");
mysqli_autocommit($handle, true);
echo "|";
$auto = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$auto_row = mysqli_fetch_assoc($auto);
echo $auto_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "rollback|https://example.test|0|commit|https://committed.test|https://autocommit.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_savepoint_helpers_accept_current_placeholder_shape() {
    let execution = run_source(
        r#"<?php
$savepoint = "mysqli_savepoint";
$release = "mysqli_release_savepoint";
echo function_exists($savepoint) ? "yes" : "no";
echo "|";
echo is_callable($release) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_begin_transaction($handle);
echo "|";
echo mysqli_savepoint($handle, "wp") ? "savepoint" : "failed";
echo "|";
echo mysqli_release_savepoint($handle, "wp") ? "release" : "failed";
echo "|";
echo $savepoint($handle, "wp2") ? "dynamic-savepoint" : "failed";
echo "|";
echo $release($handle, "wp2") ? "dynamic-release" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|savepoint|release|dynamic-savepoint|dynamic-release"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_savepoints_restore_current_wordpress_option_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_begin_transaction($handle);
mysqli_savepoint($handle, "before_home");
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://changed.test' WHERE option_name = 'siteurl'");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'yes')");
echo mysqli_rollback($handle, 0, "before_home") ? "savepoint" : "failed";
echo "|";
$site = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$site_row = mysqli_fetch_assoc($site);
echo $site_row["option_value"];
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($home);
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://after-savepoint.test' WHERE option_name = 'siteurl'");
echo "|";
echo mysqli_commit($handle) ? "commit" : "failed";
echo "|";
$committed = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$committed_row = mysqli_fetch_assoc($committed);
echo $committed_row["option_value"];
mysqli_begin_transaction($handle);
mysqli_savepoint($handle, "released");
mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://released.test' WHERE option_name = 'siteurl'");
mysqli_release_savepoint($handle, "released");
mysqli_rollback($handle, 0, "released");
echo "|";
$released = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$released_row = mysqli_fetch_assoc($released);
echo $released_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "savepoint|https://example.test|0|commit|https://after-savepoint.test|https://released.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_transactions_restore_current_wordpress_schema_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_schema_base (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_begin_transaction($handle);
mysqli_query($handle, "ALTER TABLE wp_schema_base ADD COLUMN checksum varchar(64) NOT NULL default '', ADD KEY checksum (checksum)");
mysqli_query($handle, "CREATE TABLE wp_schema_temp (id bigint(20) unsigned NOT NULL, PRIMARY KEY  (id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
echo mysqli_rollback($handle) ? "rollback" : "failed";
echo "|temp=";
$temp = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp_schema_temp'");
echo mysqli_num_rows($temp);
echo "|base=";
$base = mysqli_query($handle, "DESCRIBE wp_schema_base");
while ($column = mysqli_fetch_assoc($base)) {
    echo $column["Field"], ":", $column["Key"], ";";
}
mysqli_begin_transaction($handle);
mysqli_savepoint($handle, "before_extra");
mysqli_query($handle, "ALTER TABLE wp_schema_base ADD COLUMN extra varchar(20) NULL");
echo "|savepoint=";
echo mysqli_rollback($handle, 0, "before_extra") ? "rollback" : "failed";
echo "|extra=";
$extra = mysqli_query($handle, "SHOW FULL COLUMNS FROM wp_schema_base LIKE 'extra'");
echo mysqli_num_rows($extra);
mysqli_commit($handle);
mysqli_begin_transaction($handle);
mysqli_query($handle, "ALTER TABLE wp_schema_base ADD COLUMN committed varchar(20) NULL");
echo "|";
echo mysqli_commit($handle) ? "commit" : "failed";
echo "|committed=";
$committed = mysqli_query($handle, "SHOW FULL COLUMNS FROM wp_schema_base LIKE 'committed'");
echo mysqli_num_rows($committed);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "rollback|temp=0|base=id:PRI;slug:MUL;|savepoint=rollback|extra=0|commit|committed=1"
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
echo get_class($autoload);
echo "|";
echo mysqli_num_rows($autoload);
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
mysqli_free_result($autoload);
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
        "mysqli_result|0|fallback-result|prime-result|single-result|columns-result|describe-result|0||0|clean"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_insert_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_insert_id($handle);
echo "|";
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')") ? "inserted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://duplicate.test', 'no')") ? "duplicate-inserted" : "duplicate-rejected";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
$missing = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0|inserted|1|1|duplicate-rejected|0|1|https://example.test|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_name_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$result = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_name"];
echo "|";
echo mysqli_num_rows($result);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$missing = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
echo "|";
mysqli_query($handle, "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES ('home', 'https://home.test', 'auto-on')");
$home = mysqli_query($handle, "SELECT `option_name` FROM `wp_options` WHERE `option_name` = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "siteurl|1|0|0|home");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_upsert_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://upserted.test', 'no') ON DUPLICATE KEY UPDATE option_name = VALUES(option_name), option_value = VALUES(option_value), autoload = VALUES(autoload)") ? "updated" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'yes') ON DUPLICATE KEY UPDATE option_value = VALUES(option_value), autoload = VALUES(autoload)") ? "inserted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|2|2|https://upserted.test|inserted|1|3|https://home.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_replace_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://replaced.test', 'no')") ? "replaced" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
echo mysqli_query($handle, "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'yes')") ? "inserted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$home = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "replaced|2|2|https://replaced.test|inserted|1|3|https://home.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_autoload_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://replaced.test', 'no')");
$result = mysqli_query($handle, "SELECT autoload FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["autoload"];
echo "|";
echo mysqli_num_rows($result);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$missing = mysqli_query($handle, "SELECT autoload FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
echo "|";
mysqli_query($handle, "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES ('home', 'https://home.test', 'auto-on')");
$home = mysqli_query($handle, "SELECT `autoload` FROM `wp_options` WHERE `option_name` = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["autoload"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT `autoload` FROM `wp_options` WHERE `option_name` = ? LIMIT 1", array("home"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["autoload"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT autoload FROM wp_options WHERE option_name = ?");
$name = "home";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "prepared" : "failed";
echo ":";
$prepared = mysqli_stmt_get_result($stmt);
$prepared_row = mysqli_fetch_assoc($prepared);
echo $prepared_row["autoload"];
echo "|";
$name = "missing";
mysqli_stmt_execute($stmt);
$missing_prepared = mysqli_stmt_get_result($stmt);
echo "missing:", mysqli_num_rows($missing_prepared), ":", mysqli_num_fields($missing_prepared);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "no|1|0|0|auto-on|auto-on|prepared:auto-on|missing:0:0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_value_and_autoload_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'no')");
$result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo mysqli_num_rows($result);
echo ":";
echo mysqli_num_fields($result);
echo ":";
echo $row["option_value"], ":", $row["autoload"];
echo "|";
$missing = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
echo ":";
echo mysqli_num_fields($missing);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1", array("siteurl"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_value"], ":", $direct_row["autoload"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$prepared = mysqli_stmt_get_result($stmt);
$prepared_row = mysqli_fetch_assoc($prepared);
echo $prepared_row["option_value"], ":", $prepared_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:2:https://example.test:no|0:0|https://example.test:no|executed|https://example.test:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_full_row_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'no')");
$result = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo mysqli_num_rows($result);
echo ":";
echo mysqli_num_fields($result);
echo ":";
echo $row["option_name"], ":", $row["option_value"], ":", $row["autoload"];
echo "|";
$missing = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
echo ":";
echo mysqli_num_fields($missing);
echo "|";
$direct = mysqli_execute_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1", array("siteurl"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_name"], ":", $direct_row["option_value"], ":", $direct_row["autoload"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` = ? LIMIT 1");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$prepared = mysqli_stmt_get_result($stmt);
$prepared_row = mysqli_fetch_assoc($prepared);
echo $prepared_row["option_name"], ":", $prepared_row["option_value"], ":", $prepared_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:3:siteurl:https://example.test:no|0:0|siteurl:https://example.test:no|executed|siteurl:https://example.test:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_full_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$all = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options");
echo mysqli_num_rows($all);
echo ":";
echo mysqli_num_fields($all);
echo ":";
$all_first = mysqli_fetch_assoc($all);
$all_second = mysqli_fetch_assoc($all);
$all_third = mysqli_fetch_assoc($all);
echo $all_first["option_name"], ":", $all_first["option_value"], ":", $all_first["autoload"];
echo ",";
echo $all_second["option_name"], ":", $all_second["option_value"], ":", $all_second["autoload"];
echo ",";
echo $all_third["option_name"], ":", $all_third["option_value"], ":", $all_third["autoload"];
echo "|";
$autoload = mysqli_query($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `autoload` IN ( 'yes', 'on', 'auto-on', 'auto' )");
echo mysqli_num_rows($autoload);
echo ":";
$autoload_first = mysqli_fetch_assoc($autoload);
$autoload_second = mysqli_fetch_assoc($autoload);
echo $autoload_first["option_name"], ":", $autoload_first["autoload"];
echo ",";
echo $autoload_second["option_name"], ":", $autoload_second["autoload"];
echo "|";
$named = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name IN ('theme_mods','missing','home')");
echo mysqli_num_rows($named);
echo ":";
$named_first = mysqli_fetch_assoc($named);
$named_second = mysqli_fetch_assoc($named);
echo $named_first["option_name"], ":", $named_first["autoload"];
echo ",";
echo $named_second["option_name"], ":", $named_second["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3:3:home:https://home.test:no,siteurl:https://example.test:yes,theme_mods:theme-db:on|2:siteurl:yes,theme_mods:on|2:theme_mods:on,home:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_id_full_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$single = mysqli_query($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` = 'siteurl' LIMIT 1");
$single_row = mysqli_fetch_assoc($single);
echo mysqli_num_rows($single);
echo ":";
echo mysqli_num_fields($single);
echo ":";
echo $single_row["option_id"], ":", $single_row["option_name"], ":", $single_row["option_value"], ":", $single_row["autoload"];
echo "|";
$all = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options");
$all_first = mysqli_fetch_assoc($all);
$all_second = mysqli_fetch_assoc($all);
$all_third = mysqli_fetch_assoc($all);
echo mysqli_num_rows($all);
echo ":";
echo $all_first["option_id"], ":", $all_first["option_name"];
echo ",";
echo $all_second["option_id"], ":", $all_second["option_name"];
echo ",";
echo $all_third["option_id"], ":", $all_third["option_name"];
echo "|";
$autoload = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
$autoload_first = mysqli_fetch_assoc($autoload);
$autoload_second = mysqli_fetch_assoc($autoload);
echo mysqli_num_rows($autoload);
echo ":";
echo $autoload_first["option_id"], ":", $autoload_first["option_name"];
echo ",";
echo $autoload_second["option_id"], ":", $autoload_second["option_name"];
echo "|";
$named = mysqli_query($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` IN ('theme_mods','missing','home')");
$named_first = mysqli_fetch_assoc($named);
$named_second = mysqli_fetch_assoc($named);
echo mysqli_num_rows($named);
echo ":";
echo $named_first["option_id"], ":", $named_first["option_name"];
echo ",";
echo $named_second["option_id"], ":", $named_second["option_name"];
echo "|";
$direct = mysqli_execute_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1", array("theme_mods"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_id"], ":", $direct_row["option_name"], ":", $direct_row["autoload"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` = ? LIMIT 1");
$name = "home";
mysqli_stmt_bind_param($stmt, "s", $name);
mysqli_stmt_execute($stmt);
$prepared = mysqli_stmt_get_result($stmt);
$prepared_row = mysqli_fetch_assoc($prepared);
echo $prepared_row["option_id"], ":", $prepared_row["option_name"], ":", $prepared_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:4:1:siteurl:https://example.test:yes|3:2:home,1:siteurl,3:theme_mods|2:1:siteurl,3:theme_mods|2:3:theme_mods,2:home|3:theme_mods:on|2:home:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_star_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$single = mysqli_query($handle, "SELECT * FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$single_row = mysqli_fetch_object($single);
echo mysqli_num_rows($single);
echo ":";
echo mysqli_num_fields($single);
echo ":";
echo $single_row->option_id, ":", $single_row->option_name, ":", $single_row->option_value, ":", $single_row->autoload;
echo "|";
$all = mysqli_query($handle, "SELECT * FROM wp_options");
$all_first = mysqli_fetch_assoc($all);
$all_second = mysqli_fetch_assoc($all);
$all_third = mysqli_fetch_assoc($all);
echo mysqli_num_rows($all);
echo ":";
echo $all_first["option_id"], ":", $all_first["option_name"];
echo ",";
echo $all_second["option_id"], ":", $all_second["option_name"];
echo ",";
echo $all_third["option_id"], ":", $all_third["option_name"];
echo "|";
$autoload = mysqli_execute_query($handle, "SELECT * FROM `wp_options` WHERE `autoload` IN (?, ?)", array("yes", "on"));
$autoload_first = mysqli_fetch_assoc($autoload);
$autoload_second = mysqli_fetch_assoc($autoload);
echo mysqli_num_rows($autoload);
echo ":";
echo $autoload_first["option_id"], ":", $autoload_first["option_name"];
echo ",";
echo $autoload_second["option_id"], ":", $autoload_second["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT * FROM wp_options WHERE option_name IN (?, ?, ?)");
$one = "theme_mods";
$two = "missing";
$three = "home";
mysqli_stmt_bind_param($stmt, "sss", $one, $two, $three);
mysqli_stmt_execute($stmt);
$named = mysqli_stmt_get_result($stmt);
$named_first = mysqli_fetch_assoc($named);
$named_second = mysqli_fetch_assoc($named);
echo mysqli_num_rows($named);
echo ":";
echo $named_first["option_id"], ":", $named_first["option_name"];
echo ",";
echo $named_second["option_id"], ":", $named_second["option_name"];
echo "|";
$prepared_single = mysqli_execute_query($handle, "SELECT * FROM wp_options WHERE option_name = ? LIMIT 1", array("theme_mods"));
$prepared_single_row = mysqli_fetch_assoc($prepared_single);
echo $prepared_single_row["option_id"], ":", $prepared_single_row["option_name"], ":", $prepared_single_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:4:1:siteurl:https://example.test:yes|3:2:home,1:siteurl,3:theme_mods|2:1:siteurl,3:theme_mods|2:3:theme_mods,2:home|3:theme_mods:on"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_transient_prefix_option_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '12345', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%'");
$first = mysqli_fetch_assoc($rows);
$second = mysqli_fetch_assoc($rows);
echo mysqli_num_rows($rows);
echo ":";
echo $first["option_name"], "=", $first["option_value"];
echo ",";
echo $second["option_name"], "=", $second["option_value"];
echo "|";
$star = mysqli_query($handle, "SELECT * FROM `wp_options` WHERE `option_name` LIKE '\\_transient\\_%'");
$star_first = mysqli_fetch_assoc($star);
$star_second = mysqli_fetch_assoc($star);
echo mysqli_num_fields($star);
echo ":";
echo $star_first["option_id"], ":", $star_first["option_name"];
echo ",";
echo $star_second["option_id"], ":", $star_second["option_name"];
echo "|";
$values = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name LIKE '_transient_timeout_%'");
$timeout = mysqli_fetch_assoc($values);
echo $timeout["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_timeout_update_plugins=12345,_transient_update_plugins=plugin-payload|4:2:_transient_timeout_update_plugins,1:_transient_update_plugins|12345"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_applies_mysql_like_wildcards_to_wordpress_option_reads() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$wildcard = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '_transient_%'");
$wildcard_first = mysqli_fetch_assoc($wildcard);
$wildcard_second = mysqli_fetch_assoc($wildcard);
echo mysqli_num_rows($wildcard);
echo ":";
echo $wildcard_first["option_name"], ",";
echo $wildcard_second["option_name"];
echo "|";
$escaped = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '\\_transient\\_%'");
$escaped_row = mysqli_fetch_assoc($escaped);
echo mysqli_num_rows($escaped);
echo ":";
echo $escaped_row["option_name"], "=", $escaped_row["option_value"];
echo "|";
$custom_escape = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '!_site!_transient!_%' ESCAPE '!'");
$custom_row = mysqli_fetch_assoc($custom_escape);
echo mysqli_num_rows($custom_escape);
echo ":";
echo $custom_row["option_name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_update_plugins,xtransient-update-plugins|1:_transient_update_plugins=plugin-payload|1:_site_transient_update_plugins"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_applies_mysql_like_wildcards_to_wordpress_option_reads() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$wildcard = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ?", array("_transient_%"));
$wildcard_first = mysqli_fetch_assoc($wildcard);
$wildcard_second = mysqli_fetch_assoc($wildcard);
echo mysqli_num_rows($wildcard);
echo ":";
echo $wildcard_first["option_name"], ",";
echo $wildcard_second["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ?");
$escaped = "\\_transient\\_%";
mysqli_stmt_bind_param($stmt, "s", $escaped);
mysqli_stmt_execute($stmt);
$escaped_rows = mysqli_stmt_get_result($stmt);
$escaped_row = mysqli_fetch_assoc($escaped_rows);
echo mysqli_num_rows($escaped_rows);
echo ":";
echo $escaped_row["option_name"], "=", $escaped_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_update_plugins,xtransient-update-plugins|1:_transient_update_plugins=plugin-payload"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_option_like_scans_apply_escape_clause() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_feed_mod', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-feed-mod', 'wildcard', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_feed_mod', 'site-payload', 'no')");
$names = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '!' ORDER BY option_name", array("!_transient!_%"));
$first = mysqli_fetch_assoc($names);
$second = mysqli_fetch_assoc($names);
echo mysqli_num_rows($names);
echo ":";
echo $first["option_name"], ",";
echo $second["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!' ORDER BY `option_name` ASC");
mysqli_stmt_execute($stmt, array("!_site!_transient!_%"));
$site = mysqli_stmt_get_result($stmt);
$site_row = mysqli_fetch_assoc($site);
echo mysqli_num_fields($site);
echo ":";
echo $site_row["option_name"], "=", $site_row["option_value"], ":", $site_row["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_timeout_feed_mod,_transient_update_feed_mod|3:_site_transient_update_feed_mod=site-payload:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_prepared_transient_prefix_option_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '12345', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ?");
mysqli_stmt_execute($stmt, array("\\_transient\\_%"));
$rows = mysqli_stmt_get_result($stmt);
$first = mysqli_fetch_assoc($rows);
$second = mysqli_fetch_assoc($rows);
echo mysqli_num_rows($rows);
echo ":";
echo $first["option_name"], "=", $first["option_value"];
echo ",";
echo $second["option_name"], "=", $second["option_value"];
echo "|";
$values = mysqli_execute_query($handle, "SELECT `option_value` FROM `wp_options` WHERE `option_name` LIKE ?", array("_transient_timeout_%"));
$timeout = mysqli_fetch_assoc($values);
echo $timeout["option_value"];
echo "|";
$star = mysqli_prepare($handle, "SELECT * FROM `wp_options` WHERE `option_name` LIKE ?");
$site_prefix = "\\_site_transient\\_%";
mysqli_stmt_bind_param($star, "s", $site_prefix);
mysqli_stmt_execute($star);
$site_rows = mysqli_stmt_get_result($star);
$site_row = mysqli_fetch_assoc($site_rows);
echo mysqli_num_fields($site_rows);
echo ":";
echo $site_row["option_id"], ":", $site_row["option_name"], ":", $site_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_timeout_update_plugins=12345,_transient_update_plugins=plugin-payload|12345|4:3:_site_transient_update_plugins:site-plugin-payload"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_ordered_transient_prefix_option_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_themes', 'theme-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_themes', '555', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$direct = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%' ORDER BY option_name");
$direct_first = mysqli_fetch_assoc($direct);
$direct_second = mysqli_fetch_assoc($direct);
echo mysqli_num_rows($direct);
echo ":";
echo $direct_first["option_name"], "=", $direct_first["option_value"];
echo ",";
echo $direct_second["option_name"], "=", $direct_second["option_value"];
echo "|";
$prepared = mysqli_execute_query($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` LIKE ? ORDER BY `option_name` ASC", array("\\_transient\\_%"));
$prepared_first = mysqli_fetch_assoc($prepared);
$prepared_second = mysqli_fetch_assoc($prepared);
echo mysqli_num_fields($prepared);
echo ":";
echo $prepared_first["option_name"], ":", $prepared_first["autoload"];
echo ",";
echo $prepared_second["option_name"], ":", $prepared_second["autoload"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT * FROM `wp_options` WHERE `option_name` LIKE ? ORDER BY `option_name`");
mysqli_stmt_execute($stmt, array("_transient_timeout_%"));
$timeout = mysqli_stmt_get_result($stmt);
$timeout_row = mysqli_fetch_assoc($timeout);
echo mysqli_num_rows($timeout);
echo ":";
echo $timeout_row["option_id"], ":", $timeout_row["option_name"], ":", $timeout_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_timeout_update_themes=555,_transient_update_themes=theme-payload|3:_transient_timeout_update_themes:no,_transient_update_themes:no|1:2:_transient_timeout_update_themes:555"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_expired_transient_timeout_option_names_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_themes', '250', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_core', '900', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$direct = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300 ORDER BY option_name");
$direct_first = mysqli_fetch_assoc($direct);
$direct_second = mysqli_fetch_assoc($direct);
echo mysqli_num_rows($direct);
echo ":";
echo $direct_first["option_name"], ",";
echo $direct_second["option_name"];
echo "|";
$prepared = mysqli_execute_query($handle, "SELECT `option_name` FROM `wp_options` WHERE `option_name` LIKE ? AND `option_value` < ? ORDER BY `option_name` ASC", array("\\_transient\\_timeout\\_%", "300"));
$prepared_first = mysqli_fetch_assoc($prepared);
$prepared_second = mysqli_fetch_assoc($prepared);
echo mysqli_num_fields($prepared);
echo ":";
echo $prepared_first["option_name"], ",";
echo $prepared_second["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? AND option_value < ? ORDER BY option_name");
mysqli_stmt_execute($stmt, array("_transient_timeout_%", 101));
$timeout = mysqli_stmt_get_result($stmt);
$timeout_row = mysqli_fetch_assoc($timeout);
echo mysqli_num_rows($timeout);
echo ":";
echo $timeout_row["option_name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:_transient_timeout_update_plugins,_transient_timeout_update_themes|1:_transient_timeout_update_plugins,_transient_timeout_update_themes|1:_transient_timeout_update_plugins"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_expired_transient_timeout_predicates_apply_escape_clause() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_timeout_feed_mod', '120', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_fresh', '900', 'no')");
$select = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '!' AND option_value < ? ORDER BY option_name", array("!_transient!_timeout!_%", "300"));
$selected = mysqli_fetch_assoc($select);
echo mysqli_num_rows($select);
echo ":";
echo $selected["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name LIKE ? ESCAPE '!' AND option_value < ?");
mysqli_stmt_execute($stmt, array("!_transient!_timeout!_%", 300));
echo mysqli_stmt_affected_rows($stmt);
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!' AND `option_value` < ?", array("!_transient!_timeout!_%", "300"));
echo mysqli_affected_rows($handle);
echo "|";
$left = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '%transient%timeout%' ORDER BY option_name");
while ($row = mysqli_fetch_assoc($left)) {
    echo $row["option_name"], ",";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:_transient_timeout_feed_mod|1|1|_site_transient_timeout_feed_mod,_transient_timeout_fresh,xtransient-timeout-feed_mod,"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_row_sets_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options");
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
$all = mysqli_stmt_get_result($stmt);
$all_first = mysqli_fetch_assoc($all);
$all_second = mysqli_fetch_assoc($all);
$all_third = mysqli_fetch_assoc($all);
echo "|";
echo mysqli_num_rows($all);
echo ":";
echo mysqli_num_fields($all);
echo ":";
echo $all_first["option_id"], ":", $all_first["option_name"];
echo ",";
echo $all_second["option_id"], ":", $all_second["option_name"];
echo ",";
echo $all_third["option_id"], ":", $all_third["option_name"];
echo "|";
$autoload_stmt = mysqli_prepare($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `autoload` IN ( 'yes', 'on', 'auto-on', 'auto' )");
mysqli_stmt_execute($autoload_stmt);
$autoload = mysqli_stmt_get_result($autoload_stmt);
$autoload_first = mysqli_fetch_assoc($autoload);
$autoload_second = mysqli_fetch_assoc($autoload);
echo mysqli_num_rows($autoload);
echo ":";
echo $autoload_first["option_name"], ":", $autoload_first["autoload"];
echo ",";
echo $autoload_second["option_name"], ":", $autoload_second["autoload"];
echo "|";
$named = mysqli_execute_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name IN ('theme_mods','missing','home')");
$named_first = mysqli_fetch_assoc($named);
$named_second = mysqli_fetch_assoc($named);
echo mysqli_num_rows($named);
echo ":";
echo $named_first["option_id"], ":", $named_first["option_name"], ":", $named_first["autoload"];
echo ",";
echo $named_second["option_id"], ":", $named_second["option_name"], ":", $named_second["autoload"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|3:4:2:home,1:siteurl,3:theme_mods|2:siteurl:yes,theme_mods:on|2:3:theme_mods:on,2:home:no"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_reads_current_wordpress_option_name_value_row_sets_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options");
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
$all = mysqli_stmt_get_result($stmt);
$all_first = mysqli_fetch_assoc($all);
$all_second = mysqli_fetch_assoc($all);
$all_third = mysqli_fetch_assoc($all);
echo "|";
echo mysqli_num_rows($all);
echo ":";
echo mysqli_num_fields($all);
echo ":";
echo $all_first["option_name"], "=", $all_first["option_value"];
echo ",";
echo $all_second["option_name"], "=", $all_second["option_value"];
echo ",";
echo $all_third["option_name"], "=", $all_third["option_value"];
echo "|";
$autoload_stmt = mysqli_prepare($handle, "SELECT `option_name`, `option_value` FROM `wp_options` WHERE `autoload` IN ( 'yes', 'on', 'auto-on', 'auto' )");
mysqli_stmt_execute($autoload_stmt);
$autoload = mysqli_stmt_get_result($autoload_stmt);
$autoload_first = mysqli_fetch_assoc($autoload);
$autoload_second = mysqli_fetch_assoc($autoload);
echo mysqli_num_rows($autoload);
echo ":";
echo $autoload_first["option_name"], "=", $autoload_first["option_value"];
echo ",";
echo $autoload_second["option_name"], "=", $autoload_second["option_value"];
echo "|";
$named = mysqli_execute_query($handle, "SELECT `option_name`, `option_value` FROM `wp_options` WHERE `option_name` IN ('theme_mods','missing','home')");
$named_first = mysqli_fetch_assoc($named);
$named_second = mysqli_fetch_assoc($named);
echo mysqli_num_rows($named);
echo ":";
echo $named_first["option_name"], "=", $named_first["option_value"];
echo ",";
echo $named_second["option_name"], "=", $named_second["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "executed|3:2:home=https://home.test,siteurl=https://example.test,theme_mods=theme-db|2:siteurl=https://example.test,theme_mods=theme-db|2:theme_mods=theme-db,home=https://home.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_option_autoload_equality_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogname', 'Example Blog', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'auto-on')");
$direct = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload = 'yes'");
$direct_first = mysqli_fetch_assoc($direct);
$direct_second = mysqli_fetch_assoc($direct);
echo mysqli_num_rows($direct);
echo ":";
echo $direct_first["option_name"], "=", $direct_first["option_value"];
echo ",";
echo $direct_second["option_name"], "=", $direct_second["option_value"];
echo "|";
$prepared = mysqli_execute_query($handle, "SELECT `option_name`, `option_value` FROM `wp_options` WHERE `autoload` = ?", array("auto-on"));
$prepared_row = mysqli_fetch_assoc($prepared);
echo mysqli_num_rows($prepared);
echo ":";
echo $prepared_row["option_name"], "=", $prepared_row["option_value"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload = ?");
$autoload = "yes";
mysqli_stmt_bind_param($stmt, "s", $autoload);
$autoload = "no";
mysqli_stmt_execute($stmt);
$stmt_rows = mysqli_stmt_get_result($stmt);
$stmt_row = mysqli_fetch_assoc($stmt_rows);
echo mysqli_num_rows($stmt_rows);
echo ":";
echo $stmt_row["option_name"], "=", $stmt_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:blogname=Example Blog,siteurl=https://example.test|1:theme_mods=theme-db|1:home=https://home.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_prepared_option_value_limit_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '12345', 'no')");
$direct = mysqli_execute_query($handle, "SELECT `option_value` FROM `wp_options` WHERE `option_name` = ? LIMIT 1", array("_transient_update_plugins"));
$direct_row = mysqli_fetch_assoc($direct);
echo mysqli_num_rows($direct);
echo ":";
echo $direct_row["option_value"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT option_value FROM wp_options WHERE option_name = ? LIMIT 1");
$name = "_transient_update_plugins";
mysqli_stmt_bind_param($stmt, "s", $name);
$name = "_transient_timeout_update_plugins";
mysqli_stmt_execute($stmt);
$rows = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($rows);
echo mysqli_num_rows($rows);
echo ":";
echo $row["option_value"];
echo "|";
$missing = mysqli_execute_query($handle, "SELECT option_value FROM wp_options WHERE option_name = ? LIMIT 1", array("_transient_missing"));
echo mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:plugin-payload|1:12345|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_option_object_rows_without_limit_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$direct = mysqli_query($handle, "SELECT * FROM wp_options WHERE option_name = 'siteurl'");
$direct_row = mysqli_fetch_object($direct);
echo mysqli_num_rows($direct);
echo ":";
echo $direct_row->option_id, ":", $direct_row->option_name, ":", $direct_row->option_value, ":", $direct_row->autoload;
echo "|";
$prepared = mysqli_execute_query($handle, "SELECT * FROM `wp_options` WHERE `option_name` = ?", array("theme_mods"));
$prepared_row = mysqli_fetch_object($prepared);
echo mysqli_num_fields($prepared);
echo ":";
echo $prepared_row->option_id, ":", $prepared_row->option_name, ":", $prepared_row->option_value, ":", $prepared_row->autoload;
echo "|";
$missing = mysqli_execute_query($handle, "SELECT * FROM wp_options WHERE option_name = ?", array("missing"));
echo mysqli_num_rows($missing);
echo ":";
echo mysqli_fetch_object($missing) === false ? "missing" : "row";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:1:siteurl:https://example.test:yes|4:2:theme_mods:theme-db:on|0:missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_reads_current_wordpress_explicit_option_rows_without_limit_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('theme_mods', 'theme-db', 'on')");
$direct = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = 'siteurl'");
$direct_row = mysqli_fetch_object($direct);
echo mysqli_num_fields($direct);
echo ":";
echo $direct_row->option_id, ":", $direct_row->option_name, ":", $direct_row->option_value, ":", $direct_row->autoload;
echo "|";
$prepared = mysqli_execute_query($handle, "SELECT `option_id`, `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE `option_name` = ?", array("theme_mods"));
$prepared_value = mysqli_fetch_column($prepared, 2);
echo mysqli_num_rows($prepared);
echo ":";
echo $prepared_value;
echo "|";
$missing = mysqli_execute_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ?", array("missing"));
echo mysqli_num_fields($missing);
echo ":";
echo mysqli_fetch_assoc($missing) === false ? "missing" : "row";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "4:1:siteurl:https://example.test:yes|1:theme-db|0:missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_id_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$result = mysqli_query($handle, "SELECT option_id FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo mysqli_num_rows($result);
echo ":";
echo mysqli_num_fields($result);
echo ":";
echo $row["option_id"];
echo "|";
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'duplicate', 'no')") ? "duplicate" : "rejected";
echo "|";
$again = mysqli_query($handle, "SELECT `option_id` FROM `wp_options` WHERE `option_name` = 'siteurl' LIMIT 1");
$again_row = mysqli_fetch_assoc($again);
echo $again_row["option_id"];
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('home', 'https://home.test', 'yes')");
$direct = mysqli_execute_query($handle, "SELECT option_id FROM wp_options WHERE option_name = ? LIMIT 1", array("home"));
$direct_row = mysqli_fetch_assoc($direct);
echo $direct_row["option_id"];
echo "|";
$stmt = mysqli_prepare($handle, "SELECT `option_id` FROM `wp_options` WHERE `option_name` = ? LIMIT 1");
$name = "siteurl";
mysqli_stmt_bind_param($stmt, "s", $name);
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$prepared = mysqli_stmt_get_result($stmt);
$prepared_row = mysqli_fetch_assoc($prepared);
echo $prepared_row["option_id"];
echo "|";
$missing = mysqli_query($handle, "SELECT option_id FROM wp_options WHERE option_name = 'missing' LIMIT 1");
echo mysqli_num_rows($missing);
echo ":";
echo mysqli_num_fields($missing);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:1:1|rejected|1|2|executed|1|0:0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_escaped_wordpress_option_literals() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$name = "owner's option";
$value = "O'Reilly";
$escaped_name = mysqli_real_escape_string($handle, $name);
$escaped_value = mysqli_real_escape_string($handle, $value);
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('$escaped_name', '$escaped_value', 'yes')") ? "insert" : "failed";
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = '$escaped_name' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
$new_value = "updated owner's value";
$escaped_new_value = mysqli_real_escape_string($handle, $new_value);
echo mysqli_query($handle, "UPDATE wp_options SET option_value = '$escaped_new_value' WHERE option_name = '$escaped_name'") ? "update" : "failed";
echo "|";
$updated = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = '$escaped_name' LIMIT 1");
$updated_row = mysqli_fetch_assoc($updated);
echo $updated_row["option_value"];
echo "|";
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name = '$escaped_name'") ? "delete" : "failed";
echo "|";
$deleted = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = '$escaped_name' LIMIT 1");
echo mysqli_num_rows($deleted);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "insert|O'Reilly|update|updated owner's value|delete|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_update_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "UPDATE wp_options SET option_value = 'https://updated.test' WHERE option_name = 'siteurl'") ? "updated" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
echo mysqli_query($handle, "UPDATE wp_options SET option_value = 'missing' WHERE option_name = 'home'") ? "missing-update" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|1|https://updated.test|missing-update|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_value_and_autoload_update_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'before', 'no')");
echo mysqli_query($handle, "UPDATE wp_options SET option_value = 'after', autoload = 'auto-on' WHERE option_name = 'blogdescription'") ? "updated" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"], ":", $row["autoload"];
echo "|";
echo mysqli_query($handle, "UPDATE `wp_options` SET `option_value` = 'missing-value', `autoload` = 'yes' WHERE `option_name` = 'missing'") ? "missing-update" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "updated|1|after:auto-on|missing-update|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_autoload_update_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'value-kept', 'no')");
echo mysqli_query($handle, "UPDATE wp_options SET autoload = 'auto-off' WHERE option_name = 'blogdescription'") ? "updated" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"], ":", $row["autoload"];
echo "|";
echo mysqli_query($handle, "UPDATE `wp_options` SET `autoload` = 'yes' WHERE `option_name` = 'missing'") ? "missing-update" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "updated|1|value-kept:auto-off|missing-update|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_records_current_wordpress_option_delete_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name = 'siteurl'") ? "deleted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
echo mysqli_num_rows($result);
echo "|";
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name = 'home'") ? "missing-delete" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "deleted|1|0|missing-delete|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_deletes_current_wordpress_option_name_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_feed_mod', 'cached-feed', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '123456', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name IN ('_transient_feed_mod','_transient_timeout_feed_mod','missing','_transient_feed_mod')") ? "deleted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$transient = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN ('_transient_feed_mod','_transient_timeout_feed_mod','siteurl')");
echo mysqli_num_rows($transient);
echo "|";
$row = mysqli_fetch_assoc($transient);
echo $row["option_name"], "=", $row["option_value"];
echo "|";
mysqli_query($handle, "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES ('_transient_feed_mod', 'cached-again', 'no')");
echo mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` IN ('_transient_feed_mod','missing')") ? "execute-deleted" : "execute-failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$again = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = '_transient_feed_mod' LIMIT 1");
echo mysqli_num_rows($again);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "deleted|2|1|siteurl=https://example.test|execute-deleted|1|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_statement_deletes_current_wordpress_option_name_lists_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_feed_mod', 'cached-feed', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '123456', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name IN (?, ?, ?, ?)");
$one = "_transient_feed_mod";
$two = "_transient_timeout_feed_mod";
$missing = "missing";
$duplicate = "_transient_feed_mod";
mysqli_stmt_bind_param($stmt, "ssss", $one, $two, $missing, $duplicate);
echo mysqli_stmt_execute($stmt) ? "deleted" : "failed";
echo "|";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN ('_transient_feed_mod','_transient_timeout_feed_mod','siteurl')");
echo mysqli_num_rows($rows);
echo "|";
$row = mysqli_fetch_assoc($rows);
echo $row["option_name"], "=", $row["option_value"];
echo "|";
mysqli_query($handle, "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES ('_transient_feed_mod', 'cached-again', 'no')");
$direct = mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` IN (?, ?)", array("_transient_feed_mod", "missing"));
echo $direct ? "execute-deleted" : "execute-failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$again = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = '_transient_feed_mod' LIMIT 1");
echo mysqli_num_rows($again);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "deleted|2|2|1|siteurl=https://example.test|execute-deleted|1|0"
    );
    assert_eq!(execution.exit_code, 0);

    let non_string = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name IN (?, ?)");
$name = "siteurl";
$other = 42;
mysqli_stmt_bind_param($stmt, "si", $name, $other);
mysqli_stmt_execute($stmt);
"#,
    )
    .unwrap_err();

    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 9);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call mysqli_stmt_execute(): prepared wp_options option-name-list delete requires string option name parameters in the current subset"
    );
}

#[test]
fn mysqli_deletes_current_wordpress_transient_prefix_options_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_feed_mod', 'cached-feed', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '123456', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_theme_roots', '789000', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'core-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_timeout_update_core', '456789', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name LIKE '_transient_timeout_%'") ? "direct" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$remaining = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE '_transient_%'");
$row = mysqli_fetch_assoc($remaining);
echo mysqli_num_rows($remaining);
echo ":";
echo $row["option_name"], "=", $row["option_value"];
echo "|";
$stmt = mysqli_prepare($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ?");
$site_prefix = "\\_site_transient\\_%";
mysqli_stmt_bind_param($stmt, "s", $site_prefix);
echo mysqli_stmt_execute($stmt) ? "prepared" : "failed";
echo ":";
echo mysqli_stmt_affected_rows($stmt);
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "DELETE FROM wp_options WHERE `option_name` LIKE ?", array("_transient_%")) ? "execute" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$left = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $left[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $left);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct:2|1:_transient_feed_mod=cached-feed|prepared:2:2|execute:1|siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_like_deletes_apply_wordpress_option_wildcards() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'site-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name LIKE ?");
mysqli_stmt_execute($stmt, array("_transient_%"));
echo "wild=", mysqli_stmt_affected_rows($stmt), ":", mysqli_affected_rows($handle);
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
echo mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!'", array("!_transient!_%")) ? "escape" : "failed";
echo ":", mysqli_affected_rows($handle);
echo "|";
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$left = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $left[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $left);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wild=2:2|escape:1|_site_transient_update_core=site-payload,siteurl=https://example.test,xtransient-update-plugins=wildcard-payload"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_direct_like_deletes_apply_wordpress_option_escape_wildcards() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'site-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name LIKE '_transient_%'") ? "wild" : "failed";
echo ":", mysqli_affected_rows($handle);
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-update-plugins', 'wildcard-payload', 'no')");
echo mysqli_query($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE '!_transient!_%' ESCAPE '!'") ? "escape" : "failed";
echo ":", mysqli_affected_rows($handle);
echo "|";
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$left = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $left[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $left);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wild:2|escape:1|_site_transient_update_core=site-payload,siteurl=https://example.test,xtransient-update-plugins=wildcard-payload"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_deletes_current_wordpress_expired_transient_timeout_options_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '500', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_timeout_update_core', '120', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300") ? "direct" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$remaining = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '_transient_timeout_%' ORDER BY option_name");
echo mysqli_num_rows($remaining);
echo ":";
$row = mysqli_fetch_assoc($remaining);
echo $row["option_name"];
echo "|";
$stmt = mysqli_prepare($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ? AND `option_value` < ?");
$prefix = "\\_site_transient\\_timeout\\_%";
$threshold = "300";
mysqli_stmt_bind_param($stmt, "ss", $prefix, $threshold);
echo mysqli_stmt_execute($stmt) ? "prepared" : "failed";
echo ":";
echo mysqli_stmt_affected_rows($stmt);
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "DELETE FROM wp_options WHERE `option_name` LIKE ? AND `option_value` < ?", array("_transient_timeout_%", 600)) ? "execute" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$left = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$parts = array();
while ($row = mysqli_fetch_assoc($left)) {
    $parts[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $parts);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct:1|1:_transient_timeout_update_plugins|prepared:1:1|execute:1|_transient_update_plugins=payload,siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_expired_transient_timeout_deletes_apply_wordpress_option_wildcards() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_fresh', '900', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name LIKE '_transient_timeout_%' AND option_value < 300") ? "wild" : "failed";
echo ":", mysqli_affected_rows($handle);
echo "|";
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-feed_mod', '110', 'no')");
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name LIKE ? AND option_value < ?");
mysqli_stmt_execute($stmt, array("\\_transient\\_timeout\\_%", 300));
echo "escape=", mysqli_stmt_affected_rows($stmt), ":", mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ? AND `option_value` < ?", array("_transient_timeout_%", "300")) ? "execute" : "failed";
echo ":", mysqli_affected_rows($handle);
echo "|";
$rows = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$left = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $left[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $left);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wild:2|escape=1:1|execute:1|_transient_timeout_fresh=900,siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_deletes_current_wordpress_expired_transient_payload_pairs_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_feed_mod', 'cached-feed', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_feed_mod', '100', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '500', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_core', 'core-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_timeout_update_core', '120', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE a, b FROM wp_options a, wp_options b WHERE a.option_name LIKE '_transient_%' AND a.option_name NOT LIKE '_transient_timeout_%' AND b.option_name = CONCAT( '_transient_timeout_', SUBSTRING( a.option_name, 12 ) ) AND b.option_value < 300") ? "direct" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$stmt = mysqli_prepare($handle, "DELETE a, b FROM wp_options a, wp_options b WHERE a.option_name LIKE ? AND a.option_name NOT LIKE ? AND b.option_name = CONCAT( '_site_transient_timeout_', SUBSTRING( a.option_name, 17 ) ) AND b.option_value < ?");
mysqli_stmt_execute($stmt, array("\\_site_transient\\_%", "\\_site_transient\\_timeout\\_%", "300"));
echo mysqli_stmt_affected_rows($stmt);
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_execute_query($handle, "DELETE a, b FROM wp_options a, wp_options b WHERE a.option_name LIKE ? AND a.option_name NOT LIKE ? AND b.option_name = CONCAT( '_transient_timeout_', SUBSTRING( a.option_name, 12 ) ) AND b.option_value < ?", array("_transient_%", "_transient_timeout_%", 600)) ? "execute" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
echo "|";
$left = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
$parts = array();
while ($row = mysqli_fetch_assoc($left)) {
    $parts[] = $row["option_name"] . "=" . $row["option_value"];
}
echo implode(",", $parts);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "direct:2|2:2|execute:2|siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_stmt_insert_id_tracks_current_wordpress_prepared_option_insert_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$stmt = mysqli_prepare($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)");
$name = "siteurl";
$value = "https://example.test";
$autoload = "yes";
mysqli_stmt_bind_param($stmt, "sss", $name, $value, $autoload);
echo mysqli_stmt_execute($stmt) ? "insert" : "failed";
echo ":";
echo mysqli_stmt_insert_id($stmt);
echo ":";
echo mysqli_insert_id($handle);
echo "|";
$name = "_transient_feed_mod";
$value = "cached-feed";
$autoload = "no";
echo mysqli_stmt_execute($stmt) ? "transient" : "failed";
echo ":";
echo mysqli_stmt_insert_id($stmt);
echo ":";
echo mysqli_insert_id($handle);
echo "|";
$name = "siteurl";
$value = "duplicate";
$autoload = "no";
echo mysqli_stmt_execute($stmt) ? "duplicate" : "duplicate-rejected";
echo ":";
echo mysqli_stmt_insert_id($stmt);
echo ":";
echo mysqli_stmt_affected_rows($stmt);
echo "|";
$update = mysqli_prepare($handle, "UPDATE wp_options SET option_value = ? WHERE option_name = ?");
$new_value = "cached-new";
$name = "_transient_feed_mod";
mysqli_stmt_execute($update, array($new_value, $name));
echo mysqli_stmt_insert_id($update);
echo ":";
echo mysqli_stmt_affected_rows($update);
echo "|";
$rows = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options");
$parts = array();
while ($row = mysqli_fetch_assoc($rows)) {
    $parts[] = $row["option_id"] . ":" . $row["option_name"] . "=" . $row["option_value"] . ":" . $row["autoload"];
}
echo implode(",", $parts);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "insert:1:1|transient:2:2|duplicate-rejected:0:0|0:1|2:_transient_feed_mod=cached-new:no,1:siteurl=https://example.test:yes"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_reads_current_wordpress_option_rows_from_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogname', 'Example Blog', 'no')");
$autoload = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
echo mysqli_num_rows($autoload);
echo "|";
$row = mysqli_fetch_assoc($autoload);
echo $row["option_name"], "=", $row["option_value"];
echo "|";
$all = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
echo mysqli_num_rows($all);
echo "|";
$primed = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN ('blogname','missing','siteurl')");
echo mysqli_num_rows($primed);
echo "|";
$first = mysqli_fetch_assoc($primed);
$second = mysqli_fetch_assoc($primed);
echo $first["option_name"], "=", $first["option_value"], ";", $second["option_name"], "=", $second["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|siteurl=https://example.test|2|2|blogname=Example Blog;siteurl=https://example.test"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_exposes_current_wordpress_options_schema_probe_rows() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$tables = mysqli_query($handle, "SHOW TABLES LIKE 'wp_options'");
$table = mysqli_fetch_row($tables);
echo mysqli_num_rows($tables);
echo ":";
echo $table[0];
echo "|";
$describe = mysqli_query($handle, "DESCRIBE wp_options;");
echo mysqli_num_fields($describe);
echo ":";
$first = mysqli_fetch_assoc($describe);
echo $first["Field"], ":", $first["Type"], ":", $first["Key"], ":", $first["Extra"];
echo "|";
$columns = mysqli_query($handle, "SHOW FULL COLUMNS FROM `wp_options`");
echo mysqli_num_rows($columns);
echo ":";
while ($column = mysqli_fetch_assoc($columns)) {
    if ($column["Field"] === "autoload") {
        echo $column["Type"], ":", $column["Default"], ":", $column["Collation"];
    }
}
echo "|";
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_options");
echo mysqli_num_fields($indexes);
echo ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Column_name"], ":", $index["Non_unique"], ":", $index["Index_type"], ";";
}
echo "|";
$keys = mysqli_query($handle, "SHOW KEYS FROM `wp_options`");
echo mysqli_num_rows($keys);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:wp_options|6:option_id:bigint(20) unsigned:PRI:auto_increment|4:varchar(20):yes:utf8mb4_unicode_ci|15:PRIMARY:option_id:0:BTREE;option_name:option_name:0:BTREE;|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_tracks_bounded_wordpress_schema_create_alter_state() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_query($handle, "CREATE TABLE wp_phptest (id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', payload longtext NOT NULL, PRIMARY KEY  (id), KEY slug (slug)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci") ? "created" : "failed";
echo ":";
echo mysqli_affected_rows($handle);
mysqli_query($handle, "ALTER TABLE wp_phptest ADD COLUMN checksum varchar(64) NOT NULL default '', ADD UNIQUE KEY checksum (checksum)");
$tables = mysqli_query($handle, "SHOW TABLES LIKE 'wp_phptest'");
$table = mysqli_fetch_row($tables);
echo "|table=", mysqli_num_rows($tables), ":", $table[0];
$describe = mysqli_query($handle, "DESCRIBE `wp_phptest`");
echo "|describe=", mysqli_num_fields($describe), ":";
while ($column = mysqli_fetch_assoc($describe)) {
    echo $column["Field"], ":", $column["Type"], ":", $column["Key"], ":", $column["Default"], ";";
}
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_phptest");
echo "|index=", mysqli_num_rows($indexes), ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Column_name"], ":", $index["Non_unique"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "created:0|table=1:wp_phptest|describe=6:id:bigint(20) unsigned:PRI:;slug:varchar(191):MUL:;payload:longtext::;checksum:varchar(64):UNI:;|index=3:PRIMARY:id:0;slug:slug:1;checksum:checksum:0;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_tracks_bounded_wordpress_schema_multi_column_index_parts() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_posts (ID bigint(20) unsigned NOT NULL auto_increment, post_name varchar(200) NOT NULL default '', post_type varchar(20) NOT NULL default 'post', post_status varchar(20) NOT NULL default 'publish', post_date datetime NOT NULL default '0000-00-00 00:00:00', PRIMARY KEY  (ID), KEY type_status_date (post_type, post_status, post_date, ID), KEY post_name (post_name(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_posts ADD KEY name_date (post_name(191), post_date)");
$indexes = mysqli_query($handle, "SHOW INDEX FROM `wp_probe_posts`");
echo mysqli_num_rows($indexes), ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Seq_in_index"], ":", $index["Column_name"], ":", $index["Sub_part"], ";";
}
echo "|";
$columns = mysqli_query($handle, "DESCRIBE wp_probe_posts");
while ($column = mysqli_fetch_assoc($columns)) {
    echo $column["Field"], ":", $column["Key"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "8:PRIMARY:1:ID:;type_status_date:1:post_type:;type_status_date:2:post_status:;type_status_date:3:post_date:;type_status_date:4:ID:;post_name:1:post_name:191;name_date:1:post_name:191;name_date:2:post_date:;|ID:PRI;post_name:MUL;post_type:MUL;post_status:MUL;post_date:MUL;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_tracks_bounded_wordpress_schema_index_ordering_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_lookup (lookup_id bigint(20) unsigned NOT NULL auto_increment, object_id bigint(20) unsigned NOT NULL default 0, meta_key varchar(191) NOT NULL default '', updated_at datetime NOT NULL default '0000-00-00 00:00:00', PRIMARY KEY  (lookup_id), KEY object_recent (object_id ASC, updated_at DESC), KEY meta_recent (meta_key(100) DESC)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_lookup ADD KEY object_meta_recent (object_id, meta_key(100) ASC, updated_at DESC)");
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_probe_lookup");
echo mysqli_num_rows($indexes), ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Seq_in_index"], ":", $index["Column_name"], ":", $index["Sub_part"], ":", $index["Collation"], ";";
}
$create = mysqli_query($handle, "SHOW CREATE TABLE wp_probe_lookup");
$row = mysqli_fetch_assoc($create);
echo "|", $row["Create Table"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "7:PRIMARY:1:lookup_id::A;object_recent:1:object_id::A;object_recent:2:updated_at::D;meta_recent:1:meta_key:100:D;object_meta_recent:1:object_id::A;object_meta_recent:2:meta_key:100:A;object_meta_recent:3:updated_at::D;|CREATE TABLE `wp_probe_lookup` (\n  `lookup_id` bigint(20) unsigned NOT NULL auto_increment,\n  `object_id` bigint(20) unsigned NOT NULL DEFAULT '0',\n  `meta_key` varchar(191) NOT NULL DEFAULT '',\n  `updated_at` datetime NOT NULL DEFAULT '0000-00-00 00:00:00',\n  PRIMARY KEY (`lookup_id`),\n  KEY `object_recent` (`object_id`,`updated_at` DESC),\n  KEY `meta_recent` (`meta_key`(100) DESC),\n  KEY `object_meta_recent` (`object_id`,`meta_key`(100),`updated_at` DESC)\n) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_preserves_bounded_wordpress_schema_column_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_column_meta (inline_id bigint(20) unsigned NOT NULL auto_increment PRIMARY KEY, slug varchar(191) NOT NULL default 'draft\\'s', flag tinyint(1) NOT NULL default 0, maybe varchar(20) DEFAULT NULL, unique_code varchar(32) NOT NULL UNIQUE KEY, plain_key varchar(32) KEY) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_column_meta ADD COLUMN token varchar(20) NOT NULL default 'x' KEY, MODIFY COLUMN flag tinyint(1) NOT NULL default 1, CHANGE COLUMN slug post_slug varchar(191) NOT NULL default 'post' KEY");
$describe = mysqli_query($handle, "DESCRIBE wp_probe_column_meta");
while ($column = mysqli_fetch_assoc($describe)) {
    echo $column["Field"], ":", $column["Type"], ":", $column["Null"], ":", $column["Key"], ":", $column["Default"], ":", $column["Extra"], ";";
}
echo "|";
$full = mysqli_query($handle, "SHOW FULL COLUMNS FROM wp_probe_column_meta LIKE 'post_%'");
$full_row = mysqli_fetch_assoc($full);
echo $full_row["Field"], ":", $full_row["Collation"], ":", $full_row["Default"], ":", $full_row["Key"];
echo "|";
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_probe_column_meta");
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Column_name"], ":", $index["Non_unique"], ";";
}
echo "|";
$create = mysqli_query($handle, "SHOW CREATE TABLE wp_probe_column_meta");
$create_row = mysqli_fetch_assoc($create);
echo $create_row["Create Table"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "inline_id:bigint(20) unsigned:NO:PRI::auto_increment;post_slug:varchar(191):NO:MUL:post:;flag:tinyint(1):NO::1:;maybe:varchar(20):YES:::;unique_code:varchar(32):NO:UNI::;plain_key:varchar(32):YES:MUL::;token:varchar(20):NO:MUL:x:;|post_slug:utf8mb4_unicode_ci:post:MUL|PRIMARY:inline_id:0;unique_code:unique_code:0;plain_key:plain_key:1;token:token:1;post_slug:post_slug:1;|CREATE TABLE `wp_probe_column_meta` (\n  `inline_id` bigint(20) unsigned NOT NULL auto_increment,\n  `post_slug` varchar(191) NOT NULL DEFAULT 'post',\n  `flag` tinyint(1) NOT NULL DEFAULT '1',\n  `maybe` varchar(20) NULL DEFAULT NULL,\n  `unique_code` varchar(32) NOT NULL,\n  `plain_key` varchar(32) NULL,\n  `token` varchar(20) NOT NULL DEFAULT 'x',\n  PRIMARY KEY (`inline_id`),\n  UNIQUE KEY `unique_code` (`unique_code`),\n  KEY `plain_key` (`plain_key`),\n  KEY `token` (`token`),\n  KEY `post_slug` (`post_slug`)\n) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_tracks_bounded_wordpress_schema_fulltext_spatial_index_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_search (ID bigint(20) unsigned NOT NULL auto_increment, post_title text NOT NULL, post_content longtext NOT NULL, geo point NOT NULL, PRIMARY KEY  (ID), FULLTEXT KEY title_content (post_title, post_content), SPATIAL KEY geo_lookup (geo)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_search ADD FULLTEXT INDEX content_only (post_content), ADD SPATIAL INDEX geo_recent (geo)");
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_probe_search");
echo mysqli_num_rows($indexes), ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Seq_in_index"], ":", $index["Column_name"], ":", $index["Non_unique"], ":", $index["Index_type"], ";";
}
$create = mysqli_query($handle, "SHOW CREATE TABLE wp_probe_search");
$row = mysqli_fetch_assoc($create);
echo "|", $row["Create Table"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "6:PRIMARY:1:ID:0:BTREE;title_content:1:post_title:1:FULLTEXT;title_content:2:post_content:1:FULLTEXT;geo_lookup:1:geo:1:SPATIAL;content_only:1:post_content:1:FULLTEXT;geo_recent:1:geo:1:SPATIAL;|CREATE TABLE `wp_probe_search` (\n  `ID` bigint(20) unsigned NOT NULL auto_increment,\n  `post_title` text NOT NULL,\n  `post_content` longtext NOT NULL,\n  `geo` point NOT NULL,\n  PRIMARY KEY (`ID`),\n  FULLTEXT KEY `title_content` (`post_title`,`post_content`),\n  SPATIAL KEY `geo_lookup` (`geo`),\n  FULLTEXT KEY `content_only` (`post_content`),\n  SPATIAL KEY `geo_recent` (`geo`)\n) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_tracks_bounded_wordpress_schema_column_and_index_changes() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_terms (term_id bigint(20) unsigned NOT NULL auto_increment, slug varchar(191) NOT NULL default '', payload longtext NOT NULL, checksum varchar(64) NOT NULL default '', PRIMARY KEY  (term_id), KEY slug (slug), KEY checksum (checksum)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_terms CHANGE COLUMN slug name varchar(200) NOT NULL default '', MODIFY COLUMN payload longtext NULL, DROP COLUMN checksum, DROP KEY slug, ADD KEY name (name(191))");
echo "affected=", mysqli_affected_rows($handle);
$columns = mysqli_query($handle, "DESCRIBE wp_probe_terms");
echo "|columns=";
while ($column = mysqli_fetch_assoc($columns)) {
    echo $column["Field"], ":", $column["Type"], ":", $column["Null"], ":", $column["Key"], ";";
}
$indexes = mysqli_query($handle, "SHOW INDEX FROM wp_probe_terms");
echo "|indexes=", mysqli_num_rows($indexes), ":";
while ($index = mysqli_fetch_assoc($indexes)) {
    echo $index["Key_name"], ":", $index["Column_name"], ":", $index["Sub_part"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "affected=0|columns=term_id:bigint(20) unsigned:NO:PRI;name:varchar(200):NO:MUL;payload:longtext:YES:;|indexes=2:PRIMARY:term_id:;name:name:191;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_exposes_bounded_wordpress_schema_show_create_table() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_links (link_id bigint(20) unsigned NOT NULL auto_increment, link_url varchar(255) NOT NULL default '', link_name varchar(255) NOT NULL default '', link_visible varchar(20) NOT NULL default 'Y', PRIMARY KEY  (link_id), KEY link_visible (link_visible), KEY link_name (link_name(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "ALTER TABLE wp_probe_links ADD COLUMN link_updated datetime NOT NULL default '0000-00-00 00:00:00', DROP KEY link_visible, ADD KEY visible_name (link_visible, link_name(191))");
$result = mysqli_query($handle, "SHOW CREATE TABLE `wp_probe_links`");
echo mysqli_num_fields($result), ":";
$row = mysqli_fetch_assoc($result);
echo $row["Table"], "|";
echo $row["Create Table"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2:wp_probe_links|CREATE TABLE `wp_probe_links` (\n  `link_id` bigint(20) unsigned NOT NULL auto_increment,\n  `link_url` varchar(255) NOT NULL DEFAULT '',\n  `link_name` varchar(255) NOT NULL DEFAULT '',\n  `link_visible` varchar(20) NOT NULL DEFAULT 'Y',\n  `link_updated` datetime NOT NULL DEFAULT '0000-00-00 00:00:00',\n  PRIMARY KEY (`link_id`),\n  KEY `link_name` (`link_name`(191)),\n  KEY `visible_name` (`link_visible`,`link_name`(191))\n) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_exposes_bounded_wordpress_schema_table_status() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NULL, PRIMARY KEY  (meta_id), KEY meta_key (meta_key(191))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$status = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp_probe_meta'");
echo mysqli_num_fields($status), ":";
$row = mysqli_fetch_assoc($status);
echo $row["Name"], "|", $row["Engine"], "|", $row["Rows"], "|", $row["Collation"], "|", $row["Create_options"];
echo "|";
$where = mysqli_query($handle, "SHOW TABLE STATUS WHERE Name = 'wp_probe_meta'");
echo mysqli_num_rows($where);
echo "|";
$missing = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp_missing_meta'");
echo mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "18:wp_probe_meta|InnoDB|0|utf8mb4_unicode_520_ci||1|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_filters_bounded_wordpress_schema_column_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_options (option_id bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, autoload varchar(20) NOT NULL default 'yes', PRIMARY KEY  (option_id), UNIQUE KEY option_name (option_name)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$like = mysqli_query($handle, "SHOW FULL COLUMNS FROM `wp_probe_options` LIKE 'option_name'");
echo mysqli_num_rows($like), ":";
$like_row = mysqli_fetch_assoc($like);
echo $like_row["Field"], ":", $like_row["Type"], ":", $like_row["Collation"], ":", $like_row["Key"];
echo "|";
$where = mysqli_query($handle, "SHOW COLUMNS FROM wp_probe_options WHERE Field = 'option_value'");
echo mysqli_num_rows($where), ":";
$where_row = mysqli_fetch_assoc($where);
echo $where_row["Field"], ":", $where_row["Null"], ":", $where_row["Collation"];
echo "|";
$describe = mysqli_query($handle, "DESCRIBE wp_probe_options autoload");
echo mysqli_num_rows($describe), ":";
$describe_row = mysqli_fetch_assoc($describe);
echo $describe_row["Field"], ":", $describe_row["Default"];
echo "|";
$missing = mysqli_query($handle, "SHOW FULL COLUMNS FROM wp_probe_options LIKE 'missing_column'");
echo mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1:option_name:varchar(191):utf8mb4_unicode_ci:UNI|1:option_value:NO:utf8mb4_unicode_ci|1:autoload:yes|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_filters_bounded_wordpress_schema_metadata_with_percent_like() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_options (option_id bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', option_value longtext NOT NULL, autoload varchar(20) NOT NULL default 'yes', PRIMARY KEY  (option_id), UNIQUE KEY option_name (option_name)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, option_id bigint(20) unsigned NOT NULL default 0, meta_key varchar(255) NULL, PRIMARY KEY  (meta_id), KEY option_id (option_id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$tables = mysqli_query($handle, "SHOW TABLES LIKE 'wp_probe_%'");
echo "tables=", mysqli_num_rows($tables), ":";
while ($table = mysqli_fetch_row($tables)) {
    echo $table[0], ";";
}
$status = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp_probe_%'");
echo "|status=", mysqli_num_rows($status), ":";
while ($row = mysqli_fetch_assoc($status)) {
    echo $row["Name"], ":", $row["Collation"], ";";
}
$columns = mysqli_query($handle, "SHOW FULL COLUMNS FROM `wp_probe_options` LIKE 'option_%'");
echo "|columns=", mysqli_num_rows($columns), ":";
while ($column = mysqli_fetch_assoc($columns)) {
    echo $column["Field"], ":", $column["Key"], ";";
}
$where = mysqli_query($handle, "SHOW COLUMNS FROM wp_probe_meta WHERE Field LIKE 'meta_%'");
echo "|where=", mysqli_num_rows($where), ":";
while ($column = mysqli_fetch_assoc($where)) {
    echo $column["Field"], ":", $column["Null"], ";";
}
$missing = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp_missing_%'");
echo "|missing=", mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "tables=2:wp_probe_meta;wp_probe_options;|status=2:wp_probe_meta:utf8mb4_unicode_520_ci;wp_probe_options:utf8mb4_unicode_ci;|columns=3:option_id:PRI;option_name:UNI;option_value:;|where=2:meta_id:NO;meta_key:YES;|missing=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_filters_bounded_wordpress_schema_metadata_with_underscore_and_escaped_like() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_meta (meta_id bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NULL, metadata varchar(20) NULL, PRIMARY KEY  (meta_id), KEY meta_key (meta_key)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
mysqli_query($handle, "CREATE TABLE wpXprobeXmeta (id bigint(20) unsigned NOT NULL, PRIMARY KEY  (id)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_520_ci");
$underscore = mysqli_query($handle, "SHOW TABLES LIKE 'wp_probe_meta'");
echo "underscore=", mysqli_num_rows($underscore), ":";
while ($table = mysqli_fetch_row($underscore)) {
    echo $table[0], ";";
}
$escaped = mysqli_query($handle, "SHOW TABLES LIKE 'wp\\_probe\\_meta'");
echo "|escaped=", mysqli_num_rows($escaped), ":";
while ($table = mysqli_fetch_row($escaped)) {
    echo $table[0], ";";
}
$single = mysqli_query($handle, "SHOW TABLE STATUS LIKE 'wp\\_probe\\_met_'");
echo "|status=", mysqli_num_rows($single), ":";
while ($row = mysqli_fetch_assoc($single)) {
    echo $row["Name"], ":", $row["Collation"], ";";
}
$columns = mysqli_query($handle, "SHOW FULL COLUMNS FROM `wp_probe_meta` LIKE 'meta\\_%'");
echo "|columns=", mysqli_num_rows($columns), ":";
while ($column = mysqli_fetch_assoc($columns)) {
    echo $column["Field"], ";";
}
$where = mysqli_query($handle, "SHOW COLUMNS FROM wp_probe_meta WHERE Field LIKE 'meta__d'");
echo "|where=", mysqli_num_rows($where), ":";
while ($column = mysqli_fetch_assoc($where)) {
    echo $column["Field"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "underscore=2:wpXprobeXmeta;wp_probe_meta;|escaped=1:wp_probe_meta;|status=1:wp_probe_meta:utf8mb4_unicode_ci;|columns=2:meta_id;meta_key;|where=1:meta_id;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_filters_bounded_wordpress_schema_index_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_index_filter (ID bigint(20) unsigned NOT NULL auto_increment, option_name varchar(191) NOT NULL default '', meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, post_content longtext NOT NULL, PRIMARY KEY  (ID), UNIQUE KEY option_name (option_name), KEY meta_lookup (meta_key(191), meta_value(10)), FULLTEXT KEY content_search (post_content)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$exact = mysqli_query($handle, "SHOW INDEXES FROM `wp_probe_index_filter` WHERE Key_name = 'meta_lookup'");
echo "exact=", mysqli_num_rows($exact), ":";
while ($index = mysqli_fetch_assoc($exact)) {
    echo $index["Key_name"], ":", $index["Seq_in_index"], ":", $index["Column_name"], ":", $index["Sub_part"], ";";
}
$like = mysqli_query($handle, "SHOW INDEX FROM wp_probe_index_filter WHERE `Key_name` LIKE 'content_%'");
echo "|like=", mysqli_num_rows($like), ":";
while ($index = mysqli_fetch_assoc($like)) {
    echo $index["Key_name"], ":", $index["Column_name"], ":", $index["Index_type"], ";";
}
$primary = mysqli_query($handle, "SHOW KEYS FROM wp_probe_index_filter WHERE `Key_name` = 'PRIMARY'");
$primary_row = mysqli_fetch_assoc($primary);
echo "|primary=", mysqli_num_rows($primary), ":", $primary_row["Key_name"], ":", $primary_row["Non_unique"];
$missing = mysqli_query($handle, "SHOW INDEX FROM wp_probe_index_filter WHERE Key_name LIKE 'missing_%'");
echo "|missing=", mysqli_num_rows($missing);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "exact=2:meta_lookup:1:meta_key:191;meta_lookup:2:meta_value:10;|like=1:content_search:post_content:FULLTEXT;|primary=1:PRIMARY:0|missing=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_filters_bounded_wordpress_schema_like_with_escape_clause() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_escape_filter (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10)), KEY literal_percent (meta_key(20))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$wild = mysqli_query($handle, "SHOW INDEX FROM wp_probe_escape_filter WHERE Key_name LIKE 'meta_%'");
echo "wild=", mysqli_num_rows($wild), ":";
while ($index = mysqli_fetch_assoc($wild)) {
    echo $index["Key_name"], ";";
}
$escaped = mysqli_query($handle, "SHOW INDEX FROM wp_probe_escape_filter WHERE `Key_name` LIKE 'meta!_%' ESCAPE '!'");
echo "|escaped=", mysqli_num_rows($escaped), ":";
while ($index = mysqli_fetch_assoc($escaped)) {
    echo $index["Key_name"], ":", $index["Column_name"], ";";
}
$percent = mysqli_query($handle, "SHOW KEYS FROM wp_probe_escape_filter WHERE Key_name LIKE 'literal!_%' ESCAPE '!'");
echo "|percent=", mysqli_num_rows($percent), ":";
$row = mysqli_fetch_assoc($percent);
echo $row["Key_name"], ":", $row["Sub_part"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "wild=2:meta_lookup;metaXlookup;|escaped=1:meta_lookup:meta_key;|percent=1:literal_percent:20"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_applies_bounded_no_backslash_escapes_to_schema_like_filters() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_sql_mode_filter (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10))) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$default = mysqli_query($handle, "SHOW INDEX FROM wp_probe_sql_mode_filter WHERE Key_name LIKE 'meta\\_%'");
echo "default=", mysqli_num_rows($default), ":";
while ($index = mysqli_fetch_assoc($default)) {
    echo $index["Key_name"], ";";
}
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = mysqli_query($handle, "SHOW INDEX FROM wp_probe_sql_mode_filter WHERE Key_name LIKE 'meta\\_%'");
echo "|mode=", mysqli_num_rows($mode);
$explicit = mysqli_query($handle, "SHOW INDEX FROM wp_probe_sql_mode_filter WHERE Key_name LIKE 'meta!_%' ESCAPE '!'");
echo "|explicit=", mysqli_num_rows($explicit), ":";
while ($index = mysqli_fetch_assoc($explicit)) {
    echo $index["Key_name"], ":", $index["Column_name"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "default=1:meta_lookup;|mode=0|explicit=1:meta_lookup:meta_key;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_applies_bounded_no_backslash_escapes_to_option_reads_and_deletes() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('mode\\\\_target', 'with-backslash', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('plain_target', 'without-backslash', 'no')");
$default = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'mode\\_target' LIMIT 1");
echo "default=", mysqli_num_rows($default);
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = 'mode\\_target' LIMIT 1");
$mode_row = mysqli_fetch_assoc($mode);
echo "|mode=", mysqli_num_rows($mode), ":", $mode_row["option_name"], "=", $mode_row["option_value"], ":", $mode_row["autoload"];
echo "|delete=", mysqli_query($handle, "DELETE FROM wp_options WHERE option_name IN ('mode\\_target')") ? "ok" : "failed";
echo ":", mysqli_affected_rows($handle);
$remaining = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'plain_target' LIMIT 1");
$remaining_row = mysqli_fetch_assoc($remaining);
echo "|left=", mysqli_num_rows($remaining), ":", $remaining_row["option_value"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "default=0|mode=1:mode\\_target=with-backslash:no|delete=ok:1|left=1:without-backslash"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_query_direct_option_like_filters_honor_explicit_backslash_escape_under_no_backslash_escapes(
) {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_mode_target', '200', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-mode-target', '200', 'no')");
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$implicit = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ORDER BY option_name");
echo "implicit=", mysqli_num_rows($implicit);
$explicit = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ESCAPE '\\\\' ORDER BY option_name");
$explicit_row = mysqli_fetch_assoc($explicit);
echo "|explicit=", mysqli_num_rows($explicit), ":", $explicit_row["option_name"];
$expired = mysqli_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE '\\_transient\\_timeout\\_%' ESCAPE '\\\\' AND option_value < 300 ORDER BY option_name");
$expired_row = mysqli_fetch_assoc($expired);
echo "|expired=", mysqli_num_rows($expired), ":", $expired_row["option_name"];
echo "|delete=", mysqli_query($handle, "DELETE FROM wp_options WHERE option_name LIKE '\\_transient\\_%' ESCAPE '\\\\'") ? "ok" : "failed";
echo ":", mysqli_affected_rows($handle);
$left = mysqli_query($handle, "SELECT option_name FROM wp_options");
echo "|left=";
while ($row = mysqli_fetch_assoc($left)) {
    echo $row["option_name"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "implicit=0|explicit=2:_transient_mode_target|expired=1:_transient_timeout_mode_target|delete=ok:2|left=xtransient-mode-target;xtransient-timeout-mode-target;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_option_like_filters_apply_bounded_no_backslash_escapes() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$default = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ?", array("\\_transient\\_%"));
$default_row = mysqli_fetch_assoc($default);
echo "default=", mysqli_num_rows($default), ":", $default_row["option_name"];
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ?", array("\\_transient\\_%"));
echo "|mode=", mysqli_num_rows($mode);
$explicit = mysqli_execute_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ? ESCAPE '!'", array("!_transient!_%"));
$explicit_row = mysqli_fetch_assoc($explicit);
echo "|explicit=", mysqli_num_rows($explicit), ":", $explicit_row["option_name"], "=", $explicit_row["option_value"];
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name LIKE ?");
mysqli_stmt_execute($stmt, array("\\_transient\\_%"));
echo "|delete-mode=", mysqli_stmt_affected_rows($stmt), ":", mysqli_affected_rows($handle);
echo "|delete-explicit=", mysqli_execute_query($handle, "DELETE FROM `wp_options` WHERE `option_name` LIKE ? ESCAPE '!'", array("!_transient!_%")) ? "ok" : "failed";
echo ":", mysqli_affected_rows($handle);
$left = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options");
echo "|left=";
while ($row = mysqli_fetch_assoc($left)) {
    echo $row["option_name"], "=", $row["option_value"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "default=1:_transient_mode_target|mode=0|explicit=1:_transient_mode_target=payload|delete-mode=0:0|delete-explicit=ok:1|left=siteurl=https://example.test;xtransient-mode-target=wildcard;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_option_like_filters_honor_explicit_backslash_escape_under_no_backslash_escapes()
{
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_mode_target', 'payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-mode-target', 'wildcard', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_mode_target', '200', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('xtransient-timeout-mode-target', '200', 'no')");
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$implicit = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? ORDER BY option_name", array("\\_transient\\_%"));
echo "implicit=", mysqli_num_rows($implicit);
$explicit = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\' ORDER BY option_name", array("\\_transient\\_%"));
$explicit_row = mysqli_fetch_assoc($explicit);
echo "|explicit=", mysqli_num_rows($explicit), ":", $explicit_row["option_name"];
$expired = mysqli_execute_query($handle, "SELECT option_name FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\' AND option_value < ? ORDER BY option_name", array("\\_transient\\_timeout\\_%", "300"));
$expired_row = mysqli_fetch_assoc($expired);
echo "|expired=", mysqli_num_rows($expired), ":", $expired_row["option_name"];
$stmt = mysqli_prepare($handle, "DELETE FROM wp_options WHERE option_name LIKE ? ESCAPE '\\\\'");
mysqli_stmt_execute($stmt, array("\\_transient\\_%"));
echo "|delete=", mysqli_stmt_affected_rows($stmt), ":", mysqli_affected_rows($handle);
$left = mysqli_query($handle, "SELECT option_name FROM wp_options");
echo "|left=";
while ($row = mysqli_fetch_assoc($left)) {
    echo $row["option_name"], ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "implicit=0|explicit=2:_transient_mode_target|expired=1:_transient_timeout_mode_target|delete=2:2|left=xtransient-mode-target;xtransient-timeout-mode-target;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_prepared_queries_filter_bounded_wordpress_schema_metadata() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "CREATE TABLE wp_probe_prepared_schema (ID bigint(20) unsigned NOT NULL auto_increment, meta_key varchar(255) NOT NULL default '', meta_value longtext NOT NULL, post_content longtext NOT NULL, PRIMARY KEY  (ID), KEY meta_lookup (meta_key(191)), KEY metaXlookup (meta_value(10)), FULLTEXT KEY content_search (post_content)) DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
$like = mysqli_execute_query($handle, "SHOW INDEX FROM wp_probe_prepared_schema WHERE Key_name LIKE ?", array("meta_%"));
echo "like=", mysqli_num_rows($like), ":";
while ($index = mysqli_fetch_assoc($like)) {
    echo $index["Key_name"], ":", $index["Column_name"], ";";
}
$escaped = mysqli_execute_query($handle, "SHOW INDEX FROM wp_probe_prepared_schema WHERE `Key_name` LIKE ? ESCAPE '!'", array("meta!_%"));
echo "|escaped=", mysqli_num_rows($escaped), ":";
while ($index = mysqli_fetch_assoc($escaped)) {
    echo $index["Key_name"], ":", $index["Sub_part"], ";";
}
$columns = mysqli_execute_query($handle, "SHOW FULL COLUMNS FROM wp_probe_prepared_schema LIKE ?", array("meta_%"));
echo "|columns=", mysqli_num_rows($columns), ":";
while ($column = mysqli_fetch_assoc($columns)) {
    echo $column["Field"], ":", $column["Key"], ";";
}
$field = mysqli_execute_query($handle, "SHOW COLUMNS FROM wp_probe_prepared_schema WHERE Field = ?", array("post_content"));
$field_row = mysqli_fetch_assoc($field);
echo "|field=", mysqli_num_rows($field), ":", $field_row["Field"], ":", $field_row["Type"];
$key = mysqli_execute_query($handle, "SHOW INDEX FROM wp_probe_prepared_schema WHERE Key_name = ?", array("content_search"));
$key_row = mysqli_fetch_assoc($key);
echo "|key=", mysqli_num_rows($key), ":", $key_row["Key_name"], ":", $key_row["Index_type"];
$status = mysqli_execute_query($handle, "SHOW TABLE STATUS LIKE ?", array("wp_probe_prepared_schema"));
$status_row = mysqli_fetch_assoc($status);
echo "|status=", mysqli_num_rows($status), ":", $status_row["Collation"];
$stmt = mysqli_prepare($handle, "SHOW TABLES LIKE ?");
mysqli_stmt_execute($stmt, array("wp_probe_prepared_%"));
$tables = mysqli_stmt_get_result($stmt);
echo "|tables=", mysqli_num_rows($tables), ":";
$table = mysqli_fetch_assoc($tables);
echo $table["Tables_in_wordpress (wp_probe_prepared_%)"];
mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");
$mode = mysqli_execute_query($handle, "SHOW INDEX FROM wp_probe_prepared_schema WHERE Key_name LIKE ?", array("meta\\_%"));
echo "|mode=", mysqli_num_rows($mode);
$explicit = mysqli_execute_query($handle, "SHOW INDEX FROM wp_probe_prepared_schema WHERE Key_name LIKE ? ESCAPE '!'", array("meta!_%"));
echo "|mode-explicit=", mysqli_num_rows($explicit);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "like=2:meta_lookup:meta_key;metaXlookup:meta_value;|escaped=1:meta_lookup:191;|columns=2:meta_key:MUL;meta_value:MUL;|field=1:post_content:longtext|key=1:content_search:FULLTEXT|status=1:utf8mb4_unicode_ci|tables=1:wp_probe_prepared_schema|mode=0|mode-explicit=1"
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
$info = "mysqli_info";
$warning_list = "mysqli_get_warnings";
$error_list = "mysqli_error_list";
echo function_exists($sqlstate) ? "yes" : "no";
echo "|";
echo is_callable($warnings) ? "callable" : "missing";
echo "|";
echo is_callable($info) ? "info-callable" : "info-missing";
echo "|";
echo is_callable($warning_list) ? "warnings-callable" : "warnings-missing";
echo "|";
echo is_callable($error_list) ? "errors-callable" : "errors-missing";
$handle = mysqli_init();
echo "|";
echo mysqli_errno($handle);
echo "|";
echo mysqli_error($handle);
echo "|";
echo count(mysqli_error_list($handle));
echo "|";
echo mysqli_sqlstate($handle);
echo "|";
echo mysqli_warning_count($handle);
echo "|";
echo mysqli_info($handle) === null ? "null" : mysqli_info($handle);
echo "|";
echo mysqli_get_warnings($handle) === false ? "false" : "warning";
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo $sqlstate($handle);
echo "|";
echo $warnings($handle);
echo "|";
echo $info($handle) === null ? "null" : $info($handle);
echo "|";
echo $warning_list($handle) === false ? "false" : "warning";
echo "|";
echo count($error_list($handle));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|info-callable|warnings-callable|errors-callable|0||0|00000|0|null|false|00000|0|null|false|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_real_escape_string_escapes_current_scalar_subset() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_real_escape_string";
$alias = "mysqli_escape_string";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo function_exists($alias) ? "alias-exists" : "alias-missing";
echo "|";
echo is_callable($alias) ? "alias-callable" : "alias-missing";
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
echo "|";
echo mysqli_escape_string($handle, $data);
echo "|";
echo $alias($handle, true);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        r#"yes|callable|alias-exists|alias-callable|quote\'\"\\\n\r|1|42||quote\'\"\\\n\r|1"#
    );
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

    let error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 4096);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_real_connect(): only MYSQLI_CLIENT_* flag combinations are supported in the current subset, unsupported bits 4096"
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

    let bad_kill_handle = run_source(
        r#"<?php
mysqli_kill("not-a-handle", 1);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_kill_handle.phase, Phase::Runtime);
    assert_eq!(bad_kill_handle.line, 2);
    assert_eq!(bad_kill_handle.column, 1);
    assert_eq!(
        bad_kill_handle.message,
        "unsupported call mysqli_kill(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_kill_process_id = run_source(
        r#"<?php
mysqli_kill(mysqli_init(), "1");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_kill_process_id.phase, Phase::Runtime);
    assert_eq!(bad_kill_process_id.line, 2);
    assert_eq!(bad_kill_process_id.column, 1);
    assert_eq!(
        bad_kill_process_id.message,
        "unsupported call mysqli_kill(): process_id argument must be int in the current subset, got string"
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
fn mysqli_links_stats_reject_forms_outside_current_boundary() {
    let error = run_source(
        r#"<?php
mysqli_get_links_stats(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "arity mismatch for mysqli_get_links_stats(): expected 0 argument(s), got 1"
    );
}

#[test]
fn mysqli_change_user_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_change_user("not-a-handle", "user", "pass", "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_change_user(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_username = run_source(
        r#"<?php
mysqli_change_user(mysqli_init(), 1, "pass", "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_username.phase, Phase::Runtime);
    assert_eq!(bad_username.line, 2);
    assert_eq!(bad_username.column, 1);
    assert_eq!(
        bad_username.message,
        "unsupported call mysqli_change_user(): username argument must be string in the current subset, got int"
    );

    let bad_password = run_source(
        r#"<?php
mysqli_change_user(mysqli_init(), "user", null, "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_password.phase, Phase::Runtime);
    assert_eq!(bad_password.line, 2);
    assert_eq!(bad_password.column, 1);
    assert_eq!(
        bad_password.message,
        "unsupported call mysqli_change_user(): password argument must be string in the current subset, got null"
    );

    let bad_database = run_source(
        r#"<?php
mysqli_change_user(mysqli_init(), "user", "pass", false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_database.phase, Phase::Runtime);
    assert_eq!(bad_database.line, 2);
    assert_eq!(bad_database.column, 1);
    assert_eq!(
        bad_database.message,
        "unsupported call mysqli_change_user(): database argument must be string or null in the current subset, got bool"
    );
}

#[test]
fn mysqli_refresh_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_refresh("not-a-handle", MYSQLI_REFRESH_LOG);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_refresh(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_flags = run_source(
        r#"<?php
mysqli_refresh(mysqli_init(), "tables");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_flags.phase, Phase::Runtime);
    assert_eq!(bad_flags.line, 2);
    assert_eq!(bad_flags.column, 1);
    assert_eq!(
        bad_flags.message,
        "unsupported call mysqli_refresh(): flags argument must be int in the current subset, got string"
    );

    let zero_flags = run_source(
        r#"<?php
mysqli_refresh(mysqli_init(), 0);
"#,
    )
    .unwrap_err();

    assert_eq!(zero_flags.phase, Phase::Runtime);
    assert_eq!(zero_flags.line, 2);
    assert_eq!(zero_flags.column, 1);
    assert_eq!(
        zero_flags.message,
        "unsupported call mysqli_refresh(): flags argument must include at least one MYSQLI_REFRESH_* flag in the current subset"
    );

    let unsupported_flags = run_source(
        r#"<?php
mysqli_refresh(mysqli_init(), 256);
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_flags.phase, Phase::Runtime);
    assert_eq!(unsupported_flags.line, 2);
    assert_eq!(unsupported_flags.column, 1);
    assert_eq!(
        unsupported_flags.message,
        "unsupported call mysqli_refresh(): only MYSQLI_REFRESH_* flag combinations are supported in the current subset, unsupported bits 256"
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
fn mysqli_options_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_options("not-a-handle", MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_options(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_option = run_source(
        r#"<?php
mysqli_options(mysqli_init(), 999, true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_option.phase, Phase::Runtime);
    assert_eq!(bad_option.line, 2);
    assert_eq!(bad_option.column, 1);
    assert_eq!(
        bad_option.message,
        "unsupported call mysqli_options(): unsupported mysqli option in the current subset, got 999"
    );

    let bad_value = run_source(
        r#"<?php
mysqli_options(mysqli_init(), MYSQLI_OPT_INT_AND_FLOAT_NATIVE, "yes");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_value.phase, Phase::Runtime);
    assert_eq!(bad_value.line, 2);
    assert_eq!(bad_value.column, 1);
    assert_eq!(
        bad_value.message,
        "unsupported call mysqli_options(): value must be bool or int for the selected mysqli option in the current subset, got string"
    );

    let bad_alias_option = run_source(
        r#"<?php
mysqli_set_opt(mysqli_init(), 999, true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_alias_option.phase, Phase::Runtime);
    assert_eq!(bad_alias_option.line, 2);
    assert_eq!(bad_alias_option.column, 1);
    assert_eq!(
        bad_alias_option.message,
        "unsupported call mysqli_set_opt(): unsupported mysqli option in the current subset, got 999"
    );

    let bad_alias_value = run_source(
        r#"<?php
mysqli_set_opt(mysqli_init(), MYSQLI_OPT_INT_AND_FLOAT_NATIVE, "yes");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_alias_value.phase, Phase::Runtime);
    assert_eq!(bad_alias_value.line, 2);
    assert_eq!(bad_alias_value.column, 1);
    assert_eq!(
        bad_alias_value.message,
        "unsupported call mysqli_set_opt(): value must be bool or int for the selected mysqli option in the current subset, got string"
    );

    let bad_timeout_value = run_source(
        r#"<?php
mysqli_options(mysqli_init(), MYSQLI_OPT_CONNECT_TIMEOUT, "5");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_timeout_value.phase, Phase::Runtime);
    assert_eq!(bad_timeout_value.line, 2);
    assert_eq!(bad_timeout_value.column, 1);
    assert_eq!(
        bad_timeout_value.message,
        "unsupported call mysqli_options(): value must be int for the selected mysqli option in the current subset, got string"
    );

    let bad_command_value = run_source(
        r#"<?php
mysqli_options(mysqli_init(), MYSQLI_INIT_COMMAND, false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_command_value.phase, Phase::Runtime);
    assert_eq!(bad_command_value.line, 2);
    assert_eq!(bad_command_value.column, 1);
    assert_eq!(
        bad_command_value.message,
        "unsupported call mysqli_options(): value must be string for the selected mysqli option in the current subset, got bool"
    );
}

#[test]
fn mysqli_ssl_set_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_ssl_set("not-a-handle", null, null, null, null, null);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_ssl_set(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_value = run_source(
        r#"<?php
mysqli_ssl_set(mysqli_init(), null, false, null, null, null);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_value.phase, Phase::Runtime);
    assert_eq!(bad_value.line, 2);
    assert_eq!(bad_value.column, 1);
    assert_eq!(
        bad_value.message,
        "unsupported call mysqli_ssl_set(): third argument must be string or null in the current subset, got bool"
    );
}

#[test]
fn mysqli_connect_error_state_rejects_forms_outside_current_boundary() {
    let bad_errno = run_source(
        r#"<?php
mysqli_connect_errno("extra");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_errno.phase, Phase::Runtime);
    assert_eq!(bad_errno.line, 2);
    assert_eq!(bad_errno.column, 1);
    assert_eq!(
        bad_errno.message,
        "arity mismatch for mysqli_connect_errno(): expected 0 argument(s), got 1"
    );

    let bad_error = run_source(
        r#"<?php
mysqli_connect_error("extra");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_error.phase, Phase::Runtime);
    assert_eq!(bad_error.line, 2);
    assert_eq!(bad_error.column, 1);
    assert_eq!(
        bad_error.message,
        "arity mismatch for mysqli_connect_error(): expected 0 argument(s), got 1"
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
fn mysqli_savepoint_helpers_reject_forms_outside_current_boundary() {
    let bad_savepoint_handle = run_source(
        r#"<?php
mysqli_savepoint("not-a-handle", "wp");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_savepoint_handle.phase, Phase::Runtime);
    assert_eq!(bad_savepoint_handle.line, 2);
    assert_eq!(bad_savepoint_handle.column, 1);
    assert_eq!(
        bad_savepoint_handle.message,
        "unsupported call mysqli_savepoint(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_release_handle = run_source(
        r#"<?php
mysqli_release_savepoint("not-a-handle", "wp");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_release_handle.phase, Phase::Runtime);
    assert_eq!(bad_release_handle.line, 2);
    assert_eq!(bad_release_handle.column, 1);
    assert_eq!(
        bad_release_handle.message,
        "unsupported call mysqli_release_savepoint(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_savepoint_name = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_savepoint($handle, false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_savepoint_name.phase, Phase::Runtime);
    assert_eq!(bad_savepoint_name.line, 3);
    assert_eq!(bad_savepoint_name.column, 1);
    assert_eq!(
        bad_savepoint_name.message,
        "unsupported call mysqli_savepoint(): name argument must be string in the current subset, got bool"
    );

    let bad_release_name = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_release_savepoint($handle, null);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_release_name.phase, Phase::Runtime);
    assert_eq!(bad_release_name.line, 3);
    assert_eq!(bad_release_name.column, 1);
    assert_eq!(
        bad_release_name.message,
        "unsupported call mysqli_release_savepoint(): name argument must be string in the current subset, got null"
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
fn mysqli_real_query_accepts_current_wordpress_charset_setup_placeholder() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_real_query";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_real_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "charset-ok" : "charset-failed";
echo "|";
echo $call($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "dynamic" : "failed";
echo "|";
echo mysqli_store_result($handle) === false ? "no-pending" : "pending";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|charset-ok|dynamic|no-pending"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_real_query_stores_current_pending_result_placeholders() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_real_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
$result = mysqli_store_result($handle);
echo get_class($result);
echo "|";
echo mysqli_field_count($handle);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["ID"], ":", $row["post_title"];
echo "|";
echo mysqli_store_result($handle) === false ? "drained" : "pending";

$handle2 = mysqli_init();
mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_real_query($handle2, "SELECT * FROM wp_posts WHERE 1 = 0");
$empty = mysqli_use_result($handle2);
echo "|";
echo get_class($empty), ":", mysqli_num_rows($empty), ":", mysqli_num_fields($empty);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "queued|2|mysqli_result|0|1:Hello world placeholder|drained|mysqli_result:0:0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_multi_query_accepts_current_wordpress_charset_setup_placeholder() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_multi_query";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_multi_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "charset-ok" : "charset-failed";
echo "|";
echo $call($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "dynamic" : "failed";
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "done";
echo "|";
echo mysqli_store_result($handle) === false ? "no-pending" : "pending";
echo "|";
echo mysqli_multi_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle);
$result = mysqli_store_result($handle);
echo "|";
echo get_class($result);
echo "|";
echo mysqli_field_count($handle);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "done";
echo "|";
echo mysqli_store_result($handle) === false ? "drained" : "pending";

$handle2 = mysqli_init();
mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_multi_query($handle2, "SELECT * FROM wp_posts WHERE 1 = 0");
$empty = mysqli_use_result($handle2);
echo "|";
echo get_class($empty), ":", mysqli_num_rows($empty), ":", mysqli_num_fields($empty);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|charset-ok|dynamic|done|done|no-pending|queued|2|mysqli_result|0|1:Hello world placeholder|done|done|drained|mysqli_result:0:0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_multi_query_tracks_current_deterministic_multi_result_queue() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_multi_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1; SELECT * FROM wp_posts WHERE 1 = 0") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "blocked";
$result = mysqli_store_result($handle);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "blocked";
echo "|";
echo mysqli_field_count($handle);
$empty = mysqli_store_result($handle);
echo "|";
echo get_class($empty), ":", mysqli_num_rows($empty), ":", mysqli_num_fields($empty);
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "done";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "queued|2|more|blocked|1:Hello world placeholder|more|next|0|mysqli_result:0:0|done|done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_multi_query_tracks_current_mixed_no_result_and_result_queue() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_multi_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'; SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-result" : "result";
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "blocked";
echo "|";
echo mysqli_field_count($handle);
$result = mysqli_store_result($handle);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
echo "|";
echo mysqli_more_results($handle) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle) ? "next" : "done";

$handle2 = mysqli_init();
mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_multi_query($handle2, "SELECT ID, post_title FROM wp_posts WHERE ID = 1; SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'") ? "queued" : "failed";
$first = mysqli_store_result($handle2);
$first_row = mysqli_fetch_assoc($first);
echo "|";
echo $first_row["ID"], ":", $first_row["post_title"];
echo "|";
echo mysqli_more_results($handle2) ? "more" : "done";
echo "|";
echo mysqli_next_result($handle2) ? "next" : "blocked";
echo "|";
echo mysqli_field_count($handle2);
echo "|";
echo mysqli_store_result($handle2) === false ? "no-result" : "result";
echo "|";
echo mysqli_more_results($handle2) ? "more" : "done";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "queued|0|no-result|more|next|2|1:Hello world placeholder|done|done|queued|1:Hello world placeholder|more|next|0|no-result|done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_multi_query_tracks_current_sql_mode_no_result_queue() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_real_query($handle, "SELECT @@SESSION.sql_mode") ? "real-ok" : "real-failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-result" : "result";

$handle2 = mysqli_init();
mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_multi_query($handle2, "SELECT @@SESSION.sql_mode") ? "multi-ok" : "multi-failed";
echo "|";
echo mysqli_field_count($handle2);
echo "|";
echo mysqli_store_result($handle2) === false ? "no-result" : "result";
echo "|";
echo mysqli_more_results($handle2) ? "more" : "done";

$handle3 = mysqli_init();
mysqli_real_connect($handle3, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_multi_query($handle3, "SELECT @@SESSION.sql_mode; SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle3);
echo "|";
echo mysqli_store_result($handle3) === false ? "no-result" : "result";
echo "|";
echo mysqli_next_result($handle3) ? "next" : "blocked";
$result = mysqli_store_result($handle3);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "real-ok|0|no-result|multi-ok|0|no-result|done|queued|0|no-result|next|1:Hello world placeholder"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_queries_accept_current_sql_mode_assignment_no_result_placeholder() {
    let execution = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_query($handle, "SET SESSION sql_mode=''") ? "query-ok" : "query-failed";
echo "|";
echo mysqli_real_query($handle, "SET SESSION sql_mode='NO_ENGINE_SUBSTITUTION'") ? "real-ok" : "real-failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-result" : "result";

$handle2 = mysqli_init();
mysqli_real_connect($handle2, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_multi_query($handle2, "SET SESSION sql_mode='STRICT_TRANS_TABLES,NO_ZERO_DATE'; SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle2);
echo "|";
echo mysqli_store_result($handle2) === false ? "no-result" : "result";
echo "|";
echo mysqli_next_result($handle2) ? "next" : "blocked";
$result = mysqli_store_result($handle2);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "query-ok|real-ok|0|no-result|queued|0|no-result|next|1:Hello world placeholder"
    );
    assert_eq!(execution.exit_code, 0);

    let unsupported = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_query($handle, "SET SESSION sql_mode='ansi'");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported.phase, Phase::Runtime);
    assert_eq!(unsupported.line, 3);
    assert_eq!(unsupported.column, 1);
    assert_eq!(
        unsupported.message,
        "unsupported call mysqli_query(): only the WordPress SQL mode probe, SQL-mode assignment, charset setup query, empty/exact wp_options SELECT placeholders, and exact wp_options insert/replace/update/delete state-island queries are implemented in the current subset; got SET SESSION sql_mode='ansi'"
    );
}

#[test]
fn mysqli_reap_async_query_returns_current_clean_placeholder_state() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_reap_async_query";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo "|";
echo mysqli_reap_async_query($handle) === false ? "no-async" : "async";
echo "|";
echo $call($handle) === false ? "dynamic" : "async";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|no-async|dynamic");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_poll_is_visible_but_async_readiness_is_an_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_poll";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo defined("MYSQLI_ASYNC") ? MYSQLI_ASYNC : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|8");
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
$read = [];
$error = [];
$reject = [];
mysqli_poll($read, $error, $reject, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_poll(): async socket readiness and by-reference array mutation are not implemented in the current subset"
    );
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
echo "|";
echo mysqli_store_result($handle) === false ? "no-store" : "stored";
echo "|";
echo mysqli_use_result($handle) === false ? "no-use" : "using";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mysqli_result|0|no-field|no-row|freed|done|done|no-store|no-use"
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
fn mysqli_fetch_lengths_reports_last_seed_post_row_lengths() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_fetch_lengths";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_fetch_lengths($result) === false ? "no-lengths" : "lengths";
$row = mysqli_fetch_row($result);
$lengths = mysqli_fetch_lengths($result);
echo "|";
echo $lengths[0];
echo ",";
echo $lengths[1];
echo "|";
$lengths = $call($result);
echo $lengths[0];
echo ",";
echo $lengths[1];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|no-lengths|1,23|1,23");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mysqli_fetch_all_returns_current_seed_post_rows_for_supported_modes() {
    let execution = run_source(
        r#"<?php
$fetch_all = "mysqli_fetch_all";
echo function_exists($fetch_all) ? "yes" : "no";
echo "|";
echo is_callable($fetch_all) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$rows = mysqli_fetch_all($result);
echo "|";
echo $rows[0][0];
echo ",";
echo $rows[0][1];
echo ",";
echo isset($rows[0]["ID"]) ? "assoc" : "no-assoc";
echo ",";
echo mysqli_fetch_assoc($result) === false ? "drained" : "row";
$lengths = mysqli_fetch_lengths($result);
echo ",";
echo $lengths[0];
echo ",";
echo $lengths[1];
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$rows = mysqli_fetch_all($result, MYSQLI_ASSOC);
echo "|";
echo $rows[0]["ID"];
echo ",";
echo $rows[0]["post_title"];
echo ",";
echo isset($rows[0][0]) ? "num" : "no-num";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$rows = $fetch_all($result, MYSQLI_BOTH);
echo "|";
echo $rows[0][0];
echo ",";
echo $rows[0]["ID"];
echo ",";
echo $rows[0][1];
echo ",";
echo $rows[0]["post_title"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|1,Hello world placeholder,no-assoc,drained,1,23|1,Hello world placeholder,no-num|1,1,Hello world placeholder,Hello world placeholder"
    );
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_fetch_all($result, 99);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_fetch_all(): mode must be MYSQLI_ASSOC, MYSQLI_NUM, or MYSQLI_BOTH in the current subset, got int"
    );
}

#[test]
fn mysqli_fetch_column_returns_current_seed_post_columns() {
    let execution = run_source(
        r#"<?php
$fetch_column = "mysqli_fetch_column";
echo function_exists($fetch_column) ? "yes" : "no";
echo "|";
echo is_callable($fetch_column) ? "callable" : "missing";
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_fetch_column($result);
echo ",";
echo mysqli_fetch_column($result) === false ? "no-row" : "row";
$lengths = mysqli_fetch_lengths($result);
echo ",";
echo $lengths[0];
echo ",";
echo $lengths[1];
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo $fetch_column($result, 1);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_fetch_column($result, 99) === null ? "null" : "value";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|1,no-row,1,23|Hello world placeholder|null"
    );
    assert_eq!(execution.exit_code, 0);

    let error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_fetch_column($result, "1");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call mysqli_fetch_column(): column must be int in the current subset, got string"
    );
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

    let bad_store_result_handle = run_source(
        r#"<?php
mysqli_store_result("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_store_result_handle.phase, Phase::Runtime);
    assert_eq!(bad_store_result_handle.line, 2);
    assert_eq!(bad_store_result_handle.column, 1);
    assert_eq!(
        bad_store_result_handle.message,
        "unsupported call mysqli_store_result(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_use_result_handle = run_source(
        r#"<?php
mysqli_use_result("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_use_result_handle.phase, Phase::Runtime);
    assert_eq!(bad_use_result_handle.line, 2);
    assert_eq!(bad_use_result_handle.column, 1);
    assert_eq!(
        bad_use_result_handle.message,
        "unsupported call mysqli_use_result(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_real_query_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_real_query("not-a-handle", "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_real_query(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_query = run_source(
        r#"<?php
mysqli_real_query(mysqli_init(), false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_query.phase, Phase::Runtime);
    assert_eq!(bad_query.line, 2);
    assert_eq!(bad_query.column, 1);
    assert_eq!(
        bad_query.message,
        "unsupported call mysqli_real_query(): query argument must be string in the current subset, got bool"
    );

    let unsupported_select = run_source(
        r#"<?php
mysqli_real_query(mysqli_init(), "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_select.phase, Phase::Runtime);
    assert_eq!(unsupported_select.line, 2);
    assert_eq!(unsupported_select.column, 1);
    assert_eq!(
        unsupported_select.message,
        "unsupported call mysqli_real_query(): general result-producing mysqli_real_query() SQL is not implemented; only deterministic pending result placeholders are supported in the current subset; got SELECT 1"
    );

    let unsupported_mutation = run_source(
        r#"<?php
mysqli_real_query(mysqli_init(), "UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_mutation.phase, Phase::Runtime);
    assert_eq!(unsupported_mutation.line, 2);
    assert_eq!(unsupported_mutation.column, 1);
    assert_eq!(
        unsupported_mutation.message,
        "unsupported call mysqli_real_query(): mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'"
    );
}

#[test]
fn mysqli_multi_query_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_multi_query("not-a-handle", "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_multi_query(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_query = run_source(
        r#"<?php
mysqli_multi_query(mysqli_init(), false);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_query.phase, Phase::Runtime);
    assert_eq!(bad_query.line, 2);
    assert_eq!(bad_query.column, 1);
    assert_eq!(
        bad_query.message,
        "unsupported call mysqli_multi_query(): query argument must be string in the current subset, got bool"
    );

    let unsupported_multi_statement = run_source(
        r#"<?php
mysqli_multi_query(mysqli_init(), "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'; SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_multi_statement.phase, Phase::Runtime);
    assert_eq!(unsupported_multi_statement.line, 2);
    assert_eq!(unsupported_multi_statement.column, 1);
    assert_eq!(
        unsupported_multi_statement.message,
        "unsupported call mysqli_multi_query(): multi-statement mysqli_multi_query() SQL is not implemented; only deterministic known no-result/result queues are supported in the current subset; got SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'; SELECT 1"
    );

    let unsupported_select = run_source(
        r#"<?php
mysqli_multi_query(mysqli_init(), "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_select.phase, Phase::Runtime);
    assert_eq!(unsupported_select.line, 2);
    assert_eq!(unsupported_select.column, 1);
    assert_eq!(
        unsupported_select.message,
        "unsupported call mysqli_multi_query(): general result-producing mysqli_multi_query() SQL is not implemented; only deterministic pending result placeholders are supported in the current subset; got SELECT 1"
    );

    let unsupported_mutation = run_source(
        r#"<?php
mysqli_multi_query(mysqli_init(), "UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'");
"#,
    )
    .unwrap_err();

    assert_eq!(unsupported_mutation.phase, Phase::Runtime);
    assert_eq!(unsupported_mutation.line, 2);
    assert_eq!(unsupported_mutation.column, 1);
    assert_eq!(
        unsupported_mutation.message,
        "unsupported call mysqli_multi_query(): mutation SQL is not implemented in the current subset; affected-row and insert-id state are deterministic clean placeholders only; got UPDATE wp_options SET option_value = '1' WHERE option_name = 'blog_public'"
    );
}

#[test]
fn mysqli_reap_async_query_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_reap_async_query("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_reap_async_query(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_dump_debug_info_rejects_forms_outside_current_boundary() {
    let bad_handle = run_source(
        r#"<?php
mysqli_dump_debug_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_handle.phase, Phase::Runtime);
    assert_eq!(bad_handle.line, 2);
    assert_eq!(bad_handle.column, 1);
    assert_eq!(
        bad_handle.message,
        "unsupported call mysqli_dump_debug_info(): first argument must be mysqli object in the current subset, got string"
    );
}

#[test]
fn mysqli_debug_rejects_forms_outside_current_boundary() {
    let bad_options = run_source(
        r#"<?php
mysqli_debug([]);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_options.phase, Phase::Runtime);
    assert_eq!(bad_options.line, 2);
    assert_eq!(bad_options.column, 1);
    assert_eq!(
        bad_options.message,
        "unsupported call mysqli_debug(): options argument arrays are not implemented in the current subset"
    );
}

#[test]
fn mysqli_client_stats_rejects_forms_outside_current_boundary() {
    let bad_arity = run_source(
        r#"<?php
mysqli_get_client_stats("unused");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_arity.phase, Phase::Runtime);
    assert_eq!(bad_arity.line, 2);
    assert_eq!(bad_arity.column, 1);
    assert_eq!(
        bad_arity.message,
        "arity mismatch for mysqli_get_client_stats(): expected 0 argument(s), got 1"
    );
}

#[test]
fn mysqli_thread_safe_rejects_forms_outside_current_boundary() {
    let bad_arity = run_source(
        r#"<?php
mysqli_thread_safe("unused");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_arity.phase, Phase::Runtime);
    assert_eq!(bad_arity.line, 2);
    assert_eq!(bad_arity.column, 1);
    assert_eq!(
        bad_arity.message,
        "arity mismatch for mysqli_thread_safe(): expected 0 argument(s), got 1"
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

    let bad_alias_handle = run_source(
        r#"<?php
mysqli_escape_string("not-a-handle", "value");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_alias_handle.phase, Phase::Runtime);
    assert_eq!(bad_alias_handle.line, 2);
    assert_eq!(bad_alias_handle.column, 1);
    assert_eq!(
        bad_alias_handle.message,
        "unsupported call mysqli_escape_string(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_alias_data = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_escape_string($handle, ["value"]);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_alias_data.phase, Phase::Runtime);
    assert_eq!(bad_alias_data.line, 3);
    assert_eq!(bad_alias_data.column, 1);
    assert_eq!(
        bad_alias_data.message,
        "unsupported call mysqli_escape_string(): data argument arrays are not implemented in the current subset"
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

    let bad_info_handle = run_source(
        r#"<?php
mysqli_info("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_info_handle.phase, Phase::Runtime);
    assert_eq!(bad_info_handle.line, 2);
    assert_eq!(bad_info_handle.column, 1);
    assert_eq!(
        bad_info_handle.message,
        "unsupported call mysqli_info(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_get_warnings_handle = run_source(
        r#"<?php
mysqli_get_warnings("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_get_warnings_handle.phase, Phase::Runtime);
    assert_eq!(bad_get_warnings_handle.line, 2);
    assert_eq!(bad_get_warnings_handle.column, 1);
    assert_eq!(
        bad_get_warnings_handle.message,
        "unsupported call mysqli_get_warnings(): first argument must be mysqli object in the current subset, got string"
    );

    let bad_error_list_handle = run_source(
        r#"<?php
mysqli_error_list("not-a-handle");
"#,
    )
    .unwrap_err();

    assert_eq!(bad_error_list_handle.phase, Phase::Runtime);
    assert_eq!(bad_error_list_handle.line, 2);
    assert_eq!(bad_error_list_handle.column, 1);
    assert_eq!(
        bad_error_list_handle.message,
        "unsupported call mysqli_error_list(): first argument must be mysqli object in the current subset, got string"
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
fn dynamic_mysqli_connect_calls_return_placeholder_handles() {
    let execution = run_source(
        r#"<?php
$call = "mysqli_connect";
$handle = $call("localhost");
echo get_class($handle);
echo "|";
echo mysqli_get_host_info($handle);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mysqli|localhost via TCP/IP (phpc-placeholder)"
    );
    assert_eq!(execution.exit_code, 0);
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
echo function_exists("mysqli_kill") ? "1" : "0";
echo is_callable("mysqli_kill") ? "1" : "0";
echo function_exists("mysqli_change_user") ? "1" : "0";
echo is_callable("mysqli_change_user") ? "1" : "0";
echo function_exists("mysqli_refresh") ? "1" : "0";
echo is_callable("mysqli_refresh") ? "1" : "0";
echo function_exists("mysqli_get_charset") ? "1" : "0";
echo is_callable("mysqli_get_charset") ? "1" : "0";
echo function_exists("mysqli_character_set_name") ? "1" : "0";
echo is_callable("mysqli_character_set_name") ? "1" : "0";
echo function_exists("mysqli_field_count") ? "1" : "0";
echo is_callable("mysqli_field_count") ? "1" : "0";
echo function_exists("mysqli_close") ? "1" : "0";
echo is_callable("mysqli_close") ? "1" : "0";
echo function_exists("mysqli_options") ? "1" : "0";
echo is_callable("mysqli_options") ? "1" : "0";
echo function_exists("mysqli_set_opt") ? "1" : "0";
echo is_callable("mysqli_set_opt") ? "1" : "0";
echo function_exists("mysqli_ssl_set") ? "1" : "0";
echo is_callable("mysqli_ssl_set") ? "1" : "0";
echo function_exists("mysqli_connect_errno") ? "1" : "0";
echo is_callable("mysqli_connect_errno") ? "1" : "0";
echo function_exists("mysqli_connect_error") ? "1" : "0";
echo is_callable("mysqli_connect_error") ? "1" : "0";
echo function_exists("mysqli_error_list") ? "1" : "0";
echo is_callable("mysqli_error_list") ? "1" : "0";
echo function_exists("mysqli_get_connection_stats") ? "1" : "0";
echo is_callable("mysqli_get_connection_stats") ? "1" : "0";
echo function_exists("mysqli_get_links_stats") ? "1" : "0";
echo is_callable("mysqli_get_links_stats") ? "1" : "0";
echo function_exists("mysqli_get_client_stats") ? "1" : "0";
echo is_callable("mysqli_get_client_stats") ? "1" : "0";
echo function_exists("mysqli_thread_safe") ? "1" : "0";
echo is_callable("mysqli_thread_safe") ? "1" : "0";
echo function_exists("mysqli_stmt_init") ? "1" : "0";
echo is_callable("mysqli_stmt_init") ? "1" : "0";
echo function_exists("mysqli_prepare") ? "1" : "0";
echo is_callable("mysqli_prepare") ? "1" : "0";
echo function_exists("mysqli_stmt_prepare") ? "1" : "0";
echo is_callable("mysqli_stmt_prepare") ? "1" : "0";
echo function_exists("mysqli_stmt_param_count") ? "1" : "0";
echo is_callable("mysqli_stmt_param_count") ? "1" : "0";
echo function_exists("mysqli_stmt_get_warnings") ? "1" : "0";
echo is_callable("mysqli_stmt_get_warnings") ? "1" : "0";
echo function_exists("mysqli_stmt_error_list") ? "1" : "0";
echo is_callable("mysqli_stmt_error_list") ? "1" : "0";
echo function_exists("mysqli_stmt_bind_param") ? "1" : "0";
echo is_callable("mysqli_stmt_bind_param") ? "1" : "0";
echo function_exists("mysqli_stmt_bind_result") ? "1" : "0";
echo is_callable("mysqli_stmt_bind_result") ? "1" : "0";
echo function_exists("mysqli_stmt_execute") ? "1" : "0";
echo is_callable("mysqli_stmt_execute") ? "1" : "0";
echo function_exists("mysqli_execute") ? "1" : "0";
echo is_callable("mysqli_execute") ? "1" : "0";
echo function_exists("mysqli_stmt_get_result") ? "1" : "0";
echo is_callable("mysqli_stmt_get_result") ? "1" : "0";
echo function_exists("mysqli_stmt_close") ? "1" : "0";
echo is_callable("mysqli_stmt_close") ? "1" : "0";
echo function_exists("mysqli_stmt_errno") ? "1" : "0";
echo is_callable("mysqli_stmt_errno") ? "1" : "0";
echo function_exists("mysqli_stmt_error") ? "1" : "0";
echo is_callable("mysqli_stmt_error") ? "1" : "0";
echo function_exists("mysqli_stmt_affected_rows") ? "1" : "0";
echo is_callable("mysqli_stmt_affected_rows") ? "1" : "0";
echo function_exists("mysqli_stmt_store_result") ? "1" : "0";
echo is_callable("mysqli_stmt_store_result") ? "1" : "0";
echo function_exists("mysqli_stmt_num_rows") ? "1" : "0";
echo is_callable("mysqli_stmt_num_rows") ? "1" : "0";
echo function_exists("mysqli_stmt_fetch") ? "1" : "0";
echo is_callable("mysqli_stmt_fetch") ? "1" : "0";
echo function_exists("mysqli_stmt_result_metadata") ? "1" : "0";
echo is_callable("mysqli_stmt_result_metadata") ? "1" : "0";
echo function_exists("mysqli_stmt_field_count") ? "1" : "0";
echo is_callable("mysqli_stmt_field_count") ? "1" : "0";
echo function_exists("mysqli_stmt_free_result") ? "1" : "0";
echo is_callable("mysqli_stmt_free_result") ? "1" : "0";
echo function_exists("mysqli_stmt_data_seek") ? "1" : "0";
echo is_callable("mysqli_stmt_data_seek") ? "1" : "0";
echo function_exists("mysqli_stmt_attr_get") ? "1" : "0";
echo is_callable("mysqli_stmt_attr_get") ? "1" : "0";
echo function_exists("mysqli_stmt_attr_set") ? "1" : "0";
echo is_callable("mysqli_stmt_attr_set") ? "1" : "0";
echo function_exists("mysqli_stmt_send_long_data") ? "1" : "0";
echo is_callable("mysqli_stmt_send_long_data") ? "1" : "0";
echo function_exists("mysqli_stmt_reset") ? "1" : "0";
echo is_callable("mysqli_stmt_reset") ? "1" : "0";
echo function_exists("mysqli_stmt_more_results") ? "1" : "0";
echo is_callable("mysqli_stmt_more_results") ? "1" : "0";
echo function_exists("mysqli_stmt_next_result") ? "1" : "0";
echo is_callable("mysqli_stmt_next_result") ? "1" : "0";
echo function_exists("mysqli_stmt_sqlstate") ? "1" : "0";
echo is_callable("mysqli_stmt_sqlstate") ? "1" : "0";
echo function_exists("mysqli_stmt_warning_count") ? "1" : "0";
echo is_callable("mysqli_stmt_warning_count") ? "1" : "0";
echo function_exists("mysqli_stmt_insert_id") ? "1" : "0";
echo is_callable("mysqli_stmt_insert_id") ? "1" : "0";
echo function_exists("mysqli_execute_query") ? "1" : "0";
echo is_callable("mysqli_execute_query") ? "1" : "0";
echo function_exists("mysqli_dump_debug_info") ? "1" : "0";
echo is_callable("mysqli_dump_debug_info") ? "1" : "0";
echo function_exists("mysqli_debug") ? "1" : "0";
echo is_callable("mysqli_debug") ? "1" : "0";
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
echo function_exists("mysqli_savepoint") ? "1" : "0";
echo is_callable("mysqli_savepoint") ? "1" : "0";
echo function_exists("mysqli_release_savepoint") ? "1" : "0";
echo is_callable("mysqli_release_savepoint") ? "1" : "0";
echo function_exists("mysqli_set_charset") ? "1" : "0";
echo is_callable("mysqli_set_charset") ? "1" : "0";
echo function_exists("mysqli_query") ? "1" : "0";
echo is_callable("mysqli_query") ? "1" : "0";
echo function_exists("mysqli_real_query") ? "1" : "0";
echo is_callable("mysqli_real_query") ? "1" : "0";
echo function_exists("mysqli_multi_query") ? "1" : "0";
echo is_callable("mysqli_multi_query") ? "1" : "0";
echo function_exists("mysqli_errno") ? "1" : "0";
echo is_callable("mysqli_errno") ? "1" : "0";
echo function_exists("mysqli_error") ? "1" : "0";
echo is_callable("mysqli_error") ? "1" : "0";
echo function_exists("mysqli_sqlstate") ? "1" : "0";
echo is_callable("mysqli_sqlstate") ? "1" : "0";
echo function_exists("mysqli_warning_count") ? "1" : "0";
echo is_callable("mysqli_warning_count") ? "1" : "0";
echo function_exists("mysqli_info") ? "1" : "0";
echo is_callable("mysqli_info") ? "1" : "0";
echo function_exists("mysqli_get_warnings") ? "1" : "0";
echo is_callable("mysqli_get_warnings") ? "1" : "0";
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
echo function_exists("mysqli_escape_string") ? "1" : "0";
echo is_callable("mysqli_escape_string") ? "1" : "0";
echo function_exists("mysqli_fetch_object") ? "1" : "0";
echo is_callable("mysqli_fetch_object") ? "1" : "0";
echo function_exists("mysqli_fetch_assoc") ? "1" : "0";
echo is_callable("mysqli_fetch_assoc") ? "1" : "0";
echo function_exists("mysqli_fetch_row") ? "1" : "0";
echo is_callable("mysqli_fetch_row") ? "1" : "0";
echo function_exists("mysqli_fetch_array") ? "1" : "0";
echo is_callable("mysqli_fetch_array") ? "1" : "0";
echo function_exists("mysqli_fetch_all") ? "1" : "0";
echo is_callable("mysqli_fetch_all") ? "1" : "0";
echo function_exists("mysqli_fetch_column") ? "1" : "0";
echo is_callable("mysqli_fetch_column") ? "1" : "0";
echo function_exists("mysqli_fetch_field") ? "1" : "0";
echo is_callable("mysqli_fetch_field") ? "1" : "0";
echo function_exists("mysqli_fetch_fields") ? "1" : "0";
echo is_callable("mysqli_fetch_fields") ? "1" : "0";
echo function_exists("mysqli_fetch_field_direct") ? "1" : "0";
echo is_callable("mysqli_fetch_field_direct") ? "1" : "0";
echo function_exists("mysqli_num_fields") ? "1" : "0";
echo is_callable("mysqli_num_fields") ? "1" : "0";
echo function_exists("mysqli_num_rows") ? "1" : "0";
echo is_callable("mysqli_num_rows") ? "1" : "0";
echo function_exists("mysqli_fetch_lengths") ? "1" : "0";
echo is_callable("mysqli_fetch_lengths") ? "1" : "0";
echo function_exists("mysqli_data_seek") ? "1" : "0";
echo is_callable("mysqli_data_seek") ? "1" : "0";
echo function_exists("mysqli_field_seek") ? "1" : "0";
echo is_callable("mysqli_field_seek") ? "1" : "0";
echo function_exists("mysqli_field_tell") ? "1" : "0";
echo is_callable("mysqli_field_tell") ? "1" : "0";
echo function_exists("mysqli_free_result") ? "1" : "0";
echo is_callable("mysqli_free_result") ? "1" : "0";
echo function_exists("mysqli_more_results") ? "1" : "0";
echo is_callable("mysqli_more_results") ? "1" : "0";
echo function_exists("mysqli_next_result") ? "1" : "0";
echo is_callable("mysqli_next_result") ? "1" : "0";
echo function_exists("mysqli_store_result") ? "1" : "0";
echo is_callable("mysqli_store_result") ? "1" : "0";
echo function_exists("mysqli_use_result") ? "1" : "0";
echo is_callable("mysqli_use_result") ? "1" : "0";
echo function_exists("mysqli_reap_async_query") ? "1" : "0";
echo is_callable("mysqli_reap_async_query") ? "1" : "0";
echo function_exists("mysqli_poll") ? "1" : "0";
echo is_callable("mysqli_poll") ? "1" : "0";
echo function_exists("mysqli_report") ? "1" : "0";
echo is_callable("mysqli_report") ? "1" : "0";
echo function_exists("mysqli_init") ? "1" : "0";
echo is_callable("mysqli_init") ? "1" : "0";
echo defined("MYSQLI_REPORT_OFF") ? "1" : "0";
echo defined("MYSQLI_ASSOC") ? "1" : "0";
echo defined("MYSQLI_NUM") ? "1" : "0";
echo defined("MYSQLI_BOTH") ? "1" : "0";
echo defined("MYSQLI_ASYNC") ? "1" : "0";
echo defined("MYSQLI_CLIENT_SSL") ? "1" : "0";
echo defined("MYSQLI_CLIENT_COMPRESS") ? "1" : "0";
echo defined("MYSQLI_CLIENT_INTERACTIVE") ? "1" : "0";
echo defined("MYSQLI_CLIENT_IGNORE_SPACE") ? "1" : "0";
echo defined("MYSQLI_CLIENT_NO_SCHEMA") ? "1" : "0";
echo defined("MYSQLI_CLIENT_FOUND_ROWS") ? "1" : "0";
echo defined("MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT") ? "1" : "0";
echo defined("MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT") ? "1" : "0";
echo defined("MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS") ? "1" : "0";
echo defined("MYSQLI_OPT_CONNECT_TIMEOUT") ? "1" : "0";
echo defined("MYSQLI_OPT_LOCAL_INFILE") ? "1" : "0";
echo defined("MYSQLI_OPT_LOAD_DATA_LOCAL_DIR") ? "1" : "0";
echo defined("MYSQLI_INIT_COMMAND") ? "1" : "0";
echo defined("MYSQLI_OPT_READ_TIMEOUT") ? "1" : "0";
echo defined("MYSQLI_OPT_NET_CMD_BUFFER_SIZE") ? "1" : "0";
echo defined("MYSQLI_OPT_NET_READ_BUFFER_SIZE") ? "1" : "0";
echo defined("MYSQLI_OPT_INT_AND_FLOAT_NATIVE") ? "1" : "0";
echo defined("MYSQLI_OPT_SSL_VERIFY_SERVER_CERT") ? "1" : "0";
echo defined("MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS") ? "1" : "0";
echo defined("MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH") ? "1" : "0";
echo defined("MYSQLI_STMT_ATTR_CURSOR_TYPE") ? "1" : "0";
echo defined("MYSQLI_STMT_ATTR_PREFETCH_ROWS") ? "1" : "0";
echo defined("MYSQLI_CURSOR_TYPE_NO_CURSOR") ? "1" : "0";
echo defined("MYSQLI_CURSOR_TYPE_READ_ONLY") ? "1" : "0";
echo defined("MYSQLI_CURSOR_TYPE_FOR_UPDATE") ? "1" : "0";
echo defined("MYSQLI_CURSOR_TYPE_SCROLLABLE") ? "1" : "0";
echo defined("MYSQLI_REFRESH_GRANT") ? "1" : "0";
echo defined("MYSQLI_REFRESH_LOG") ? "1" : "0";
echo defined("MYSQLI_REFRESH_TABLES") ? "1" : "0";
echo defined("MYSQLI_REFRESH_HOSTS") ? "1" : "0";
echo defined("MYSQLI_REFRESH_STATUS") ? "1" : "0";
echo defined("MYSQLI_REFRESH_THREADS") ? "1" : "0";
echo defined("MYSQLI_REFRESH_SLAVE") ? "1" : "0";
echo defined("MYSQLI_REFRESH_REPLICA") ? "1" : "0";
echo defined("MYSQLI_REFRESH_MASTER") ? "1" : "0";
echo defined("MYSQLI_REFRESH_BACKUP_LOG") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 255, "{ir}");
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
mysqli_execute_query(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array(1));
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
mysqli_options(mysqli_init(), MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_set_opt(mysqli_init(), MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_ssl_set(mysqli_init(), null, null, null, null, null);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_connect_errno();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_connect_error();
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
mysqli_kill(mysqli_init(), 1);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_change_user(mysqli_init(), "user", "pass", "wordpress");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_refresh(mysqli_init(), MYSQLI_REFRESH_LOG);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_links_stats();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_dump_debug_info(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_client_stats();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_thread_safe();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_init(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_prepare(mysqli_init(), "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_prepare(mysqli_init(), "SELECT 1");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_param_count(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_get_warnings(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_error_list(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_bind_param(mysqli_init(), "s", $value);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_bind_result(mysqli_init(), $value);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_execute(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_execute(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_get_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_close(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_errno(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_error(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_affected_rows(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_store_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_num_rows(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_fetch(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_result_metadata(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_field_count(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_free_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_data_seek(mysqli_init(), 0);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_attr_get(mysqli_init(), 1);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_attr_set(mysqli_init(), 1, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_send_long_data(mysqli_init(), 0, "blob");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_reset(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_more_results(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_next_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_sqlstate(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_warning_count(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_stmt_insert_id(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_fetch_fields(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_fetch_field_direct(mysqli_init(), 0);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_field_seek(mysqli_init(), 0);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_field_tell(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_fetch_lengths(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_fetch_all(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_fetch_column(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_debug("d:t:o,/tmp/phpc.trace");
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
mysqli_savepoint(mysqli_init(), "wp");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_release_savepoint(mysqli_init(), "wp");
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
mysqli_real_query(mysqli_init(), "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_multi_query(mysqli_init(), "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
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
mysqli_error_list(mysqli_init());
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
mysqli_info(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_get_warnings(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_store_result(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_use_result(mysqli_init());
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
mysqli_reap_async_query(mysqli_init());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
mysqli_poll(1, 2, 3, 0);
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

    let error = emit_ir_source(
        r#"<?php
mysqli_escape_string(mysqli_init(), "value");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
