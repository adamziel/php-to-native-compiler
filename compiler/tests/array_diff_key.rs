use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_diff_key_preserves_first_array_entries_with_missing_keys() {
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
$right[-1] = "ignored";
$right["extra"] = "ignored";

$diffed = array_diff_key($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["02"], "|", $diffed["drop"], "|", $diffed[6], "\n";
$diffed[] = "after";
echo $diffed[7], "\n";
print_r($left);
print_r($right);

$call = "array_diff_key";
$again = $call($left, $right);
echo $again["02"], "|", $again["drop"], "|", $again[6], "\n";

$empty = array_diff_key([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff_key(["missing" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff_key(["name" => "x"], $right);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [02] => zero two\n    [drop] => drop\n    [6] => next\n)\n3\nzero two|drop|next\nafter\nArray\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [-1] => negative\n    [drop] => drop\n    [6] => next\n)\nArray\n(\n    [name] => ignored\n    [5] => ignored\n    [2] => ignored\n    [-1] => ignored\n    [extra] => ignored\n)\nzero two|drop|next\nArray\n(\n)\n0\nArray\n(\n    [missing] => x\n)\n1\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_key_requires_array_first_argument() {
    let error = runtime_error("<?php\n$right = [];\necho array_diff_key(42, $right);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_diff_key(): first argument must be array, got int"
    );
}

#[test]
fn array_diff_key_requires_array_second_argument() {
    let error = runtime_error("<?php\n$left = [];\necho array_diff_key($left, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_diff_key(): second argument must be array, got int"
    );
}

#[test]
fn array_diff_key_rejects_variadic_operands_until_supported() {
    let error = runtime_error(
        "<?php\n$left = [];\n$right = [];\n$third = [];\necho array_diff_key($left, $right, $third);\n",
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_diff_key(): expected 2 argument(s), got 3"
    );
}

#[test]
fn emit_ir_rejects_array_diff_key_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_diff_key([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
