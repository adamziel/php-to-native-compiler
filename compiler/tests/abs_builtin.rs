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
fn abs_executes_current_int_and_finite_float_subset() {
    let execution = run_source(
        r#"<?php
echo abs(-42), "|";
echo abs(7), "|";
echo abs(-2.5);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "42|7|2.5");
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
    assert_eq!(missing.stderr, "");
    assert_eq!(missing.exit_code, 255);
    assert!(
        missing.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function abs(), 0 passed in Command line code on line 2 and exactly 1 expected"
        ),
        "{}",
        missing.stdout
    );

    let string = runtime_error(
        r#"<?php
echo abs("-42");
"#,
    );
    assert_eq!(string.line, 2);
    assert_eq!(string.column, 6);
    assert_eq!(
        string.message,
        "unsupported call abs(): argument must be int or finite float in the current subset, got string"
    );

    let array = runtime_error(
        r#"<?php
echo abs([-1]);
"#,
    );
    assert_eq!(array.line, 2);
    assert_eq!(array.column, 6);
    assert_eq!(
        array.message,
        "unsupported call abs(): argument must be int or finite float in the current subset, got array"
    );
}

#[test]
fn emit_ir_rejects_abs_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho abs(-1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
