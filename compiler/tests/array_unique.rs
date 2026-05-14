use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_unique_preserves_first_entries_by_scalar_string_form() {
    let source = r#"<?php
$items = [];
$items[5] = "five";
$items[9] = "five";
$items[2] = "two";
$items["null"] = null;
$items["false"] = false;
$items["empty"] = "";
$items["true"] = true;
$items["one"] = 1;
$items["string-one"] = "1";
$items["int-ten"] = 10;
$items["float-ten"] = 10.0;
$items["string-ten-float"] = "10.0";
$items["text"] = "abc";
$items["dup-text"] = "abc";
$items[] = "next";

$unique = array_unique($items);
print_r($unique);
echo count($unique), "\n";
echo $unique[5], "|", $unique[2], "|", $unique["true"], "|", $unique["int-ten"], "|", $unique["string-ten-float"], "|", $unique[10], "\n";
$unique[] = "after";
echo $unique[11], "\n";
print_r($items);

$call = "array_unique";
$again = $call($items);
echo $again[5], "|", $again[2], "|", $again["true"], "|", $again["int-ten"], "|", $again["string-ten-float"], "|", $again[10], "\n";

$empty = array_unique([]);
print_r($empty);
echo count($empty);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [5] => five\n    [2] => two\n    [null] => \n    [true] => 1\n    [int-ten] => 10\n    [string-ten-float] => 10.0\n    [text] => abc\n    [10] => next\n)\n8\nfive|two|1|10|10.0|next\nafter\nArray\n(\n    [5] => five\n    [9] => five\n    [2] => two\n    [null] => \n    [false] => \n    [empty] => \n    [true] => 1\n    [one] => 1\n    [string-one] => 1\n    [int-ten] => 10\n    [float-ten] => 10\n    [string-ten-float] => 10.0\n    [text] => abc\n    [dup-text] => abc\n    [10] => next\n)\nfive|two|1|10|10.0|next\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unique_accepts_sort_string_flag() {
    let source = r#"<?php
$items = ["first" => "10", "second" => 10, "third" => "10.0", "fourth" => false, "fifth" => ""];
$unique = array_unique($items, SORT_STRING);
print_r($unique);
echo $unique["first"], "|", $unique["third"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_STRING"));
echo $again["first"], "|", defined("SORT_STRING"), "|", SORT_STRING;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [first] => 10\n    [third] => 10.0\n    [fourth] => \n)\n10|10.0|3\n10|1|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unique_accepts_sort_regular_flag_for_scalar_values() {
    let source = r#"<?php
$items = [
    "s10" => "10",
    "i10" => 10,
    "f10" => 10.0,
    "s10f" => "10.0",
    "true" => true,
    "one" => 1,
    "false" => false,
    "empty" => "",
    "null" => null,
    "zero" => 0,
    "s0" => "0",
    "text" => "abc",
    "dup-text" => "abc",
];

$unique = array_unique($items, SORT_REGULAR);
print_r($unique);
echo $unique["s10"], "|", $unique["one"], "|", $unique["false"], "|", $unique["text"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_REGULAR"));
echo $again["s10"], "|", $again["one"], "|", defined("SORT_REGULAR"), "|", SORT_REGULAR;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [s10] => 10\n    [one] => 1\n    [false] => \n    [text] => abc\n)\n10|1||abc|4\n10|1|1|0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unique_accepts_sort_numeric_flag_for_numeric_values() {
    let source = r#"<?php
$items = [
    "first" => "10",
    "second" => 10,
    "third" => "10.0",
    "fourth" => 10.5,
    "fifth" => "010.50",
    "sixth" => 11,
    "seventh" => "11.0",
    "eighth" => 0,
    "ninth" => false,
    "tenth" => null,
];

$unique = array_unique($items, SORT_NUMERIC);
print_r($unique);
echo $unique["first"], "|", $unique["fourth"], "|", $unique["sixth"], "|", $unique["eighth"], "|", count($unique), "\n";

$call = "array_unique";
$again = $call($items, constant("SORT_NUMERIC"));
echo $again["first"], "|", defined("SORT_NUMERIC"), "|", SORT_NUMERIC;
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [first] => 10\n    [fourth] => 10.5\n    [sixth] => 11\n    [eighth] => 0\n)\n10|10.5|11|0|4\n10|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unique_requires_array_argument() {
    let error = runtime_error("<?php\necho array_unique(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_unique(): argument must be array, got int"
    );
}

#[test]
fn array_unique_rejects_non_scalar_value_comparisons() {
    let array_error = runtime_error("<?php\n$items = [[]];\necho array_unique($items);\n");

    assert_eq!(array_error.line, 3);
    assert_eq!(array_error.column, 6);
    assert_eq!(
        array_error.message,
        "unsupported call array_unique(): values must be scalar in the current subset, got array"
    );

    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$items = [$box];
echo array_unique($items);
"#,
    );

    assert_eq!(object_error.line, 5);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call array_unique(): values must be scalar in the current subset, got object"
    );
}

#[test]
fn array_unique_sort_numeric_rejects_values_outside_current_numeric_subset() {
    let string_error = runtime_error(
        "<?php\n$items = [\"10\", \"not numeric\"];\necho array_unique($items, SORT_NUMERIC);\n",
    );

    assert_eq!(string_error.line, 3);
    assert_eq!(string_error.column, 6);
    assert_eq!(
        string_error.message,
        "unsupported call array_unique(): values must be numeric in the current subset, got non-numeric string"
    );

    let array_error =
        runtime_error("<?php\n$items = [10, []];\necho array_unique($items, SORT_NUMERIC);\n");

    assert_eq!(array_error.line, 3);
    assert_eq!(array_error.column, 6);
    assert_eq!(
        array_error.message,
        "unsupported call array_unique(): values must be numeric scalar in the current subset, got array"
    );
}

#[test]
fn array_unique_rejects_sort_flags_outside_supported_subset() {
    let error = runtime_error("<?php\n$items = [\"a\"];\necho array_unique($items, 3);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_unique(): sort flags other than SORT_REGULAR, SORT_NUMERIC, or SORT_STRING are not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_array_unique_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_unique([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
