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
fn array_reduce_callback_requires_callable_value() {
    let error = runtime_error("<?php\n$items = [\"Ada\"];\necho array_reduce($items, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reduce(): callback must evaluate to string, closure, or array callable in the current subset, got int"
    );
}

#[test]
fn array_reduce_accepts_closure_callback() {
    let execution = run_source(
        "<?php\n$items = [\"Ada\"];\n$callback = fn($carry, $value) => $value;\necho array_reduce($items, $callback);\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reduce_user_callbacks_with_reference_params_warn_and_receive_values() {
    let execution = run_source(
        r#"<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function reduce_first_ref(&$carry, $value) {
    return $carry + $value;
}

function reduce_second_ref($carry, &$value) {
    return $carry + $value;
}

$closure = function(&$carry, $value) {
    return $carry + $value;
};

class Reducer {
    public static function stat(&$carry, $value) {
        return $carry + $value;
    }

    public function inst($carry, &$value) {
        return $carry + $value;
    }
}

echo array_reduce([1, 2], "reduce_first_ref", 0), "\n";
echo array_reduce([1, 2], "reduce_second_ref", 0), "\n";
echo array_reduce([1, 2], $closure, 0), "\n";
echo array_reduce([1, 2], ["Reducer", "stat"], 0), "\n";
echo array_reduce([1, 2], [new Reducer(), "inst"], 0), "\n";

foreach ($messages as $message) {
    echo $message, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3\n3\n3\n3\n3\n\
reduce_first_ref(): Argument #1 ($carry) must be passed by reference, value given\n\
reduce_first_ref(): Argument #1 ($carry) must be passed by reference, value given\n\
reduce_second_ref(): Argument #2 ($value) must be passed by reference, value given\n\
reduce_second_ref(): Argument #2 ($value) must be passed by reference, value given\n\
{closure}(): Argument #1 ($carry) must be passed by reference, value given\n\
{closure}(): Argument #1 ($carry) must be passed by reference, value given\n\
Reducer::stat(): Argument #1 ($carry) must be passed by reference, value given\n\
Reducer::stat(): Argument #1 ($carry) must be passed by reference, value given\n\
Reducer::inst(): Argument #2 ($value) must be passed by reference, value given\n\
Reducer::inst(): Argument #2 ($value) must be passed by reference, value given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reduce_callback_reports_unknown_function() {
    let execution =
        run_source("<?php\n$items = [\"Ada\"];\necho array_reduce($items, \"missing_reduce\");\n")
            .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: Call to undefined function missing_reduce() in Command line code:3\nStack trace:\n#0 {main}\n  thrown in Command line code on line 3"
    );
    assert_eq!(execution.exit_code, 255);
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
    let execution = run_source(
        "<?php\nfunction needs_three($carry, $value, $extra) { return $carry; }\necho array_reduce([1], \"needs_three\");\n",
    )
    .unwrap();

    assert!(
        execution.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function needs_three(), 2 passed and exactly 3 expected in Command line code:3"
        ),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 255);
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
