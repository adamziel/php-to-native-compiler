use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

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

    assert_eq!(execution.stdout, "2|utf8mb4|utf8mb4_unicode_ci");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn compact_uses_function_local_scope_and_dynamic_string_calls() {
    let execution = run_source(
        r#"<?php
$name = "global";
function build_compact() {
    $name = "local";
    $call = "compact";
    return $call("name");
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
fn compact_rejects_forms_outside_current_subset() {
    let array_arg = runtime_error(
        r#"<?php
$name = "value";
compact(["name"]);
"#,
    );
    assert_eq!(array_arg.line, 3);
    assert_eq!(array_arg.column, 1);
    assert_eq!(
        array_arg.message,
        "unsupported call compact(): variable names must be direct strings in the current subset, got array"
    );

    let invalid_name = runtime_error(
        r#"<?php
compact("not-valid");
"#,
    );
    assert_eq!(invalid_name.line, 2);
    assert_eq!(invalid_name.column, 1);
    assert_eq!(
        invalid_name.message,
        "unsupported call compact(): variable names must be non-empty simple identifiers in the current subset"
    );

    let call_user_func_boundary = runtime_error(
        r#"<?php
$name = "value";
call_user_func("compact", "name");
"#,
    );
    assert_eq!(call_user_func_boundary.line, 3);
    assert_eq!(call_user_func_boundary.column, 1);
    assert_eq!(
        call_user_func_boundary.message,
        "unsupported call compact(): caller-scope variable lookup is only implemented for direct and dynamic compact() calls in the current subset"
    );
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
