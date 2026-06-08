use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_first_and_array_last_return_values_or_null() {
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

$empty = [];
var_dump(array_first($empty));
var_dump(array_last($empty));

$call_first = "array_first";
$call_last = "array_last";
var_dump($call_first($items));
var_dump($call_last($items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "string(3) \"Ada\"\n",
            "string(4) \"next\"\n",
            "NULL\n",
            "NULL\n",
            "string(3) \"Ada\"\n",
            "string(4) \"next\"\n"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_first_and_array_last_work_with_reference_backed_slots() {
    let source = r#"<?php
$str = "hello world";
$left = [&$str, 1];
$right = [1, &$str];

echo json_encode($left), "\n";
echo json_encode($right), "\n";
var_dump(array_first($left));
var_dump(array_last($right));
var_dump(array_first($left) === $left[array_key_first($left)]);
var_dump(array_last($right) === $right[array_key_last($right)]);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "[\"hello world\",1]\n",
            "[1,\"hello world\"]\n",
            "string(11) \"hello world\"\n",
            "string(11) \"hello world\"\n",
            "bool(true)\n",
            "bool(true)\n"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_first_and_array_last_preserve_nested_values() {
    let source = r#"<?php
$items = [];
$items[] = ["first" => 1];
$items[] = ["last" => 2];

var_dump(array_first($items));
var_dump(array_last($items));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        concat!(
            "array(1) {\n",
            "  [\"first\"]=>\n",
            "  int(1)\n",
            "}\n",
            "array(1) {\n",
            "  [\"last\"]=>\n",
            "  int(2)\n",
            "}\n"
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_first_requires_array_argument() {
    let error = runtime_error("<?php\necho array_first(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_first(): argument must be array, got int"
    );
}

#[test]
fn array_last_requires_array_argument() {
    let error = runtime_error("<?php\necho array_last(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_last(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_array_first_last_until_native_call_lowering_exists() {
    let first_error = emit_ir_source("<?php\necho array_first([1]);\n").unwrap_err();
    let last_error = emit_ir_source("<?php\necho array_last([1]);\n").unwrap_err();

    assert_eq!(first_error.phase, Phase::Codegen);
    assert!(
        first_error.message.contains("function calls"),
        "{}",
        first_error.message
    );
    assert_eq!(last_error.phase, Phase::Codegen);
    assert!(
        last_error.message.contains("function calls"),
        "{}",
        last_error.message
    );
}
