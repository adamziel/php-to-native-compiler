use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn call_user_func_invokes_current_string_callable_subset() {
    let execution = run_source(
        r#"<?php
function greet($name) {
    return "hi " . $name;
}
echo call_user_func("greet", "Ada"), "\n";
echo call_user_func("str_replace", " ", "_", "hello world"), "\n";
$call = "call_user_func";
echo $call("strlen", "four");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "hi Ada\nhello_world\n4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_invokes_current_string_callable_and_positional_array_subset() {
    let execution = run_source(
        r#"<?php
function join_names($first, $second = "Grace") {
    return $first . "+" . $second;
}
echo call_user_func_array("join_names", array("Ada", "Linus")), "\n";
echo call_user_func_array("str_replace", array(" ", "_", "hello world")), "\n";
$call = "call_user_func_array";
echo $call("strlen", array("four"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada+Linus\nhello_world\n4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_literal_reference_arguments_for_user_callbacks() {
    let execution = run_source(
        r#"<?php
function update_option_like(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$option = "autoload";
$callback = "update_option_like";
echo call_user_func_array($callback, array(&$option, "cache")), "\n";
echo $option;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "autoload:cache\nautoload:cache");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_binds_literal_reference_arguments_for_object_array_callbacks() {
    let execution = run_source(
        r#"<?php
class OptionFilter {
    public function update(&$value, $suffix) {
        $value = $value . ":" . $suffix;
        return $value;
    }
}

$filter = new OptionFilter();
$option = "autoload";
echo call_user_func_array(array($filter, "update"), array(&$option, "object-cache")), "\n";
echo $option;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "autoload:object-cache\nautoload:object-cache"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_array_invokes_current_array_callable_subset() {
    let execution = run_source(
        r#"<?php
class Formatter {
    public $prefix = ">";

    public function wrap($value) {
        return $this->prefix . $value;
    }

    public static function join($left, $right) {
        return $left . ":" . $right;
    }
}

$formatter = new Formatter();
echo call_user_func_array(array($formatter, "wrap"), array("item")), "\n";
echo call_user_func_array(array("Formatter", "join"), array("a", "b"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, ">item\na:b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_is_available_through_function_metadata_builtins() {
    let execution = run_source(
        r#"<?php
echo function_exists("call_user_func") ? "yes" : "no";
echo "|";
echo is_callable("call_user_func") ? "callable" : "missing";
echo "|";
echo function_exists("call_user_func_array") ? "yes" : "no";
echo "|";
echo is_callable("call_user_func_array") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|yes|callable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn call_user_func_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo call_user_func();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for call_user_func(): expected at least 1 argument(s), got 0"
    );

    let non_string = runtime_error(
        r#"<?php
echo call_user_func(42);
"#,
    );
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 6);
    assert_eq!(
        non_string.message,
        "unsupported call call_user_func(): callback must evaluate to string in the current subset, got int"
    );

    let array_callable = runtime_error(
        r#"<?php
echo call_user_func(["ClassName", "method"]);
"#,
    );
    assert_eq!(array_callable.line, 2);
    assert_eq!(array_callable.column, 6);
    assert_eq!(
        array_callable.message,
        "unsupported call call_user_func(): array callables are not implemented in the current subset"
    );

    let unknown = runtime_error(
        r#"<?php
echo call_user_func("missing_function");
"#,
    );
    assert_eq!(unknown.line, 2);
    assert_eq!(unknown.column, 6);
    assert_eq!(unknown.message, "undefined function missing_function()");

    let missing_array_arg = runtime_error(
        r#"<?php
echo call_user_func_array("strlen");
"#,
    );
    assert_eq!(missing_array_arg.line, 2);
    assert_eq!(missing_array_arg.column, 6);
    assert_eq!(
        missing_array_arg.message,
        "arity mismatch for call_user_func_array(): expected 2 argument(s), got 1"
    );

    let non_array_args = runtime_error(
        r#"<?php
echo call_user_func_array("strlen", "four");
"#,
    );
    assert_eq!(non_array_args.line, 2);
    assert_eq!(non_array_args.column, 6);
    assert_eq!(
        non_array_args.message,
        "unsupported call call_user_func_array(): argument array must be array in the current subset, got string"
    );

    let named_args = runtime_error(
        r#"<?php
echo call_user_func_array("strlen", array("value" => "four"));
"#,
    );
    assert_eq!(named_args.line, 2);
    assert_eq!(named_args.column, 6);
    assert_eq!(
        named_args.message,
        "unsupported call call_user_func_array(): string-keyed named arguments are not implemented in the current subset"
    );

    let bad_array_callable = runtime_error(
        r#"<?php
echo call_user_func_array(array("ClassName"), array());
"#,
    );
    assert_eq!(bad_array_callable.line, 2);
    assert_eq!(bad_array_callable.column, 6);
    assert_eq!(
        bad_array_callable.message,
        "unsupported call call_user_func_array(): array callback must be [object-or-class, method] in the current subset"
    );
}

#[test]
fn emit_ir_rejects_call_user_func_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho call_user_func('strlen', 'abc');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let metadata = emit_ir_source(
        r#"<?php
echo function_exists("call_user_func_array") ? "1" : "0";
echo is_callable("call_user_func_array") ? "1" : "0";
"#,
    )
    .unwrap();
    assert_eq!(metadata.matches("c\"1\\00\"").count(), 2, "{metadata}");

    let error =
        emit_ir_source("<?php\necho call_user_func_array('strlen', ['abc']);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
