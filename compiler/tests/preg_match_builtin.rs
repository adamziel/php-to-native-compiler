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
fn preg_match_writes_direct_matches_variable_for_current_subset() {
    let execution = run_source(
        r#"<?php
$result = preg_match('/dex/', 'index.php', $matches);
echo $result;
echo "|";
echo $matches[0];
echo "|";
$matches = ["old"];
$result = preg_match('/missing/', 'index.php', $matches);
echo $result;
echo "|";
echo count($matches);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|dex|0|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_writes_wordpress_db_host_named_matches() {
    let execution = run_source(
        r#"<?php
$ipv4 = '#^(?P<host>[^:/]*)(?::(?P<port>[\d]+))?#';
$ipv6 = '#^(?:\[)?(?P<host>[0-9a-fA-F:]+)(?:\]:(?P<port>[\d]+))?#';
echo preg_match($ipv4, 'db.example:3306', $matches);
echo "|";
echo $matches[0];
echo "|";
echo $matches['host'];
echo "|";
echo $matches[1];
echo "|";
echo $matches['port'];
echo "|";
echo $matches[2];
echo "|";
echo preg_match($ipv6, '[2001:db8::1]:3306', $matches);
echo "|";
echo $matches[0];
echo "|";
echo $matches['host'];
echo "|";
echo $matches[1];
echo "|";
echo $matches['port'];
echo "|";
echo $matches[2];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|db.example:3306|db.example|db.example|3306|3306|1|[2001:db8::1]:3306|2001:db8::1|2001:db8::1|3306|3306"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_handles_wordpress_table_prefix_guard() {
    let execution = run_source(
        r#"<?php
echo preg_match('|[^a-z0-9_]|i', 'wp_');
echo "|";
echo preg_match('|[^a-z0-9_]|i', 'wp-Bad', $matches);
echo "|";
echo $matches[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|1|-");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_handles_wordpress_safe_collation_query_guard() {
    let execution = run_source(
        r#"<?php
$pattern = '/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\s/i';
echo preg_match($pattern, "SHOW TABLES", $matches);
echo "|";
echo $matches[0];
echo "|";
echo preg_match($pattern, "select * from wp_options");
echo "|";
echo preg_match($pattern, "create\tTABLE wp_posts", $matches);
echo "|";
echo $matches[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|SHOW |0|1|create\t");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_handles_wordpress_ascii_check_guard() {
    let execution = run_source(
        r#"<?php
$pattern = '/[^\x00-\x7F]/';
echo preg_match($pattern, "SELECT option_name");
echo "|";
echo preg_match($pattern, "café");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_handles_wordpress_query_type_classifiers() {
    let execution = run_source(
        r#"<?php
$ddl = '/^\s*(create|alter|truncate|drop)\s/i';
$dml = '/^\s*(insert|delete|update|replace)\s/i';
$insert = '/^\s*(insert|replace)\s/i';
echo preg_match($ddl, "  CREATE TABLE wp_posts", $matches);
echo "|";
echo $matches[0];
echo "|";
echo preg_match($ddl, "SELECT option_name FROM wp_options");
echo "|";
echo preg_match($dml, "\tupdate wp_options set option_value = 'x'", $matches);
echo "|";
echo $matches[0];
echo "|";
echo preg_match($dml, "show tables");
echo "|";
echo preg_match($insert, " replace into wp_options", $matches);
echo "|";
echo $matches[0];
echo "|";
echo preg_match($insert, "delete from wp_options");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|  CREATE |0|1|\tupdate |0|1| replace |0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_rejects_forms_outside_current_subset() {
    let output_args = runtime_error(
        r#"<?php
$bag = [];
preg_match('/wp/', 'wp-settings', $bag['matches']);
"#,
    );
    assert_eq!(output_args.line, 3);
    assert_eq!(output_args.column, 1);
    assert_eq!(
        output_args.message,
        "unsupported call preg_match(): matches output must be a direct variable in the current subset"
    );

    let flags_args = runtime_error(
        r#"<?php
preg_match('/wp/', 'wp-settings', $matches, 0);
"#,
    );
    assert_eq!(flags_args.line, 2);
    assert_eq!(flags_args.column, 1);
    assert_eq!(
        flags_args.message,
        "unsupported call preg_match(): flags and offset arguments are not implemented; pass at most a direct matches variable in the current subset"
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
