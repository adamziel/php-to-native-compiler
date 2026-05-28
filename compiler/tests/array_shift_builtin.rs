use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;
use php_compiler::run_source_with_source_file;

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_shift_mutates_direct_variable_arrays_and_returns_first_value() {
    let execution = run_source(
        r#"<?php
$items = array(2 => "two", "name" => "Ada", 5 => "five");
echo array_shift($items), "|";
echo count($items), "|";
echo $items[0], "|", $items["name"], "|";
var_dump(array_shift($items));
var_dump(array_shift($items));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "two|2|five|Ada|string(3) \"Ada\"\nstring(4) \"five\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_shift_call_result_warns_and_shifts_temporary_array() {
    let execution = run_source_with_source_file(
        r#"<?php
$stack = array(array("two"));
var_dump(array_shift(array_shift($stack)));
$original = array(array("one"));
$stack = $original;
var_dump(array_shift(array_shift($stack)));
echo $original[0][0];
"#,
        "virtual/pass012.php",
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Notice: Only variables should be passed by reference in virtual/pass012.php on line 3"
    ));
    assert!(execution.stdout.contains(
        "Notice: Only variables should be passed by reference in virtual/pass012.php on line 6"
    ));
    let semantic_lines = execution
        .stdout
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("Notice:"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(semantic_lines, "string(3) \"two\"\nstring(3) \"one\"\none");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_shift_is_available_through_string_valued_direct_calls() {
    let execution = run_source(
        r#"<?php
$call = "array_shift";
$items = array("head", "tail");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($items);
echo "|";
echo count($items), "|", $items[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|head|1|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_shift_rejects_non_array_direct_variables() {
    let non_array = run_source("<?php\n$value = 1;\narray_shift($value);\n").unwrap_err();
    assert_eq!(non_array.phase, Phase::Runtime);
    assert_eq!(non_array.line, 3);
    assert_eq!(non_array.column, 1);
    assert_eq!(
        non_array.message,
        "unsupported call array_shift(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_folds_array_shift_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("array_shift") ? "1" : "0";
echo is_callable("array_shift") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$items = 1;\narray_shift($items);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
