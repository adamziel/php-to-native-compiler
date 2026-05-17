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
        "unsupported call mysqli_stmt_bind_result(): result bindings must be direct variables in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "no|1|0|0|auto-on");
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
