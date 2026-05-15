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
fn preg_replace_executes_current_wordpress_path_tail_cleanup() {
    let execution = run_source(
        r#"<?php
echo preg_replace('#/[^/]*$#i', '', '/index.php');
echo "|";
echo preg_replace('#/[^/]*$#i', '', '/wp-admin/admin.php?page=site');
echo "|";
echo preg_replace('#/[^/]*$#i', '', 'index.php');
echo "|";
$call = "preg_replace";
echo $call('#/[^/]*$#i', '', '/wp/wp-login.php');
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "|/wp-admin|index.php|/wp");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_executes_current_wordpress_redirect_sanitizer_cleanup() {
    let execution = run_source(
        r#"<?php
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/wp-admin/install.php');
echo "|";
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/path/<bad>"quote"');
echo "|";
echo preg_replace('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/p%C3%A5th/%C3%A9.php');
echo "|";
$call = "preg_replace";
echo $call('|[^a-z0-9-~+_.?#=&;,/:%!*\[\]()@]|i', '', '/bad space/');
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "/wp-admin/install.php|/path/badquote|/p%C3%A5th/%C3%A9.php|/badspace/"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_executes_current_wordpress_mail_host_cleanup() {
    let execution = run_source(
        r#"<?php
echo preg_replace('#^www\.#', '', 'www.example.test');
echo "|";
echo preg_replace('#^www\.#', '', 'mail.example.test');
echo "|";
echo preg_replace('#^www\.#', '', 'www2.example.test');
echo "|";
$call = "preg_replace";
echo $call('#^www\.#', '', 'www.wordpress.org');
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "example.test|mail.example.test|www2.example.test|wordpress.org"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_executes_current_wordpress_kses_control_char_cleanup() {
    let execution = run_source(
        "<?php\n\
echo preg_replace('/[\\x00-\\x08\\x0B\\x0C\\x0E-\\x1F]/', '', \"a\u{0}b\u{7}c\u{b}d\u{1f}e\");\n\
echo \"|\";\n\
$call = \"preg_replace\";\n\
echo $call('/[\\x00-\\x08\\x0B\\x0C\\x0E-\\x1F]/', '', \"x\u{c}y\");\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "abcde|xy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_executes_current_wordpress_kses_slash_zero_cleanup() {
    let execution = run_source(
        r#"<?php
echo preg_replace('/\\\\+0+/', '', 'a\\0b\\\\00c');
echo "|";
echo preg_replace('/\\\\+0+/', '', 'keep\\\\slash');
echo "|";
$call = "preg_replace";
echo $call('/\\\\+0+/', '', '\\000x');
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "abc|keep\\\\slash|x");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_replace_executes_current_wordpress_wpdb_prepare_placeholder_escape() {
    let execution = run_source(
        r#"<?php
$allowed_format = '(?:[1-9][0-9]*[$])?[-+0-9]*(?: |0|\'.)?[-+0-9]*(?:\.[0-9]+)?';
echo preg_replace(
    "/%(?:%|$|(?!($allowed_format)?[sdfFi]))/",
    '%%\\1',
    'SELECT %s, %05d, 100%, %q, %%s, %1$s'
);
echo "\n";
echo preg_replace(
    "/%(?:%|$|(?!($allowed_format)?[sdfFi]))/",
    '%%\\1',
    'LIKE %foo% AND rate %1$q'
);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "SELECT %s, %05d, 100%%, %%q, %%s, %1$s\nLIKE %foo%% AND rate %%1$q"
    );
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
        "unsupported call preg_replace(): only the WordPress database-version cleanup pattern /[^0-9.].*/, path-tail pattern #/[^/]*$#i, redirect sanitizer cleanup pattern |[^a-z0-9-~+_.?#=&;,/:%!*\\[\\]()@]|i, mail host cleanup pattern #^www\\.#, KSES null cleanup patterns /[\\x00-\\x08\\x0B\\x0C\\x0E-\\x1F]/ and /\\\\+0+/, and wpdb prepare placeholder escape pattern are implemented in the current subset"
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
