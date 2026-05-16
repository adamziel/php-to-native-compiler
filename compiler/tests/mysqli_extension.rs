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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|stmt-callable|prepare-exists|prepare-callable"
    );
    assert_eq!(execution.exit_code, 0);

    let stmt_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_stmt_init($handle);
"#,
    )
    .unwrap_err();

    assert_eq!(stmt_error.phase, Phase::Runtime);
    assert_eq!(stmt_error.line, 3);
    assert_eq!(stmt_error.column, 1);
    assert_eq!(
        stmt_error.message,
        "unsupported call mysqli_stmt_init(): mysqli statement objects and prepared statement lifecycle are not implemented in the current subset"
    );

    let prepare_error = run_source(
        r#"<?php
$handle = mysqli_init();
mysqli_prepare($handle, "SELECT option_value FROM wp_options WHERE option_name = ?");
"#,
    )
    .unwrap_err();

    assert_eq!(prepare_error.phase, Phase::Runtime);
    assert_eq!(prepare_error.line, 3);
    assert_eq!(prepare_error.column, 1);
    assert_eq!(
        prepare_error.message,
        "unsupported call mysqli_prepare(): mysqli prepared statement parsing, statement objects, binding, execution, and result metadata are not implemented in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|prepare-callable|param-count-exists|param-count-callable|warnings-exists|warnings-callable|error-list-exists|error-list-callable"
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
        "unsupported call mysqli_stmt_prepare(): mysqli statement objects, prepared SQL parsing, prepared statement state, and host database execution are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_param_count(): mysqli statement objects, prepared SQL parsing, parameter metadata, and statement lifecycle state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_get_warnings(): mysqli statement objects, statement warning chains, and statement diagnostic state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_error_list(): mysqli statement objects, statement error-list tracking, and statement diagnostic state are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_bind_and_execute_are_visible_but_explicit_boundaries() {
    let execution = run_source(
        r#"<?php
$bind = "mysqli_stmt_bind_param";
$execute = "mysqli_stmt_execute";
echo function_exists($bind) ? "yes" : "no";
echo "|";
echo is_callable($bind) ? "bind-callable" : "bind-missing";
echo "|";
echo function_exists($execute) ? "execute-exists" : "execute-missing";
echo "|";
echo is_callable($execute) ? "execute-callable" : "execute-missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|bind-callable|execute-exists|execute-callable"
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
        "unsupported call mysqli_stmt_bind_param(): mysqli statement objects, by-reference parameter binding, type strings, and prepared statement execution are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_execute(): mysqli statement objects, bound parameters, array parameter execution, result state, and host database execution are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_bind_result_is_visible_but_explicit_boundary() {
    let execution = run_source(
        r#"<?php
$bind_result = "mysqli_stmt_bind_result";
echo function_exists($bind_result) ? "yes" : "no";
echo "|";
echo is_callable($bind_result) ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable");
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
        "unsupported call mysqli_stmt_bind_result(): mysqli statement objects, by-reference result binding, result buffer mutation, and fetch integration are not implemented in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|result-callable|close-exists|close-callable"
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
        "unsupported call mysqli_stmt_get_result(): mysqli statement objects, statement result materialization, result metadata, and mysqlnd result transfer are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_close(): mysqli statement objects, statement resource cleanup, and statement lifecycle state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_errno(): mysqli statement objects and statement error-state metadata are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_error(): mysqli statement objects and statement error-message metadata are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_affected_rows(): mysqli statement objects, statement execution state, and affected-row metadata are not implemented in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|store-callable|num-rows-exists|num-rows-callable|fetch-exists|fetch-callable"
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
        "unsupported call mysqli_stmt_store_result(): mysqli statement objects, result buffering, and statement result state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_num_rows(): mysqli statement objects, buffered statement result state, and statement row-count metadata are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_fetch(): mysqli statement objects, bound result buffers, cursor advancement, and host database rows are not implemented in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|metadata-callable|field-count-exists|field-count-callable|free-result-exists|free-result-callable"
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
        "unsupported call mysqli_stmt_result_metadata(): mysqli statement objects, statement result metadata objects, and field metadata transfer are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_field_count(): mysqli statement objects, statement result metadata, and statement field-count state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_free_result(): mysqli statement objects, statement result buffers, and statement result cleanup state are not implemented in the current subset"
    );
}

#[test]
fn mysqli_statement_positioning_and_attributes_are_visible_but_explicit_boundary() {
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|data-seek-callable|attr-get-exists|attr-get-callable|attr-set-exists|attr-set-callable"
    );
    assert_eq!(execution.exit_code, 0);

    let data_seek_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_data_seek($stmt, 0);
"#,
    )
    .unwrap_err();

    assert_eq!(data_seek_error.phase, Phase::Runtime);
    assert_eq!(data_seek_error.line, 3);
    assert_eq!(data_seek_error.column, 1);
    assert_eq!(
        data_seek_error.message,
        "unsupported call mysqli_stmt_data_seek(): mysqli statement objects, buffered result cursors, offset seeking, and statement result state are not implemented in the current subset"
    );

    let attr_get_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_attr_get($stmt, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(attr_get_error.phase, Phase::Runtime);
    assert_eq!(attr_get_error.line, 3);
    assert_eq!(attr_get_error.column, 1);
    assert_eq!(
        attr_get_error.message,
        "unsupported call mysqli_stmt_attr_get(): mysqli statement objects, statement attributes, and option registry state are not implemented in the current subset"
    );

    let attr_set_error = run_source(
        r#"<?php
$stmt = mysqli_init();
mysqli_stmt_attr_set($stmt, 1, 1);
"#,
    )
    .unwrap_err();

    assert_eq!(attr_set_error.phase, Phase::Runtime);
    assert_eq!(attr_set_error.line, 3);
    assert_eq!(attr_set_error.column, 1);
    assert_eq!(
        attr_set_error.message,
        "unsupported call mysqli_stmt_attr_set(): mysqli statement objects, statement attributes, option mutation, and option registry state are not implemented in the current subset"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|send-long-callable|reset-exists|reset-callable|more-results-exists|more-results-callable|next-result-exists|next-result-callable"
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
        "unsupported call mysqli_stmt_send_long_data(): mysqli statement objects, long-parameter streaming, packet buffering, and statement parameter state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_reset(): mysqli statement objects, statement state reset, buffered results, and parameter/result lifecycle state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_more_results(): mysqli statement objects, multi-result state, and pending statement result queues are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_next_result(): mysqli statement objects, multi-result cursor advancement, and pending statement result queues are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_sqlstate(): mysqli statement objects, statement SQLSTATE tracking, and statement diagnostic state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_warning_count(): mysqli statement objects, statement warning tracking, and statement diagnostic state are not implemented in the current subset"
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
        "unsupported call mysqli_stmt_insert_id(): mysqli statement objects, statement execution state, and statement insert-id metadata are not implemented in the current subset"
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
$direct = mysqli_fetch_field_direct($result, 1);
echo $direct->name;
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
        "yes|fetch-fields-callable|fetch-direct-exists|fetch-direct-callable|field-seek-exists|field-seek-callable|field-tell-exists|field-tell-callable|stmt-fields-missing|stmt-field-missing|ID,post_title|post_title|0|seek|1|post_title|2|no-direct|no-seek"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|alias-exists|alias-callable|201|set|set|alias-set|dynamic-alias"
    );
    assert_eq!(execution.exit_code, 0);
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
mysqli_options(mysqli_init(), 0, true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_option.phase, Phase::Runtime);
    assert_eq!(bad_option.line, 2);
    assert_eq!(bad_option.column, 1);
    assert_eq!(
        bad_option.message,
        "unsupported call mysqli_options(): only MYSQLI_OPT_INT_AND_FLOAT_NATIVE is supported in the current subset, got 0"
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
        "unsupported call mysqli_options(): value must be bool or int for MYSQLI_OPT_INT_AND_FLOAT_NATIVE in the current subset, got string"
    );

    let bad_alias_option = run_source(
        r#"<?php
mysqli_set_opt(mysqli_init(), 0, true);
"#,
    )
    .unwrap_err();

    assert_eq!(bad_alias_option.phase, Phase::Runtime);
    assert_eq!(bad_alias_option.line, 2);
    assert_eq!(bad_alias_option.column, 1);
    assert_eq!(
        bad_alias_option.message,
        "unsupported call mysqli_set_opt(): only MYSQLI_OPT_INT_AND_FLOAT_NATIVE is supported in the current subset, got 0"
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
        "unsupported call mysqli_set_opt(): value must be bool or int for MYSQLI_OPT_INT_AND_FLOAT_NATIVE in the current subset, got string"
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
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|charset-ok|dynamic|done|done|no-pending"
    );
    assert_eq!(execution.exit_code, 0);
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
        "unsupported call mysqli_real_query(): result-producing mysqli_real_query() SQL is not implemented because pending result state for mysqli_store_result()/mysqli_use_result() is not modeled; got SELECT 1"
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
        "unsupported call mysqli_multi_query(): multi-statement mysqli_multi_query() SQL is not implemented because pending result queues and mysqli_more_results()/mysqli_next_result() state are not modeled; got SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'; SELECT 1"
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
        "unsupported call mysqli_multi_query(): result-producing mysqli_multi_query() SQL is not implemented because pending result queues and mysqli_store_result()/mysqli_use_result() state are not modeled; got SELECT 1"
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
echo defined("MYSQLI_OPT_INT_AND_FLOAT_NATIVE") ? "1" : "0";
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

    assert_eq!(ir.matches("c\"1\\00\"").count(), 226, "{ir}");
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
