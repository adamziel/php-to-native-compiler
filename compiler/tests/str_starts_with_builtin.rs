use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn str_starts_with_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo str_starts_with("wp-admin/admin-ajax.php", "wp-admin") ? "yes" : "no";
echo "|";
echo str_starts_with("index.php", "php") ? "yes" : "no";
echo "|";
echo str_starts_with("index.php", "") ? "empty" : "no";
echo "|";
echo str_starts_with(42, "4") ? "coerced" : "no";
echo "|";
echo str_starts_with(null, "") ? "null-empty" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no|empty|coerced|null-empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_starts_with_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "str_starts_with";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("wp-content/plugins/example.php", "wp-content") ? "prefix" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|prefix");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_starts_with_rejects_forms_outside_current_subset() {
    let array_haystack = run_source("<?php\nstr_starts_with(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call str_starts_with(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstr_starts_with('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call str_starts_with(): needle argument arrays are not implemented in the current subset"
    );

    let too_few = run_source("<?php\nstr_starts_with('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for str_starts_with(): expected 2 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_str_starts_with_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_starts_with") ? "1" : "0";
echo is_callable("str_starts_with") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nstr_starts_with('abc', 'a');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
