use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_chunk_splits_values_into_reindexed_chunks() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[] = "next";

$chunks = array_chunk($items, 2);
echo count($chunks), "|", count($chunks[0]), "|", count($chunks[1]), "|", count($chunks[2]), "\n";
echo $chunks[0][0], "|", $chunks[0][1], "|", $chunks[1][0], "|", $chunks[1][1], "|", $chunks[2][0], "\n";
if (array_key_exists("02", $chunks[1])) {
    echo "string-key-kept\n";
} else {
    echo "string-key-reindexed\n";
}
$second = $chunks[1];
$second[] = "after";
echo $second[2], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[6], "\n";

$one = array_chunk($items, 99);
echo count($one), "|", count($one[0]), "|", $one[0][4], "\n";

$empty = array_chunk([], 2);
echo count($empty), "\n";

$call = "array_chunk";
$again = $call($items, 3);
echo count($again), "|", $again[0][0], "|", $again[0][2], "|", $again[1][0], "|", $again[1][1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3|2|2|1\nAda|five|two|zero two|next\nstring-key-reindexed\nafter\nAda|five|two|zero two|next\n1|5|next\n0\n2|Ada|two|zero two|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_chunk_supports_boolean_preserve_keys_argument() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$preserved = array_chunk($items, 2, true);
echo count($preserved[0]), "|", $preserved[0]["name"], "|", $preserved[0][5], "\n";
if (array_key_exists(0, $preserved[0])) {
    echo "first-reindexed\n";
} else {
    echo "first-preserved\n";
}
$first = $preserved[0];
$first[] = "after";
echo $first[6], "\n";

$second = $preserved[1];
echo $second[2], "|", $second["02"], "\n";
$second[] = "after-second";
echo $second[3], "\n";

$third = $preserved[2];
echo $third[-1], "|", $third[6], "\n";
$third[] = "after-third";
echo $third[7], "\n";

$default_false = array_chunk($items, 2, false);
echo $default_false[0][0], "|", $default_false[0][1], "\n";
if (array_key_exists("name", $default_false[0])) {
    echo "default-false-preserved\n";
} else {
    echo "default-false-reindexed\n";
}

$call = "array_chunk";
$dynamic = $call($items, 3, true);
echo count($dynamic), "|", $dynamic[0]["name"], "|", $dynamic[0][5], "|", $dynamic[0][2], "|", $dynamic[1]["02"], "|", $dynamic[1][-1], "|", $dynamic[1][6], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "2|Ada|five\nfirst-preserved\nafter\ntwo|zero two\nafter-second\nnegative|next\nafter-third\nAda|five\ndefault-false-reindexed\n2|Ada|five|two|zero two|negative|next\nAda|five|two|zero two|negative|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_chunk_preserves_reference_backed_value_slots() {
    let source = r#"<?php
$first = "one";
$second = "two";
$third = "three";
$items = [3 => &$first, "name" => "plain", 2 => &$second, 1 => &$third];

$chunks = array_chunk($items, 2);
var_dump($chunks);
$second = "changed";
var_dump($chunks);

$preserved = array_chunk($items, 2, true);
var_dump($preserved);
$third = "later";
var_dump($preserved);
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "array(2) {\n  [0]=>\n  array(2) {\n    [0]=>\n    &string(3) \"one\"\n    [1]=>\n    string(5) \"plain\"\n  }\n  [1]=>\n  array(2) {\n    [0]=>\n    &string(3) \"two\"\n    [1]=>\n    &string(5) \"three\"\n  }\n}\narray(2) {\n  [0]=>\n  array(2) {\n    [0]=>\n    &string(3) \"one\"\n    [1]=>\n    string(5) \"plain\"\n  }\n  [1]=>\n  array(2) {\n    [0]=>\n    &string(7) \"changed\"\n    [1]=>\n    &string(5) \"three\"\n  }\n}\narray(2) {\n  [0]=>\n  array(2) {\n    [3]=>\n    &string(3) \"one\"\n    [\"name\"]=>\n    string(5) \"plain\"\n  }\n  [1]=>\n  array(2) {\n    [2]=>\n    &string(7) \"changed\"\n    [1]=>\n    &string(5) \"three\"\n  }\n}\narray(2) {\n  [0]=>\n  array(2) {\n    [3]=>\n    &string(3) \"one\"\n    [\"name\"]=>\n    string(5) \"plain\"\n  }\n  [1]=>\n  array(2) {\n    [2]=>\n    &string(7) \"changed\"\n    [1]=>\n    &string(5) \"later\"\n  }\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_chunk_requires_array_first_argument() {
    let error = runtime_error("<?php\necho array_chunk(42, 2);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): first argument must be array, got int"
    );
}

#[test]
fn array_chunk_requires_int_length_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_chunk($items, \"2\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): length argument must be int in the current subset, got string"
    );
}

#[test]
fn array_chunk_non_positive_lengths_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
$items = [1, 2, 3];
foreach ([0, -1] as $length) {
    try {
        var_dump(array_chunk($items, $length));
    } catch (ValueError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
    try {
        var_dump(array_chunk($items, $length, true));
    } catch (ValueError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
    try {
        var_dump(array_chunk($items, $length, false));
    } catch (ValueError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n\
ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n\
ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n\
ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n\
ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n\
ValueError:array_chunk(): Argument #2 ($length) must be greater than 0\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_chunk_coerces_scalar_preserve_keys_and_reports_bool_type_errors() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$truthy_int = array_chunk($items, 2, 1);
echo $truthy_int[0]["name"], "|", $truthy_int[0][5], "\n";

$falsey_int = array_chunk($items, 2, 0);
echo $falsey_int[0][0], "|", $falsey_int[0][1], "\n";

$truthy_string = array_chunk($items, 2, "yes");
echo $truthy_string[2][-1], "|", $truthy_string[2][6], "\n";

$falsey_string = array_chunk($items, 2, "0");
echo $falsey_string[2][0], "|", $falsey_string[2][1], "\n";

$falsey_null = array_chunk($items, 2, null);
echo $falsey_null[0][0], "|", $falsey_null[0][1], "\n";

$truthy_float = array_chunk($items, 2, 0.25);
echo $truthy_float[1][2], "|", $truthy_float[1]["02"], "\n";

$call = "array_chunk";
$dynamic = $call($items, 2, "1");
echo $dynamic[0]["name"], "|", $dynamic[0][5], "\n";

try {
    array_chunk($items, 0, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Ada|five\n\
Ada|five\n\
negative|next\n\
negative|next\n\
Ada|five\n\
two|zero two\n\
Ada|five\n\
array_chunk(): Argument #3 ($preserve_keys) must be of type bool, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_chunk_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_chunk([1], 1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
