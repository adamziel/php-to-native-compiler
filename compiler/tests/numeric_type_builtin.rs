use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn is_numeric_matches_current_numeric_scalar_subset() {
    let execution = run_source(
        r#"<?php
class Box {}

$box = new Box();
$values = [
    0,
    -7,
    3.5,
    "0",
    " 42 ",
    "+8",
    "-.5",
    "5.",
    "8e2",
    "",
    " ",
    "8foo",
    "0x10",
    true,
    null,
    ["1"],
    $box,
];
foreach ($values as $value) {
    echo is_numeric($value) ? "1" : "0";
}
echo "\n";
$call = "is_numeric";
echo $call("10.5") ? "1" : "0", $call("text") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111111100000000\n10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_direct_scalar_null_is_numeric_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo is_numeric(0) ? "1" : "0";
echo is_numeric(3.5) ? "1" : "0";
echo is_numeric(" 42 ") ? "1" : "0";
echo is_numeric("-.5") ? "1" : "0";
echo is_numeric("5.") ? "1" : "0";
echo is_numeric("8e2") ? "1" : "0";
echo is_numeric("") ? "1" : "0";
echo is_numeric(" ") ? "1" : "0";
echo is_numeric("8foo") ? "1" : "0";
echo is_numeric("0x10") ? "1" : "0";
echo is_numeric(true) ? "1" : "0";
echo is_numeric(null) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert_eq!(ir.matches("c\"0\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("is_numeric"), "{ir}");
}

#[test]
fn emit_ir_rejects_dynamic_is_numeric_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
$call = "is_numeric";
echo $call("1") ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_uniform_tracked_string_is_numeric_calls() {
    let ir = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$flag = $x === 3;
$numeric = $flag ? "1" : "2.5";
$text = $flag ? "x" : "y";
echo is_numeric($numeric) ? "1" : "0";
echo is_numeric($text) ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.contains("select i1"), "{ir}");
    for expected in ["c\"1\\00\"", "c\"0\\00\""] {
        assert!(ir.contains(expected), "{ir}");
    }
}

#[test]
fn emit_ir_rejects_mixed_tracked_string_is_numeric_until_runtime_checks_exist() {
    let error = emit_ir_source(
        r#"<?php
$x = 1 + 2;
$flag = $x === 3;
$value = $flag ? "1" : "text";
echo is_numeric($value) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
