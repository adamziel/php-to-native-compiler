use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn modulo_operator_handles_integer_coercions_and_php_precedence() {
    let execution = run_source(
        r#"<?php
var_dump(7 % 3);
var_dump(-7 % 3);
var_dump(7 % -3);
var_dump(7.9 % 3);
var_dump("8" % true);
var_dump(null % 3);
var_dump(10 % 4 * 2);
var_dump(10 % 4 + 1);
var_dump("x" . 5 % 2);
var_dump(5 % 2 == 1);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(1)\nint(-1)\nint(1)\nint(1)\nint(0)\nint(0)\nint(4)\nint(3)\nstring(2) \"x1\"\nbool(true)\n"
    );
}

#[test]
fn modulo_operator_accepts_assignment_expression_operands() {
    let execution = run_source(
        r#"<?php
$value = 0;
$result = ($value = 11) % 4;
var_dump($result);
var_dump($value);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(3)\nint(11)\n");
}

#[test]
fn modulo_by_zero_has_stable_runtime_error() {
    let error = run_source("<?php\necho 5 % 0;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "invalid arithmetic for %: modulo by zero");
}

#[test]
fn modulo_non_numeric_string_has_stable_runtime_error() {
    let error = run_source("<?php\necho \"abc\" % 2;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for %: string is not numeric"
    );
}

#[test]
fn emit_ir_lowers_integer_modulo() {
    let ir = emit_ir_source("<?php\n$value = 10 % 4;\necho $value % 2;\n").unwrap();

    assert!(ir.contains("srem i64 10, 4"), "{ir}");
    assert!(ir.contains("srem i64 %t0, 2"), "{ir}");
    assert!(ir.contains("@printf"), "{ir}");
}

#[test]
fn emit_ir_rejects_modulo_by_zero_until_native_runtime_checks_exist() {
    let error = emit_ir_source("<?php\necho 7 % 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM modulo lowering rejects modulo by zero; phpc run reports a runtime diagnostic"
    );
}

#[test]
fn emit_ir_rejects_non_integer_modulo_until_native_coercions_exist() {
    let error = emit_ir_source("<?php\necho 7.5 % 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM modulo lowering currently requires integer operands; phpc run handles the broader int-coercion subset"
    );
}
