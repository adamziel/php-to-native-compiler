use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn user_functions_use_local_scope_without_clobbering_globals() {
    let execution = run_source(
        r#"<?php
$value = "global";
function shadow($value) {
    $value = $value . "-local";
    echo $value, "\n";
}
shadow("arg");
echo $value, "\n";
function make_local() {
    $value = "function";
    return $value;
}
echo make_local(), "\n";
echo $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "arg-local\nglobal\nfunction\nglobal\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_functions_do_not_import_global_variables_implicitly() {
    let error = runtime_error(
        r#"<?php
$value = "global";
function read_value() {
    return $value;
}
echo read_value();
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 12);
    assert_eq!(error.message, "undefined variable '$value'");
}

#[test]
fn global_declaration_has_stable_unsupported_runtime_error() {
    let error = runtime_error(
        r#"<?php
$value = 1;
function read_global() {
    global $value;
    return $value;
}
echo read_global();
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 5);
    assert_eq!(
        error.message,
        "unsupported global declaration: importing globals into function scope is not implemented"
    );
}

#[test]
fn recursive_user_functions_can_return_values() {
    let execution = run_source(
        r#"<?php
function factorial($n) {
    if ($n <= 1) {
        return 1;
    }
    return $n * factorial($n - 1);
}
echo factorial(5), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "120\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_function_default_parameters_are_used_for_omitted_arguments() {
    let execution = run_source(
        r#"<?php
function greet($name = "world", $suffix = "!") {
    echo "hello ", $name, $suffix, "\n";
}
greet();
greet("Ada");
greet("Ada", ".");
function scale($value, $factor = 2, $offset = 1) {
    return $value * $factor + $offset;
}
echo scale(3), "\n";
echo scale(3, 4), "\n";
echo scale(3, 4, 5), "\n";
function default_items($items = ["first", "second" => 2]) {
    echo count($items), ":", $items[0], ":", $items["second"], "\n";
}
default_items();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "hello world!\nhello Ada!\nhello Ada.\n7\n13\n17\n2:first:2\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn default_parameter_values_can_reference_global_constants() {
    let execution = run_source(
        r#"<?php
define("RUNTIME_FACTOR", 3);
const BASE = "compiler";
function describe($label = BASE . ":" . ARRAY_FILTER_USE_KEY, $factor = RUNTIME_FACTOR + 1, $items = [BASE => ARRAY_FILTER_USE_BOTH]) {
    echo $label, "|", $factor, "|", $items["compiler"], "\n";
}
describe();
function late_default($value = LATE_DEFAULT) {
    return $value;
}
const LATE_DEFAULT = "late";
echo late_default(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "compiler:2|4|1\nlate\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_default_parameter_constant_reports_runtime_error_when_omitted() {
    let error = runtime_error(
        r#"<?php
function missing_default($value = MISSING_DEFAULT) {
    return $value;
}
echo missing_default();
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 35);
    assert_eq!(error.message, "undefined constant MISSING_DEFAULT");
}

#[test]
fn default_parameter_arity_errors_report_supported_range() {
    let too_few = runtime_error(
        r#"<?php
function label($value, $suffix = "!") {
    return $value . $suffix;
}
echo label();
"#,
    );

    assert_eq!(too_few.line, 5);
    assert_eq!(too_few.column, 6);
    assert_eq!(
        too_few.message,
        "arity mismatch for label(): expected 1 to 2 argument(s), got 0"
    );

    let too_many = runtime_error(
        r#"<?php
function label($value, $suffix = "!") {
    return $value . $suffix;
}
echo label("a", "b", "c");
"#,
    );

    assert_eq!(too_many.line, 5);
    assert_eq!(too_many.column, 6);
    assert_eq!(
        too_many.message,
        "arity mismatch for label(): expected 1 to 2 argument(s), got 3"
    );
}

#[test]
fn default_parameter_values_must_be_constant_expressions_in_current_subset() {
    let error = parse_error(
        r#"<?php
$fallback = "value";
function invalid($value = $fallback) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 27);
    assert_eq!(
        error.message,
        "default parameter values only support constant expressions in the current subset"
    );
}

#[test]
fn required_parameters_after_defaults_are_rejected_in_current_subset() {
    let error = parse_error(
        r#"<?php
function invalid($first = 1, $second) {
    return $second;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 30);
    assert_eq!(
        error.message,
        "required parameter cannot follow a default parameter in the current subset"
    );
}

#[test]
fn variadic_parameters_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function collect(...$items) {
    return count($items);
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 18);
    assert_eq!(
        error.message,
        "unsupported variadic parameter: variadics are not implemented"
    );
}

#[test]
fn reference_parameters_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function mutate(&$value) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 17);
    assert_eq!(
        error.message,
        "unsupported reference parameter: references are not implemented"
    );
}

#[test]
fn reference_returns_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function &identity($value) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "unsupported reference return: returning functions by reference is not implemented"
    );
}

#[test]
fn reference_expressions_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
$value = 1;
$alias =& $value;
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "unsupported reference expression: references are not implemented"
    );
}

#[test]
fn closures_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
$fn = function ($value) {
    return $value;
};
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "unsupported closure: anonymous functions are not implemented"
    );
}

#[test]
fn arrow_functions_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
$fn = fn($value) => $value;
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "unsupported closure: arrow functions are not implemented"
    );
}

#[test]
fn variadic_argument_unpacking_is_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function first($value) {
    return $value;
}
$items = [1];
echo first(...$items);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported argument unpacking: variadic calls are not implemented"
    );
}

#[test]
fn named_arguments_are_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
function greet($name) {
    return $name;
}
echo greet(name: "Ada");
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported named argument: named arguments are not implemented"
    );
}

#[test]
fn strict_types_declare_is_rejected_with_stable_parse_error() {
    let error = parse_error(
        r#"<?php
declare(strict_types=1);
function identity($value) {
    return $value;
}
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported declare directive: strict_types is not implemented"
    );
}
