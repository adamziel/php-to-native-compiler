use php_compiler::error::Phase;
use php_compiler::run_source;
use php_compiler::{emit_asm_source, emit_ir_source};

const LLVM_OUTPUT_BUFFER_REJECTION: &str = "LLVM output-buffer lowering rejects ob_start(), ob_get_level(), ob_get_contents(), and ob_get_clean() until native stdout capture buffers, shutdown flushing, output-started tracking, SAPI interaction, references/copy-on-write, and exact native diagnostics exist; phpc run handles current bounded output-buffer behavior";

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
fn output_buffer_builtins_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$start = "ob_start";
$level = "ob_get_level";
$contents = "ob_get_contents";
$clean = "ob_get_clean";
echo function_exists($start) ? "yes" : "no";
echo "|";
echo is_callable($level) ? "callable" : "missing";
echo "|";
$start();
echo "captured";
echo "|";
echo $level();
$peek = $contents();
$captured = $clean();
echo "clean=";
echo $captured;
echo "|peek=";
echo $peek;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|clean=captured|1|peek=captured|1"
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
}

#[test]
fn emit_ir_rejects_output_buffer_builtins_until_native_state_exists() {
    let error = emit_ir_source("<?php\nob_start();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_OUTPUT_BUFFER_REJECTION);
}

#[test]
fn emit_asm_rejects_output_buffer_builtins_until_native_state_exists() {
    let error = emit_asm_source("<?php\necho ob_get_clean();\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_OUTPUT_BUFFER_REJECTION);
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
echo function_exists("ob_get_clean") ? "1" : "0";
echo is_callable("ob_get_clean") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 8, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
