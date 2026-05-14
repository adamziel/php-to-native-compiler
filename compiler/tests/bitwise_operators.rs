use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn bitwise_operators_handle_ints_and_php_precedence() {
    let execution = run_source(
        r#"<?php
var_dump(6 & 3);
var_dump(6 | 3);
var_dump(6 ^ 3);
var_dump((1 + 2) & 6);
var_dump(1 == 1 & 0);
var_dump(1 | 2 && false);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(2)\nint(7)\nint(5)\nint(2)\nint(0)\nbool(false)\n"
    );
}

#[test]
fn bitwise_operators_handle_string_string_and_numeric_mixed_operands() {
    let execution = run_source(
        r#"<?php
var_dump("ab" & "AB");
var_dump("A@" | " !");
var_dump("az" ^ "  ");
var_dump("ABC" & "xy");
var_dump("A" | "CD");
var_dump("az" ^ " ");
var_dump("6" & 3);
var_dump(8 | "2");
var_dump("7" ^ true);
var_dump(null | 2);
var_dump(false | 2);
var_dump(true & 3);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(2) \"AB\"\nstring(2) \"aa\"\nstring(2) \"AZ\"\nstring(2) \"@@\"\nstring(2) \"CD\"\nstring(1) \"A\"\nint(2)\nint(10)\nint(6)\nint(2)\nint(2)\nint(1)\n"
    );
}

#[test]
fn bitwise_operators_accept_assignment_expression_operands() {
    let execution = run_source(
        r#"<?php
$value = 0;
$result = ($value = 4) & 6;
var_dump($result);
var_dump($value);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(4)\nint(4)\n");
}

#[test]
fn bitwise_non_numeric_mixed_string_has_stable_runtime_error() {
    let error = run_source("<?php\necho \"abc\" & 1;\n").unwrap_err();

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for &: string is not numeric"
    );
}

#[test]
fn emit_ir_rejects_bitwise_operators_until_lowering_exists() {
    let error = emit_ir_source("<?php\necho true & 3;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM bitwise lowering rejects unsupported bitwise or shift operators or operands until native PHP bitwise string semantics, scalar-to-int coercion, shift diagnostics, references/copy-on-write, and exact native error behavior exist; phpc run handles current bitwise/shift behavior"
    );
}
