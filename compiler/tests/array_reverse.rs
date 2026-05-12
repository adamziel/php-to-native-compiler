use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_reverse_reverses_order_reindexes_int_keys_and_preserves_string_keys() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

$reversed = array_reverse($items);
echo count($reversed), "\n";
echo $reversed[0], "|", $reversed[1], "|", $reversed["02"], "|", $reversed[2], "|", $reversed[3], "|", $reversed["name"], "\n";
$reversed[] = "after";
echo $reversed[4], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6], "\n";

$call = "array_reverse";
$again = $call($items);
echo $again[0], "|", $again["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "6\nnext|negative|zero two|two updated|five|Ada\nafter\nAda|five|two updated|zero two|negative|next\nnext|Ada"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reverse_requires_array_argument() {
    let error = runtime_error("<?php\necho array_reverse(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reverse(): argument must be array, got int"
    );
}

#[test]
fn array_reverse_rejects_preserve_keys_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_reverse($items, true);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_reverse(): preserve_keys argument is not implemented"
    );
}

#[test]
fn emit_ir_rejects_array_reverse_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_reverse([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
