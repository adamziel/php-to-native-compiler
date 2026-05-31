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
fn output_buffer_builtins_reject_forms_outside_current_subset() {
    let start_arg = runtime_error(
        r#"<?php
ob_start("handler");
"#,
    );
    assert_eq!(start_arg.line, 2);
    assert_eq!(start_arg.column, 1);
    assert_eq!(
        start_arg.message,
        "arity mismatch for ob_start(): expected 0 argument(s), got 1"
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
