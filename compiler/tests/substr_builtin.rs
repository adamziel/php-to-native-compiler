use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn substr_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo substr("abcdef", 2);
echo "|";
echo substr("abcdef", 2, 3);
echo "|";
echo substr("abcdef", -2);
echo "|";
echo substr("abcdef", 0, -1);
echo "|";
echo substr("abcdef", -4, 2);
echo "|";
echo substr("abcdef", 99) === "" ? "empty" : "nonempty";
echo "|";
echo substr(12345, 1, 3);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "cdef|cde|ef|abcde|cd|empty|234");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "substr";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("abcdef", -3, 2);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|de");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_rejects_forms_outside_current_subset() {
    let array_string = run_source("<?php\nsubstr(['abc'], 1);\n").unwrap_err();
    assert_eq!(array_string.phase, Phase::Runtime);
    assert_eq!(array_string.line, 2);
    assert_eq!(array_string.column, 1);
    assert_eq!(
        array_string.message,
        "unsupported call substr(): string argument arrays are not implemented in the current subset"
    );

    let bad_offset = run_source("<?php\nsubstr('abc', '1');\n").unwrap_err();
    assert_eq!(bad_offset.phase, Phase::Runtime);
    assert_eq!(bad_offset.line, 2);
    assert_eq!(bad_offset.column, 1);
    assert_eq!(
        bad_offset.message,
        "unsupported call substr(): offset argument must be int in the current subset, got string"
    );

    let bad_length = run_source("<?php\nsubstr('abc', 0, '1');\n").unwrap_err();
    assert_eq!(bad_length.phase, Phase::Runtime);
    assert_eq!(bad_length.line, 2);
    assert_eq!(bad_length.column, 1);
    assert_eq!(
        bad_length.message,
        "unsupported call substr(): length argument must be int in the current subset, got string"
    );

    let too_few = run_source("<?php\nsubstr('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for substr(): expected 2 to 3 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_substr_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("substr") ? "1" : "0";
echo is_callable("substr") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nsubstr('abc', 1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
