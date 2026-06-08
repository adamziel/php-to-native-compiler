use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source, run_source_with_source_file};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn current_returns_first_ordered_array_value_or_false_for_empty_arrays() {
    let execution = run_source(
        r#"<?php
$items = array("name" => "Ada", 5 => "five", "2" => "two");
echo current($items), "|";
$items["name"] = "Grace";
echo current($items), "|";
var_dump(current(array()));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|Grace|bool(false)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn current_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "current";
$items = array("head", "tail");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|head");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn next_advances_direct_variable_array_pointers() {
    let execution = run_source(
        r#"<?php
$items = array("first", "second");
echo current($items), "|";
echo next($items), "|";
echo current($items), "|";
var_dump(next($items));
$call = "next";
$more = array("a", "b");
echo $call($more);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "first|second|second|bool(false)\nb");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn key_prev_reset_and_end_observe_and_mutate_array_pointers() {
    let execution = run_source(
        r#"<?php
$items = array("first" => "a", "second" => "b", "third" => "c");
echo key($items), "|", current($items), "|";
next($items);
echo key($items), "|", current($items), "|";
echo end($items), "|", key($items), "|";
echo prev($items), "|", key($items), "|";
echo reset($items), "|", key($items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "first|a|second|b|c|third|b|second|a|first"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn current_observes_append_after_exhausted_tail_unset() {
    let execution = run_source(
        r#"<?php
$array = ["foo" => 1, "bar" => 2, "baz" => 3];
reset($array);
while ($cur = current($array)) {
    var_dump($cur);
    next($array);
}

unset($array["baz"]);
$array[] = 4;
var_dump(current($array));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(1)\nint(2)\nint(3)\nint(4)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn next_advances_direct_object_property_array_offsets() {
    let execution = run_source(
        r#"<?php
class HookLike {
    public $iterations = array();

    public function run() {
        $level = 0;
        $this->iterations[$level] = array(10, 20);
        echo current($this->iterations[$level]), "|";
        echo next($this->iterations[$level]), "|";
        echo current($this->iterations[$level]);
    }
}

$hook = new HookLike();
$hook->run();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10|20|20");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn next_accepts_function_result_temporary_with_notice() {
    let execution = run_source_with_source_file(
        r#"<?php
function f() {
    return array(1, 2);
}
var_dump(next(f()));
"#,
        "/tmp/array_next_error1.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Notice: Only variables should be passed by reference in /tmp/array_next_error1.php on line 5\nint(2)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn next_array_literal_reports_php_reference_argument_fatal() {
    let execution = run_source_with_source_file(
        r#"<?php
function f() {
    return array(1, 2);
}
var_dump(next(array(1, 2)));
echo "after";
"#,
        "/tmp/array_next_error2.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: next(): Argument #1 ($array) could not be passed by reference in /tmp/array_next_error2.php:5\nStack trace:\n#0 {main}\n  thrown in /tmp/array_next_error2.php on line 5"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn current_rejects_forms_outside_current_subset() {
    let error = run_source("<?php\necho current(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call current(): argument must be array, got int"
    );

    let next_non_array = run_source("<?php\n$value = 42;\necho next($value);\n").unwrap_err();
    assert_eq!(next_non_array.phase, Phase::Runtime);
    assert_eq!(next_non_array.line, 3);
    assert_eq!(next_non_array.column, 6);
    assert_eq!(
        next_non_array.message,
        "unsupported call next(): argument must be array, got int"
    );

    let value_call = run_source(
        r#"<?php
function warn_ref($errno, $errstr) {
    echo str_contains($errstr, "must be passed by reference") ? "warning" : "other";
    echo "|";
    return true;
}
set_error_handler("warn_ref", E_WARNING);
$items = array("a", "b");
echo call_user_func("next", $items), "|", current($items);
"#,
    )
    .unwrap();
    assert_eq!(value_call.stdout, "warning|b|a");
    assert_eq!(value_call.exit_code, 0);
}

#[test]
fn emit_ir_folds_current_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("current") ? "1" : "0";
echo is_callable("current") ? "1" : "0";
echo function_exists("next") ? "1" : "0";
echo is_callable("next") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho current([1]);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);

    let error = emit_ir_source("<?php\necho next([1, 2]);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
