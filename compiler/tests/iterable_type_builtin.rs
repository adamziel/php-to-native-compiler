use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn is_iterable_matches_current_array_only_subset() {
    let execution = run_source(
        r#"<?php
class Box {}

$box = new Box();
$values = [null, false, true, 0, 3.5, "", [], [1], $box];
foreach ($values as $value) {
    echo is_iterable($value) ? "1" : "0";
}
echo "\n";
$call = "is_iterable";
echo $call([]) ? "1" : "0", $call("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "000000110\n10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_direct_scalar_null_is_iterable_calls_to_false() {
    let ir = emit_ir_source(
        r#"<?php
echo is_iterable(null) ? "1" : "0";
echo is_iterable(false) ? "1" : "0";
echo is_iterable(0) ? "1" : "0";
echo is_iterable(3.5) ? "1" : "0";
echo is_iterable("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("is_iterable"), "{ir}");
}

#[test]
fn emit_ir_rejects_array_is_iterable_until_native_array_lowering_exists() {
    let error = emit_ir_source("<?php\necho is_iterable([]) ? 1 : 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}

#[test]
fn emit_ir_rejects_dynamic_is_iterable_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
$call = "is_iterable";
echo $call([]) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
