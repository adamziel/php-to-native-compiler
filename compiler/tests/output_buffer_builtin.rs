use php_compiler::error::Phase;
use php_compiler::run_source;
use php_compiler::{emit_asm_source, emit_ir_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn ob_get_clean_captures_echoed_output_from_active_buffer() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "outer";
echo "|";
echo ob_get_level();
ob_start();
print "inner";
$inner = ob_get_clean();
echo "|inner=" . $inner;
echo "|level=" . ob_get_level();
$outer = ob_get_clean();
echo "clean=[" . $outer . "]";
echo "|level=" . ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "clean=[outer|1|inner=inner|level=1]|level=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unclosed_output_buffers_flush_when_program_finishes() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "outer";
ob_start();
echo "|inner";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "outer|inner");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_clean_without_active_buffer_returns_false() {
    let execution = run_source(
        r#"<?php
echo ob_get_level();
echo "|";
echo ob_get_clean() === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_flush_closes_active_buffer_flushes_and_returns_contents() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "outer:";
ob_start();
echo "inner";
$inner = ob_get_flush();
echo "|after-inner";
$outer = ob_get_flush();
echo "|inner=[" . $inner . "]";
echo "|outer=[" . $outer . "]";
echo "|level=" . ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "outer:inner|after-inner|inner=[inner]|outer=[outer:inner|after-inner]|level=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_flush_is_available_through_string_valued_call() {
    let execution = run_source(
        r#"<?php
$get_flush = "ob_get_flush";
ob_start();
echo "captured";
$result = $get_flush();
echo "|return=" . $result;
echo "|level=" . ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "captured|return=captured|level=0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_contents_peeks_without_closing_active_buffer() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "first";
$peek = ob_get_contents();
echo "|second";
$clean = ob_get_clean();
echo "peek=[" . $peek . "]";
echo "|clean=[" . $clean . "]";
echo "|level=" . ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "peek=[first]|clean=[first|second]|level=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_contents_without_active_buffer_returns_false() {
    let execution = run_source(
        r#"<?php
echo ob_get_contents() === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_length_reports_active_buffer_byte_length() {
    let execution = run_source(
        r#"<?php
echo ob_get_length() === false ? "initial=false" : "initial=true";
ob_start();
echo "abc";
echo "|len=" . ob_get_length();
ob_start();
echo "xy";
echo "|inner-len=" . ob_get_length();
$inner = ob_get_clean();
echo "|after-inner-len=" . ob_get_length();
$outer = ob_get_clean();
echo "outer=[" . $outer . "]";
echo "|inner=[" . $inner . "]";
echo "|final=" . (ob_get_length() === false ? "false" : "true");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial=falseouter=[abc|len=3|after-inner-len=9]|inner=[xy|inner-len=2]|final=false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_list_handlers_reports_default_handlers_for_active_buffers() {
    let execution = run_source(
        r#"<?php
echo count(ob_list_handlers());
ob_start();
$outer = ob_list_handlers();
echo "outer";
ob_start();
$inner = ob_list_handlers();
echo "|inner";
$inner_capture = ob_get_clean();
echo "|after-inner=" . count(ob_list_handlers());
$outer_capture = ob_get_clean();
echo "outer-count=" . count($outer);
echo "|outer-handler=" . $outer[0];
echo "|inner-count=" . count($inner);
echo "|inner-handlers=" . $inner[0] . "," . $inner[1];
echo "|outer-capture=[" . $outer_capture . "]";
echo "|inner-capture=[" . $inner_capture . "]";
echo "|final=" . count(ob_list_handlers());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0outer-count=1|outer-handler=default output handler|inner-count=2|inner-handlers=default output handler,default output handler|outer-capture=[outer|after-inner=1]|inner-capture=[|inner]|final=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_get_status_reports_default_handler_status_for_active_buffers() {
    let execution = run_source(
        r#"<?php
$initial = ob_get_status();
ob_start();
echo "outer";
$outer = ob_get_status();
ob_start();
echo "xy";
$inner = ob_get_status();
$full = ob_get_status(true);
$inner_capture = ob_get_clean();
$after_inner = ob_get_status();
$outer_capture = ob_get_clean();
echo "initial=" . count($initial);
echo "|outer=" . $outer["name"] . ":" . $outer["level"] . ":" . $outer["buffer_used"] . ":" . $outer["chunk_size"] . ":" . $outer["buffer_size"];
echo "|inner=" . $inner["name"] . ":" . $inner["level"] . ":" . $inner["buffer_used"];
echo "|full=" . count($full) . ":" . $full[0]["level"] . ":" . $full[0]["buffer_used"] . ":" . $full[1]["level"] . ":" . $full[1]["buffer_used"];
echo "|after-inner=" . $after_inner["level"] . ":" . $after_inner["buffer_used"];
echo "|captures=" . $outer_capture . "/" . $inner_capture;
echo "|final=" . count(ob_get_status(true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "initial=0|outer=default output handler:0:5:0:16384|inner=default output handler:1:2|full=2:0:5:1:2|after-inner=0:5|captures=outer/xy|final=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_flush_and_ob_end_flush_move_nested_buffer_output_outward() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "outer:";
ob_start();
echo "inner";
$flushed = ob_flush();
echo "|after-inner";
$ended = ob_end_flush();
echo "|after-end";
$outer = ob_get_clean();
echo "outer=[" . $outer . "]";
echo "|flushed=" . ($flushed ? "true" : "false");
echo "|ended=" . ($ended ? "true" : "false");
echo "|level=" . ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "outer=[outer:inner|after-inner|after-end]|flushed=true|ended=true|level=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_clean_and_ob_end_clean_discard_active_buffer_contents() {
    let execution = run_source(
        r#"<?php
ob_start();
echo "keep";
ob_start();
echo "discard";
$cleaned = ob_clean();
echo "inner-after-clean";
$ended = ob_end_clean();
echo "|outer-after-discard";
$outer = ob_get_clean();
echo "outer=[" . $outer . "]";
echo "|cleaned=" . ($cleaned ? "true" : "false");
echo "|ended=" . ($ended ? "true" : "false");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "outer=[keep|outer-after-discard]|cleaned=true|ended=true"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_buffer_flush_and_end_without_active_buffer_return_false() {
    let execution = run_source(
        r#"<?php
echo ob_clean() === false ? "clean=false" : "clean=true";
echo "|";
echo ob_flush() === false ? "flush=false" : "flush=true";
echo "|";
echo ob_get_flush() === false ? "get-flush=false" : "get-flush=true";
echo "|";
echo ob_end_clean() === false ? "end-clean=false" : "end-clean=true";
echo "|";
echo ob_end_flush() === false ? "end-flush=false" : "end-flush=true";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Notice: ob_clean(): Failed to delete buffer. No buffer to delete in Command line code on line 2\n\
clean=false|\n\
Notice: ob_flush(): Failed to flush buffer. No buffer to flush in Command line code on line 4\n\
flush=false|\n\
Notice: ob_get_flush(): Failed to delete and flush buffer. No buffer to delete or flush in Command line code on line 6\n\
get-flush=false|\n\
Notice: ob_end_clean(): Failed to delete buffer. No buffer to delete in Command line code on line 8\n\
end-clean=false|\n\
Notice: ob_end_flush(): Failed to delete and flush buffer. No buffer to delete or flush in Command line code on line 10\n\
end-flush=false"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_buffer_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$start = "ob_start";
$level = "ob_get_level";
$contents = "ob_get_contents";
$length = "ob_get_length";
$handlers = "ob_list_handlers";
$status = "ob_get_status";
$flush = "ob_flush";
$end_flush = "ob_end_flush";
$clean = "ob_get_clean";
echo function_exists($start) ? "yes" : "no";
echo "|";
echo is_callable($level) ? "callable" : "missing";
echo "|";
$start();
echo "captured";
$start();
echo "|inner";
$flush();
echo "|after-flush";
$ended = $end_flush();
echo "|";
echo $level();
echo "|len=" . $length();
echo "|handlers=" . count($handlers());
$active = $status();
echo "|status=" . $active["level"] . ":" . $active["buffer_used"];
$peek = $contents();
$captured = $clean();
echo "clean=";
echo $captured;
echo "|peek=";
echo $peek;
echo "|ended=" . ($ended ? "true" : "false");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|clean=captured|inner|after-flush|1|len=28|handlers=1|status=0:46|peek=captured|inner|after-flush|1|len=28|handlers=1|status=0:46|ended=true"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_output_handler_converts_utf8_to_euc_jp_for_matching_content_types() {
    let execution = run_source(
        r#"<?php
$callable = function_exists("mb_output_handler") && is_callable("mb_output_handler") ? "callable" : "missing";
mb_http_output("EUC-JP");
$current = mb_http_output();

header("Content-Type: text/html");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$html = bin2hex(ob_get_clean());

header("Content-Type: text/plain");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$plain = bin2hex(ob_get_clean());

header("Content-Type: application/xhtml+xml");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$xhtml = bin2hex(ob_get_clean());

header("Content-Type: application/octet-stream");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$octet = bin2hex(ob_get_clean());

ini_set("mbstring.http_output_conv_mimetypes", "html");
header("Content-Type: text/plain");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$plain_filtered = bin2hex(ob_get_clean());

header("Content-Type: application/xhtml+xml");
ob_start();
ob_start("mb_output_handler");
echo "テスト";
ob_end_flush();
$xhtml_filtered = bin2hex(ob_get_clean());

echo $callable, "|", $current, "\n";
echo $html, "\n", $plain, "\n", $xhtml, "\n", $octet, "\n", $plain_filtered, "\n", $xhtml_filtered, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "callable|EUC-JP\n\
a5c6a5b9a5c8\n\
a5c6a5b9a5c8\n\
a5c6a5b9a5c8\n\
e38386e382b9e38388\n\
e38386e382b9e38388\n\
a5c6a5b9a5c8\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_start_closure_handlers_process_flushed_output_without_changing_raw_peeks() {
    let execution = run_source(
        r#"<?php
ob_start(function ($output) {
    return "outer(" . $output . ")";
});
ob_start(function ($output) {
    return "inner[" . $output . "]";
});
echo "body";
$rawInner = ob_get_contents();
$handlers = ob_list_handlers();
$status = ob_get_status();
ob_end_flush();
$rawOuter = ob_get_contents();
ob_end_flush();
$reflection = new ReflectionFunction("ob_start");
echo "|rawInner=" . $rawInner;
echo "|rawOuter=" . $rawOuter;
echo "|handlers=" . $handlers[0] . "," . $handlers[1];
echo "|status=" . $status["name"] . ":" . $status["type"] . ":" . $status["flags"];
echo "|reflection=" . $reflection->getNumberOfRequiredParameters() . "/" . $reflection->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "outer(inner[body])|rawInner=body|rawOuter=inner[body]|handlers=Closure::__invoke,Closure::__invoke|status=Closure::__invoke:1:113|reflection=0/3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_buffer_handlers_discard_produced_output_and_emit_deprecation_on_flush() {
    let execution = run_source(
        r#"<?php
function handler($string) {
    if ($string === "DO ECHO\n") {
        echo "handler-output";
    }
    return $string;
}
ob_start("handler");
echo "DO ECHO\n";
ob_flush();
echo "NO ECHO\n";
ob_flush();
ob_end_clean();
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains(
            "Deprecated: ob_flush(): Producing output from user output handler handler is deprecated"
        ),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("DO ECHO\nNO ECHO\n"));
    assert!(!execution.stdout.contains("handler-output"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn nested_output_buffer_handler_deprecations_flow_through_parent_handlers() {
    let execution = run_source(
        r#"<?php
$log = [];

function first_handler($string) {
    global $log;
    $log[] = "first_handler: <<<" . $string . ">>>";
    echo "first-produced";
    return "FIRST\n";
}

function second_handler($string) {
    global $log;
    $log[] = "second_handler: <<<" . $string . ">>>";
    echo "second-produced";
    return "SECOND\n";
}

function third_handler($string) {
    global $log;
    $log[] = "third_handler: <<<" . $string . ">>>";
    echo "third-produced";
    return "THIRD\n";
}

ob_start("first_handler");
ob_start("second_handler");
ob_start("third_handler");
echo "Testing...";
ob_end_flush();
ob_end_flush();
ob_end_flush();

echo "\nLog:\n";
echo implode("\n", $log);
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.starts_with(
            "Deprecated: ob_end_flush(): Producing output from user output handler first_handler"
        ),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("\nFIRST\n\nLog:\n"));
    assert!(execution.stdout.contains("third_handler: <<<Testing...>>>"));
    assert!(execution.stdout.contains(
        "second_handler: <<<\nDeprecated: ob_end_flush(): Producing output from user output handler third_handler"
    ));
    assert!(execution.stdout.contains(
        "first_handler: <<<\nDeprecated: ob_end_flush(): Producing output from user output handler second_handler"
    ));
    assert!(!execution.stdout.contains("first-produced"));
    assert!(!execution.stdout.contains("second-produced"));
    assert!(!execution.stdout.contains("third-produced"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_handler_deprecation_converted_to_error_exception_preserves_returned_output() {
    let execution = run_source(
        r#"<?php
set_error_handler(function ($errno, $errstr, $errfile, $errline) {
    throw new ErrorException($errstr, 0, $errno, $errfile, $errline);
});

function handler($string) {
    echo "handler-output";
    return "RETURNED\n";
}

ob_start("handler");
echo "BODY";
try {
    ob_end_flush();
} catch (ErrorException $e) {
    echo "caught:", get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "RETURNED\ncaught:ErrorException:ob_end_flush(): Producing output from user output handler handler is deprecated\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_handler_that_throws_is_disabled_after_caught_flush_failure() {
    let execution = run_source(
        r#"<?php
ob_start(function () {
    throw new Exception("ob_start");
});
try {
    ob_flush();
} catch (Throwable $e) {
    echo "caught\n";
}
ob_flush();
echo "done";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "caught\ndone");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_start_chunk_size_runs_handlers_on_threshold_clean_and_final_clean() {
    let execution = run_source(
        r#"<?php
$statuses = array();
function output_buffer_phase_probe($str, $flags) {
    global $statuses;
    $statuses[] = "$flags: $str";
    return $str;
}
ob_start("output_buffer_phase_probe", 3);
echo "yes";
echo "!\n";
ob_flush();
echo "no";
ob_clean();
echo "yes!\n";
echo "no";
ob_end_clean();
print_r($statuses);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes!\nyes!\nArray\n(\n    [0] => 1: yes\n    [1] => 4: !\n\n    [2] => 2: no\n    [3] => 0: yes!\n\n    [4] => 10: no\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_start_chunk_size_cascades_into_parent_threshold() {
    let execution = run_source(
        r#"<?php
function output_buffer_wrap($str, $flags) {
    return "[$str]";
}
ob_start("output_buffer_wrap", 3);
ob_start("output_buffer_wrap", 3);
echo "abc";
echo "d";
while (ob_get_level()) {
    ob_end_flush();
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "[[abc]][[d]][]");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_start_flags_control_status_bits_and_unerasable_buffers() {
    let execution = run_source(
        r#"<?php
ob_start(
    function ($value) { return "handled[" . $value . "]"; },
    0,
    PHP_OUTPUT_HANDLER_STDFLAGS |
    PHP_OUTPUT_HANDLER_STARTED |
    PHP_OUTPUT_HANDLER_DISABLED |
    PHP_OUTPUT_HANDLER_PROCESSED
);
$status = ob_get_status();
echo ($status["flags"] & PHP_OUTPUT_HANDLER_STDFLAGS) . "|";
echo ($status["flags"] & 1) . "|";
echo ($status["flags"] & PHP_OUTPUT_HANDLER_STARTED) . "|";
echo ($status["flags"] & PHP_OUTPUT_HANDLER_DISABLED) . "|";
echo ($status["flags"] & PHP_OUTPUT_HANDLER_PROCESSED);
ob_end_flush();
echo "\n";
ob_start(function ($value) { return "locked[" . $value . "]"; }, 0, false);
echo "payload";
$peek = ob_get_contents();
var_dump($peek);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "handled[112|1|0|0|0]\nlocked[payloadstring(7) \"payload\"\n]"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ob_start_invalid_handlers_warn_and_do_not_create_buffers() {
    let execution = run_source(
        r#"<?php
class C {
    public static function g($string) { return $string; }
    public function h($string) { return $string; }
}
$c = new C();
var_dump(ob_start(1.5));
var_dump(ob_start(array("Missing", "f")));
var_dump(ob_start(array("C", "missing")));
var_dump(ob_start("C::h"));
var_dump(ob_start(array($c)));
var_dump(ob_start(array($c, "missing")));
echo "level=", ob_get_level(), "\n";
var_dump(ob_start(array($c, "h")));
echo "payload";
ob_end_flush();
echo "|level=", ob_get_level();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: ob_start(): no array or string given in Command line code on line 7\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 7\n\
bool(false)\n\
\n\
Warning: ob_start(): class \"Missing\" not found in Command line code on line 8\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 8\n\
bool(false)\n\
\n\
Warning: ob_start(): class C does not have a method \"missing\" in Command line code on line 9\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 9\n\
bool(false)\n\
\n\
Warning: ob_start(): non-static method C::h() cannot be called statically in Command line code on line 10\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 10\n\
bool(false)\n\
\n\
Warning: ob_start(): array callback must have exactly two members in Command line code on line 11\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 11\n\
bool(false)\n\
\n\
Warning: ob_start(): class C does not have a method \"missing\" in Command line code on line 12\n\
\n\
Notice: ob_start(): Failed to create buffer in Command line code on line 12\n\
bool(false)\n\
level=0\n\
bool(true)\n\
payload|level=0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn output_buffer_builtins_reject_forms_outside_current_subset() {
    let start_arg = runtime_error(
        r#"<?php
ob_start(null, 0, PHP_OUTPUT_HANDLER_STDFLAGS, 1);
"#,
    );
    assert_eq!(start_arg.line, 2);
    assert_eq!(start_arg.column, 1);
    assert_eq!(
        start_arg.message,
        "arity mismatch for ob_start(): expected 0 to 3 argument(s), got 4"
    );

    let level_arg = runtime_error(
        r#"<?php
echo ob_get_level(1);
"#,
    );
    assert_eq!(level_arg.line, 2);
    assert_eq!(level_arg.column, 6);
    assert_eq!(
        level_arg.message,
        "arity mismatch for ob_get_level(): expected 0 argument(s), got 1"
    );

    let clean_arg = runtime_error(
        r#"<?php
echo ob_get_clean(1);
"#,
    );
    assert_eq!(clean_arg.line, 2);
    assert_eq!(clean_arg.column, 6);
    assert_eq!(
        clean_arg.message,
        "arity mismatch for ob_get_clean(): expected 0 argument(s), got 1"
    );

    let get_flush_arg = runtime_error(
        r#"<?php
echo ob_get_flush(1);
"#,
    );
    assert_eq!(get_flush_arg.line, 2);
    assert_eq!(get_flush_arg.column, 6);
    assert_eq!(
        get_flush_arg.message,
        "arity mismatch for ob_get_flush(): expected 0 argument(s), got 1"
    );

    let contents_arg = runtime_error(
        r#"<?php
echo ob_get_contents(1);
"#,
    );
    assert_eq!(contents_arg.line, 2);
    assert_eq!(contents_arg.column, 6);
    assert_eq!(
        contents_arg.message,
        "arity mismatch for ob_get_contents(): expected 0 argument(s), got 1"
    );

    let length_arg = runtime_error(
        r#"<?php
echo ob_get_length(1);
"#,
    );
    assert_eq!(length_arg.line, 2);
    assert_eq!(length_arg.column, 6);
    assert_eq!(
        length_arg.message,
        "arity mismatch for ob_get_length(): expected 0 argument(s), got 1"
    );

    let list_handlers_arg = runtime_error(
        r#"<?php
echo ob_list_handlers(1);
"#,
    );
    assert_eq!(list_handlers_arg.line, 2);
    assert_eq!(list_handlers_arg.column, 6);
    assert_eq!(
        list_handlers_arg.message,
        "arity mismatch for ob_list_handlers(): expected 0 argument(s), got 1"
    );

    let status_arg = runtime_error(
        r#"<?php
echo ob_get_status(true, false);
"#,
    );
    assert_eq!(status_arg.line, 2);
    assert_eq!(status_arg.column, 6);
    assert_eq!(
        status_arg.message,
        "arity mismatch for ob_get_status(): expected 0 to 1 argument(s), got 2"
    );

    let status_type = runtime_error(
        r#"<?php
echo ob_get_status(1);
"#,
    );
    assert_eq!(status_type.line, 2);
    assert_eq!(status_type.column, 6);
    assert_eq!(
        status_type.message,
        "unsupported call ob_get_status(): full_status argument must be bool in the current subset, got int"
    );

    let clean_buffer_arg = runtime_error(
        r#"<?php
echo ob_clean(1);
"#,
    );
    assert_eq!(clean_buffer_arg.line, 2);
    assert_eq!(clean_buffer_arg.column, 6);
    assert_eq!(
        clean_buffer_arg.message,
        "arity mismatch for ob_clean(): expected 0 argument(s), got 1"
    );

    let flush_arg = runtime_error(
        r#"<?php
echo ob_flush(1);
"#,
    );
    assert_eq!(flush_arg.line, 2);
    assert_eq!(flush_arg.column, 6);
    assert_eq!(
        flush_arg.message,
        "arity mismatch for ob_flush(): expected 0 argument(s), got 1"
    );

    let end_clean_arg = runtime_error(
        r#"<?php
echo ob_end_clean(1);
"#,
    );
    assert_eq!(end_clean_arg.line, 2);
    assert_eq!(end_clean_arg.column, 6);
    assert_eq!(
        end_clean_arg.message,
        "arity mismatch for ob_end_clean(): expected 0 argument(s), got 1"
    );

    let end_flush_arg = runtime_error(
        r#"<?php
echo ob_end_flush(1);
"#,
    );
    assert_eq!(end_flush_arg.line, 2);
    assert_eq!(end_flush_arg.column, 6);
    assert_eq!(
        end_flush_arg.message,
        "arity mismatch for ob_end_flush(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_routes_output_buffer_builtins_through_native_runtime_abi() {
    let ir = emit_ir_source(
        r#"<?php
ob_start();
echo ob_get_length();
ob_list_handlers();
ob_get_status();
echo ob_get_flush();
"#,
    )
    .unwrap();

    assert!(
        ir.contains(
            "declare %phpc.NativeValueHandle @phpc_native_output_buffer_operation_with_diagnostic"
        ),
        "{ir}"
    );
    assert!(
        ir.matches("@phpc_native_output_buffer_operation_with_diagnostic")
            .count()
            >= 5,
        "{ir}"
    );
    assert!(!ir.contains("output-buffer lowering rejects"), "{ir}");
}

#[test]
fn emit_asm_routes_output_buffer_builtins_through_native_runtime_abi() {
    let asm = emit_asm_source("<?php\nob_start();\necho ob_get_clean();\n").unwrap();

    assert!(
        asm.contains("phpc_native_output_buffer_operation_with_diagnostic"),
        "{asm}"
    );
    assert!(!asm.contains("output-buffer lowering rejects"), "{asm}");
}

#[test]
fn emit_ir_includes_output_buffer_builtins_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("ob_start") ? "1" : "0";
echo is_callable("ob_start") ? "1" : "0";
echo function_exists("ob_get_level") ? "1" : "0";
echo is_callable("ob_get_level") ? "1" : "0";
echo function_exists("ob_get_contents") ? "1" : "0";
echo is_callable("ob_get_contents") ? "1" : "0";
echo function_exists("ob_get_length") ? "1" : "0";
echo is_callable("ob_get_length") ? "1" : "0";
echo function_exists("ob_list_handlers") ? "1" : "0";
echo is_callable("ob_list_handlers") ? "1" : "0";
echo function_exists("ob_get_status") ? "1" : "0";
echo is_callable("ob_get_status") ? "1" : "0";
echo function_exists("ob_get_clean") ? "1" : "0";
echo is_callable("ob_get_clean") ? "1" : "0";
echo function_exists("ob_get_flush") ? "1" : "0";
echo is_callable("ob_get_flush") ? "1" : "0";
echo function_exists("ob_clean") ? "1" : "0";
echo is_callable("ob_clean") ? "1" : "0";
echo function_exists("ob_flush") ? "1" : "0";
echo is_callable("ob_flush") ? "1" : "0";
echo function_exists("ob_end_clean") ? "1" : "0";
echo is_callable("ob_end_clean") ? "1" : "0";
echo function_exists("ob_end_flush") ? "1" : "0";
echo is_callable("ob_end_flush") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 24, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
