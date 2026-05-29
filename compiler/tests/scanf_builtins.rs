use php_compiler::run_source;

#[test]
fn sscanf_returns_array_and_assigns_direct_targets() {
    let execution = run_source(
        r#"<?php
$values = sscanf("Part: Widget Serial Number: 1234789 Stock: 25", "Part: %s Serial Number: %d Stock: %d");
echo $values[0] . "|" . $values[1] . "|" . $values[2] . "\n";
$count = sscanf("-11 +11 11", "%u %u %u", $first, $second, $third);
var_dump($count, $first, $second, $third);
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
        "Widget|1234789|25\nint(3)\nstring(10) \"4294967285\"\nint(11)\nint(11)\ncaught:Variable is not assigned by any conversion specifiers\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
