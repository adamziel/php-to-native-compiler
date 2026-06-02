use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_unshift_mutates_direct_variable_arrays_and_reindexes_integer_keys() {
    let execution = run_source(
        r#"<?php
$items = array(2 => "two", "name" => "Ada", 5 => "five");
$count = array_unshift($items, "new", "first");
echo $count, "|";
echo $items[0], "|", $items[1], "|", $items[2], "|", $items["name"], "|", $items[3], "|";
$items[] = "tail";
echo $items[4];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5|new|first|two|Ada|five|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unshift_preserves_reference_backed_existing_slots() {
    let execution = run_source(
        r#"<?php
$value = "ref";
$items = ["first" => "plain"];
$items["ref"] =& $value;
$items[] = "tail";

$count = array_unshift($items, "head");
$value = "changed";
echo $count, "|", $items[0], "|", $items["first"], "|", $items["ref"], "|", $items[1], "\n";

$items["ref"] = "through-item";
echo $value;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "4|head|plain|changed|tail\nthrough-item");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unshift_is_available_through_string_valued_direct_calls() {
    let execution = run_source(
        r#"<?php
$call = "array_unshift";
$items = array("tail");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($items, "head");
echo "|";
echo $items[0], "|", $items[1];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|2|head|tail");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_unshift_rejects_forms_outside_current_subset() {
    let non_variable = run_source("<?php\narray_unshift(['tail'], 'head');\n").unwrap_err();
    assert_eq!(non_variable.phase, Phase::Runtime);
    assert_eq!(non_variable.line, 2);
    assert_eq!(non_variable.column, 1);
    assert_eq!(
        non_variable.message,
        "unsupported call array_unshift(): first argument must be a direct variable array path in the current subset"
    );

    let non_array = run_source("<?php\n$value = 1;\narray_unshift($value, 'head');\n").unwrap_err();
    assert_eq!(non_array.phase, Phase::Runtime);
    assert_eq!(non_array.line, 3);
    assert_eq!(non_array.column, 1);
    assert_eq!(
        non_array.message,
        "unsupported call array_unshift(): argument must be array, got int"
    );

    let value_call = run_source(
        r#"<?php
function warn_ref($errno, $errstr) {
    echo str_contains($errstr, "must be passed by reference") ? "warning" : "other";
    echo "|";
    return true;
}
set_error_handler("warn_ref", E_WARNING);
$items = array("tail");
echo call_user_func("array_unshift", $items, "head"), "|", implode(",", $items);
"#,
    )
    .unwrap();
    assert_eq!(value_call.stdout, "warning|2|tail");
    assert_eq!(value_call.exit_code, 0);
}

#[test]
fn array_push_and_shift_mutate_direct_variable_arrays() {
    let execution = run_source(
        r#"<?php
$items = array(2 => "two", "name" => "Ada", 5 => "five");
echo array_push($items, "tail", "end"), "|";
echo $items[6], "|", $items[7], "|";
echo array_shift($items), "|";
echo implode(",", array_keys($items)), "|";
echo array_push($items), "|";
echo count($items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5|tail|end|two|name,0,1,2|4|4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_push_shift_unshift_accept_direct_nested_array_paths_and_empty_spread() {
    let execution = run_source(
        r#"<?php
$items = array("outer" => array("a", "b"));
$empty = array();
echo array_push($items["outer"], ...$empty), "|";
echo array_push($items["outer"], array("nested")), "|";
echo array_shift($items["outer"]), "|";
echo array_unshift($items["outer"], ...$empty), "|";
echo array_unshift($items["outer"], "front"), "|";
var_dump($items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2|3|a|2|3|array(1) {\n  [\"outer\"]=>\n  array(3) {\n    [0]=>\n    string(5) \"front\"\n    [1]=>\n    string(1) \"b\"\n    [2]=>\n    array(1) {\n      [0]=>\n      string(6) \"nested\"\n    }\n  }\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_shift_by_value_expression_emits_notice_and_returns_copy() {
    let execution = run_source(
        r#"<?php
$stack = array(array(array("zero", "one"), "tail"), "after");
var_dump(array_shift(array_shift(array_shift($stack))));
echo "|", count($stack);
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Notice: Only variables should be passed by reference"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(4) \"zero\""),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.ends_with("|1"), "{}", execution.stdout);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_array_unshift_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("array_unshift") ? "1" : "0";
echo is_callable("array_unshift") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$items = 1;\narray_unshift($items, 'head');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
