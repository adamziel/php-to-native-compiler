use php_compiler::error::Phase;
use php_compiler::{emit_asm_source, emit_ir_source};
use php_compiler::{run_source, run_source_with_source_file};

const LLVM_HEADER_STATE_REJECTION: &str = "LLVM header-state lowering rejects header(), header_remove(), headers_list(), headers_sent(), http_response_code(), and setcookie() until native response-header storage, output-started tracking, status-code handling, cookie formatting, SAPI emission, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI header-state behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn header_accepts_current_noop_signature() {
    let execution = run_source(
        r#"<?php
echo "before";
$result = header("HTTP/1.1 500 Internal Server Error", true, 500);
echo $result === null ? "|null" : "|not-null";
header("Content-Type: text/html; charset=utf-8");
header("X-No-Replace: one", false);
echo "|after";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "before|null|after");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_list_returns_current_cli_header_log() {
    let execution = run_source(
        r#"<?php
$initial = count(headers_list());
header("X-First: one");
header("Content-Type: text/plain", true, 200);
header("X-First: two", false);
$headers = headers_list();
echo $initial;
echo "|";
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $headers[1];
echo "|";
echo $headers[2];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0|3|X-First: one|Content-Type: text/plain|X-First: two"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_replaces_matching_cli_header_names_by_default() {
    let execution = run_source(
        r#"<?php
header("X-Replace: one");
header("X-Keep: one");
header("x-replace: two");
header("X-Keep: two", false);
$headers = headers_list();
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $headers[1];
echo "|";
echo $headers[2];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3|X-Keep: one|x-replace: two|X-Keep: two");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_and_http_response_code_track_bounded_status_state() {
    let execution = run_source(
        r#"<?php
$out = array();
$initial = http_response_code();
$out[] = $initial === false ? "false" : "not-false";
$previous = http_response_code(201);
$out[] = $previous === true ? "true" : "not-true";
$out[] = (string) http_response_code();
header("Location: /wp-admin/");
$out[] = (string) http_response_code();
http_response_code(404);
header("Location: /wp-login.php");
$out[] = (string) http_response_code();
header("HTTP/1.1 503 Service Unavailable");
$out[] = (string) http_response_code();
header("X-Test: one", true, 204);
$out[] = (string) http_response_code();
header("HTTP/1.1 500 Internal Server Error", true, 0);
$out[] = (string) http_response_code();
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "false|true|201|201|302|503|204|500");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "header";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
$result = $call("X-Test: dynamic", true, 204);
echo $result === null ? "|null" : "|not-null";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|null");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_remove_mutates_current_cli_header_log() {
    let execution = run_source(
        r#"<?php
header("Last-Modified: today");
header("Content-Type: text/plain");
header("Last-Modified: tomorrow");
$result = header_remove("Last-Modified");
$headers = headers_list();
header_remove();
$after_clear = count(headers_list());
echo $result === null ? "null" : "not-null";
echo "|";
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $after_clear;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "null|1|Content-Type: text/plain|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_remove_matches_header_names_case_insensitively() {
    let execution = run_source(
        r#"<?php
header("Last-Modified: today");
header("X-Keep: one");
header("last-modified: tomorrow", false);
header_remove("LAST-MODIFIED");
$headers = headers_list();
echo count($headers);
echo "|";
echo $headers[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|X-Keep: one");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_remove_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "header_remove";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
header("Last-Modified: today");
$result = $call("Last-Modified");
echo $result === null ? "|null" : "|not-null";
echo "|";
echo count(headers_list());
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|null|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn setcookie_appends_set_cookie_header_to_current_cli_header_log() {
    let execution = run_source(
        r#"<?php
$initial = count(headers_list());
$result = setcookie("wordpress_test_cookie", "WP Cookie check");
setcookie("empty_cookie");
$headers = headers_list();
echo $initial;
echo "|";
echo $result ? "true" : "false";
echo "|";
echo count($headers);
echo "|";
echo $headers[0];
echo "|";
echo $headers[1];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0|true|2|Set-Cookie: wordpress_test_cookie=WP%20Cookie%20check|Set-Cookie: empty_cookie="
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn setcookie_formats_bounded_attributes_encoding_and_name_replacement() {
    let execution = run_source(
        r#"<?php
$out = array();
setcookie("wordpress_test_cookie", "old value");
$out[] = count(headers_list());
$first = setcookie("wordpress_test_cookie", "WP Cookie check", 1700000000, "/wp-admin", "example.test", true, true);
$second = setcookie("logged_in", "delete me", ["expires" => 1, "path" => "/", "secure" => false, "httponly" => true, "samesite" => "Lax"]);
$headers = headers_list();
$out[] = $first ? "first" : "first-failed";
$out[] = $second ? "second" : "second-failed";
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|first|second|2|Set-Cookie: wordpress_test_cookie=WP%20Cookie%20check; expires=Tue, 14 Nov 2023 22:13:20 GMT; path=/wp-admin; domain=example.test; secure; HttpOnly|Set-Cookie: logged_in=delete%20me; expires=Thu, 01 Jan 1970 00:00:01 GMT; path=/; HttpOnly; SameSite=Lax"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn setcookie_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "setcookie";
$result = $call("wordpress_test_cookie", "1");
$headers = headers_list();
$exists = function_exists($call) ? "yes" : "no";
$callable = is_callable($call) ? "callable" : "missing";
echo $exists;
echo "|";
echo $callable;
echo $result ? "|true" : "|false";
echo "|";
echo $headers[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|true|Set-Cookie: wordpress_test_cookie=1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_sent_accepts_current_no_header_state_signature() {
    let execution = run_source(
        r#"<?php
echo headers_sent() ? "sent" : "open";
header("X-Test: one");
echo "|";
$call = "headers_sent";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call() ? "sent" : "open";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "open|yes|callable|sent");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_sent_reports_first_unbuffered_output_location() {
    let execution = run_source_with_source_file(
        r#"<?php
$first = headers_sent($file, $line);
echo ($first ? "sent" : "open") . ":" . $file . ":" . $line;
echo "|bytes";
$second = headers_sent($file, $line);
echo "|" . ($second ? "sent" : "open") . ":" . $file . ":" . $line;
"#,
        "virtual/request.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "open::0|bytes|sent:virtual/request.php:3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_sent_stays_open_until_output_buffer_flushes_to_stdout() {
    let execution = run_source_with_source_file(
        r#"<?php
ob_start();
echo "buffered";
$before = headers_sent($file, $line);
echo "|" . ($before ? "sent" : "open") . ":" . $file . ":" . $line;
ob_flush();
$after = headers_sent($file, $line);
echo "|" . ($after ? "sent" : "open") . ":" . $file . ":" . $line;
"#,
        "virtual/buffered.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "buffered|open::0|sent:virtual/buffered.php:6"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn header_mutations_after_output_started_do_not_change_header_log() {
    let execution = run_source(
        r#"<?php
header("X-Before: one");
echo "body";
header("X-After: two");
$cookie = setcookie("late", "1");
header_remove("X-Before");
$headers = headers_list();
echo "|" . count($headers) . "|" . $headers[0] . "|" . ($cookie ? "cookie-true" : "cookie-false");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "body|1|X-Before: one|cookie-false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn late_header_mutations_emit_recoverable_warnings_through_error_handler() {
    let execution = run_source_with_source_file(
        r#"<?php
function late_header_warning($errno, $errstr, $errfile, $errline) {
    echo "|warn:" . $errno;
    echo ":" . (str_contains($errstr, "Cannot modify header information") ? "cannot" : "other");
    echo ":" . (str_contains($errstr, "output started at") ? "started" : "missing");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
header("X-Before: one");
set_error_handler("late_header_warning", E_WARNING);
echo "body";
$header_result = header("X-Late: two");
$cookie_result = setcookie("late_cookie", "1");
$remove_result = header_remove("X-Before");
$headers = headers_list();
echo "|returns:" . ($header_result === null ? "null" : "other");
echo ":" . ($cookie_result ? "cookie-true" : "cookie-false");
echo ":" . ($remove_result === null ? "remove-null" : "remove-other");
echo "|headers:" . count($headers) . ":" . $headers[0];
restore_error_handler();
"#,
        "tests/fixtures/milestone1433/late_header_warnings.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "body|warn:2:cannot:started:late_header_warnings.php:12|warn:2:cannot:started:late_header_warnings.php:13|warn:2:cannot:started:late_header_warnings.php:14|returns:null:cookie-false:remove-null|headers:1:X-Before: one"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn headers_list_rejects_forms_outside_current_subset() {
    let too_many = runtime_error(
        r#"<?php
echo headers_list("extra");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for headers_list(): expected 0 argument(s), got 1"
    );
}

#[test]
fn header_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo header();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for header(): expected 1 to 3 argument(s), got 0"
    );

    let header = runtime_error(
        r#"<?php
echo header(42);
"#,
    );
    assert_eq!(header.line, 2);
    assert_eq!(header.column, 6);
    assert_eq!(
        header.message,
        "unsupported call header(): header argument must be string in the current subset, got int"
    );

    let replace = runtime_error(
        r#"<?php
echo header("X-Test: one", "yes");
"#,
    );
    assert_eq!(replace.line, 2);
    assert_eq!(replace.column, 6);
    assert_eq!(
        replace.message,
        "unsupported call header(): replace argument must be bool in the current subset, got string"
    );

    let response_code = runtime_error(
        r#"<?php
echo header("X-Test: one", true, "500");
"#,
    );
    assert_eq!(response_code.line, 2);
    assert_eq!(response_code.column, 6);
    assert_eq!(
        response_code.message,
        "unsupported call header(): response_code argument must be int in the current subset, got string"
    );
}

#[test]
fn headers_sent_rejects_forms_outside_current_subset() {
    let output_arg = runtime_error(
        r#"<?php
echo headers_sent("file");
"#,
    );
    assert_eq!(output_arg.line, 2);
    assert_eq!(output_arg.column, 19);
    assert_eq!(
        output_arg.message,
        "unsupported call headers_sent(): filename output argument must be a direct variable in the current subset"
    );

    let line_arg = runtime_error(
        r#"<?php
$file = "";
echo headers_sent($file, 0);
"#,
    );
    assert_eq!(line_arg.line, 3);
    assert_eq!(line_arg.column, 26);
    assert_eq!(
        line_arg.message,
        "unsupported call headers_sent(): line output argument must be a direct variable in the current subset"
    );

    let too_many = runtime_error(
        r#"<?php
echo headers_sent("", 0, "extra");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for headers_sent(): expected 0 to 2 argument(s), got 3"
    );
}

#[test]
fn http_response_code_rejects_forms_outside_current_subset() {
    let non_int = runtime_error(
        r#"<?php
echo http_response_code("404");
"#,
    );
    assert_eq!(non_int.line, 2);
    assert_eq!(non_int.column, 6);
    assert_eq!(
        non_int.message,
        "unsupported call http_response_code(): response code argument must be int in the current subset, got string"
    );

    let too_many = runtime_error(
        r#"<?php
echo http_response_code(200, 404);
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for http_response_code(): expected 0 to 1 argument(s), got 2"
    );
}

#[test]
fn header_remove_rejects_forms_outside_current_subset() {
    let non_string = runtime_error(
        r#"<?php
echo header_remove(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call header_remove(): header name argument must be string in the current subset, got int"
    );

    let too_many = runtime_error(
        r#"<?php
echo header_remove("A", "B");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for header_remove(): expected 0 to 1 argument(s), got 2"
    );
}

#[test]
fn setcookie_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo setcookie();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for setcookie(): expected 1 to 7 argument(s), got 0"
    );

    let non_string_name = runtime_error(
        r#"<?php
echo setcookie(42);
"#,
    );
    assert_eq!(non_string_name.line, 2);
    assert_eq!(non_string_name.column, 6);
    assert_eq!(
        non_string_name.message,
        "unsupported call setcookie(): name argument must be string in the current subset, got int"
    );

    let non_string_value = runtime_error(
        r#"<?php
echo setcookie("wordpress_test_cookie", 1);
"#,
    );
    assert_eq!(non_string_value.line, 2);
    assert_eq!(non_string_value.column, 6);
    assert_eq!(
        non_string_value.message,
        "unsupported call setcookie(): value argument must be string in the current subset, got int"
    );

    let bad_expires = runtime_error(
        r#"<?php
echo setcookie("A", "B", "soon");
"#,
    );
    assert_eq!(bad_expires.line, 2);
    assert_eq!(bad_expires.column, 6);
    assert_eq!(
        bad_expires.message,
        "unsupported call setcookie(): expires argument must be int or options array in the current subset, got string"
    );

    let too_many = runtime_error(
        r#"<?php
echo setcookie("A", "B", 0, "", "", false, false, "extra");
"#,
    );
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for setcookie(): expected 1 to 7 argument(s), got 8"
    );
}

#[test]
fn emit_ir_rejects_header_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
header("Content-Type: text/html");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_rejects_header_remove_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\nheader_remove('Last-Modified');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_rejects_headers_list_until_native_header_state_exists() {
    let error = emit_ir_source("<?php\necho headers_list();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_asm_rejects_headers_list_during_native_header_state_lowering() {
    let error = emit_asm_source("<?php\necho headers_list();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_rejects_headers_sent_until_native_header_state_exists() {
    let error = emit_ir_source("<?php\necho headers_sent();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_rejects_http_response_code_until_native_header_state_exists() {
    let error = emit_ir_source("<?php\necho http_response_code();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_rejects_setcookie_until_native_header_state_exists() {
    let error = emit_ir_source("<?php\nsetcookie('wordpress_test_cookie', '1');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_HEADER_STATE_REJECTION);
}

#[test]
fn emit_ir_includes_header_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("header") ? "1" : "0";
echo is_callable("header") ? "1" : "0";
echo function_exists("header_remove") ? "1" : "0";
echo is_callable("header_remove") ? "1" : "0";
echo function_exists("headers_list") ? "1" : "0";
echo is_callable("headers_list") ? "1" : "0";
echo function_exists("headers_sent") ? "1" : "0";
echo is_callable("headers_sent") ? "1" : "0";
echo function_exists("http_response_code") ? "1" : "0";
echo is_callable("http_response_code") ? "1" : "0";
echo function_exists("setcookie") ? "1" : "0";
echo is_callable("setcookie") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 12, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
