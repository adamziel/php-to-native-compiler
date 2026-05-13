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
fn bare_global_constants_resolve_runtime_defined_and_builtin_values() {
    let execution = run_source(
        r#"<?php
define("APP_NAME", "compiler");
define("APP_VERSION", 2);
echo APP_NAME, "|", APP_VERSION + 3, "\n";
echo ARRAY_FILTER_USE_KEY, "|", ARRAY_FILTER_USE_BOTH, "\n";

$items = ["name" => "Ada", "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "|", $copy["name"], "\n";

function read_constant_inside_function() {
    define("FUNCTION_CONSTANT", "inside");
    return APP_NAME . ":" . FUNCTION_CONSTANT;
}

echo read_constant_inside_function(), "\n";
$call = "define";
$call("DYNAMIC_CONSTANT", "dynamic");
echo DYNAMIC_CONSTANT, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|5\n2|1\nAda|1|changed\ncompiler:inside\ndynamic\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_bare_global_constants_have_stable_runtime_errors() {
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
        let error = runtime_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, format!("undefined constant {name}"));
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
        "unsupported call constant(): constant PHP_VERSION is not defined in the current runtime-defined or built-in constant subset"
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
fn defined_builtin_introspects_current_constant_table() {
    let execution = run_source(
        r#"<?php
echo defined("ARRAY_FILTER_USE_KEY"), "|", defined("ARRAY_FILTER_USE_BOTH"), "\n";
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
define("APP_NAME", "compiler");
echo defined("APP_NAME"), "|", defined("MISSING_CONST"), "\n";
$call = "defined";
echo $call("APP_NAME"), "|", $call("MISSING_CONST"), "\n";

function check_defined_inside_function() {
    define("INSIDE_DEFINED", 1);
    return defined("INSIDE_DEFINED") . ":" . defined("APP_NAME");
}

echo check_defined_inside_function(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|1\n|\n1|\n1|\n1:1\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn defined_builtin_requires_string_names_and_supported_names() {
    let non_string = runtime_error(
        r#"<?php
echo defined(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call defined(): name argument must be string in the current subset, got int"
    );

    let bad_name = runtime_error(
        r#"<?php
echo defined("123BAD");
"#,
    );
    assert_eq!(bad_name.line, 2);
    assert_eq!(bad_name.column, 6);
    assert_eq!(
        bad_name.message,
        "unsupported call defined(): constant name must be a non-empty unqualified identifier in the current subset, got 123BAD"
    );
}

#[test]
fn top_level_const_declarations_populate_constant_table() {
    let execution = run_source(
        r#"<?php
const APP_NAME = "compiler";
CONST APP_VERSION = 2;
const APP_SCALE = 1 + 2 * 3;
const APP_ITEMS = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "\n";
echo constant("APP_NAME"), "|", defined("APP_ITEMS"), "|", defined("MISSING_CONST"), "\n";
$copy = APP_ITEMS;
$copy["name"] = "changed";
echo count($copy), "|", $copy["name"], "|", APP_ITEMS["name"], "|", APP_ITEMS["nested"]["x"], "\n";
function read_declared_const() {
    return APP_NAME . ":" . APP_VERSION;
}
echo read_declared_const(), "\n";
$name = "APP_NAME";
echo constant($name), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|2|7\ncompiler|1|\n3|changed|Ada|1\ncompiler:2\ncompiler\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn grouped_top_level_const_declarations_execute_left_to_right() {
    let execution = run_source(
        r#"<?php
const APP_NAME = "compiler", APP_VERSION = 2, APP_SCALE = 1 + 2 * 3;
CONST APP_FLAGS = ["env" => "dev", "nested" => ["x" => 1]], APP_EMPTY = [];
echo APP_NAME, "|", APP_VERSION, "|", APP_SCALE, "|", defined("APP_EMPTY"), "\n";
$copy = APP_FLAGS;
$copy["env"] = "prod";
echo $copy["env"], "|", APP_FLAGS["env"], "|", APP_FLAGS["nested"]["x"], "\n";
function read_grouped_const() {
    return APP_NAME . ":" . APP_VERSION . ":" . APP_FLAGS["nested"]["x"];
}
echo read_grouped_const(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler|2|7|1\nprod|dev|1\ncompiler:2:1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn top_level_const_declaration_values_can_reference_prior_constants() {
    let execution = run_source(
        r#"<?php
define("RUNTIME_BASE", 3);
const FROM_DEFINE = RUNTIME_BASE + 1;
const BASE = "compiler";
const VERSION = 2, DOUBLE_VERSION = VERSION * 2, LABEL = BASE . ":" . DOUBLE_VERSION;
const FILTER_MODE = ARRAY_FILTER_USE_BOTH;
const ITEMS = [BASE => LABEL, "mode" => FILTER_MODE, "key-mode" => ARRAY_FILTER_USE_KEY, "from-define" => FROM_DEFINE];
echo LABEL, "|", FILTER_MODE, "|", ITEMS["compiler"], "|", ITEMS["mode"], "|", ITEMS["key-mode"], "|", ITEMS["from-define"], "\n";
function read_referenced_const() {
    return LABEL . ":" . ARRAY_FILTER_USE_KEY;
}
echo read_referenced_const(), "\n";
$name = "DOUBLE_VERSION";
echo constant($name), "|", FROM_DEFINE, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "compiler:4|1|compiler:4|1|2|4\ncompiler:4:2\n4|4\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn forward_const_declaration_references_have_stable_runtime_diagnostics() {
    let error = runtime_error(
        r#"<?php
const FORWARD = LATER, LATER = "done";
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 17);
    assert_eq!(error.message, "undefined constant LATER");
}

#[test]
fn duplicate_top_level_const_declarations_have_stable_diagnostics() {
    let duplicate = runtime_error(
        r#"<?php
const APP_NAME = "compiler";
const APP_NAME = "again";
"#,
    );
    assert_eq!(duplicate.line, 3);
    assert_eq!(duplicate.column, 1);
    assert_eq!(duplicate.message, "constant APP_NAME is already defined");

    let builtin = runtime_error(
        r#"<?php
const ARRAY_FILTER_USE_KEY = 4;
"#,
    );
    assert_eq!(builtin.line, 2);
    assert_eq!(builtin.column, 1);
    assert_eq!(
        builtin.message,
        "constant ARRAY_FILTER_USE_KEY is already defined"
    );

    let grouped = runtime_error(
        r#"<?php
const APP_NAME = "compiler", APP_VERSION = 1, APP_NAME = "again";
"#,
    );
    assert_eq!(grouped.line, 2);
    assert_eq!(grouped.column, 47);
    assert_eq!(grouped.message, "constant APP_NAME is already defined");
}

#[test]
fn unsupported_const_declaration_forms_have_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
if (true) {
    const INSIDE = 1;
}
"#,
            3,
            5,
            "unsupported const declaration: only top-level constant declarations are implemented",
        ),
        (
            r#"<?php
const APP_NAME = $name;
"#,
            2,
            18,
            "const declaration values only support constant expressions in the current subset",
        ),
        (
            r#"<?php
class Box {}
const BOX = new Box();
"#,
            3,
            13,
            "const declaration values only support constant expressions in the current subset",
        ),
        (
            r#"<?php
const APP\NAME = 1;
"#,
            2,
            10,
            "unsupported const declaration: namespace-qualified constant declarations are not implemented",
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
fn define_builtin_populates_runtime_constant_table() {
    let execution = run_source(
        r#"<?php
define("APP_NAME", "compiler");
echo define("APP_VERSION", 1), "\n";
echo constant("APP_NAME"), "|", constant("APP_VERSION"), "\n";

$items = ["name" => "Ada", "count" => 2, "nested" => ["x" => 1]];
define("APP_ITEMS", $items);
$copy = constant("APP_ITEMS");
$copy["name"] = "changed";
$again = constant("APP_ITEMS");
echo count($copy), "|", $copy["name"], "|", $again["name"], "|", $again["nested"]["x"], "\n";

function constant_scope() {
    define("INSIDE_FUNCTION", "inside");
    return constant("APP_NAME") . ":" . constant("INSIDE_FUNCTION");
}

echo constant_scope(), "\n";
$call = "define";
echo $call("DYNAMIC_NAME", "dynamic"), "\n";
echo constant("DYNAMIC_NAME"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1\ncompiler|1\n3|changed|Ada|1\ncompiler:inside\n1\ndynamic\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn duplicate_runtime_constant_definitions_have_stable_diagnostics() {
    let error = runtime_error(
        r#"<?php
define("APP_NAME", "compiler");
define("APP_NAME", "again");
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "constant APP_NAME is already defined");
}

#[test]
fn define_rejects_builtin_constant_redefinition() {
    let error = runtime_error(
        r#"<?php
define("ARRAY_FILTER_USE_KEY", 4);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "constant ARRAY_FILTER_USE_KEY is already defined"
    );
}

#[test]
fn define_requires_string_names_and_supported_values() {
    let non_string = runtime_error(
        r#"<?php
define(42, "bad");
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call define(): name argument must be string in the current subset, got int"
    );

    let bad_name = runtime_error(
        r#"<?php
define("123BAD", "bad");
"#,
    );
    assert_eq!(bad_name.line, 2);
    assert_eq!(bad_name.column, 1);
    assert_eq!(
        bad_name.message,
        "unsupported call define(): constant name must be a non-empty unqualified identifier in the current subset, got 123BAD"
    );

    let unsupported_value = runtime_error(
        r#"<?php
class Box {}
define("BOX", new Box());
"#,
    );
    assert_eq!(unsupported_value.line, 3);
    assert_eq!(unsupported_value.column, 1);
    assert_eq!(
        unsupported_value.message,
        "unsupported call define(): value must be null, bool, int, float, string, or array values in the current subset, got object"
    );
}

#[test]
fn define_rejects_case_insensitive_legacy_flag() {
    let error = runtime_error(
        r#"<?php
define("APP_NAME", "compiler", true);
"#,
    );

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call define(): case-insensitive constant definitions are not implemented; pass exactly two arguments in the current subset"
    );
}

#[test]
fn emit_ir_rejects_constant_lookup_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho constant(\"ARRAY_FILTER_USE_KEY\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_define_until_user_constant_lowering_exists() {
    let error = emit_ir_source("<?php\ndefine(\"APP_NAME\", \"compiler\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_defined_until_constant_introspection_lowering_exists() {
    let error = emit_ir_source("<?php\necho defined(\"APP_NAME\");\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}

#[test]
fn emit_ir_rejects_const_declarations_until_native_lowering_exists() {
    let error =
        emit_ir_source("<?php\nconst APP_NAME = \"compiler\", APP_VERSION = 2;\necho APP_NAME;\n")
            .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("global-constant lowering"),
        "{}",
        error.message
    );
}
