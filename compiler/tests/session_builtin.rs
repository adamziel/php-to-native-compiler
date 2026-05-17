use php_compiler::error::Phase;
use php_compiler::interpreter::{run_program_with_source_file_and_options, RunOptions};
use php_compiler::{emit_asm_source, emit_ir_source, parse, run_source};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const LLVM_SESSION_STATE_REJECTION: &str = "LLVM session-state lowering rejects $_SESSION and session_start(), session_status(), session_cache_limiter(), session_cache_expire(), session_id(), and session_write_close() until native request/session storage, session id persistence, cache limiter state, locking, cookie/header emission, save handlers, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded CLI session-state behavior";

fn format_http_date(timestamp: i64) -> String {
    const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let days = timestamp.div_euclid(86_400);
    let seconds = timestamp.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds / 3_600;
    let minute = (seconds % 3_600) / 60;
    let second = seconds % 60;
    let weekday = WEEKDAYS[days.rem_euclid(7) as usize];
    let month_name = MONTHS[(month - 1) as usize];
    format!("{weekday}, {day:02} {month_name} {year:04} {hour:02}:{minute:02}:{second:02} GMT")
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

#[test]
fn session_start_materializes_session_superglobal_and_status() {
    let execution = run_source(
        r#"<?php
$previous = session_id("phpctestsession");
$before = session_status();
$started = session_start();
echo $before === PHP_SESSION_NONE ? "none" : "other";
echo "|";
echo $previous === "" ? "empty-id" : "had-id";
echo "|";
echo $started ? "started" : "failed";
echo "|";
echo session_status() === PHP_SESSION_ACTIVE ? "active" : "inactive";
echo "|";
echo session_id();
$_SESSION["auth"]["user"] = "admin";
$_SESSION["count"] = 2;
echo "|";
echo $_SESSION["auth"]["user"];
echo ":";
echo $_SESSION["count"];
echo "|";
echo count(headers_list());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "none|empty-id|started|active|phpctestsession|admin:2|4"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_emits_cookie_header_unless_use_cookies_is_false() {
    let execution = run_source(
        r#"<?php
session_id("phpccookie");
$out = array();
$first = session_start();
$headers = headers_list();
$out[] = $first ? "started" : "failed";
$out[] = count($headers);
$out[] = $headers[0];
session_write_close();
header_remove();
session_id("phpcnocookie");
$second = session_start(["use_cookies" => false]);
$out[] = $second ? "second" : "second-failed";
$out[] = count(headers_list());
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "started|4|Set-Cookie: PHPSESSID=phpccookie|second|3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_emits_default_nocache_headers() {
    let execution = run_source(
        r#"<?php
session_id("phpccache");
$out = array();
$started = session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = $started ? "started" : "failed";
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
header("Cache-Control: custom");
session_write_close();
session_id("phpccachetwo");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "started|3|Expires: Thu, 19 Nov 1981 08:52:00 GMT|Cache-Control: no-store, no-cache, must-revalidate|Pragma: no-cache|3|Expires: Thu, 19 Nov 1981 08:52:00 GMT|Cache-Control: no-store, no-cache, must-revalidate|Pragma: no-cache"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_cache_apis_configure_bounded_cache_headers() {
    let execution = run_source(
        r#"<?php
$out = array();
$out[] = session_cache_limiter();
$out[] = session_cache_expire();
$out[] = session_cache_expire(42);
$out[] = session_cache_expire();
$out[] = session_cache_limiter("");
$out[] = session_cache_limiter();
session_id("phpcnocacheoff");
session_start(["use_cookies" => false]);
$out[] = count(headers_list());
session_write_close();
header_remove();
$out[] = session_cache_limiter("nocache");
$out[] = session_cache_limiter();
session_id("phpcnocacheon");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "nocache|180|180|42|nocache||0||nocache|3|Expires: Thu, 19 Nov 1981 08:52:00 GMT"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_cache_limiter_emits_private_and_public_cache_headers() {
    let execution = run_source(
        r#"<?php
$out = array();
session_cache_limiter("private");
session_cache_expire(2);
session_id("phpcprivatecache");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
session_write_close();
header_remove();
session_cache_limiter("private_no_expire");
session_cache_expire(3);
session_id("phpcprivatenoexpire");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
session_write_close();
header_remove();
session_cache_limiter("public");
session_cache_expire(1);
session_id("phpcpubliccache");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3|Expires: Thu, 19 Nov 1981 08:52:00 GMT|Cache-Control: private, max-age=120|Last-Modified: Thu, 01 Jan 1970 00:00:00 GMT|2|Cache-Control: private, max-age=180|Last-Modified: Thu, 01 Jan 1970 00:00:00 GMT|3|Expires: Thu, 01 Jan 1970 00:01:00 GMT|Cache-Control: public, max-age=60|Last-Modified: Thu, 01 Jan 1970 00:00:00 GMT"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_cache_limiter_uses_request_time_and_main_script_mtime() {
    let source = r#"<?php
$out = array();
session_cache_limiter("public");
session_cache_expire(1);
session_id("phpcmtimecache");
session_start(["use_cookies" => false]);
$headers = headers_list();
$out[] = $_SERVER["REQUEST_TIME"];
$out[] = count($headers);
$out[] = $headers[0];
$out[] = $headers[1];
$out[] = $headers[2];
echo implode("|", $out);
"#;
    let path = std::env::temp_dir().join(format!(
        "phpc-session-cache-mtime-{}-{}.php",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ));
    fs::write(&path, source).expect("temporary session-cache source can be written");
    let metadata = fs::metadata(&path).expect("temporary session-cache source has metadata");
    let modified = metadata
        .modified()
        .expect("temporary session-cache source has mtime")
        .duration_since(UNIX_EPOCH)
        .expect("temporary session-cache source mtime is after Unix epoch")
        .as_secs() as i64;

    let program = parse(source).unwrap();
    let execution = run_program_with_source_file_and_options(
        &program,
        path.display().to_string(),
        RunOptions {
            request_time: Some(978_307_200),
            ..RunOptions::default()
        },
    )
    .unwrap();
    let _ = fs::remove_file(path);

    assert_eq!(
        execution.stdout,
        format!(
            "978307200|3|Expires: Mon, 01 Jan 2001 00:01:00 GMT|Cache-Control: public, max-age=60|Last-Modified: {}",
            format_http_date(modified)
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_cache_apis_warn_when_changed_too_late() {
    let execution = run_source(
        r#"<?php
function cache_warning($errno, $errstr, $errfile, $errline) {
    echo "warn:" . $errno;
    echo ":" . (str_contains($errstr, "cache") ? "cache" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
set_error_handler("cache_warning", E_WARNING);
session_start(["use_cookies" => false]);
$limiter = session_cache_limiter("");
$expire = session_cache_expire(5);
echo "|active:" . ($limiter === false ? "limiter-false" : "limiter-value");
echo ":" . $expire;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "warn:2:cache:Command line code:10warn:2:other:Command line code:11|active:limiter-false:"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_rejects_unknown_cache_limiter_before_state_changes() {
    let error = run_source(
        r#"<?php
session_cache_limiter("must-revalidate");
session_id("phpcunknowncache");
session_start();
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call session_start(): session cache limiter 'must-revalidate' is not supported in the current subset"
    );
}

#[test]
fn session_start_emits_bounded_cookie_attributes_from_options() {
    let execution = run_source(
        r#"<?php
session_id("phpcattrs");
$out = array();
$started = session_start([
    "cookie_lifetime" => 3600,
    "cookie_path" => "/wp-admin",
    "cookie_domain" => "example.test",
    "cookie_secure" => true,
    "cookie_httponly" => true,
    "cookie_samesite" => "Lax",
]);
$headers = headers_list();
$out[] = $started ? "started" : "failed";
$out[] = count($headers);
$out[] = $headers[0];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "started|4|Set-Cookie: PHPSESSID=phpcattrs; Max-Age=3600; path=/wp-admin; domain=example.test; secure; HttpOnly; SameSite=Lax"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_write_close_keeps_session_data_visible_but_closes_status() {
    let execution = run_source(
        r#"<?php
session_start();
$_SESSION["stage"] = "open";
$closed = session_write_close();
echo $closed ? "closed" : "failed";
echo "|";
echo session_status() === PHP_SESSION_NONE ? "none" : "active";
echo "|";
echo $_SESSION["stage"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "closed|none|open");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_read_and_close_option_closes_after_materializing_session() {
    let execution = run_source(
        r#"<?php
session_id("phpcreadclose");
$out = array();
$out[] = session_start(["read_and_close" => true]) ? "started" : "failed";
$out[] = session_status() === PHP_SESSION_NONE ? "closed" : "active";
$out[] = session_id();
$_SESSION["after"] = "visible";
$out[] = $_SESSION["after"];
$out[] = session_start(["read_and_close" => false]) ? "again-started" : "again-failed";
$out[] = session_status() === PHP_SESSION_ACTIVE ? "active" : "closed";
$_SESSION["during"] = "open";
session_write_close();
$out[] = session_status() === PHP_SESSION_NONE ? "closed" : "active";
$out[] = $_SESSION["during"];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "started|closed|phpcreadclose|visible|again-started|active|closed|open"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_restart_reloads_last_closed_snapshot() {
    let execution = run_source(
        r#"<?php
session_id("phpcpersist");
$out = array();
session_start();
$_SESSION["token"] = "saved";
$_SESSION["nested"]["role"] = "admin";
session_write_close();
$_SESSION["token"] = "closed-edit";
$_SESSION["nested"]["role"] = "guest";
$out[] = $_SESSION["token"] . ":" . $_SESSION["nested"]["role"];
session_start();
$out[] = $_SESSION["token"] . ":" . $_SESSION["nested"]["role"];
$_SESSION["token"] = "second";
session_write_close();
$_SESSION["token"] = "third";
session_start(["read_and_close" => true]);
$out[] = session_status() === PHP_SESSION_NONE ? "closed" : "active";
$out[] = $_SESSION["token"];
$_SESSION["token"] = "after-read-close";
session_start();
$out[] = $_SESSION["token"];
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "closed-edit:guest|saved:admin|closed|second|second"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_save_path_persists_across_interpreter_invocations() {
    let save_path = unique_session_save_path("phpc-session-file-persist");
    fs::create_dir_all(&save_path).unwrap();
    let save_path_php = save_path.display().to_string().replace('\\', "\\\\");
    let write_source = format!(
        r#"<?php
ini_set("session.save_path", "{save_path_php}");
session_id("phpcfilepersist");
session_start(["use_cookies" => false]);
$_SESSION["token"] = "saved";
$_SESSION["nested"]["count"] = 2;
session_write_close();
echo "wrote";
"#
    );
    let read_source = format!(
        r#"<?php
ini_set("session.save_path", "{save_path_php}");
session_id("phpcfilepersist");
session_start(["use_cookies" => false]);
echo $_SESSION["token"], "|", $_SESSION["nested"]["count"];
"#
    );

    let write = run_source(&write_source).unwrap();
    assert_eq!(write.stdout, "wrote");

    let read = run_source(&read_source).unwrap();
    assert_eq!(read.stdout, "saved|2");

    let session_file = save_path.join("sess_phpcfilepersist");
    let stored = fs::read_to_string(&session_file).unwrap();
    assert!(stored.contains("token|s:5:\"saved\""), "{stored}");
    assert!(
        stored.contains("nested|a:1:{s:5:\"count\";i:2;}"),
        "{stored}"
    );
    let _ = fs::remove_file(session_file);
    let _ = fs::remove_dir(save_path);
}

#[test]
fn session_start_rejects_explicit_ids_outside_bounded_file_safe_subset() {
    let execution = run_source(
        r#"<?php
function invalid_session_id_warning($errno, $errstr, $errfile, $errline) {
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "bounded file-safe subset") ? "id" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
set_error_handler("invalid_session_id_warning", E_WARNING);
session_id("bad/slash");
$started = session_start(["use_cookies" => false]);
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_NONE ? "none" : "active");
echo "|headers:" . count(headers_list());
echo "|session:" . (isset($_SESSION) ? "set" : "unset");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "warning:2:id:Command line code:10|return:false|status:none|headers:0|session:unset"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_warns_and_recovers_from_malformed_session_file() {
    let save_path = unique_session_save_path("phpc-session-file-malformed");
    fs::create_dir_all(&save_path).unwrap();
    fs::write(
        save_path.join("sess_phpcmalformed"),
        "token|s:5:\"short\";bad",
    )
    .unwrap();
    let save_path_php = save_path.display().to_string().replace('\\', "\\\\");
    let source = format!(
        r#"<?php
function malformed_session_file_warning($errno, $errstr, $errfile, $errline) {{
    echo "warning:" . $errno;
    echo ":" . (str_contains($errstr, "Malformed session file") ? "malformed" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}}
ini_set("session.save_path", "{save_path_php}");
set_error_handler("malformed_session_file_warning", E_WARNING);
session_id("phpcmalformed");
$started = session_start(["use_cookies" => false]);
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_ACTIVE ? "active" : "none");
echo "|session:" . (isset($_SESSION["token"]) ? $_SESSION["token"] : "empty");
"#
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "warning:2:malformed:Command line code:11|return:true|status:active|session:empty"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    let _ = fs::remove_file(save_path.join("sess_phpcmalformed"));
    let _ = fs::remove_dir(save_path);
}

#[test]
fn session_start_while_active_emits_notice_and_keeps_session_open() {
    let execution = run_source(
        r#"<?php
function active_session_notice($errno, $errstr, $errfile, $errline) {
    echo "notice:" . $errno;
    echo ":" . (str_contains($errstr, "already active") ? "active" : "other");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
session_id("phpcactiverestart");
$first = session_start();
$_SESSION["phase"] = "open";
set_error_handler("active_session_notice", E_NOTICE);
$second = session_start(["read_and_close" => true]);
echo "|" . ($first ? "first" : "first-failed");
echo "|" . ($second ? "second" : "second-failed");
echo "|" . (session_status() === PHP_SESSION_ACTIVE ? "active" : "closed");
echo "|" . $_SESSION["phase"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "notice:8:active:Command line code:12|first|second|active|open"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_superglobal_reference_aliases_survive_function_scope_writes() {
    let execution = run_source(
        r#"<?php
session_start();
$_SESSION["payload"] = ["slot" => "start"];
$alias =& $_SESSION["payload"]["slot"];

function wp_session_refcow_update($suffix) {
    $_SESSION["payload"]["slot"] = $_SESSION["payload"]["slot"] . ":" . $suffix;
}

wp_session_refcow_update("function");
$alias = $alias . ":alias";
echo $_SESSION["payload"]["slot"], "|", $alias;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "start:function:alias|start:function:alias"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$id = "session_id";
$start = "session_start";
$status = "session_status";
$close = "session_write_close";
$exists = function_exists($start) ? "yes" : "no";
$callable = is_callable($start) ? "callable" : "missing";
$id("dynamic-session");
$started = $start() ? "started" : "failed";
echo $exists;
echo "|";
echo $callable;
echo "|";
echo $started;
echo "|";
echo $status() === PHP_SESSION_ACTIVE ? "active" : "inactive";
echo "|";
echo $close() ? "closed" : "failed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|started|active|closed");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_after_unbuffered_output_returns_false() {
    let execution = run_source(
        r#"<?php
echo "sent";
$started = session_start();
echo $started ? "|started" : "|failed";
echo "|";
echo session_status() === PHP_SESSION_NONE ? "none" : "active";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "sent|failed|none");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_start_after_unbuffered_output_emits_recoverable_warning() {
    let execution = run_source(
        r#"<?php
function late_session_warning($errno, $errstr, $errfile, $errline) {
    echo "|warn:" . $errno;
    echo ":" . (str_contains($errstr, "Session cannot be started") ? "session" : "other");
    echo ":" . (str_contains($errstr, "headers have already been sent") ? "headers" : "missing");
    echo ":" . basename($errfile) . ":" . $errline;
    return true;
}
set_error_handler("late_session_warning", E_WARNING);
echo "body";
$started = session_start();
echo "|return:" . ($started ? "true" : "false");
echo "|status:" . (session_status() === PHP_SESSION_NONE ? "none" : "active");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "body|warn:2:session:headers:Command line code:11|return:false|status:none"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn session_builtins_reject_forms_outside_current_subset() {
    let too_many = run_source("<?php\nsession_start(array(), array());\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for session_start(): expected 0 to 1 argument(s), got 2"
    );

    let bad_options = run_source("<?php\nsession_start('read_and_close=1');\n").unwrap_err();
    assert_eq!(bad_options.phase, Phase::Runtime);
    assert_eq!(bad_options.line, 2);
    assert_eq!(bad_options.column, 1);
    assert_eq!(
        bad_options.message,
        "unsupported call session_start(): options argument must be array in the current subset, got string"
    );

    let bad_cookie_path =
        run_source("<?php\nsession_start(['cookie_path' => 123]);\n").unwrap_err();
    assert_eq!(bad_cookie_path.phase, Phase::Runtime);
    assert_eq!(bad_cookie_path.line, 2);
    assert_eq!(bad_cookie_path.column, 1);
    assert_eq!(
        bad_cookie_path.message,
        "unsupported call session_start(): cookie_path option must be string in the current subset, got int"
    );

    let bad_id = run_source("<?php\nsession_id(42);\n").unwrap_err();
    assert_eq!(bad_id.phase, Phase::Runtime);
    assert_eq!(bad_id.line, 2);
    assert_eq!(bad_id.column, 1);
    assert_eq!(
        bad_id.message,
        "unsupported call session_id(): id argument must be string in the current subset, got int"
    );

    let bad_limiter = run_source("<?php\nsession_cache_limiter(42);\n").unwrap_err();
    assert_eq!(bad_limiter.phase, Phase::Runtime);
    assert_eq!(bad_limiter.line, 2);
    assert_eq!(bad_limiter.column, 1);
    assert_eq!(
        bad_limiter.message,
        "unsupported call session_cache_limiter(): cache_limiter argument must be string in the current subset, got int"
    );

    let bad_expire = run_source("<?php\nsession_cache_expire('42');\n").unwrap_err();
    assert_eq!(bad_expire.phase, Phase::Runtime);
    assert_eq!(bad_expire.line, 2);
    assert_eq!(bad_expire.column, 1);
    assert_eq!(
        bad_expire.message,
        "unsupported call session_cache_expire(): new_cache_expire argument must be int in the current subset, got string"
    );
}

#[test]
fn emit_ir_folds_session_metadata_but_rejects_stateful_calls_and_superglobal() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("session_start") ? "1" : "0";
echo is_callable("session_write_close") ? "1" : "0";
echo function_exists("session_cache_limiter") ? "1" : "0";
echo is_callable("session_cache_expire") ? "1" : "0";
echo defined("PHP_SESSION_ACTIVE") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");

    let call_error = emit_ir_source("<?php\nsession_cache_limiter();\n").unwrap_err();
    assert_eq!(call_error.phase, Phase::Codegen);
    assert_eq!(call_error.line, 2);
    assert_eq!(call_error.column, 1);
    assert_eq!(call_error.message, LLVM_SESSION_STATE_REJECTION);

    let superglobal_error = emit_ir_source("<?php\necho $_SESSION['x'];\n").unwrap_err();
    assert_eq!(superglobal_error.phase, Phase::Codegen);
    assert_eq!(superglobal_error.line, 2);
    assert_eq!(superglobal_error.column, 6);
    assert!(superglobal_error.message.contains("request-superglobal"));
}

#[test]
fn emit_asm_rejects_session_state_before_backend_execution() {
    let error = emit_asm_source("<?php\nsession_status();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_SESSION_STATE_REJECTION);
}

fn unique_session_save_path(prefix: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
