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
fn array_reduce_rejects_initial_values_for_now() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\"];\necho array_reduce($items, \"strlen\", \"start\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reduce(): initial values are not supported in the current subset"
    );
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
