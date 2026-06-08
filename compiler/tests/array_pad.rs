use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_pad_supports_right_left_and_noop_padding() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$right = array_pad($items, 8, "pad");
print_r($right);
echo count($right), "|", $right["name"], "|", $right[0], "|", $right[1], "|", $right["02"], "|", $right[2], "|", $right[3], "|", $right[4], "|", $right[5], "\n";
$right[] = "after";
echo $right[6], "\n";

$left = array_pad($items, -8, "pad");
print_r($left);
echo count($left), "|", $left[0], "|", $left[1], "|", $left["name"], "|", $left[2], "|", $left[3], "|", $left["02"], "|", $left[4], "|", $left[5], "\n";
$left[] = "after-left";
echo $left[6], "\n";

$noop = array_pad($items, 3, "pad");
if (array_key_exists(0, $noop)) {
    echo "noop-reindexed\n";
} else {
    echo "noop-preserved\n";
}
echo $noop["name"], "|", $noop[5], "|", $noop[2], "|", $noop["02"], "|", $noop[-1], "|", $noop[6], "\n";
$noop[] = "after-noop";
echo $noop[7], "\n";

$empty_right = array_pad([], 3, "pad");
echo count($empty_right), "|", $empty_right[0], "|", $empty_right[1], "|", $empty_right[2], "\n";

$empty_left = array_pad([], -2, "left");
echo count($empty_left), "|", $empty_left[0], "|", $empty_left[1], "\n";

$call = "array_pad";
$dynamic = $call(["first" => "Ada"], 3, "pad");
print_r($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Ada\n    [0] => five\n    [1] => two\n    [02] => zero two\n    [2] => negative\n    [3] => next\n    [4] => pad\n    [5] => pad\n)\n8|Ada|five|two|zero two|negative|next|pad|pad\nafter\nArray\n(\n    [0] => pad\n    [1] => pad\n    [name] => Ada\n    [2] => five\n    [3] => two\n    [02] => zero two\n    [4] => negative\n    [5] => next\n)\n8|pad|pad|Ada|five|two|zero two|negative|next\nafter-left\nnoop-preserved\nAda|five|two|zero two|negative|next\nafter-noop\n3|pad|pad|pad\n2|left|left\nArray\n(\n    [first] => Ada\n    [0] => pad\n    [1] => pad\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_pad_requires_array_first_argument() {
    let error = runtime_error("<?php\necho array_pad(42, 3, \"pad\");\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_pad(): first argument must be array, got int"
    );
}

#[test]
fn array_pad_requires_int_length_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_pad($items, \"3\", \"pad\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_pad(): length argument must be int in the current subset, got string"
    );
}

#[test]
fn array_pad_too_large_padding_is_catchable_value_error() {
    let source = r#"<?php
function test($length) {
    try {
        var_dump(array_pad(array("", -1, 2.0), $length, 0));
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}

test(PHP_INT_MIN);
test(PHP_INT_MAX);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_pad(): Argument #2 ($length) must not exceed the maximum allowed array size\narray_pad(): Argument #2 ($length) must not exceed the maximum allowed array size\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_pad_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_pad([1], 3, 0);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
