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
fn non_string_casts_have_stable_parse_error() {
    let error = run_source("<?php\necho (int) \"42\";\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported cast expression: only (string) casts are implemented"
    );
}

#[test]
fn emit_ir_rejects_string_cast_until_native_cast_lowering_exists() {
    let error = emit_ir_source("<?php\necho (string) 42;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_UNARY_REJECTION);
}
