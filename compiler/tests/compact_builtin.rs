use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn compact_collects_direct_string_variable_names_from_current_scope() {
    let execution = run_source(
        r#"<?php
$charset = "utf8mb4";
$collate = "utf8mb4_unicode_ci";
$result = compact("charset", "missing", "collate");
echo count($result);
echo "|";
echo $result["charset"];
echo "|";
echo $result["collate"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: compact(): Undefined variable $missing in Command line code on line 4\n2|utf8mb4|utf8mb4_unicode_ci"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn compact_uses_function_local_scope() {
    let execution = run_source(
        r#"<?php
$name = "global";
function build_compact() {
    $name = "local";
    return compact("name");
}
$result = build_compact();
echo $result["name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "local");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn compact_collects_array_arguments_and_warns_for_invalid_values() {
    let execution = run_source(
        r#"<?php
$name = "value";
$result = compact(["name", ["missing"]], true, "not-valid");
echo count($result);
echo "|";
echo $result["name"];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: compact(): Undefined variable $missing in Command line code on line 3\n\nWarning: compact(): Argument #2 must be string or array of strings, true given in Command line code on line 3\n\nWarning: compact(): Undefined variable $not-valid in Command line code on line 3\n1|value"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn compact_rejects_dynamic_call_boundary() {
    let execution = run_source(
        r#"<?php
$name = "value";
call_user_func("compact", "name");
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Cannot call compact() dynamically"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn emit_ir_folds_compact_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("compact") ? "1" : "0";
echo is_callable("compact") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$charset = 'utf8mb4';\ncompact('charset');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
