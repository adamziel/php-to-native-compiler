use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_merge_reindexes_integer_keys_and_overwrites_string_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$merged = array_merge($left, $right);
echo count($merged), "\n";
echo $merged["name"], "|", $merged[0], "|", $merged[1], "|", $merged["02"], "|", $merged[2], "|", $merged[3], "|", $merged[4], "|", $merged["extra"], "\n";
$merged[] = "after";
echo $merged[5], "\n";
echo $left["name"], "|", $left[5], "|", $left[2], "|", $left["02"], "|", $left[6], "\n";
echo $right["name"], "|", $right[7], "|", $right["02"], "|", $right[8], "|", $right["extra"], "\n";

$call = "array_merge";
$again = $call($left, $right);
echo $again["name"], "|", $again[0], "|", $again["02"], "|", $again["extra"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "8\nBea|five|two|zero two right|left next|seven|right next|extra\nafter\nAda|five|two|zero two|left next\nBea|seven|zero two right|right next|extra\nBea|five|zero two right|extra"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_merge_requires_array_first_argument() {
    let error = runtime_error("<?php\n$right = [];\necho array_merge(42, $right);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_merge(): first argument must be array, got int"
    );
}

#[test]
fn array_merge_requires_array_second_argument() {
    let error = runtime_error("<?php\n$left = [];\necho array_merge($left, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_merge(): second argument must be array, got int"
    );
}

#[test]
fn array_merge_requires_exactly_two_arguments() {
    let error = runtime_error("<?php\n$items = [];\necho array_merge($items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_merge(): expected 2 argument(s), got 1"
    );
}

#[test]
fn emit_ir_rejects_array_merge_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_merge([1], [2]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
