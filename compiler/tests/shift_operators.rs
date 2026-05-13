use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn shift_operators_handle_ints_coercions_and_php_precedence() {
    let execution = run_source(
        r#"<?php
var_dump(8 << 1);
var_dump(8 >> 1);
var_dump(-8 >> 1);
var_dump(1 + 2 << 3);
var_dump(1 << 2 + 1);
var_dump("x" . 1 << 2);
var_dump(1 << 2 < 8);
var_dump("8" << true);
var_dump(null >> 1);
var_dump(8 << 64);
var_dump(-1 >> 64);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(16)\nint(4)\nint(-4)\nint(24)\nint(8)\nstring(2) \"x4\"\nbool(true)\nint(16)\nint(0)\nint(0)\nint(-1)\n"
    );
}

#[test]
fn shift_operators_accept_assignment_expression_operands() {
    let execution = run_source(
        r#"<?php
$value = 0;
$result = ($value = 4) << 2;
var_dump($result);
var_dump($value);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(16)\nint(4)\n");
}

#[test]
fn shift_negative_count_has_stable_runtime_error() {
    let error = run_source("<?php\necho 8 << -1;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for <<: bit shift by negative number"
    );
}

#[test]
fn shift_non_numeric_string_has_stable_runtime_error() {
    let error = run_source("<?php\necho \"abc\" >> 1;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for >>: string is not numeric"
    );
}

#[test]
fn emit_ir_rejects_shift_operators_until_lowering_exists() {
    let error = emit_ir_source("<?php\necho 8 << 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "bitwise operators are supported by phpc run for the current int/string subset but not LLVM IR emission yet"
    );
}
