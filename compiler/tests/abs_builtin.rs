use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn abs_executes_current_int_and_finite_float_subset() {
    let execution = run_source(
        r#"<?php
echo abs(-42), "|";
echo abs(7), "|";
echo abs(-2.5), "|";
echo abs("-42"), "|";
echo abs("-2.5"), "|";
echo abs(true), "|";
var_dump(abs(PHP_INT_MIN));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "42|7|2.5|42|2.5|1|float(9.223372036854776E+18)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn abs_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "abs";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call(-9);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|9");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn abs_rejects_forms_outside_current_subset() {
    let missing = run_source(
        r#"<?php
echo abs();
"#,
    )
    .unwrap();
    assert_eq!(missing.exit_code, 255);
    assert!(missing
        .stdout
        .contains("Too few arguments to function abs(), 0 passed"));

    let string = run_source(
        r#"<?php
try {
    echo abs("not numeric");
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        string.stdout,
        "abs(): Argument #1 ($num) must be of type int|float, string given"
    );

    let array = run_source(
        r#"<?php
try {
    echo abs([-1]);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array.stdout,
        "abs(): Argument #1 ($num) must be of type int|float, array given"
    );
}

#[test]
fn abs_null_emits_deprecation_and_returns_zero() {
    let execution = run_source(
        r#"<?php
var_dump(abs(null));
"#,
    )
    .unwrap();

    assert!(execution.stdout.starts_with(
        "Deprecated: abs(): Passing null to parameter #1 ($num) of type int|float is deprecated"
    ));
    assert!(execution.stdout.ends_with("int(0)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_abs_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho abs(-1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
