use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::{run_source, run_source_with_source_file};

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
fn parameter_type_declarations_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
function label(string $value) {
    return $value;
}
"#,
            16,
        ),
        (
            r#"<?php
function nullable(?string $value) {
    return $value;
}
"#,
            19,
        ),
        (
            r#"<?php
function union(int|string $value) {
    return $value;
}
"#,
            16,
        ),
        (
            r#"<?php
function intersection(Iterator&Countable $value) {
    return $value;
}
"#,
            23,
        ),
    ];

    for (source, column) in cases {
        let error = parse_error(source);

        assert_eq!(error.line, 2);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported parameter type declaration: parameter type enforcement is not implemented"
        );
    }
}

#[test]
fn return_type_declarations_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
function label($value): string {
    return $value;
}
"#,
            23,
        ),
        (
            r#"<?php
function nullable($value): ?string {
    return $value;
}
"#,
            26,
        ),
        (
            r#"<?php
function union($value): int|string {
    return $value;
}
"#,
            23,
        ),
        (
            r#"<?php
function void_result(): void {
}
"#,
            23,
        ),
    ];

    for (source, column) in cases {
        let error = parse_error(source);

        assert_eq!(error.line, 2);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported return type declaration: return type enforcement is not implemented"
        );
    }
}

#[test]
fn static_local_declarations_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
function counter() {
    static $count;
}
"#,
            3,
            5,
        ),
        (
            r#"<?php
function counter() {
    static $count = 0;
}
"#,
            3,
            5,
        ),
        (
            r#"<?php
function counter($enabled) {
    if ($enabled)
        static $count = 0;
}
"#,
            4,
            9,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);

        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported static local variable declaration: function-local static storage is not implemented"
        );
    }
}

#[test]
fn magic_line_constant_evaluates_from_expression_source_span() {
    let execution = run_source(
        r#"<?php
echo __LINE__, "\n";
$line = __LINE__;
echo $line, "\n";
function default_line($line = __LINE__) {
    echo $line, "\n";
    echo __LINE__, "\n";
}
const DECLARED_LINE = __LINE__;
default_line();
echo DECLARED_LINE, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2\n3\n5\n7\n9\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_line_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __LINE__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(
        error.message.contains("__LINE__") && error.message.contains("not LLVM IR emission yet"),
        "{}",
        error.message
    );
}

#[test]
fn magic_file_constant_evaluates_from_current_source_file_when_available() {
    let execution = run_source_with_source_file(
        r#"<?php
echo __FILE__, "\n";
$file = __FILE__;
echo $file, "\n";
function default_file($file = __FILE__) {
    echo $file, "\n";
}
const DECLARED_FILE = __FILE__;
default_file();
echo DECLARED_FILE, "\n";
"#,
        "virtual/input.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "virtual/input.php\nvirtual/input.php\nvirtual/input.php\nvirtual/input.php\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_file_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __FILE__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(
        error.message.contains("__FILE__") && error.message.contains("not LLVM IR emission yet"),
        "{}",
        error.message
    );
}

#[test]
fn magic_dir_constant_evaluates_from_current_source_file_directory_when_available() {
    let execution = run_source_with_source_file(
        r#"<?php
echo __DIR__, "\n";
$dir = __DIR__;
echo $dir, "\n";
function default_dir($dir = __DIR__) {
    echo $dir, "\n";
}
const DECLARED_DIR = __DIR__;
default_dir();
echo DECLARED_DIR, "\n";
"#,
        "virtual/input.php",
    )
    .unwrap();

    assert_eq!(execution.stdout, "virtual\nvirtual\nvirtual\nvirtual\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn magic_dir_constant_uses_dot_for_source_file_without_parent_directory() {
    let execution =
        run_source_with_source_file("<?php\necho __DIR__, \"\\n\";\n", "input.php").unwrap();

    assert_eq!(execution.stdout, ".\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_dir_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __DIR__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(
        error.message.contains("__DIR__") && error.message.contains("not LLVM IR emission yet"),
        "{}",
        error.message
    );
}

#[test]
fn magic_function_constant_evaluates_from_current_user_function_context() {
    let execution = run_source(
        r#"<?php
echo "top:", __FUNCTION__, "\n";
function current_name($default = __FUNCTION__) {
    echo "default:", $default, "\n";
    echo "body:", __FUNCTION__, "\n";
}
function caller() {
    current_name();
    echo "caller:", __FUNCTION__, "\n";
}
current_name("manual");
caller();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "top:\ndefault:manual\nbody:current_name\ndefault:current_name\nbody:current_name\ncaller:caller\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_magic_function_until_native_source_mapping_exists() {
    let error = emit_ir_source("<?php\necho __FUNCTION__;\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert!(
        error.message.contains("__FUNCTION__")
            && error.message.contains("not LLVM IR emission yet"),
        "{}",
        error.message
    );
}

#[test]
fn magic_constants_except_line_file_dir_and_function_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
class Box {
    public function label() {
        return __METHOD__;
    }
}
"#,
            4,
            16,
            "__METHOD__",
        ),
        (
            r#"<?php
class Box {
    public function label() {
        return __CLASS__;
    }
}
"#,
            4,
            16,
            "__CLASS__",
        ),
        (
            r#"<?php
echo __TRAIT__;
"#,
            2,
            6,
            "__TRAIT__",
        ),
        (
            r#"<?php
echo __NAMESPACE__;
"#,
            2,
            6,
            "__NAMESPACE__",
        ),
    ];

    for (source, line, column, magic_name) in cases {
        let error = parse_error(source);

        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            format!(
                "unsupported magic constant {magic_name}: source-aware magic constant evaluation is not implemented"
            )
        );
    }
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
