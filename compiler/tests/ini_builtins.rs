use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn ini_get_reads_current_deterministic_registry() {
    let execution = run_source(
        r#"<?php
echo ini_get("memory_limit"), "|";
echo ini_get("MEMORY_LIMIT"), "|";
echo ini_get("mbstring.func_overload"), "|";
echo ini_get("missing.option") === false ? "false" : "not-false";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|128M|0|false");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_get_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "ini_get";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("memory_limit");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|128M");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ini_get_rejects_forms_outside_current_subset() {
    let non_string = run_source("<?php\nini_get(42);\n").unwrap_err();
    assert_eq!(non_string.phase, Phase::Runtime);
    assert_eq!(non_string.line, 2);
    assert_eq!(non_string.column, 1);
    assert_eq!(
        non_string.message,
        "unsupported call ini_get(): option argument must be string in the current subset, got int"
    );

    let too_many = run_source("<?php\nini_get('memory_limit', true);\n").unwrap_err();
    assert_eq!(too_many.phase, Phase::Runtime);
    assert_eq!(too_many.line, 2);
    assert_eq!(too_many.column, 1);
    assert_eq!(
        too_many.message,
        "arity mismatch for ini_get(): expected 1 argument(s), got 2"
    );
}

#[test]
fn emit_ir_folds_ini_get_metadata_but_rejects_direct_ini_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("ini_get") ? "1" : "0";
echo is_callable("ini_get") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nini_get('memory_limit');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
