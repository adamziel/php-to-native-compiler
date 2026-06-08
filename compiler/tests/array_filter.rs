use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_filter_without_callback_removes_falsey_values_and_preserves_keys() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["true"] = true;
$items["zero"] = 0;
$items["float-zero"] = 0.0;
$items["one"] = 1;
$items["empty-string"] = "";
$items["zero-string"] = "0";
$items["space"] = " ";
$items["text"] = "Ada";
$items["empty-array"] = [];
$items["nested-array"] = ["kept"];
$items[7] = "seven";
$items[] = "next";

$filtered = array_filter($items);
print_r(array_keys($filtered));
echo count($filtered), "\n";
echo $filtered["true"], "|", $filtered["one"], "|", $filtered["space"], "|", $filtered["text"], "|", count($filtered["nested-array"]), "|", $filtered[7], "|", $filtered[8], "\n";
if (array_key_exists("null", $filtered)) {
    echo "null kept\n";
} else {
    echo "null removed\n";
}
if (array_key_exists("empty-array", $filtered)) {
    echo "empty array kept\n";
} else {
    echo "empty array removed\n";
}
$filtered[] = "after";
echo $filtered[9], "\n";

$call = "array_filter";
$again = $call($items);
echo count($again), "|", count($again["nested-array"]);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => true\n    [1] => one\n    [2] => space\n    [3] => text\n    [4] => nested-array\n    [5] => 7\n    [6] => 8\n)\n7\n1|1| |Ada|1|seven|next\nnull removed\nempty array removed\nafter\n7|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_null_callback_uses_falsey_filtering_path() {
    let source = r#"<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["true"] = true;
$items["zero"] = 0;
$items["zero-string"] = "0";
$items["space"] = " ";
$items["text"] = "Ada";
$items[] = "tail";

$filtered = array_filter($items, null);
print_r(array_keys($filtered));
echo count($filtered), "\n";
echo $filtered["true"], "|", strlen($filtered["space"]), "|", $filtered["text"], "|", $filtered[0], "\n";

$call = "array_filter";
$again = $call($items, null);
echo count($again), "\n";
if (array_key_exists("null", $again)) {
    echo "null kept\n";
} else {
    echo "null removed\n";
}
if (array_key_exists("zero-string", $again)) {
    echo "zero string kept\n";
} else {
    echo "zero string removed\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => true\n    [1] => space\n    [2] => text\n    [3] => 0\n)\n4\n1|1|Ada|tail\n4\nnull removed\nzero string removed\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_integer_zero_mode_for_current_value_paths() {
    let source = r#"<?php
function keep_long($value) {
    return strlen($value) > 3;
}

$items = [];
$items["empty"] = "";
$items["zero"] = "0";
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$null_mode = array_filter($items, null, 0);
print_r(array_keys($null_mode));
echo count($null_mode), "|", $null_mode["short"], "|", $null_mode["long"], "|", $null_mode[5], "\n";

$callback_mode = array_filter($items, "keep_long", 0);
print_r(array_keys($callback_mode));
echo count($callback_mode), "|", $callback_mode["long"], "|", $callback_mode[5], "\n";

$call = "array_filter";
$builtin = $call(["empty" => "", "zero" => "0", "space" => " "], "strlen", 0);
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["zero"], "|", strlen($builtin["space"]), "\n";

$again = $call($items, null, 0);
echo count($again), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => short\n    [1] => long\n    [2] => 5\n)\n3|Ada|Grace|Linus\nArray\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => zero\n    [1] => space\n)\n2|0|1\n3\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_integer_two_mode_for_key_callbacks() {
    let source = r#"<?php
function keep_selected_key($key) {
    if ($key === "short") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    if ($key === "02") {
        return true;
    }
    return false;
}

$items = [];
$items[""] = "empty-key";
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";
$items["02"] = "zero-two";

$filtered = array_filter($items, "keep_selected_key", 2);
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["short"], "|", $filtered[5], "|", $filtered["02"], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$builtin = $call(["" => "empty", "name" => "Ada", "long-key" => "Grace"], "strlen", 2);
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["name"], "|", $builtin["long-key"], "\n";

$again = $call($items, "keep_selected_key", 2);
echo count($again), "|", $again[5];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => short\n    [1] => 5\n    [2] => 02\n)\n3|Ada|Linus|zero-two\nafter\nArray\n(\n    [0] => name\n    [1] => long-key\n)\n2|Ada|Grace\n3|Linus"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_integer_one_mode_for_value_and_key_callbacks() {
    let source = r#"<?php
