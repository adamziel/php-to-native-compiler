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
fn preg_replace_executes_current_wordpress_database_version_cleanup() {
    let execution = run_source(
        r#"<?php
echo preg_replace('/[^0-9.].*/', '', '8.0.35-MySQL');
echo "|";
echo preg_replace('/[^0-9.].*/', '', '10.6.18-MariaDB-log');
echo "|";
echo preg_replace('/[^0-9.].*/', '', 'abc');
echo "|";
$call = "preg_replace";
echo $call('/[^0-9.].*/', '', '8.3.1');
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "8.0.35|10.6.18||8.3.1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "preg_replace";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_rejects_forms_outside_current_subset() {
    let unsupported_pattern = runtime_error(
        r#"<?php
preg_replace('/[^a-z].*/', '', 'abc123');
"#,
    );
    assert_eq!(unsupported_pattern.line, 2);
    assert_eq!(unsupported_pattern.column, 1);
    assert_eq!(
        unsupported_pattern.message,
        "unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/ is implemented in the current subset"
    );

    let unsupported_replacement = runtime_error(
        r#"<?php
preg_replace('/[^0-9.].*/', 'x', '8.0.35-MySQL');
"#,
    );
    assert_eq!(unsupported_replacement.line, 2);
    assert_eq!(unsupported_replacement.column, 1);
    assert_eq!(
        unsupported_replacement.message,
        "unsupported call preg_replace(): only an empty replacement string is implemented in the current subset"
    );

    let unsupported_limit = runtime_error(
        r#"<?php
preg_replace('/[^0-9.].*/', '', '8.0.35-MySQL', 1);
"#,
    );
    assert_eq!(unsupported_limit.line, 2);
    assert_eq!(unsupported_limit.column, 1);
    assert_eq!(
        unsupported_limit.message,
        "unsupported call preg_replace(): limit and count output arguments are not implemented; pass exactly three arguments in the current subset"
    );
}

#[test]
fn emit_ir_folds_preg_replace_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_replace") ? "1" : "0";
echo is_callable("preg_replace") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error =
        emit_ir_source("<?php\npreg_replace('/[^0-9.].*/', '', '8.0.35-MySQL');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
