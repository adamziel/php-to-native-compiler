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
fn array_filter_callback_requires_string_callable() {
    let error = runtime_error("<?php\n$items = [\"Ada\"];\necho array_filter($items, 42);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): callback must evaluate to string, got int"
    );
}

#[test]
fn array_filter_callback_reports_unknown_function() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\"];\necho array_filter($items, \"missing_filter\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined function missing_filter()");
}

#[test]
fn array_filter_rejects_unsupported_integer_mode() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, \"strlen\", 3);\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): mode flag must be integer 0, 1, or 2 in the current subset, got 3"
    );
}

#[test]
fn array_filter_rejects_unsupported_non_integer_mode_flags() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, null, \"0\");\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): mode flag must be integer 0, 1, 2, or bool in the current subset, got string"
    );
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

    let constant_error = emit_ir_source("<?php\necho ARRAY_FILTER_USE_KEY;\n").unwrap_err();
    assert_eq!(constant_error.phase, Phase::Codegen);
    assert!(
        constant_error.message.contains("global-constant lowering"),
        "{}",
        constant_error.message
    );
}
