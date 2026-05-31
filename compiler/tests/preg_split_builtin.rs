use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn preg_split_executes_current_wordpress_wpdb_prepare_placeholder_extraction() {
    let execution = run_source(
        r#"<?php
$allowed_format = '(?:[1-9][0-9]*[$])?[-+0-9]*(?: |0|\'.)?[-+0-9]*(?:\.[0-9]+)?';
$pattern = "/(^|[^%]|(?:%%)+)(%(?:$allowed_format)?[sdfFi])/";
$split = preg_split($pattern, '%s', -1, PREG_SPLIT_DELIM_CAPTURE);
echo count($split), '|', $split[0], '|', $split[1], '|', $split[2], '|', $split[3], "\n";
$split = preg_split($pattern, 'SELECT * FROM t WHERE a = %s AND b = %d', -1, PREG_SPLIT_DELIM_CAPTURE);
echo count($split), '|', $split[0], '|', $split[1], '|', $split[2], '|', $split[3], '|', $split[4], '|', $split[5], '|', $split[6], "\n";
$split = preg_split($pattern, 'SELECT %i FROM t WHERE price > %.2f AND qty >= %05d', -1, PREG_SPLIT_DELIM_CAPTURE);
echo count($split), '|', $split[0], '|', $split[1], '|', $split[2], '|', $split[3], '|', $split[4], '|', $split[5], '|', $split[6], '|', $split[7], '|', $split[8], '|', $split[9], "\n";
$split = preg_split($pattern, 'WHERE raw = %%x AND ok = %s', -1, PREG_SPLIT_DELIM_CAPTURE);
echo count($split), '|', $split[0], '|', $split[1], '|', $split[2], '|', $split[3], "\n";
$split = preg_split($pattern, 'SELECT %%%%s, %s', -1, PREG_SPLIT_DELIM_CAPTURE);
echo count($split), '|', $split[0], '|', $split[1], '|', $split[2], '|', $split[3], '|', $split[4], '|', $split[5], '|', $split[6];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "4|||%s|\n7|SELECT * FROM t WHERE a =| |%s| AND b =| |%d|\n10|SELECT| |%i| FROM t WHERE price >| |%.2f| AND qty >=| |%05d|\n4|WHERE raw = %%x AND ok =| |%s|\n7|SELECT %|%%|%s|,| |%s|"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_split_is_available_through_string_valued_calls_and_constant_metadata() {
    let execution = run_source(
        r#"<?php
$allowed_format = '(?:[1-9][0-9]*[$])?[-+0-9]*(?: |0|\'.)?[-+0-9]*(?:\.[0-9]+)?';
$pattern = "/(^|[^%]|(?:%%)+)(%(?:$allowed_format)?[sdfFi])/";
$call = "preg_split";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo defined("PREG_SPLIT_DELIM_CAPTURE") ? PREG_SPLIT_DELIM_CAPTURE : "missing";
echo "|";
$split = $call($pattern, 'LIKE %s', -1, constant("PREG_SPLIT_DELIM_CAPTURE"));
echo count($split), "|", $split[0], "|", $split[1], "|", $split[2], "|", $split[3];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|2|4|LIKE| |%s|");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_split_supports_general_regex_flags_limits_and_offsets() {
    let execution = run_source(
        r#"<?php
$parts = preg_split('/[\s,]+/', 'x yy,zzz', -1, PREG_SPLIT_NO_EMPTY);
echo count($parts), "|", $parts[0], "|", $parts[1], "|", $parts[2], "\n";
$limited = preg_split('/[\s,]+/', 'x yy,zzz', 2);
echo count($limited), "|", $limited[0], "|", $limited[1], "\n";
$captured = preg_split('/(\d)/', 'a1b2', -1, PREG_SPLIT_DELIM_CAPTURE | PREG_SPLIT_OFFSET_CAPTURE);
echo count($captured), "|", $captured[1][0], ":", $captured[1][1], "|", $captured[3][0], ":", $captured[3][1];
ini_set('pcre.recursion_limit', 1);
$failed = preg_split('/(\d*)/', 'ab2c3u');
echo "\n", $failed === false ? "false" : "array", "|", preg_last_error();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3|x|yy|zzz\n2|x|yy,zzz\n5|1:1|2:3\nfalse|3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_quote_escapes_pcre_metacharacters_and_optional_delimiter() {
    let execution = run_source(
        r#"<?php
$before = '/this *-has \ metacharacters^ in $';
$quoted = preg_quote($before, '/');
echo $quoted, "\n";
echo preg_match('/' . $quoted . '/', $before);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "\\/this \\*\\-has \\\\ metacharacters\\^ in \\$\n1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_preg_split_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_split") ? "1" : "0";
echo is_callable("preg_split") ? "1" : "0";
echo function_exists("preg_quote") ? "1" : "0";
echo defined("PREG_SPLIT_NO_EMPTY") ? "1" : "0";
echo defined("PREG_SPLIT_DELIM_CAPTURE") ? "1" : "0";
echo defined("PREG_SPLIT_OFFSET_CAPTURE") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source(
        "<?php\npreg_split('/(^|[^%]|(?:%%)+)(%(?:[sdfFi]))/', '%s', -1, PREG_SPLIT_DELIM_CAPTURE);\n",
    )
    .unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
