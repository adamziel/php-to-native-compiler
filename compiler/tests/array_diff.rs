use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_diff_preserves_first_array_entries_with_absent_scalar_values() {
    let source = r#"<?php
$left = [];
$left["null"] = null;
$left["false"] = false;
$left["empty"] = "";
$left["true"] = true;
$left["one"] = 1;
$left["zero"] = 0;
$left["string-zero"] = "0";
$left["int-ten"] = 10;
$left["float-ten"] = 10.0;
$left["string-ten-float"] = "10.0";
$left["text"] = "abc";
$left[8] = "eight";
$left["keep"] = "keep";
$left[] = "next";

$right = [];
$right[] = "";
$right[] = "0";
$right[] = "1";
$right[] = "10";
$right[] = "abc";
$right[] = "missing";

$diffed = array_diff($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["string-ten-float"], "|", $diffed[8], "|", $diffed["keep"], "|", $diffed[9], "\n";
$diffed[] = "after";
echo $diffed[10], "\n";
print_r($left);
print_r($right);

$call = "array_diff";
$again = $call($left, $right);
echo $again["string-ten-float"], "|", $again[8], "|", $again["keep"], "|", $again[9], "\n";

$empty = array_diff([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff(["x" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff(["name" => "x"], ["x"]);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [string-ten-float] => 10.0\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\n4\n10.0|eight|keep|next\nafter\nArray\n(\n    [null] => \n    [false] => \n    [empty] => \n    [true] => 1\n    [one] => 1\n    [zero] => 0\n    [string-zero] => 0\n    [int-ten] => 10\n    [float-ten] => 10\n    [string-ten-float] => 10.0\n    [text] => abc\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\nArray\n(\n    [0] => \n    [1] => 0\n    [2] => 1\n    [3] => 10\n    [4] => abc\n    [5] => missing\n)\n10.0|eight|keep|next\nArray\n(\n)\n0\nArray\n(\n    [x] => x\n)\n1\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_requires_array_first_argument() {
    let error = runtime_error("<?php\n$right = [];\necho array_diff(42, $right);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_diff(): first argument must be array, got int"
    );
}

#[test]
fn array_diff_requires_array_second_argument() {
    let error = runtime_error("<?php\n$left = [];\necho array_diff($left, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_diff(): second argument must be array, got int"
    );
}

#[test]
fn array_diff_rejects_variadic_operands_in_current_slice() {
    let error = runtime_error("<?php\necho array_diff([], [], []);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_diff(): expected 2 argument(s), got 3"
    );
}

#[test]
fn array_diff_rejects_non_scalar_value_comparisons() {
    let array_error =
        runtime_error("<?php\n$left = [[]];\n$right = [];\necho array_diff($left, $right);\n");

    assert_eq!(array_error.line, 4);
    assert_eq!(array_error.column, 6);
    assert_eq!(
        array_error.message,
        "unsupported call array_diff(): values must be scalar in the current subset, got array"
    );

    let object_error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
$left = ["value"];
$right = [$box];
echo array_diff($left, $right);
"#,
    );

    assert_eq!(object_error.line, 6);
    assert_eq!(object_error.column, 6);
    assert_eq!(
        object_error.message,
        "unsupported call array_diff(): values must be scalar in the current subset, got object"
    );
}

#[test]
fn emit_ir_rejects_array_diff_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_diff([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
