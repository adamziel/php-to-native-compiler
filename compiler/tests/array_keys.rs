use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_keys_emits_integer_and_string_keys_in_order() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$keys = array_keys($items);
echo count($keys), "\n";
echo $keys[0], "|", $keys[1], "|", $keys[2], "|", $keys[3], "|", $keys[4], "|", $keys[5], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$call = "array_keys";
$again = $call($items);
echo $again[0], "|", $again[5];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "6\nname|5|2|02|-1|6\nAda|five|two updated|zero two|negative|next\nname|6"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_requires_array_argument() {
    let error = runtime_error("<?php\necho array_keys(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_keys(): argument must be array, got int"
    );
}

#[test]
fn array_keys_filters_values_with_loose_scalar_comparison() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["empty"] = "";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["numeric-string"] = "10.0";
$items["text"] = "abc";

$empty = array_keys($items, "");
echo count($empty), "\n";
echo $empty[0], "|", $empty[1], "|", $empty[2], "\n";

$zero = array_keys($items, "0");
echo count($zero), "\n";
echo $zero[0], "|", $zero[1], "|", $zero[2], "\n";

$ten = array_keys($items, "10");
echo count($ten), "\n";
echo $ten[0], "|", $ten[1], "|", $ten[2], "\n";

$text = array_keys($items, "abc");
echo count($text), "\n";
echo $text[0], "\n";

$missing = array_keys($items, "missing");
echo count($missing), "\n";

$call = "array_keys";
$dynamic = $call($items, "10.0");
echo $dynamic[0], "|", $dynamic[1], "|", $dynamic[2];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3\nnull|false|empty\n3\nfalse|int-zero|string-zero\n3\nint-ten|string-ten|numeric-string\n1\ntext\n0\nint-ten|string-ten|numeric-string"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_keys_rejects_array_and_object_search_gaps() {
    let array_search_value_error =
        runtime_error("<?php\n$items = [1];\necho array_keys($items, []);\n");

    assert_eq!(array_search_value_error.line, 3);
    assert_eq!(array_search_value_error.column, 6);
    assert_eq!(
        array_search_value_error.message,
        "unsupported call array_keys(): array search values and array values are not implemented"
    );

    let array_value_error =
        runtime_error("<?php\n$items = [[]];\necho array_keys($items, \"needle\");\n");

    assert_eq!(array_value_error.line, 3);
    assert_eq!(array_value_error.column, 6);
    assert_eq!(
        array_value_error.message,
        "unsupported call array_keys(): array search values and array values are not implemented"
    );

    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$items = [$box];
echo array_keys($items, "needle");
"#,
    );

    assert_eq!(object_error.line, 5);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call array_keys(): object search values and object values are not implemented"
    );
}

#[test]
fn emit_ir_rejects_array_keys_filter_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_keys([1], 1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
