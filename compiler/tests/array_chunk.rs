use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_chunk_splits_values_into_reindexed_chunks() {
    let source = r#"<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[] = "next";

$chunks = array_chunk($items, 2);
echo count($chunks), "|", count($chunks[0]), "|", count($chunks[1]), "|", count($chunks[2]), "\n";
echo $chunks[0][0], "|", $chunks[0][1], "|", $chunks[1][0], "|", $chunks[1][1], "|", $chunks[2][0], "\n";
if (array_key_exists("02", $chunks[1])) {
    echo "string-key-kept\n";
} else {
    echo "string-key-reindexed\n";
}
$second = $chunks[1];
$second[] = "after";
echo $second[2], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[6], "\n";

$one = array_chunk($items, 99);
echo count($one), "|", count($one[0]), "|", $one[0][4], "\n";

$empty = array_chunk([], 2);
echo count($empty), "\n";

$call = "array_chunk";
$again = $call($items, 3);
echo count($again), "|", $again[0][0], "|", $again[0][2], "|", $again[1][0], "|", $again[1][1];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "3|2|2|1\nAda|five|two|zero two|next\nstring-key-reindexed\nafter\nAda|five|two|zero two|next\n1|5|next\n0\n2|Ada|two|zero two|next"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_chunk_requires_array_first_argument() {
    let error = runtime_error("<?php\necho array_chunk(42, 2);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): first argument must be array, got int"
    );
}

#[test]
fn array_chunk_requires_int_length_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_chunk($items, \"2\");\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): length argument must be int in the current subset, got string"
    );
}

#[test]
fn array_chunk_requires_positive_length_argument() {
    let error = runtime_error("<?php\n$items = [1];\necho array_chunk($items, 0);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): length argument must be greater than 0 in the current subset, got 0"
    );
}

#[test]
fn array_chunk_rejects_preserve_key_mode_until_implemented() {
    let error = runtime_error("<?php\n$items = [1];\necho array_chunk($items, 1, true);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_chunk(): preserve_keys mode is not supported in the current subset"
    );
}

#[test]
fn emit_ir_rejects_array_chunk_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_chunk([1], 1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
