use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn trim_executes_current_default_mask_subset() {
    let execution = run_source(
        "<?php\n\
echo trim(\" \\t128M\\n\"), \"|\";\n\
echo trim(\"\\tabc\\n\"), \"|\";\n\
echo trim(null), \"|\";\n\
echo trim(42);\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "128M|abc||42");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trim_is_available_through_string_valued_calls() {
    let execution = run_source(
        "<?php\n\
$call = \"trim\";\n\
echo function_exists($call) ? \"yes\" : \"no\";\n\
echo \"|\";\n\
echo is_callable($call) ? \"callable\" : \"missing\";\n\
echo \"|\";\n\
echo $call(\" ABC \");\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|ABC");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn trim_rejects_forms_outside_current_subset() {
    let array_arg = run_source("<?php\ntrim(['ABC']);\n").unwrap_err();
    assert_eq!(array_arg.phase, Phase::Runtime);
    assert_eq!(array_arg.line, 2);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call trim(): arrays are not supported"
    );

    let custom_mask = run_source("<?php\ntrim('ABC', 'A');\n").unwrap_err();
    assert_eq!(custom_mask.phase, Phase::Runtime);
    assert_eq!(custom_mask.line, 2);
    assert_eq!(custom_mask.column, 1);
    assert_eq!(
        custom_mask.message,
        "unsupported call trim(): custom character masks are not implemented; pass exactly one argument in the current subset"
    );

    let too_few = run_source("<?php\ntrim();\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for trim(): expected 1 to 2 argument(s), got 0"
    );
}

#[test]
fn emit_ir_folds_trim_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        "<?php\n\
echo function_exists(\"trim\") ? \"1\" : \"0\";\n\
echo is_callable(\"trim\") ? \"1\" : \"0\";\n",
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\ntrim(' ABC ');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
