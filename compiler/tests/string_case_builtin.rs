use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn strtolower_executes_current_ascii_string_subset() {
    let execution = run_source(
        r#"<?php
echo strtolower("Memory_Limit"), "|";
echo strtolower("128M"), "|";
echo strtolower(null), "|";
echo strtolower(42);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "memory_limit|128m||42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtolower_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strtolower";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("ABC");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|abc");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtolower_rejects_forms_outside_current_subset() {
    let array_arg = run_source("<?php\nstrtolower(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call strtolower(): arrays are not supported"
    );

    let too_many = run_source("<?php\nstrtolower('ABC', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for strtolower(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_strtolower_metadata_but_rejects_direct_case_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strtolower") ? "1" : "0";
echo is_callable("strtolower") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nstrtolower('ABC');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
