use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn microtime_true_returns_current_float_seconds() {
    let execution = run_source(
        r#"<?php
$time = microtime(true);
echo is_float($time) ? "float" : "other";
echo "|";
echo $time > 0 ? "positive" : "non-positive";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "float|positive");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn microtime_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "microtime";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo is_float($call(true)) ? "float" : "other";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|float");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn microtime_string_mode_returns_php_shape_and_rejects_non_bool_flag() {
    let execution = run_source(
        r#"<?php
foreach ([microtime(), microtime(false)] as $value) {
    echo is_string($value) ? "string" : "other";
    echo "|", str_starts_with($value, "0.") ? "fraction" : "bad";
    echo "|", strpos($value, " ") !== false ? "space" : "bad", "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string|fraction|space\nstring|fraction|space\n"
    );
    assert_eq!(execution.exit_code, 0);

    let non_bool = run_source("<?php\nmicrotime(1);\n").unwrap_err();
    assert_eq!(non_bool.phase, Phase::Runtime);
    assert_eq!(non_bool.line, 2);
    assert_eq!(non_bool.column, 1);
    assert_eq!(
        non_bool.message,
        "unsupported call microtime(): as_float argument must be bool in the current subset, got int"
    );
}

#[test]
fn sleep_negative_seconds_is_catchable_value_error_without_widening_types() {
    let execution = run_source(
        r#"<?php
try {
    sleep(-10);
} catch (ValueError $e) {
    echo $e::class, ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:sleep(): Argument #1 ($seconds) must be greater than or equal to 0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let uncaught = run_source("<?php\nsleep(-10);\n").unwrap();
    assert_eq!(uncaught.exit_code, 255);
    assert_eq!(
        uncaught.stdout,
        "Fatal error: Uncaught ValueError: sleep(): Argument #1 ($seconds) must be greater than or equal to 0 in Command line code:2\nStack trace:\n#0 Command line code(2): sleep(-10)\n#1 {main}\n  thrown in Command line code on line 2"
    );

    let unsupported = run_source("<?php\nsleep(1.5);\n").unwrap_err();
    assert_eq!(unsupported.phase, Phase::Runtime);
    assert_eq!(unsupported.line, 2);
    assert_eq!(unsupported.column, 1);
    assert_eq!(
        unsupported.message,
        "unsupported call sleep(): seconds argument must be int in the current subset, got float"
    );
}

#[test]
fn usleep_zero_is_callable_and_returns_null() {
    let execution = run_source(
        r#"<?php
$call = "usleep";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
var_dump($call(0));
$function = new ReflectionFunction("usleep");
$params = $function->getParameters();
echo $function->getName(), "|";
echo $function->getReturnType()->getName(), "|";
echo $params[0]->getName(), ":", $params[0]->getType()->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|NULL\nusleep|void|microseconds:int"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn usleep_negative_microseconds_raises_php_value_error() {
    let execution = run_source(
        r#"<?php
try {
    usleep(-10);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError:usleep(): Argument #1 ($microseconds) must be greater than or equal to 0"
    );
    assert_eq!(execution.exit_code, 0);

    let fatal = run_source("<?php\nusleep(-10);\n").unwrap();
    assert_eq!(
        fatal.stdout,
        "Fatal error: Uncaught ValueError: usleep(): Argument #1 ($microseconds) must be greater than or equal to 0 in Command line code:2\nStack trace:\n#0 Command line code(2): usleep(-10)\n#1 {main}\n  thrown in Command line code on line 2"
    );
    assert_eq!(fatal.exit_code, 255);
}

#[test]
fn emit_ir_folds_microtime_metadata_but_rejects_direct_time_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("microtime") ? "1" : "0";
echo is_callable("microtime") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nmicrotime(true);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
