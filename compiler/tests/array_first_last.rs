use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_first_and_last_return_inserted_values_or_null() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

var_dump(array_first($items));
var_dump(array_last($items));
var_dump(array_first([]));
var_dump(array_last([]));

$first = "array_first";
$last = "array_last";
var_dump($first($items));
var_dump($last($items));
echo function_exists("array_first") ? "first-exists\n" : "first-missing\n";
echo is_callable("array_last") ? "last-callable\n" : "last-not-callable\n";

$reflection = new ReflectionFunction("array_first");
echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "\n";

$str = "hello";
$refs = [&$str, 1];
echo json_encode($refs), "|", json_encode(array_first($refs)), "|", json_encode(array_last($refs));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "string(3) \"Ada\"\nstring(4) \"next\"\nNULL\nNULL\nstring(3) \"Ada\"\nstring(4) \"next\"\nfirst-exists\nlast-callable\n1/1\n[\"hello\",1]|\"hello\"|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_first_and_last_require_array_argument() {
    let first_error = runtime_error("<?php\necho array_first(42);\n");

    assert_eq!(first_error.line, 2);
    assert_eq!(first_error.column, 6);
    assert_eq!(
        first_error.message,
        "unsupported call array_first(): argument must be array, got int"
    );

    let last_error = runtime_error("<?php\necho array_last(\"value\");\n");

    assert_eq!(last_error.line, 2);
    assert_eq!(last_error.column, 6);
    assert_eq!(
        last_error.message,
        "unsupported call array_last(): argument must be array, got string"
    );
}

#[test]
fn emit_ir_rejects_array_first_and_last_until_native_call_lowering_exists() {
    let first_error = emit_ir_source("<?php\necho array_first([1]);\n").unwrap_err();
    assert_eq!(first_error.phase, Phase::Codegen);
    assert!(
        first_error.message.contains("function calls"),
        "{}",
        first_error.message
    );

    let last_error = emit_ir_source("<?php\necho array_last([1]);\n").unwrap_err();
    assert_eq!(last_error.phase, Phase::Codegen);
    assert!(
        last_error.message.contains("function calls"),
        "{}",
        last_error.message
    );
}
