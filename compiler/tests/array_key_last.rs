use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_key_last_returns_last_integer_or_string_key_or_null() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

var_dump(array_key_last($items));

$string_last = [];
$string_last["name"] = "Ada";
$string_last[5] = "five";
$string_last["2"] = "two";
$string_last["02"] = "zero two";
$string_last["2"] = "two updated";
var_dump(array_key_last($string_last));

$int_last = [];
$int_last["name"] = "Ada";
$int_last["02"] = "zero two";
$int_last["2"] = "two";
var_dump(array_key_last($int_last));

$empty = [];
var_dump(array_key_last($empty));

$call = "array_key_last";
var_dump($call($items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "int(6)\nstring(2) \"02\"\nint(2)\nNULL\nint(6)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_last_requires_array_argument() {
    let error = runtime_error("<?php\necho array_key_last(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_key_last(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_array_key_last_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_key_last([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