function keep_value_and_key($value, $key) {
    if ($key === "short") {
        return $value === "Ada";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    if ($key === "02") {
        return $value === "zero-two";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";
$items["02"] = "zero-two";
$items["other"] = "Ada";

$filtered = array_filter($items, "keep_value_and_key", 1);
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["short"], "|", $filtered[5], "|", $filtered["02"], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$again = $call($items, "keep_value_and_key", 1);
echo count($again), "|", $again["02"], "\n";

$null_mode = $call(["empty" => "", "zero" => "0", "space" => " "], null, 1);
print_r(array_keys($null_mode));
echo count($null_mode), "|", strlen($null_mode["space"]), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => short\n    [1] => 5\n    [2] => 02\n)\n3|Ada|Linus|zero-two\nafter\n3|zero-two\nArray\n(\n    [0] => space\n)\n1|1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_named_mode_constants_for_key_and_value_key_callbacks() {
    let source = r#"<?php
function keep_named_key($key) {
    if ($key === "short") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    return false;
}

function keep_named_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 6) {
        return $value === "tail";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "tail";

$key_mode = array_filter($items, "keep_named_key", ARRAY_FILTER_USE_KEY);
print_r(array_keys($key_mode));
echo count($key_mode), "|", $key_mode["short"], "|", $key_mode[5], "\n";

$both_mode = array_filter($items, "keep_named_value_and_key", ARRAY_FILTER_USE_BOTH);
print_r(array_keys($both_mode));
echo count($both_mode), "|", $both_mode["long"], "|", $both_mode[6], "\n";
echo ARRAY_FILTER_USE_KEY, "|", ARRAY_FILTER_USE_BOTH, "\n";

$call = "array_filter";
$again = $call($items, "keep_named_value_and_key", ARRAY_FILTER_USE_BOTH);
echo count($again), "|", $again[6];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => short\n    [1] => 5\n)\n2|Ada|Linus\nArray\n(\n    [0] => long\n    [1] => 6\n)\n2|Grace|tail\n2|1\n2|tail"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_boolean_mode_flags() {
    let source = r#"<?php
function keep_long($value) {
    return strlen($value) > 3;
}

function keep_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$false_mode = array_filter($items, "keep_long", false);
print_r(array_keys($false_mode));
echo count($false_mode), "|", $false_mode["long"], "|", $false_mode[5], "\n";

$true_mode = array_filter($items, "keep_value_and_key", true);
print_r(array_keys($true_mode));
echo count($true_mode), "|", $true_mode["long"], "|", $true_mode[5], "\n";

$call = "array_filter";
$null_true = $call(["empty" => "", "zero" => "0", "space" => " "], null, true);
print_r(array_keys($null_true));
echo count($null_true), "|", strlen($null_true["space"]), "\n";

$again = $call($items, "keep_long", false);
echo count($again), "|", $again["long"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => space\n)\n1|1\n2|Grace"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_integer_string_mode_flags() {
    let source = r#"<?php
function keep_long($value) {
    return strlen($value) > 3;
}

function keep_selected_key($key) {
    if ($key === "long") {
        return true;
    }
    if ($key === 5) {
        return true;
    }
    return false;
}

function keep_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$value_mode = array_filter($items, "keep_long", "0");
print_r(array_keys($value_mode));
echo count($value_mode), "|", $value_mode["long"], "|", $value_mode[5], "\n";

$both_mode = array_filter($items, "keep_value_and_key", " 1 ");
print_r(array_keys($both_mode));
echo count($both_mode), "|", $both_mode["long"], "|", $both_mode[5], "\n";

$call = "array_filter";
$key_mode = $call($items, "keep_selected_key", "02");
print_r(array_keys($key_mode));
echo count($key_mode), "|", $key_mode["long"], "|", $key_mode[5], "\n";

$null_mode = $call(["empty" => "", "zero" => "0", "space" => " "], null, "+1");
print_r(array_keys($null_mode));
echo count($null_mode), "|", strlen($null_mode["space"]);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => space\n)\n1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_integral_float_and_float_string_mode_flags() {
    let source = r#"<?php
function keep_selected_key($key) {
    return $key === "long" || $key === 5;
}

function keep_value_and_key($value, $key) {
    if ($key === "long") {
        return $value === "Grace";
    }
    if ($key === 5) {
        return $value === "Linus";
    }
    return false;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items[5] = "Linus";
$items[] = "";

$both_mode = array_filter($items, "keep_value_and_key", 1.0);
print_r(array_keys($both_mode));
echo count($both_mode), "|", $both_mode["long"], "|", $both_mode[5], "\n";

$call = "array_filter";
$key_mode = $call($items, "keep_selected_key", "2.0");
print_r(array_keys($key_mode));
echo count($key_mode), "|", $key_mode["long"], "|", $key_mode[5], "\n";

$null_mode = $call(["empty" => "", "zero" => "0", "space" => " "], null, "0e0");
print_r(array_keys($null_mode));
echo count($null_mode), "|", strlen($null_mode["space"]);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => long\n    [1] => 5\n)\n2|Grace|Linus\nArray\n(\n    [0] => space\n)\n1|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_requires_array_argument() {
    let error = runtime_error("<?php\necho array_filter(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): argument must be array, got int"
    );
}

#[test]
fn array_filter_callback_invokes_string_named_callables() {
    let source = r#"<?php
function keep_long($value) {
    return strlen($value) > 3;
}

$items = [];
$items["short"] = "Ada";
$items["long"] = "Grace";
$items["empty"] = "";
$items[5] = "Linus";

$callback = "keep_long";
$filtered = array_filter($items, $callback);
print_r(array_keys($filtered));
echo $filtered["long"], "|", $filtered[5], "\n";
$filtered[] = "after";
echo $filtered[6], "\n";

$call = "array_filter";
$builtin = $call(["empty" => "", "zero" => "0", "space" => " "], "strlen");
print_r(array_keys($builtin));
echo count($builtin), "|", $builtin["zero"], "|", strlen($builtin["space"]);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => long\n    [1] => 5\n)\nGrace|Linus\nafter\nArray\n(\n    [0] => zero\n    [1] => space\n)\n2|0|1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_accepts_general_callables_and_reports_callback_errors() {
    let source = r#"<?php
class FilterHelper {
    public static function key_is_two($value, $key) {
        return $key === 2;
    }
}

$items = [1, 2, 3];
$closure = fn($value) => $value > 1;
print_r(array_filter($items, $closure));
print_r(array_filter($items, ["FilterHelper", "key_is_two"], ARRAY_FILTER_USE_BOTH));

try {
    array_filter($items, 42);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_filter($items, "missing_filter");
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_filter($items, "is_numeric", ARRAY_FILTER_USE_BOTH);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_filter([], mode: 999);
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [1] => 2\n    [2] => 3\n)\nArray\n(\n    [2] => 3\n)\narray_filter(): Argument #2 ($callback) must be a valid callback or null, no array or string given\narray_filter(): Argument #2 ($callback) must be a valid callback or null, function \"missing_filter\" not found or invalid function name\nis_numeric() expects exactly 1 argument, 2 given\nValueError: array_filter(): Argument #3 ($mode) must be one of ARRAY_FILTER_USE_VALUE, ARRAY_FILTER_USE_KEY, or ARRAY_FILTER_USE_BOTH\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_callback_arity_throws_argument_count_error_and_preserves_reference_slots() {
    let source = r#"<?php
try {
    array_filter([1, 2], "strlen", ARRAY_FILTER_USE_BOTH);
} catch (ArgumentCountError $e) {
    echo "builtin:", $e::class, ":", $e->getMessage(), "\n";
} catch (TypeError $e) {
    echo "builtin-type:", $e::class, ":", $e->getMessage(), "\n";
}

function needs_two($a, $b) {
    return true;
}

try {
    array_filter([1], "needs_two");
} catch (ArgumentCountError $e) {
    echo "user:", $e::class, ":", $e->getMessage(), "\n";
} catch (TypeError $e) {
    echo "user-type:", $e::class, ":", $e->getMessage(), "\n";
}

$ref = "start";
$items = [&$ref, "drop"];
$filtered = array_filter($items, fn($value) => $value === "start");
$filtered[0] = "changed";
echo $ref, "\n";
$ref = "source";
echo $filtered[0], "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "builtin:ArgumentCountError:strlen() expects exactly 1 argument, 2 given\nuser:ArgumentCountError:Too few arguments to function needs_two(), 1 passed and exactly 2 expected\nchanged\nsource\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_filter_invalid_integer_mode_throws_value_error() {
    let execution =
        run_source("<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, \"strlen\", 3);\n")
            .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught ValueError: array_filter(): Argument #3 ($mode) must be one of ARRAY_FILTER_USE_VALUE, ARRAY_FILTER_USE_KEY, or ARRAY_FILTER_USE_BOTH in Command line code:3\nStack trace:\n#0 {main}\n  thrown in Command line code on line 3"
    );
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn array_filter_rejects_unsupported_non_integer_mode_flags() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, null, \"not-a-mode\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): mode flag string must be an integral numeric string in the current subset"
    );
}

#[test]
fn array_filter_rejects_lossy_float_mode_flags() {
    let error =
        runtime_error("<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, null, 2.5);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): mode flag float must be finite and integral in the current subset"
    );
}

#[test]
fn array_filter_user_callbacks_with_reference_params_warn_and_receive_values() {
    let execution = run_source(
        r#"<?php
$messages = [];
set_error_handler(function($errno, $errstr) use (&$messages) {
    $messages[] = $errstr;
    return true;
});

function keep_ref(&$value) {
    $value = "local-" . $value;
    return true;
}

function keep_both_ref(&$value, &$key) {
    return true;
}

$closure = function(&$key) {
    return true;
};

class FilterHelper {
    public static function stat(&$value) {
        return true;
    }

    public function inst(&$value, &$key) {
        return true;
    }
}

$items = ["x" => "one", "empty" => ""];
$filtered = array_filter($items, "keep_ref");
print_r($filtered);
print_r($items);

$both = array_filter(["b" => "two"], "keep_both_ref", ARRAY_FILTER_USE_BOTH);
$key_only = array_filter(["k" => "three"], $closure, ARRAY_FILTER_USE_KEY);
$static = array_filter(["s" => "four"], ["FilterHelper", "stat"]);
$instance = array_filter(["i" => "five"], [new FilterHelper(), "inst"], ARRAY_FILTER_USE_BOTH);
echo count($both), "|", count($key_only), "|", count($static), "|", count($instance), "\n";

foreach ($messages as $message) {
    echo $message, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n    [x] => one\n    [empty] => \n)\n\
Array\n(\n    [x] => one\n    [empty] => \n)\n\
1|1|1|1\n\
keep_ref(): Argument #1 ($value) must be passed by reference, value given\n\
keep_ref(): Argument #1 ($value) must be passed by reference, value given\n\
keep_both_ref(): Argument #1 ($value) must be passed by reference, value given\n\
keep_both_ref(): Argument #2 ($key) must be passed by reference, value given\n\
{closure}(): Argument #1 ($key) must be passed by reference, value given\n\
FilterHelper::stat(): Argument #1 ($value) must be passed by reference, value given\n\
FilterHelper::inst(): Argument #1 ($value) must be passed by reference, value given\n\
FilterHelper::inst(): Argument #2 ($key) must be passed by reference, value given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_filter_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_filter([\"name\"]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );

    let callback_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\");\n").unwrap_err();
    assert_eq!(callback_error.phase, Phase::Codegen);
    assert!(
        callback_error.message.contains("function calls"),
        "{}",
        callback_error.message
    );

    let null_callback_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], null);\n").unwrap_err();
    assert_eq!(null_callback_error.phase, Phase::Codegen);
    assert!(
        null_callback_error.message.contains("function calls"),
        "{}",
        null_callback_error.message
    );

    let value_mode_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\", 0);\n").unwrap_err();
    assert_eq!(value_mode_error.phase, Phase::Codegen);
    assert!(
        value_mode_error.message.contains("function calls"),
        "{}",
        value_mode_error.message
    );

    let key_mode_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\", 2);\n").unwrap_err();
    assert_eq!(key_mode_error.phase, Phase::Codegen);
    assert!(
        key_mode_error.message.contains("function calls"),
        "{}",
        key_mode_error.message
    );

    let key_value_mode_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\", 1);\n").unwrap_err();
    assert_eq!(key_value_mode_error.phase, Phase::Codegen);
    assert!(
        key_value_mode_error.message.contains("function calls"),
        "{}",
        key_value_mode_error.message
    );

    let bool_mode_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\", false);\n").unwrap_err();
    assert_eq!(bool_mode_error.phase, Phase::Codegen);
    assert!(
        bool_mode_error.message.contains("function calls"),
        "{}",
        bool_mode_error.message
    );

    let string_mode_error =
        emit_ir_source("<?php\necho array_filter([\"name\"], \"strlen\", \"0\");\n").unwrap_err();
    assert_eq!(string_mode_error.phase, Phase::Codegen);
    assert!(
        string_mode_error.message.contains("function calls"),
        "{}",
        string_mode_error.message
    );

    let constant_error = emit_ir_source("<?php\necho ARRAY_FILTER_USE_KEY;\n").unwrap_err();
    assert_eq!(constant_error.phase, Phase::Codegen);
    assert!(
        constant_error.message.contains("global-constant lowering"),
        "{}",
        constant_error.message
    );
}
