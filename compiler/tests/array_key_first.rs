use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_key_first_returns_first_integer_or_string_key_or_null() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

var_dump(array_key_first($items));

$int_first = [];
$int_first["2"] = "two";
$int_first["02"] = "zero two";
$int_first["name"] = "Ada";
var_dump(array_key_first($int_first));

$empty = [];
var_dump(array_key_first($empty));

$call = "array_key_first";
var_dump($call($items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(4) \"name\"\nint(2)\nNULL\nstring(4) \"name\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_key_first_requires_array_argument() {
    let error = runtime_error("<?php\necho array_key_first(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_key_first(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_array_key_first_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_key_first([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
