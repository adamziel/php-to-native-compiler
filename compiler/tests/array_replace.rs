use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_replace_overwrites_matching_keys_and_appends_new_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right["5"] = "five right";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$replaced = array_replace($left, $right);
print_r($replaced);
echo count($replaced), "\n";
echo $replaced["name"], "|", $replaced[5], "|", $replaced[2], "|", $replaced["02"], "|", $replaced[6], "|", $replaced[7], "|", $replaced[8], "|", $replaced["extra"], "\n";
$replaced[] = "after";
echo $replaced[9], "\n";
print_r($left);
print_r($right);

$call = "array_replace";
$again = $call($left, $right);
echo $again["name"], "|", $again[5], "|", $again["02"], "|", $again["extra"], "\n";

$empty_replacement = array_replace($left, []);
print_r($empty_replacement);
echo count($empty_replacement), "\n";
$empty_replacement[] = "after empty";
echo $empty_replacement[7];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Bea\n    [5] => five right\n    [2] => two\n    [02] => zero two right\n    [6] => left next\n    [7] => seven\n    [8] => right next\n    [extra] => extra\n)\n8\nBea|five right|two|zero two right|left next|seven|right next|extra\nafter\nArray\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [6] => left next\n)\nArray\n(\n    [name] => Bea\n    [5] => five right\n    [7] => seven\n    [02] => zero two right\n    [8] => right next\n    [extra] => extra\n)\nBea|five right|zero two right|extra\nArray\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [6] => left next\n)\n5\nafter empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_replace_requires_array_first_argument() {
    let error = runtime_error("<?php\n$right = [];\necho array_replace(42, $right);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_replace(): first argument must be array, got int"
    );
}

#[test]
fn array_replace_requires_array_second_argument() {
    let error = runtime_error("<?php\n$left = [];\necho array_replace($left, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_replace(): second argument must be array, got int"
    );
}

#[test]
fn array_replace_rejects_variadic_replacements_until_supported() {
    let error =
        runtime_error("<?php\n$left = [];\n$right = [];\necho array_replace($left, $right, []);\n");

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_replace(): expected 2 argument(s), got 3"
    );
}

#[test]
fn emit_ir_rejects_array_replace_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_replace([1], [2]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
