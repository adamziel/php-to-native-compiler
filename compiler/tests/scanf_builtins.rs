use php_compiler::run_source;

#[test]
fn sscanf_returns_array_and_assigns_direct_targets() {
    let execution = run_source(
        r#"<?php
$values = sscanf("Part: Widget Serial Number: 1234789 Stock: 25", "Part: %s Serial Number: %d Stock: %d");
echo $values[0] . "|" . $values[1] . "|" . $values[2] . "\n";
$count = sscanf("-2147483649 2147483648 34359738369 -11 +11 11", "%u %u %u %u %u %u", $first, $second, $third, $fourth, $fifth, $sixth);
var_dump($count, $first, $second, $third, $fourth, $fifth, $sixth);
$positional = sscanf("one two", '%2$s %1$s');
echo $positional[0], "|", $positional[1], "\n";
$withCount = sscanf("ABC = DEF", "%s = %s %n");
echo $withCount[0], "|", $withCount[1], "|", $withCount[2], "\n";
$holder = array();
sscanf("ABC = DEF", "%s = %s %n", $holder[0], $holder[1], $holder[2]);
echo $holder[0], "|", $holder[1], "|", $holder[2], "\n";
sscanf(":59:58", "%s:%d:%f", $firstFail, $secondFail, $thirdFail);
echo "[", $firstFail, "][", $secondFail, "][", $thirdFail, "]\n";
$a = "old-a";
$b = "old-b";
$c = "old-c";
sscanf("15.1111::foo", "%f:%d:%s", $a, $b, $c);
echo $a, "|", $b, "|", $c, "\n";
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
        "Widget|1234789|25\nint(6)\nstring(20) \"18446744071562067967\"\nint(2147483648)\nint(34359738369)\nstring(20) \"18446744073709551605\"\nint(11)\nint(11)\ntwo|one\nABC|DEF|9\nABC|DEF|9\n[:59:58][][]\n15.1111|old-b|old-c\ncaught:Variable is not assigned by any conversion specifiers\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
