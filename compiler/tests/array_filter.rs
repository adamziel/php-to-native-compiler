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
fn array_filter_rejects_callback_forms_for_now() {
    let error =
        runtime_error("<?php\n$items = [\"Ada\", \"\"];\necho array_filter($items, \"strlen\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_filter(): callbacks and mode flags are not supported in the current subset"
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
}
