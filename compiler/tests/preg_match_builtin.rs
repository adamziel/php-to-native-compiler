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
fn preg_match_executes_current_literal_pattern_subset() {
    let execution = run_source(
        r#"<?php
echo preg_match('/^Microsoft-IIS\//', 'Microsoft-IIS/10.0');
echo "|";
echo preg_match('/^Microsoft-IIS\//', 'phpc');
echo "|";
echo preg_match('/php$/', 'index.php');
echo "|";
echo preg_match('/dex/', 'index.php');
echo "|";
echo preg_match('/^index\.php$/', 'index.php');
echo "|";
echo preg_match('//u', '');
echo "|";
echo preg_match('//u', 'wordpress');
echo "|";
echo preg_match('/^wp$/u', 'wp');
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|0|1|1|1|1|1|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "preg_match";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call('/^wp-/', 'wp-settings') ? "match" : "miss";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|match");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_rejects_forms_outside_current_subset() {
    let output_args = runtime_error(
        r#"<?php
preg_match('/wp/', 'wp-settings', null);
"#,
    );
    assert_eq!(output_args.line, 2);
    assert_eq!(output_args.column, 1);
    assert_eq!(
        output_args.message,
        "unsupported call preg_match(): matches output, flags, and offset arguments are not implemented; pass exactly two arguments in the current subset"
    );

    let unsupported_pattern = runtime_error(
        r#"<?php
preg_match('/wp.*/', 'wp-settings');
"#,
    );
    assert_eq!(unsupported_pattern.line, 2);
    assert_eq!(unsupported_pattern.column, 1);
    assert_eq!(
        unsupported_pattern.message,
        "unsupported call preg_match(): regex metacharacter * is not implemented in the current subset"
    );

    let unsupported_modifier = runtime_error(
        r#"<?php
preg_match('/wp/i', 'WP');
"#,
    );
    assert_eq!(unsupported_modifier.line, 2);
    assert_eq!(unsupported_modifier.column, 1);
    assert_eq!(
        unsupported_modifier.message,
        "unsupported call preg_match(): only the u pattern modifier is implemented in the current subset"
    );

    let array_subject = runtime_error(
        r#"<?php
preg_match('/wp/', ['wp-settings']);
"#,
    );
    assert_eq!(array_subject.line, 2);
    assert_eq!(array_subject.column, 1);
    assert_eq!(
        array_subject.message,
        "unsupported call preg_match(): subject argument arrays are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_folds_preg_match_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_match") ? "1" : "0";
echo is_callable("preg_match") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\npreg_match('/wp/', 'wp-settings');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
