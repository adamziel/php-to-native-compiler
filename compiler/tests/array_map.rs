use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_map_invokes_string_named_callbacks_and_reindexes_results() {
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
echo $mapped[0], "|", $mapped[1], "|", $mapped[2], "|", $mapped[3], "\n";
$mapped[] = "after";
echo $mapped[4], "\n";
print_r($items);

$call = "array_map";
$lengths = $call("strlen", ["empty" => "", "zero" => "0", "space" => " "]);
echo count($lengths), "|", $lengths[0], "|", $lengths[1], "|", $lengths[2];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => 0\n    [1] => 1\n    [2] => 2\n    [3] => 3\n)\nmapped:Ada|mapped:Bob|mapped:|mapped:Linus\nafter\nArray\n(\n    [first] => Ada\n    [5] => Bob\n    [empty] => \n    [6] => Linus\n)\n3|0|1|1"
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
fn array_map_rejects_multiple_input_arrays_for_now() {
    let error = runtime_error(
        "<?php\n$names = [\"Ada\"];\n$other = [\"Grace\"];\necho array_map(\"strlen\", $names, $other);\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_map(): multiple input arrays are not supported in the current subset"
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
}
