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
fn microtime_rejects_forms_outside_current_subset() {
    let no_arg = run_source("<?php\nmicrotime();\n").unwrap_err();
    assert_eq!(no_arg.phase, Phase::Runtime);
    assert_eq!(no_arg.line, 2);
    assert_eq!(no_arg.column, 1);
    assert_eq!(
        no_arg.message,
        "unsupported call microtime(): string return format is not implemented; pass true for float seconds in the current subset"
    );

    let false_arg = run_source("<?php\nmicrotime(false);\n").unwrap_err();
    assert_eq!(false_arg.phase, Phase::Runtime);
    assert_eq!(false_arg.line, 2);
    assert_eq!(false_arg.column, 1);
    assert_eq!(
        false_arg.message,
        "unsupported call microtime(): string return format is not implemented; pass true for float seconds in the current subset"
    );

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
