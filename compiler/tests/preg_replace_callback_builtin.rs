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
fn preg_replace_callback_invokes_supported_string_callables_for_bounded_patterns() {
    let execution = run_source(
        r#"<?php
function bracket_match($matches) {
    return "[" . $matches[0] . "]";
}
function star_match($matches) {
    return "*" . $matches[0] . "*";
}

echo preg_replace_callback('/a/', 'bracket_match', 'banana');
echo "|";
echo preg_replace_callback('/^go/', 'bracket_match', 'gogo');
echo "|";
$call = "preg_replace_callback";
echo $call('/go$/', 'bracket_match', 'gogo');
echo "|";
echo preg_replace_callback('/[0-9]/', 'bracket_match', 'a1b2');
echo "|";
echo preg_replace_callback('/[^a-z]/', 'bracket_match', 'ab-CD3');
echo "|";
echo preg_replace_callback('/[\\x41-\\x43]$/', 'star_match', 'goC');
echo "|";
echo preg_replace_callback('/[^\\x00-\\x7F]/', 'star_match', '/påth/é.php');
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "b[a]n[a]n[a]|[go]go|go[go]|a[1]b[2]|ab[-][C][D][3]|go*C*|/p*å*th/*é*.php"
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
        "unsupported call preg_replace_callback(): regex metacharacter . is not implemented in the current subset"
    );

    let unsupported_callback = runtime_error(
        r#"<?php
preg_replace_callback('/x/', 'other_callback', '/x');
"#,
    );
    assert_eq!(unsupported_callback.line, 2);
    assert_eq!(unsupported_callback.column, 1);
    assert_eq!(
        unsupported_callback.message,
        "undefined function other_callback()"
    );

    let unsupported_limit = runtime_error(
        r#"<?php
function cb($matches) { return $matches[0]; }
preg_replace_callback('/x/', 'cb', '/x', 1);
"#,
    );
    assert_eq!(unsupported_limit.line, 3);
    assert_eq!(unsupported_limit.column, 1);
    assert_eq!(
        unsupported_limit.message,
        "unsupported call preg_replace_callback(): limit, count output, and flags arguments are not implemented; pass exactly three arguments in the current subset"
    );

    let unsupported_empty_match = runtime_error(
        r#"<?php
function cb($matches) { return $matches[0]; }
preg_replace_callback('//', 'cb', 'abc');
"#,
    );
    assert_eq!(unsupported_empty_match.line, 3);
    assert_eq!(unsupported_empty_match.column, 1);
    assert_eq!(
        unsupported_empty_match.message,
        "unsupported call preg_replace_callback(): zero-length regex matches are not implemented in the current subset"
    );

    let unsupported_legacy_exact_gate = runtime_error(
        r#"<?php
function cb($matches) { return $matches[0]; }
preg_replace_callback('|[^a-z0-9_]|i', 'cb', 'ab-c');
"#,
    );
    assert_eq!(unsupported_legacy_exact_gate.line, 3);
    assert_eq!(unsupported_legacy_exact_gate.column, 1);
    assert_eq!(
        unsupported_legacy_exact_gate.message,
        "unsupported call preg_replace_callback(): only slash-delimited patterns are implemented in the current subset"
    );
}

#[test]
fn preg_replace_callback_executes_extended_byte_class_regexes() {
    let redirect_execution = run_source(&format!(
        r#"{WP_SANITIZE_REDIRECT_SOURCE}
echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/wp-admin/install.php');
echo "|";
echo preg_replace_callback($regex, '_wp_sanitize_utf8_in_redirect', '/påth/é.php');
"#
    ))
    .unwrap();

    assert_eq!(
        redirect_execution.stdout,
        "/wp-admin/install.php|/p%C3%A5th/%C3%A9.php"
    );
    assert_eq!(redirect_execution.exit_code, 0);

    let bounded_repeat_execution = run_source(
        r#"<?php
function bracket_match($matches) {
    return "[" . $matches[0] . "]";
}
$regex = '/
    ( [\x41-\x43]{1,2} | [\x30-\x39]{2} )
/x';
echo preg_replace_callback($regex, 'bracket_match', 'ABZ12C');
"#,
    )
    .unwrap();

    assert_eq!(bounded_repeat_execution.stdout, "[AB]Z[12][C]");
    assert_eq!(bounded_repeat_execution.exit_code, 0);
}

#[test]
fn preg_replace_callback_rejects_unsupported_extended_regex_shapes() {
    let unsupported_optional_repeat = runtime_error(
        r#"<?php
function cb($matches) { return $matches[0]; }
preg_replace_callback('/[\x41-\x43]{0,1}/x', 'cb', 'ABC');
"#,
    );

    assert_eq!(unsupported_optional_repeat.line, 3);
    assert_eq!(unsupported_optional_repeat.column, 1);
    assert_eq!(
        unsupported_optional_repeat.message,
        "unsupported call preg_replace_callback(): zero-length regex repeats are not implemented in the current subset"
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
