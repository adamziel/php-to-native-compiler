use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_pop_mutates_direct_variable_arrays_and_returns_last_value() {
    let execution = run_source(
        r#"<?php
$items = array(2 => "two", "name" => "Ada", 5 => "five");
echo array_pop($items), "|";
echo count($items), "|";
$items[] = "new";
echo $items[5], "|";
echo array_pop($items), "|";
echo array_pop($items), "|";
var_dump(array_pop($items));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "five|2|new|new|Ada|string(3) \"two\"\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_pop_is_available_through_string_valued_direct_calls() {
    let execution = run_source(
        r#"<?php
$call = "array_pop";
$items = array("head", "tail");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($items);
echo "|";
echo count($items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|tail|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_pop_rejects_forms_outside_current_subset() {
    let non_variable = run_source("<?php\narray_pop(['tail']);\n").unwrap_err();
    assert_eq!(non_variable.phase, Phase::Runtime);
    assert_eq!(non_variable.line, 2);
    assert_eq!(non_variable.column, 1);
    assert_eq!(
        non_variable.message,
        "unsupported call array_pop(): argument must be a direct variable array in the current subset"
    );

    let non_array = run_source("<?php\n$value = 1;\narray_pop($value);\n").unwrap_err();
    assert_eq!(non_array.phase, Phase::Runtime);
    assert_eq!(non_array.line, 3);
    assert_eq!(non_array.column, 1);
    assert_eq!(
        non_array.message,
        "unsupported call array_pop(): argument must be array, got int"
    );

    let value_call = run_source(
        r#"<?php
$items = array("tail");
call_user_func("array_pop", $items);
"#,
    )
    .unwrap_err();
    assert_eq!(value_call.phase, Phase::Runtime);
    assert_eq!(value_call.line, 3);
    assert_eq!(value_call.column, 1);
    assert_eq!(
        value_call.message,
        "unsupported call array_pop(): by-reference array arguments require a direct call target in the current subset"
    );
}

#[test]
fn emit_ir_folds_array_pop_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("array_pop") ? "1" : "0";
echo is_callable("array_pop") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$items = 1;\narray_pop($items);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
