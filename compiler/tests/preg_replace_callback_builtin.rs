use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

const WP_SANITIZE_REDIRECT_SOURCE: &str = r#"<?php
function _wp_sanitize_utf8_in_redirect($matches) {
    return urlencode($matches[0]);
}

$regex = '/
(
    (?: [\xC2-\xDF][\x80-\xBF]        # double-byte sequences
    |   \xE0[\xA0-\xBF][\x80-\xBF]
    |   [\xE1-\xEC][\x80-\xBF]{2}
    |   \xED[\x80-\x9F][\x80-\xBF]
    |   [\xEE-\xEF][\x80-\xBF]{2}
    |   \xF0[\x90-\xBF][\x80-\xBF]{2}
    |   [\xF1-\xF3][\x80-\xBF]{3}
    |   \xF4[\x80-\x8F][\x80-\xBF]{2}
){1,40}                              # ...one or more times
)/x';
"#;

#[test]
fn preg_replace_callback_executes_current_wordpress_redirect_sanitizer() {
    let execution = run_source(&format!(
        r#"{WP_SANITIZE_REDIRECT_SOURCE}
echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/wp-admin/install.php');
echo "|";
echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/påth/é.php');
echo "|";
$call = "preg_replace_callback";
echo $call($regex, '_wp_sanitize_utf8_in_redirect', '/über');
"#
    ))
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/wp-admin/install.php|/p%C3%A5th/%C3%A9.php|/%C3%BCber"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_callback_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "preg_replace_callback";
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
fn preg_replace_callback_rejects_forms_outside_current_subset() {
    let unsupported_pattern = runtime_error(
        r#"<?php
preg_replace_callback('/./', 'cb', 'abc');
"#,
    );
    assert_eq!(unsupported_pattern.line, 2);
    assert_eq!(unsupported_pattern.column, 1);
    assert_eq!(
        unsupported_pattern.message,
        "unsupported call preg_replace_callback(): only the WordPress wp_sanitize_redirect() UTF-8 sanitizer pattern is implemented in the current subset"
    );

    let unsupported_callback = runtime_error(&format!(
        r#"{WP_SANITIZE_REDIRECT_SOURCE}
preg_replace_callback($regex, 'other_callback', '/x');
"#
    ));
    assert_eq!(unsupported_callback.line, 19);
    assert_eq!(unsupported_callback.column, 1);
    assert_eq!(
        unsupported_callback.message,
        "unsupported call preg_replace_callback(): only the WordPress _wp_sanitize_utf8_in_redirect string callback is implemented in the current subset"
    );

    let unsupported_limit = runtime_error(&format!(
        r#"{WP_SANITIZE_REDIRECT_SOURCE}
preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/x', 1);
"#
    ));
    assert_eq!(unsupported_limit.line, 19);
    assert_eq!(unsupported_limit.column, 1);
    assert_eq!(
        unsupported_limit.message,
        "unsupported call preg_replace_callback(): limit, count output, and flags arguments are not implemented; pass exactly three arguments in the current subset"
    );
}

#[test]
fn emit_ir_folds_preg_replace_callback_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_replace_callback") ? "1" : "0";
echo is_callable("preg_replace_callback") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\npreg_replace_callback('/./', 'cb', 'x');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
