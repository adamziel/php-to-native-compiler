use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn min_executes_current_integer_subset() {
    let execution = run_source(
        r#"<?php
echo min(128, PHP_INT_MAX), "|";
echo min(5, -2, 9), "|";
echo PHP_INT_MAX > 0 ? "max" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "128|-2|max");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "min";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call(9, 4, 7);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn min_rejects_forms_outside_current_subset() {
    let array_arg = run_source("<?php\nmin([3, 2]);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call min(): array argument forms are not implemented in the current subset"
    );

    let float_arg = run_source("<?php\nmin(3, 2.5);\n").unwrap_err();
    assert_eq!(float_arg.phase, Phase::Runtime);
    assert_eq!(float_arg.line, 2);
    assert_eq!(float_arg.column, 1);
    assert_eq!(
        float_arg.message,
        "unsupported call min(): arguments must be integers in the current subset, got float"
    );

    let too_few = run_source("<?php\nmin(3);\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for min(): expected at least 2 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_min_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo defined("PHP_INT_MAX") ? "1" : "0";
echo function_exists("min") ? "1" : "0";
echo is_callable("min") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nmin(3, 2);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
