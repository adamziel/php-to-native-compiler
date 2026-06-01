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
fn preg_match_preserves_named_capture_order_and_no_auto_capture_modifier() {
    let execution = run_source(
        r#"<?php
preg_match('/(?P<first>.)(x)(?P<tail>\S+)/', 'zxax', $match, PREG_OFFSET_CAPTURE);
var_export($match);
echo "\n--\n";
preg_match_all('/(?<a>4)?(?<b>2)?\d/', '123456', $all, PREG_UNMATCHED_AS_NULL);
var_export($all);
echo "\n--\n";
preg_match_all('/(?<a>4)?(?<b>2)?\d/', '123456', $set, PREG_SET_ORDER | PREG_UNMATCHED_AS_NULL);
var_export($set[1]);
echo "\n--\n";
preg_match('/.(.)./n', 'abc', $auto);
var_export($auto);
echo "\n";
preg_match('/.(?P<named>.)./n', 'abc', $named);
var_export($named);
echo "\n";
"#,
    )
    .unwrap();

    let expected = concat!(
        "array (\n",
        "  0 => \n",
        "  array (\n",
        "    0 => 'zxax',\n",
        "    1 => 0,\n",
        "  ),\n",
        "  'first' => \n",
        "  array (\n",
        "    0 => 'z',\n",
        "    1 => 0,\n",
        "  ),\n",
        "  1 => \n",
        "  array (\n",
        "    0 => 'z',\n",
        "    1 => 0,\n",
        "  ),\n",
        "  2 => \n",
        "  array (\n",
        "    0 => 'x',\n",
        "    1 => 1,\n",
        "  ),\n",
        "  'tail' => \n",
        "  array (\n",
        "    0 => 'ax',\n",
        "    1 => 2,\n",
        "  ),\n",
        "  3 => \n",
        "  array (\n",
        "    0 => 'ax',\n",
        "    1 => 2,\n",
        "  ),\n",
        ")\n",
        "--\n",
        "array (\n",
        "  0 => \n",
        "  array (\n",
        "    0 => '1',\n",
        "    1 => '23',\n",
        "    2 => '45',\n",
        "    3 => '6',\n",
        "  ),\n",
        "  'a' => \n",
        "  array (\n",
        "    0 => NULL,\n",
        "    1 => NULL,\n",
        "    2 => '4',\n",
        "    3 => NULL,\n",
        "  ),\n",
        "  1 => \n",
        "  array (\n",
        "    0 => NULL,\n",
        "    1 => NULL,\n",
        "    2 => '4',\n",
        "    3 => NULL,\n",
        "  ),\n",
        "  'b' => \n",
        "  array (\n",
        "    0 => NULL,\n",
        "    1 => '2',\n",
        "    2 => NULL,\n",
        "    3 => NULL,\n",
        "  ),\n",
        "  2 => \n",
        "  array (\n",
        "    0 => NULL,\n",
        "    1 => '2',\n",
        "    2 => NULL,\n",
        "    3 => NULL,\n",
        "  ),\n",
        ")\n",
        "--\n",
        "array (\n",
        "  0 => '23',\n",
        "  'a' => NULL,\n",
        "  1 => NULL,\n",
        "  'b' => '2',\n",
        "  2 => '2',\n",
        ")\n",
        "--\n",
        "array (\n",
        "  0 => 'abc',\n",
        ")\n",
        "array (\n",
        "  0 => 'abc',\n",
        "  'named' => 'b',\n",
        "  1 => 'b',\n",
        ")\n",
    );
    assert_eq!(execution.stdout, expected);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_match_preserves_intermediate_unmatched_captures() {
    let execution = run_source(
        r#"<?php
preg_match('/(?P<date>(?P<year>(\d{2})?\d\d)-(?P<month>(?:\d\d|[a-zA-Z]{2,3}))-(?P<day>[0-3]?\d))/', '06-12-12', $match);
var_export($match);
echo "\n--\n";
preg_match_all('/(?P<date>(?P<year>(\d{2})?\d\d)-(?P<month>(?:\d\d|[a-zA-Z]{2,3}))-(?P<day>[0-3]?\d))/', '2006-05-13 12-Aug-37', $matches, PREG_SET_ORDER);
var_export($matches[1]);
echo "\n";
"#,
    )
    .unwrap();

    let expected = concat!(
        "array (\n",
        "  0 => '06-12-12',\n",
        "  'date' => '06-12-12',\n",
        "  1 => '06-12-12',\n",
        "  'year' => '06',\n",
        "  2 => '06',\n",
        "  3 => '',\n",
        "  'month' => '12',\n",
        "  4 => '12',\n",
        "  'day' => '12',\n",
        "  5 => '12',\n",
        ")\n",
        "--\n",
        "array (\n",
        "  0 => '12-Aug-37',\n",
        "  'date' => '12-Aug-37',\n",
        "  1 => '12-Aug-37',\n",
        "  'year' => '12',\n",
        "  2 => '12',\n",
        "  3 => '',\n",
        "  'month' => 'Aug',\n",
        "  4 => 'Aug',\n",
        "  'day' => '37',\n",
        "  5 => '37',\n",
        ")\n",
    );
    assert_eq!(execution.stdout, expected);
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
