use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn bitwise_not_handles_ints_strings_precedence_and_assignment_operands() {
    let execution = run_source(
        r#"<?php
var_dump(~0);
var_dump(~5);
var_dump(~-1);
var_dump(~"");
var_dump(~1 & 3);
$value = 0;
var_dump(~($value = 4));
var_dump($value);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(-1)\nint(-6)\nint(0)\nstring(0) \"\"\nint(2)\nint(-5)\nint(4)\n"
    );
}

#[test]
fn bitwise_not_non_utf8_string_result_has_stable_runtime_error() {
    let error = run_source("<?php\necho ~\"A\";\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for ~: binary string results outside UTF-8 are not supported"
    );
}

#[test]
fn bitwise_not_bool_operand_has_stable_runtime_error() {
    let error = run_source("<?php\necho ~true;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for ~: booleans cannot be used with unary bitwise not"
    );
}

#[test]
fn emit_ir_rejects_bitwise_not_until_lowering_exists() {
    let error = emit_ir_source("<?php\necho ~5;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM bitwise lowering rejects bitwise and shift operators until native PHP bitwise string semantics and shift diagnostics exist; phpc run handles current bitwise/shift behavior"
    );
}
