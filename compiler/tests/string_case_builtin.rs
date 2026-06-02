use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strtolower_executes_current_ascii_string_subset() {
    let execution = run_source(
        r#"<?php
echo strtolower("Memory_Limit"), "|";
echo strtolower("128M"), "|";
echo strtolower(false), "|";
echo strtolower(42);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "memory_limit|128m||42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtolower_lowercases_ascii_bytes_and_preserves_non_utf8_bytes() {
    let execution = run_source(
        r#"<?php
$value = chr(65) . chr(128) . chr(90);
echo bin2hex(strtolower($value)), "|";
$byte = chr(128);
echo bin2hex(strtolower("$byte"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "61807a|80");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtoupper_executes_current_ascii_string_subset() {
    let execution = run_source(
        r#"<?php
echo strtoupper("Memory_Limit"), "|";
echo strtoupper("128m"), "|";
echo strtoupper(false), "|";
echo strtoupper(42);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "MEMORY_LIMIT|128M||42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtoupper_uppercases_ascii_bytes_and_preserves_non_utf8_bytes() {
    let execution = run_source(
        r#"<?php
$value = chr(97) . chr(128) . chr(122);
echo bin2hex(strtoupper($value)), "|";
$byte = chr(128);
echo bin2hex(strtoupper("$byte"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "41805a|80");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtolower_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strtolower";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("ABC");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|abc");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtoupper_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strtoupper";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("abc");
echo "|";
$reflection = new ReflectionFunction($call);
echo $reflection->getName(), "|", $reflection->invoke("mixed");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|ABC|strtoupper|MIXED");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ascii_case_helpers_use_php_string_argument_boundary() {
    let execution = run_source(
        r#"<?php
class Word {
    public function __toString() {
        return "MiXeD";
    }
}

set_error_handler(function($_, $message) {
    echo $message, "|";
    return true;
});
echo strtolower(null), "|";
restore_error_handler();
echo strtolower(new Word), "|";
$call = "strtoupper";
echo $call(new Word), "|";
try {
    strtolower([]);
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    strtoupper(new stdClass);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "strtolower(): Passing null to parameter #1 ($string) of type string is deprecated||mixed|MIXED|strtolower(): Argument #1 ($string) must be of type string, array given|strtoupper(): Argument #1 ($string) must be of type string, stdClass given"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtolower_rejects_arity_mismatches() {
    let too_many = run_source("<?php\nstrtolower('ABC', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for strtolower(): expected 1 argument(s), got 2"
    );
}

#[test]
fn strtoupper_rejects_arity_mismatches() {
    let too_many = run_source("<?php\nstrtoupper('abc', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for strtoupper(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_string_case_metadata_and_routes_direct_case_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strtolower") ? "1" : "0";
echo is_callable("strtolower") ? "1" : "0";
echo function_exists("strtoupper") ? "1" : "0";
echo is_callable("strtoupper") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let direct_ir = emit_ir_source("<?php\necho strtolower('ABC');\n").unwrap();
    assert!(
        direct_ir.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{direct_ir}"
    );
    assert!(direct_ir.contains("i8 48, ptr %"), "{direct_ir}");
    assert!(
        direct_ir.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "{direct_ir}"
    );
    assert!(
        !direct_ir.contains("LLVM function-call lowering rejects"),
        "{direct_ir}"
    );

    let direct_ir = emit_ir_source("<?php\necho strtoupper('abc');\n").unwrap();
    assert!(
        direct_ir.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{direct_ir}"
    );
    assert!(direct_ir.contains("i8 49, ptr %"), "{direct_ir}");
    assert!(
        direct_ir.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "{direct_ir}"
    );
    assert!(
        !direct_ir.contains("LLVM function-call lowering rejects"),
        "{direct_ir}"
    );
}
