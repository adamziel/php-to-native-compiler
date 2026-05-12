use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_intersect_key_preserves_first_array_entries_with_matching_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[-1] = "negative";
$left["drop"] = "drop";
$left[] = "next";

$right = [];
$right["name"] = "ignored";
$right["5"] = "ignored";
$right[2] = "ignored";
$right["02"] = "ignored";
$right[-1] = "ignored";
$right["extra"] = "ignored";

$intersected = array_intersect_key($left, $right);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["name"], "|", $intersected[5], "|", $intersected[2], "|", $intersected["02"], "|", $intersected[-1], "\n";
$intersected[] = "after";
echo $intersected[6], "\n";
print_r($left);
print_r($right);

$call = "array_intersect_key";
$again = $call($left, $right);
echo $again["name"], "|", $again[5], "|", $again[2], "|", $again["02"], "|", $again[-1], "\n";

$empty = array_intersect_key([], $right);
print_r($empty);
echo count($empty), "\n";

$none = array_intersect_key(["missing" => "x"], $right);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [-1] => negative\n)\n5\nAda|five|two|zero two|negative\nafter\nArray\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [-1] => negative\n    [drop] => drop\n    [6] => next\n)\nArray\n(\n    [name] => ignored\n    [5] => ignored\n    [2] => ignored\n    [02] => ignored\n    [-1] => ignored\n    [extra] => ignored\n)\nAda|five|two|zero two|negative\nArray\n(\n)\n0\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_key_requires_array_first_argument() {
    let error = runtime_error("<?php\n$right = [];\necho array_intersect_key(42, $right);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_intersect_key(): first argument must be array, got int"
    );
}

#[test]
fn array_intersect_key_requires_array_second_argument() {
    let error = runtime_error("<?php\n$left = [];\necho array_intersect_key($left, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_intersect_key(): second argument must be array, got int"
    );
}

#[test]
fn array_intersect_key_rejects_variadic_operands_until_supported() {
    let error = runtime_error(
        "<?php\n$left = [];\n$right = [];\n$third = [];\necho array_intersect_key($left, $right, $third);\n",
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_intersect_key(): expected 2 argument(s), got 3"
    );
}

#[test]
fn emit_ir_rejects_array_intersect_key_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_intersect_key([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
