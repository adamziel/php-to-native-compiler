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
echo $again["name"], "|", $again[0], "|", $again["02"], "|", $again["extra"], "\n";

$zero = array_merge();
print_r($zero);
echo count($zero), "\n";

$single = array_merge($left);
print_r($single);
echo count($single), "\n";
$single[] = "single after";
echo $single[3], "\n";

$third = [];
$third["name"] = "Cy";
$third[10] = "ten";
$third["extra"] = "third extra";
$third[] = "third next";

$variadic = array_merge($left, $right, $third);
print_r($variadic);
echo count($variadic), "\n";
echo $variadic["name"], "|", $variadic[0], "|", $variadic[1], "|", $variadic["02"], "|", $variadic[2], "|", $variadic[3], "|", $variadic[4], "|", $variadic["extra"], "|", $variadic[5], "|", $variadic[6], "\n";

$again_three = $call($left, $right, $third);
echo $again_three["name"], "|", $again_three[5], "|", $again_three[6], "|", $again_three["extra"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "8\nBea|five|two|zero two right|left next|seven|right next|extra\nafter\nAda|five|two|zero two|left next\nBea|seven|zero two right|right next|extra\nBea|five|zero two right|extra\nArray\n(\n)\n0\nArray\n(\n    [name] => Ada\n    [0] => five\n    [1] => two\n    [02] => zero two\n    [2] => left next\n)\n5\nsingle after\nArray\n(\n    [name] => Cy\n    [0] => five\n    [1] => two\n    [02] => zero two right\n    [2] => left next\n    [3] => seven\n    [4] => right next\n    [extra] => third extra\n    [5] => ten\n    [6] => third next\n)\n10\nCy|five|two|zero two right|left next|seven|right next|third extra|ten|third next\nCy|ten|third next|third extra"
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
fn array_merge_requires_array_variadic_arguments() {
    let error =
        runtime_error("<?php\n$left = [];\n$right = [];\necho array_merge($left, $right, 42);\n");

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_merge(): third argument must be array, got int"
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
