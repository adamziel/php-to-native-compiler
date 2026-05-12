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
fn array_filter_rejects_mode_flags_for_now() {
    let error = runtime_error(
        "<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, \"strlen\", 1);\n",
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): mode flags are not supported in the current subset"
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
}
