use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_map_invokes_string_named_callbacks_and_preserves_one_array_keys() {
    let source = r#"<?php
function label_value($value) {
    return "mapped:" . $value;
}

$items = [];
$items["first"] = "Ada";
$items[5] = "Bob";
$items["empty"] = "";
$items[] = "Linus";

$mapped = array_map("label_value", $items);
print_r(array_keys($mapped));
echo $mapped["first"], "|", $mapped[5], "|", $mapped["empty"], "|", $mapped[6], "\n";
$mapped[] = "after";
echo $mapped[7], "\n";
print_r($items);

$call = "array_map";
$lengths = $call("strlen", ["empty" => "", "zero" => "0", "space" => " "]);
echo count($lengths), "|", $lengths["empty"], "|", $lengths["zero"], "|", $lengths["space"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => first\n    [1] => 5\n    [2] => empty\n    [3] => 6\n)\nmapped:Ada|mapped:Bob|mapped:|mapped:Linus\nafter\nArray\n(\n    [first] => Ada\n    [5] => Bob\n    [empty] => \n    [6] => Linus\n)\n3|0|1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_invokes_string_named_callback_with_two_arrays_and_null_padding() {
    let source = r#"<?php
function pair_label($left, $right) {
    if ($left === null) {
        $left = "NULL";
    }
    if ($right === null) {
        $right = "NULL";
    }
    return $left . ":" . $right;
}

$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$right = [];
$right["a"] = "R1";
$right["b"] = "R2";
$right["c"] = "R3";

$mapped = array_map("pair_label", $left, $right);
print_r(array_keys($mapped));
echo $mapped[0], "|", $mapped[1], "|", $mapped[2], "\n";
print_r($left);
print_r($right);

$call = "array_map";
$dynamic = $call("pair_label", ["x" => "A", "y" => "B", "z" => "C"], ["one" => "1"]);
print_r($dynamic);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n)\nL1:R1|L2:R2|NULL:R3\nArray\n(\n    [first] => L1\n    [5] => L2\n)\nArray\n(\n    [a] => R1\n    [b] => R2\n    [c] => R3\n)\nArray\n(\n    [0] => A:1\n    [1] => B:NULL\n    [2] => C:NULL\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_map_requires_array_argument() {
    let error = runtime_error("<?php\necho array_map(\"strlen\", 42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): second argument must be array, got int"
    );
}

#[test]
fn array_map_callback_requires_string_callable() {
    let error = runtime_error("<?php\n$items = [\"Ada\"];\necho array_map(42, $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): callback must evaluate to string, got int"
    );
}

#[test]
fn array_map_callback_reports_unknown_function() {
    let error =
        runtime_error("<?php\n$items = [\"Ada\"];\necho array_map(\"missing_map\", $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined function missing_map()");
}

#[test]
fn array_map_rejects_null_callbacks_for_now() {
    let error = runtime_error("<?php\n$items = [\"Ada\"];\necho array_map(null, $items);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): null callbacks are not supported in the current subset"
    );
}

#[test]
fn array_map_requires_third_array_argument() {
    let error =
        runtime_error("<?php\n$items = [\"Ada\"];\necho array_map(\"strlen\", $items, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): third argument must be array, got int"
    );
}

#[test]
fn array_map_rejects_more_than_two_input_arrays_for_now() {
    let error = runtime_error(
        "<?php\nfunction combine_three($a, $b, $c) { return $a; }\n$left = [\"Ada\"];\n$middle = [\"Grace\"];\n$right = [\"Linus\"];\necho array_map(\"combine_three\", $left, $middle, $right);\n",
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): more than two input arrays are not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_array_map_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_map(\"strlen\", [\"name\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );

    let two_array_error =
        emit_ir_source("<?php\necho array_map(\"pair\", [\"left\"], [\"right\"]);\n").unwrap_err();

    assert_eq!(two_array_error.phase, Phase::Codegen);
    assert!(
        two_array_error.message.contains("function calls"),
        "{}",
        two_array_error.message
    );
}
