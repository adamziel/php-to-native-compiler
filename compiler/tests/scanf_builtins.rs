use php_compiler::run_source;

#[test]
fn sscanf_returns_array_and_assigns_direct_targets() {
    let execution = run_source(
        r#"<?php
$values = sscanf("Part: Widget Serial Number: 1234789 Stock: 25", "Part: %s Serial Number: %d Stock: %d");
echo $values[0] . "|" . $values[1] . "|" . $values[2] . "\n";
$count = sscanf("-2147483649 2147483648 34359738369 -11 +11 11", "%u %u %u %u %u %u", $first, $second, $third, $fourth, $fifth, $sixth);
var_dump($count, $first, $second, $third, $fourth, $fifth, $sixth);
try {
    sscanf("Hello World", "%s %s", $left, $right, $extra);
} catch (ValueError $e) {
    echo "caught:" . $e->getMessage() . "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Widget|1234789|25\nint(6)\nint(2147483647)\nstring(10) \"2147483648\"\nstring(10) \"4294967295\"\nstring(10) \"4294967285\"\nint(11)\nint(11)\ncaught:Variable is not assigned by any conversion specifiers\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sscanf_positional_and_count_conversions_match_string_phpt_rows() {
    let execution = run_source(
        r#"<?php
$foo = "ABC = DEF";
$fmt = "%s = %s %n";
sscanf($foo, $fmt, $res_a[0], $res_a[1], $res_a[2]);
$res_b = sscanf($foo, $fmt);
var_dump($res_a);
var_dump($res_b);

var_dump(sscanf('one two', '%1$s %2$s'));
var_dump(sscanf('one two', '%2$s %1$s'));
sscanf('one two', '%2$s %1$s', $left, $right);
var_dump($left, $right);

$str = "a b c d e";
var_dump(sscanf("a ", '%1$s', $str));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array(3) {\n  [0]=>\n  string(3) \"ABC\"\n  [1]=>\n  string(3) \"DEF\"\n  [2]=>\n  int(9)\n}\narray(3) {\n  [0]=>\n  string(3) \"ABC\"\n  [1]=>\n  string(3) \"DEF\"\n  [2]=>\n  int(9)\n}\narray(2) {\n  [0]=>\n  string(3) \"one\"\n  [1]=>\n  string(3) \"two\"\n}\narray(2) {\n  [0]=>\n  string(3) \"two\"\n  [1]=>\n  string(3) \"one\"\n}\nstring(3) \"two\"\nstring(3) \"one\"\nint(1)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sscanf_keeps_existing_direct_targets_after_failed_later_conversion() {
    let execution = run_source(
        r#"<?php
sscanf(":59:58", "%s:%d:%f", $a, $b, $c);
echo "[$a][$b][$c]\n";
sscanf("15:01:58.2", "%d:%f:%f", $a, $b, $c);
echo "[$a][$b][$c]\n";
sscanf("15.1111::foo", "%f:%d:%s", $a, $b, $c);
echo "[$a][$b][$c]\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "[:59:58][][]\n[15][1][58.2]\n[15.1111][1][58.2]\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
