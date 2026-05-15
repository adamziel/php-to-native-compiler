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
fn sprintf_executes_current_string_placeholder_subset() {
    let execution = run_source(
        r#"<?php
echo sprintf("Hello %s", "Ada"), "\n";
echo sprintf('%2$s:%1$s', "one", "two"), "\n";
echo sprintf('%% %s %1$s', "done"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Hello Ada\ntwo:one\n% done done\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "sprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", $call('%1$s-%2$s', "wp", "php");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|wp-php");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_rejects_format_forms_outside_current_subset() {
    let format = runtime_error(
        r#"<?php
echo sprintf(42, "x");
"#,
    );
    assert_eq!(format.line, 2);
    assert_eq!(format.column, 6);
    assert_eq!(
        format.message,
        "unsupported call sprintf(): format argument must be string in the current subset, got int"
    );

    let placeholder = runtime_error(
        r#"<?php
echo sprintf("%d", 4);
"#,
    );
    assert_eq!(placeholder.line, 2);
    assert_eq!(placeholder.column, 6);
    assert_eq!(
        placeholder.message,
        "unsupported call sprintf(): unsupported format placeholder %d in the current subset"
    );

    let missing = runtime_error(
        r#"<?php
echo sprintf('%2$s', "only-one");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "unsupported call sprintf(): missing argument for placeholder 2"
    );
}

#[test]
fn emit_ir_rejects_sprintf_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo sprintf("Hello %s", "Ada");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
