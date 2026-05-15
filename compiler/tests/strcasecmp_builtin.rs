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
fn strcasecmp_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo strcasecmp("UTF-8", "utf-8") === 0 ? "same" : "diff";
echo "\n";
echo strcasecmp("abc", "ABD") < 0 ? "lt" : "not";
echo "\n";
echo strcasecmp("beta", "ALPHA") > 0 ? "gt" : "not";
echo "\n";
echo strcasecmp(123, "123") === 0 ? "coerced" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "same\nlt\ngt\ncoerced");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strcasecmp_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strcasecmp";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call("UTF8", "utf8") === 0 ? "same" : "diff";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|same");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strcasecmp_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo strcasecmp("a");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for strcasecmp(): expected 2 argument(s), got 1"
    );

    let array_left = runtime_error(
        r#"<?php
echo strcasecmp(["a"], "a");
"#,
    );
    assert_eq!(array_left.line, 2);
    assert_eq!(array_left.column, 6);
    assert_eq!(
        array_left.message,
        "unsupported call strcasecmp(): first argument arrays are not implemented in the current subset"
    );

    let array_right = runtime_error(
        r#"<?php
echo strcasecmp("a", ["a"]);
"#,
    );
    assert_eq!(array_right.line, 2);
    assert_eq!(array_right.column, 6);
    assert_eq!(
        array_right.message,
        "unsupported call strcasecmp(): second argument arrays are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_rejects_strcasecmp_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho strcasecmp('A', 'a');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
