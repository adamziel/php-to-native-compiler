use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strtolower_executes_current_ascii_string_subset() {
    let execution = run_source(
        r#"<?php
echo strtolower("Memory_Limit"), "|";
echo strtolower("128M"), "|";
echo strtolower(null), "|";
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
fn strtolower_rejects_forms_outside_current_subset() {
    let array_arg = run_source("<?php\nstrtolower(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call strtolower(): arrays are not supported"
    );

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
fn emit_ir_folds_strtolower_metadata_and_routes_direct_case_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strtolower") ? "1" : "0";
echo is_callable("strtolower") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let direct_ir = emit_ir_source("<?php\necho strtolower('ABC');\n").unwrap();
    assert!(
        direct_ir.contains("phpc_native_value_string_result_operation_with_diagnostic"),
        "{direct_ir}"
    );
    assert!(direct_ir.contains("i8 48, ptr %"), "{direct_ir}");
    assert!(
        direct_ir.contains("phpc_native_value_format_stdout_with_diagnostic"),
        "{direct_ir}"
    );
    assert!(
        !direct_ir.contains("LLVM function-call lowering rejects"),
        "{direct_ir}"
    );
}
