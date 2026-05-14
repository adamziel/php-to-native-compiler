use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_UNARY_REJECTION: &str = "LLVM unary lowering rejects unsupported unary operators, cast expressions, or operands until native PHP numeric coercion, truthiness conversion, scalar casts, overflow behavior, references/copy-on-write, and exact native error behavior exist; phpc run handles current unary and cast behavior";

#[test]
fn string_casts_execute_for_current_scalar_and_null_subset() {
    let execution = run_source(
        r#"<?php
echo "[", (string) null, "]\n";
echo "[", (string) false, "]\n";
echo (STRING) true, "|", (string) 42, "|", (string) 3.5, "|", (string) "ok", "\n";
echo ((string) true) === "1" ? "string" : "other";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "[]\n[]\n1|42|3.5|ok\nstring");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_casts_execute_for_current_scalar_and_null_subset() {
    let execution = run_source(
        r#"<?php
echo (int) null, "|", (int) false, "|", (int) true, "\n";
echo (integer) 42, "|", (int) -3.8, "|", (int) " 15 ", "|", (int) "2.9", "\n";
echo (int) "", "|", (int) "not numeric";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|0|1\n42|-3|15|2\n0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bool_casts_execute_for_current_value_subset() {
    let execution = run_source(
        r#"<?php
echo (bool) null ? "true" : "false", "|";
echo (boolean) false ? "true" : "false", "|";
echo (bool) true ? "true" : "false", "\n";
echo (bool) 0 ? "true" : "false", "|";
echo (bool) 1 ? "true" : "false", "|";
echo (bool) 0.0 ? "true" : "false", "|";
echo (bool) -0.5 ? "true" : "false", "\n";
echo (bool) "" ? "true" : "false", "|";
echo (bool) "0" ? "true" : "false", "|";
echo (bool) "false" ? "true" : "false", "\n";
echo (bool) [] ? "true" : "false", "|";
echo (bool) [0] ? "true" : "false", "\n";
class Flag {}
echo (bool) new Flag() ? "true" : "false";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "false|false|true\nfalse|true|false|true\nfalse|false|true\nfalse|true\ntrue"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_casts_reject_array_and_object_warning_paths_for_now() {
    let error = run_source("<?php\necho (string) [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call (string): array-to-string cast warning behavior is not implemented"
    );
}

#[test]
fn int_casts_reject_unimplemented_warning_paths_for_now() {
    let error = run_source("<?php\necho (int) [1];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call (int): array-to-int cast behavior is not implemented"
    );

    let error = run_source("<?php\necho (int) \"42abc\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call (int): leading-numeric string cast behavior is not implemented"
    );
}

#[test]
fn remaining_casts_have_stable_parse_error() {
    let error = run_source("<?php\necho (float) \"1\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported cast expression: only (string), (int), and (bool) casts are implemented"
    );
}

#[test]
fn emit_ir_rejects_string_cast_until_native_cast_lowering_exists() {
    let error = emit_ir_source("<?php\necho (string) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);

    let error = emit_ir_source("<?php\necho (int) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);

    let error = emit_ir_source("<?php\necho (bool) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);
}
