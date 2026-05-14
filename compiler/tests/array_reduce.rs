use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_reduce_invokes_string_named_callback_with_accumulator() {
    let source = r#"<?php
function join_values($carry, $value) {
    if ($carry === null) {
        return $value;
    }
    return $carry . ":" . $value;
}

function collect_values($carry, $value) {
    if (!$carry) {
        $carry = [];
    }
    $carry[] = $value;
    return $carry;
}

function sum_pair($carry, $value) {
    if ($carry === null) {
        $carry = 0;
    }
    return $carry + $value;
}

$items = [];
$items["first"] = "Ada";
$items[5] = "Grace";
$items[] = "Linus";

$callback = "join_values";
echo array_reduce($items, $callback), "\n";

$collected = array_reduce($items, "collect_values");
print_r($collected);

if (array_reduce([], "join_values") === null) {
    echo "empty-null\n";
}

print_r($items);

$call = "array_reduce";
echo $call([1, 2, 3], "sum_pair");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Ada:Grace:Linus\nArray\n(\n    [0] => Ada\n    [1] => Grace\n    [2] => Linus\n)\nempty-null\nArray\n(\n    [first] => Ada\n    [5] => Grace\n    [6] => Linus\n)\n6"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reduce_requires_array_argument() {
    let error = runtime_error("<?php\necho array_reduce(42, \"strlen\");\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reduce(): first argument must be array, got int"
    );
}

#[test]
fn array_reduce_callback_requires_string_callable() {
    let error = runtime_error("<?php\n$items = [\"Ada\"];\necho array_reduce($items, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reduce(): callback must evaluate to string, got int"
    );

    let closure_error = runtime_error(
        "<?php\n$items = [\"Ada\"];\n$callback = fn($carry, $value) => $value;\necho array_reduce($items, $callback);\n",
    );
    assert_eq!(closure_error.line, 4);
    assert_eq!(closure_error.column, 6);
    assert_eq!(
        closure_error.message,
        "unsupported call array_reduce(): callback must evaluate to string, got closure"
    );
}

#[test]
fn array_reduce_callback_reports_unknown_function() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\"];\necho array_reduce($items, \"missing_reduce\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined function missing_reduce()");
}

#[test]
fn array_reduce_uses_initial_value_when_supplied() {
    let source = r#"<?php
function join_with_seed($carry, $value) {
    return $carry . ":" . $value;
}

function collect_with_seed($carry, $value) {
    $carry[] = $value;
    return $carry;
}

function add_value($carry, $value) {
    return $carry + $value;
}

$items = ["Ada", "Grace", "Linus"];
echo array_reduce($items, "join_with_seed", "start"), "\n";

$collected = array_reduce($items, "collect_with_seed", ["seed"]);
print_r($collected);

if (array_reduce([], "join_with_seed", "empty") === "empty") {
    echo "empty-initial\n";
}

$call = "array_reduce";
echo $call([1, 2, 3], "add_value", 10);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "start:Ada:Grace:Linus\nArray\n(\n    [0] => seed\n    [1] => Ada\n    [2] => Grace\n    [3] => Linus\n)\nempty-initial\n16"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reduce_callback_arity_errors_come_from_callback() {
    let error = runtime_error(
        "<?php\nfunction only_one($carry) { return $carry; }\necho array_reduce([1], \"only_one\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for only_one(): expected 1 argument(s), got 2"
    );
}

#[test]
fn array_reduce_reports_arity_mismatch() {
    let error = runtime_error("<?php\necho array_reduce([]);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for array_reduce(): expected 2 to 3 argument(s), got 1"
    );
}

#[test]
fn emit_ir_rejects_array_reduce_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_reduce([1], \"strlen\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
