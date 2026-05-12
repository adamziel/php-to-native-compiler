use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn lex_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Lex);
    error
}

#[test]
fn materialized_symbol_table_preserves_static_variable_behavior() {
    let execution = run_source(
        r#"<?php
$name = "Ada";
$label = $name . "-static";
$items = [];
$items["label"] = $label;
echo isset($name), "\n";
echo $items["label"], "\n";
function shadow($name = "local") {
    $name = $name . "-scope";
    return $name;
}
echo shadow(), "\n";
echo $name, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1\nAda-static\nlocal-scope\nAda\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn dynamic_function_calls_use_runtime_lookup_for_string_callees() {
    let execution = run_source(
        r#"<?php
function greet($name, $suffix = "!") {
    return "hello " . $name . $suffix;
}
$call = "greet";
echo $call("Ada"), "\n";
$upper = "GREET";
echo $upper("Lin", "."), "\n";
$length = "strlen";
echo $length("native"), "\n";
$counter = "count";
echo $counter(["a", "b"]), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hello Ada!\nhello Lin.\n6\n2\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unresolved_dynamic_function_name_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
$call = "missing";
echo $call();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined function missing()");
}

#[test]
fn dynamic_function_callee_must_be_string_in_current_subset() {
    let error = runtime_error(
        r#"<?php
$call = 123;
echo $call();
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call dynamic function call: callable expression must evaluate to string, got int"
    );
}

#[test]
fn variable_variables_are_rejected_with_stable_lex_error() {
    let error = lex_error(
        r#"<?php
$name = "value";
$$name = "dynamic";
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported variable variable: variable variables are not implemented"
    );
}

#[test]
fn include_require_constructs_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
include 'config.php';
"#,
            2,
            1,
            "unsupported include: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
include_once 'config.php';
"#,
            2,
            1,
            "unsupported include_once: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
require 'bootstrap.php';
"#,
            2,
            1,
            "unsupported require: include/require resolution and execution are not implemented",
        ),
        (
            r#"<?php
$ok = require_once 'bootstrap.php';
"#,
            2,
            7,
            "unsupported require_once: include/require resolution and execution are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn eval_constructs_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
eval('echo "dynamic";');
"#,
            2,
            1,
        ),
        (
            r#"<?php
$result = eval('return 1;');
"#,
            2,
            11,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported eval: eval parsing and caller-scope execution are not implemented"
        );
    }
}

#[test]
fn namespace_and_use_declarations_are_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
namespace App\Demo;
"#,
            2,
            1,
            "unsupported namespace declaration: namespace-aware name resolution is not implemented",
        ),
        (
            r#"<?php
namespace App\Demo {
    echo "blocked";
}
"#,
            2,
            1,
            "unsupported namespace declaration: namespace-aware name resolution is not implemented",
        ),
        (
            r#"<?php
use App\Demo\Service;
"#,
            2,
            1,
            "unsupported use declaration: namespace imports are not implemented",
        ),
        (
            r#"<?php
use function App\Demo\make_service;
"#,
            2,
            1,
            "unsupported use declaration: namespace imports are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn namespace_qualified_function_and_class_names_are_rejected_with_stable_parse_errors() {
    let function_cases = [
        (
            r#"<?php
App\fn();
"#,
            2,
            4,
        ),
        (
            r#"<?php
$result = \App\make();
"#,
            2,
            11,
        ),
        (
            r#"<?php
$result = namespace\make();
"#,
            2,
            11,
        ),
    ];

    for (source, line, column) in function_cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported namespace-qualified function name: namespace-aware function resolution is not implemented"
        );
    }

    let class_cases = [
        (
            r#"<?php
$box = new App\Box();
"#,
            2,
            15,
        ),
        (
            r#"<?php
$box = new \App\Box();
"#,
            2,
            12,
        ),
        (
            r#"<?php
$box = new namespace\Box();
"#,
            2,
            12,
        ),
    ];

    for (source, line, column) in class_cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported namespace-qualified class name: namespace-aware class resolution is not implemented"
        );
    }
}

#[test]
fn bare_global_constants_outside_the_narrow_builtin_slice_are_rejected() {
    let cases = [
        (
            r#"<?php
echo PHP_VERSION;
"#,
            2,
            6,
            "PHP_VERSION",
        ),
        (
            r#"<?php
echo array_filter([], "strlen", CUSTOM_FILTER_MODE);
"#,
            2,
            33,
            "CUSTOM_FILTER_MODE",
        ),
    ];

    for (source, line, column, name) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            format!(
                "unsupported global constant {name}: only ARRAY_FILTER_USE_KEY and ARRAY_FILTER_USE_BOTH are implemented in the current subset"
            )
        );
    }
}

#[test]
fn constant_builtin_resolves_the_current_builtin_constant_slice() {
    let execution = run_source(
        r#"<?php
echo constant("ARRAY_FILTER_USE_KEY"), "|", constant("ARRAY_FILTER_USE_BOTH"), "\n";
$name = "ARRAY_FILTER_USE_KEY";
echo constant($name), "\n";
$call = "constant";
echo $call("ARRAY_FILTER_USE_BOTH"), "\n";

function keep_named_key($key) {
    return $key === "name";
}

$items = ["name" => "Ada", "other" => "Lin"];
$filtered = array_filter($items, "keep_named_key", constant("ARRAY_FILTER_USE_KEY"));
print_r(array_keys($filtered));
echo count($filtered), "|", $filtered["name"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2|1\n2\n1\nArray\n(\n    [0] => name\n)\n1|Ada\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn constant_builtin_rejects_unknown_constant_names() {
    let error = runtime_error(
        r#"<?php
echo constant("PHP_VERSION");
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call constant(): only ARRAY_FILTER_USE_KEY and ARRAY_FILTER_USE_BOTH are implemented in the current subset, got PHP_VERSION"
    );
}

#[test]
fn constant_builtin_requires_string_names() {
    let error = runtime_error(
        r#"<?php
echo constant(42);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call constant(): name argument must be string in the current subset, got int"
    );
}

#[test]
fn emit_ir_rejects_constant_lookup_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho constant(\"ARRAY_FILTER_USE_KEY\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
