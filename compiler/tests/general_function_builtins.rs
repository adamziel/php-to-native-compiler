use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn platform_and_version_functions_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["a", "s", "n", "r", "v", "m"] as $mode) {
    $value = php_uname($mode);
    echo is_string($value) && strlen($value) > 0 ? $mode : "bad";
}
echo "|";
echo php_uname() === php_uname("a") ? "default" : "bad";
echo "|";
echo phpversion();
echo "|";
echo phpversion("standard");
echo "|";
var_dump(phpversion("not_loaded"));
echo "|";
$pid = getmypid();
echo is_int($pid) && $pid > 0 && $pid === getmypid() ? "pid" : "bad";
echo "|";
$pid_fn = new ReflectionFunction("getmypid");
echo $pid_fn->getNumberOfRequiredParameters(), "/", $pid_fn->getNumberOfParameters();
echo "|";
foreach (["getmypid", "php_uname", "phpversion"] as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "asnrvm|default|8.3.0|8.3.0|bool(false)\n|pid|0/0|111111"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_extension_funcs_returns_bounded_standard_function_metadata() {
    let execution = run_source(
        r#"<?php
echo "Simple testcase for get_extension_funcs() function\n";
$result = get_extension_funcs("standard");
var_dump(gettype($result));
var_dump(in_array("cos", $result));
var_dump(get_extension_funcs("foo"));
var_dump(in_array("strlen", get_extension_funcs("STANDARD")));
var_dump(in_array("get_defined_functions", get_extension_funcs("standard")));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Simple testcase for get_extension_funcs() function\n\
string(5) \"array\"\n\
bool(true)\n\
bool(false)\n\
bool(true)\n\
bool(true)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn get_defined_functions_reports_bounded_internal_and_user_metadata() {
    let execution = run_source(
        r#"<?php
function foo() {}
function HelloWorld() {}

class C {
    function f1() {}
    static function f2() {}
}

$func = get_defined_functions();
echo gettype($func), "|", gettype($func["internal"]), "|", gettype($func["user"]), "\n";
echo in_array("cos", $func["internal"]) ? "cos" : "missing-cos";
echo "|", in_array("strlen", $func["internal"]) ? "strlen" : "missing-strlen";
echo "|", in_array("get_defined_functions", $func["internal"]) ? "self" : "missing-self";
echo "|", in_array("function_exists", $func["internal"]) ? "core" : "missing-core";
echo "\n";
$user = $func["user"];
echo count($user), "|";
echo in_array("foo", $user) ? "foo" : "missing-foo";
echo "|", in_array("helloworld", $user) ? "helloworld" : "missing-helloworld";
echo "|", in_array("f1", $user) ? "method" : "no-method";
echo "\n";
$withFlag = get_defined_functions(false);
echo in_array("cos", $withFlag["internal"]) ? "flag-ok" : "flag-bad";
echo "|", function_exists("get_defined_functions") ? "exists" : "missing";
echo "|", is_callable("get_defined_functions") ? "callable" : "not-callable";
$reflection = new ReflectionFunction("get_defined_functions");
echo "|", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
echo "|", $reflection->getParameters()[0]->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array|array|array\n\
cos|strlen|self|core\n\
2|foo|helloworld|no-method\n\
flag-ok|exists|callable|0/1|exclude_disabled"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn getmypid_rejects_arguments() {
    let error = run_source("<?php\ngetmypid(1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "arity mismatch for getmypid(): expected 0 argument(s), got 1"
    );
}

#[test]
fn php_uname_invalid_modes_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (["", "test", "z"] as $mode) {
    try {
        php_uname($mode);
    } catch (Throwable $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError: php_uname(): Argument #1 ($mode) must be a single character\n\
ValueError: php_uname(): Argument #1 ($mode) must be a single character\n\
ValueError: php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_platform_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("php_uname") ? "1" : "0";
echo is_callable("phpversion") ? "1" : "0";
echo function_exists("getmypid") ? "1" : "0";
echo function_exists("get_defined_functions") ? "1" : "0";
echo is_callable("get_defined_functions") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho php_uname();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho phpversion();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho getmypid();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho get_defined_functions();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
