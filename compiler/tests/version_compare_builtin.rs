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
fn version_compare_executes_current_numeric_component_subset() {
    let execution = run_source(
        r#"<?php
echo version_compare("7.2.24", "8.3.0"), "\n";
echo version_compare("8.3", "8.3.0"), "\n";
echo version_compare("8.3.1", "8.3.0"), "\n";
echo version_compare("7.2.24", PHP_VERSION, "<") ? "lt" : "ge";
echo "|", version_compare(PHP_VERSION, PHP_VERSION, "ge") ? "ge" : "lt";
echo "|", version_compare("8-3-0", "8.3.0", "==") ? "eq" : "ne";
echo "|", version_compare("8_3_1", "8.3.0", "ne") ? "ne" : "eq";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "-1\n-1\n1\nlt|ge|eq|ne");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn version_compare_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "version_compare";
echo function_exists($call) ? "yes" : "no";
echo "|", $call("8.3.0", "8.3.1", "<") ? "ok" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|ok");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn version_compare_rejects_inputs_outside_current_subset() {
    let first = runtime_error(
        r#"<?php
echo version_compare(80300, "8.3.0");
"#,
    );
    assert_eq!(first.line, 2);
    assert_eq!(first.column, 6);
    assert_eq!(
        first.message,
        "unsupported call version_compare(): first version argument must be string in the current subset, got int"
    );

    let prerelease = runtime_error(
        r#"<?php
echo version_compare("8.3.0RC1", "8.3.0");
"#,
    );
    assert_eq!(prerelease.line, 2);
    assert_eq!(prerelease.column, 6);
    assert_eq!(
        prerelease.message,
        "unsupported call version_compare(): version strings must use dot, hyphen, or underscore separated non-negative integer components in the current subset"
    );

    let operator = runtime_error(
        r#"<?php
echo version_compare("8.3.0", "8.3.1", "newer");
"#,
    );
    assert_eq!(operator.line, 2);
    assert_eq!(operator.column, 6);
    assert_eq!(
        operator.message,
        "unsupported call version_compare(): unsupported operator newer in the current subset"
    );
}

#[test]
fn emit_ir_rejects_version_compare_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo version_compare("8.3.0", "8.3.1", "<");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
