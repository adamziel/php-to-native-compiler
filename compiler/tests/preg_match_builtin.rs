use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

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
fn preg_match_supports_direct_output_flags_offsets_and_regex_modifiers() {
    let execution = run_source(
        r#"<?php
$bag = [];
echo preg_match('/wp/', 'wp-settings', $matches);
echo "|", $matches[0], "\n";
echo preg_match('/bar/', 'foo bar', $matches, 0, 4);
echo "|", $matches[0], "\n";
echo preg_match('/wp.*/', 'wp-settings');
echo "|";
echo preg_match('/wp/i', 'WP');
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|wp\n1|bar\n1|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_interleaves_named_captures_and_supports_no_auto_capture_modifier() {
    let execution = run_source(
        r#"<?php
preg_match('/(?P<name>)(\d+)/', 0xffffffff, $match);
echo implode(',', array_keys($match)), "|", $match['name'], "|", $match[1], "|", $match[2], "\n";
preg_match('/(?P<capt1>.)(x)(?P<letsmix>\S+)/', 'fjszxax', $match, PREG_OFFSET_CAPTURE);
echo implode(',', array_keys($match)), "|", $match['capt1'][0], ":", $match['capt1'][1], "|", $match[3][0], ":", $match[3][1], "\n";
preg_match('/.(.)./n', 'abc', $match);
echo implode(',', array_keys($match)), "|", count($match), "\n";
preg_match('/.(?P<test>.)./n', 'abc', $match);
echo implode(',', array_keys($match)), "|", $match['test'], "|", $match[1], "\n";
preg_match_all('/(?P<word>the)/', 'the other the', $match);
echo implode(',', array_keys($match)), "|", implode('-', $match['word']), "|", implode('-', $match[1]), "\n";
preg_match_all('/(?<a>4)?(?<b>2)?\d/', '123456', $match, PREG_SET_ORDER | PREG_UNMATCHED_AS_NULL);
echo implode(',', array_keys($match[1])), "|", ($match[1]['a'] === null ? 'NULL' : $match[1]['a']), "|", $match[1]['b'];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0,name,1,2|||4294967295\n0,capt1,1,2,letsmix,3|z:3|ax:5\n0|1\n0,test,1|b|b\n0,word,1|the-the-the|the-the-the\n0,a,1,b,2|NULL|2"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_supports_apostrophe_named_groups_and_dollar_endonly_modifier() {
    let execution = run_source(
        r#"<?php
preg_match("/(?'word'foo)/", "foo", $match);
echo implode(',', array_keys($match)), "|", $match['word'], "|", $match[1], "\n";
echo preg_match_all('/^\S+.+$/', "aeiou\n", $match), "|", $match[0][0], "\n";
echo preg_match_all('/^\S+.+$/D', "aeiou\n", $match), "|", count($match[0]), "\n";
echo preg_match_all('/^\S+\s$/D', "aeiou\n", $match), "|", ($match[0][0] === "aeiou\n" ? "full" : "partial");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0,word,1|foo|foo\n1|aeiou\n0|0\n1|full");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_handles_backtrack_offsets_utf8_and_trailing_unmatched_captures() {
    let execution = run_source(
        r#"<?php
ini_set('pcre.backtrack_limit', '1');
var_dump(preg_match('/(?:\D+|<\d+>)*[!?]/', 'foobar foobar foobar'));
var_dump(preg_last_error());
var_dump(preg_match('/a/', 'a', $simple));
var_dump(preg_last_error());
var_dump($simple);
var_dump(preg_match('/(?:a|b)*z/', 'aaaaa'));
var_dump(preg_last_error());
var_dump(preg_match('/^foo(?:a|b)*[z]/', 'aaaaa'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:a+|b+)*[z]/', 'a'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:a+|b+)*[z]/', 'ab'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:a+|b+)*[z]/', 'az'));
var_dump(preg_last_error());
var_dump(preg_match('/^foo(?:a+|b+)*[z]/', 'fooa'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:ab+|cd+)*[z]/', 'ab'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:ab+|cd+)*[z]/', 'az', $later));
var_dump(preg_last_error());
var_dump($later[0]);
var_dump(preg_match('/(?:ab+|cd+)*[z]/', 'acz', $later));
var_dump(preg_last_error());
var_dump($later[0]);
var_dump(preg_match('/(?:ab+|cd+)*[z]/', 'a z', $later));
var_dump(preg_last_error());
var_dump($later[0]);
var_dump(preg_match('/(?:ab+|cd+)*[z]/', 'xabz'));
var_dump(preg_last_error());
var_dump(preg_match('/foo(?:a+|b+)*[z]/', 'zfooa'));
var_dump(preg_last_error());
var_dump(preg_match('/foo(?:a+|b+)*[z]/', 'zzfooa'));
var_dump(preg_last_error());
var_dump(preg_match('/xfoo(?:a+|b+)*[z]/', 'zxfooa'));
var_dump(preg_last_error());
var_dump(preg_match('/foo(?:a+|b+)*[!?]/', 'zfooa!'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:a|b)*[c]/', 'a'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:a|b)*[c]/', 'd'));
var_dump(preg_last_error());
var_dump(preg_match('/(?:\D+|<\d+>)*[!?]/', '123'));
var_dump(preg_last_error());
var_dump(preg_match('/\S+/', 'foo bar', $matches, 0, 99999));
var_dump(preg_last_error());
var_dump(preg_match("/foo/i\r", 'FOO'));
var_dump(preg_last_error());
$text = json_decode('"\u2019"');
$pattern = '/\b/u';
var_dump(preg_match($pattern, $text, $matches, 0, 0));
var_dump(preg_match($pattern, $text, $matches, 0, 1));
var_dump(preg_last_error() == PREG_BAD_UTF8_OFFSET_ERROR);
$text = "VA\xff"; $text .= "LID";
var_dump(preg_match($pattern, $text, $matches, 0, 4));
var_dump(preg_match($pattern, $text, $matches, 0, 0));
var_dump(preg_last_error() == PREG_BAD_UTF8_ERROR);
var_dump(preg_match('/(?P<size>\d+)m|M/', '4M', $m));
var_dump($m);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nint(2)\nint(1)\nint(0)\narray(1) {\n  [0]=>\n  string(1) \"a\"\n}\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nbool(false)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(1)\nint(0)\nstring(1) \"z\"\nint(1)\nint(0)\nstring(1) \"z\"\nint(1)\nint(0)\nstring(1) \"z\"\nbool(false)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nbool(false)\nint(2)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nint(0)\nbool(false)\nint(1)\nint(1)\nint(0)\nint(0)\nbool(false)\nbool(true)\nint(1)\nbool(false)\nbool(true)\nint(1)\narray(1) {\n  [0]=>\n  string(1) \"M\"\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_ignores_line_breaks_after_pattern_delimiter() {
    let execution = run_source(
        r#"<?php
var_dump(preg_match("/foo/i\r", 'FOO'));
$pattern = hex2bin('2f5c583f3d3f223f3536ff3636ffffffff36a8a8a83636367a7a7a7a3d2aff2f0a');
preg_match($pattern, $pattern);
echo "DONE\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(1)\nDONE\n");
    assert_eq!(execution.exit_code, 0);
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
