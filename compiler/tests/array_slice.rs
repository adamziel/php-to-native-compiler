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
fn array_slice_requires_int_length_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_slice($items, 0, \"1\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_slice(): length argument must be int in the current subset, got string"
    );
}

#[test]
fn array_slice_rejects_preserve_keys_argument_for_now() {
    let error = runtime_error("<?php\n$items = [1];\necho array_slice($items, 0, 1, true);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_slice(): preserve_keys argument is not supported in the current subset"
    );
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
