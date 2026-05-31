use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_slice_from_offset_reindexes_integer_keys_and_preserves_string_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$slice = array_slice($items, 2);
echo count($slice), "\n";
echo $slice[0], "|", $slice["02"], "|", $slice[1], "|", $slice[2], "\n";
$slice[] = "after";
echo $slice[3], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$tail = array_slice($items, -3);
echo count($tail), "|", $tail["02"], "|", $tail[0], "|", $tail[1], "\n";

$empty = array_slice($items, 99);
echo count($empty), "\n";

$whole = array_slice($items, -99);
echo count($whole), "|", $whole["name"], "|", $whole[0], "|", $whole[1], "|", $whole["02"], "|", $whole[2], "|", $whole[3], "\n";

$call = "array_slice";
$again = $call($items, 1);
echo count($again), "|", $again[0], "|", $again["02"], "|", $again[3];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "4\ntwo|zero two|negative|next\nafter\nAda|five|two|zero two|negative|next\n3|zero two|negative|next\n0\n6|Ada|five|two|zero two|negative|next\n5|five|zero two|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_supports_integer_length_argument() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$middle = array_slice($items, 1, 3);
echo count($middle), "|", $middle[0], "|", $middle[1], "|", $middle["02"], "\n";

$zero = array_slice($items, 1, 0);
echo count($zero), "\n";

$without_tail = array_slice($items, 1, -2);
echo count($without_tail), "|", $without_tail[0], "|", $without_tail[1], "|", $without_tail["02"], "\n";

$empty = array_slice($items, 4, -3);
echo count($empty), "\n";

$negative_offset = array_slice($items, -4, 2);
echo count($negative_offset), "|", $negative_offset[0], "|", $negative_offset["02"], "\n";

$call = "array_slice";
$dynamic = $call($items, 0, 2);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[0], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3|five|two|zero two\n0\n3|five|two|zero two\n0\n2|two|zero two\n2|Ada|five\nAda|five|two|zero two|negative|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_supports_null_length_argument() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$tail = array_slice($items, 1, null);
echo count($tail), "|", $tail[0], "|", $tail[1], "|", $tail["02"], "|", $tail[2], "|", $tail[3], "\n";

$negative = array_slice($items, -3, null);
echo count($negative), "|", $negative["02"], "|", $negative[0], "|", $negative[1], "\n";

$empty = array_slice($items, 99, null);
echo count($empty), "\n";

$call = "array_slice";
$dynamic = $call($items, 0, null);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[0], "|", $dynamic[1], "|", $dynamic["02"], "|", $dynamic[2], "|", $dynamic[3], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "5|five|two|zero two|negative|next\n3|zero two|negative|next\n0\n6|Ada|five|two|zero two|negative|next\nAda|five|two|zero two|negative|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_supports_boolean_preserve_keys_argument() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$preserved = array_slice($items, 1, 3, true);
echo count($preserved), "|", $preserved[5], "|", $preserved[2], "|", $preserved["02"], "\n";
$preserved[] = "after";
echo $preserved[6], "\n";

$default_false = array_slice($items, 1, 3, false);
echo count($default_false), "|", $default_false[0], "|", $default_false[1], "|", $default_false["02"], "\n";

$int_preserved = array_slice($items, 1, 3, 1);
echo count($int_preserved), "|", $int_preserved[5], "|", $int_preserved[2], "|", $int_preserved["02"], "\n";

$int_false = array_slice($items, 1, 3, 0);
echo count($int_false), "|", $int_false[0], "|", $int_false[1], "|", $int_false["02"], "\n";

$string_preserved = array_slice($items, 1, 3, "1");
echo count($string_preserved), "|", $string_preserved[5], "|", $string_preserved[2], "|", $string_preserved["02"], "\n";

$tail = array_slice($items, -3, null, true);
echo count($tail), "|", $tail["02"], "|", $tail[-1], "|", $tail[6], "\n";

$call = "array_slice";
$dynamic = $call($items, 0, null, true);
echo count($dynamic), "|", $dynamic["name"], "|", $dynamic[5], "|", $dynamic[2], "|", $dynamic["02"], "|", $dynamic[-1], "|", $dynamic[6], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3|five|two|zero two\nafter\n3|five|two|zero two\n3|five|two|zero two\n3|five|two|zero two\n3|five|two|zero two\n3|zero two|negative|next\n6|Ada|five|two|zero two|negative|next\nAda|five|two|zero two|negative|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_preserves_reference_backed_value_slots() {
    let source = r#"<?php
$first = "one";
$second = "two";
$third = "three";
$items = [3 => &$first, 2 => &$second, 1 => &$third];
$slice = array_slice($items, 1, 2);
var_dump($slice);
$second = "changed";
var_dump(array_slice($items, 1, 2, true));
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "array(2) {\n  [0]=>\n  &string(3) \"two\"\n  [1]=>\n  &string(5) \"three\"\n}\narray(2) {\n  [2]=>\n  &string(7) \"changed\"\n  [1]=>\n  &string(5) \"three\"\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_requires_array_first_argument() {
    let error = runtime_error("<?php\necho array_slice(42, 0);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_slice(): first argument must be array, got int"
    );
}

#[test]
fn array_slice_requires_int_offset_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_slice($items, \"0\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_slice(): offset argument must be int in the current subset, got string"
    );
}

#[test]
fn array_slice_coerces_weak_length_and_reports_nullable_int_type_errors() {
    let source = r#"<?php
$items = [1, 2, 3, 4];

try {
    array_slice($items, 0, "foo");
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

$float_length = array_slice($items, 0, 2.9);
echo count($float_length), "|", $float_length[0], "|", $float_length[1], "\n";

$string_length = array_slice($items, 0, "3");
echo count($string_length), "|", $string_length[0], "|", $string_length[1], "|", $string_length[2], "\n";

try {
    array_slice($items, 0, PHP_INT_MAX * 1.0);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_slice($items, 0, "9223372036854775808");
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_slice(): Argument #3 ($length) must be of type ?int, string given\n2|1|2\n3|1|2|3\narray_slice(): Argument #3 ($length) must be of type ?int, float given\narray_slice(): Argument #3 ($length) must be of type ?int, string given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_slice_reports_bool_type_error_for_invalid_preserve_keys_argument() {
    let source = r#"<?php
$items = [1];

try {
    array_slice($items, 0, 1, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_slice(): Argument #4 ($preserve_keys) must be of type bool, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_slice_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_slice([1], 0);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
