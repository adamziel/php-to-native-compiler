use php_compiler::run_source;

#[test]
fn foreach_supports_positional_value_destructuring() {
    let execution = run_source(
        r#"<?php
$rows = [["alpha", 10, "skip"], ["beta", 20, "skip"]];
$out = [];
foreach ($rows as [$name, $score,]) {
    $out[] = $name . ":" . $score;
}
foreach ($rows as $index => list(, $score)) {
    $out[] = $index . "=" . $score;
}
echo implode("|", $out);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "alpha:10|beta:20|0=10|1=20");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_supports_nested_and_literal_keyed_list_destructuring() {
    let execution = run_source(
        r#"<?php
$multi = [
    [[1, 2], [3, 4]],
    [[5, 6], [7, 8]],
];

foreach ($multi as list(list($a, $b), list($c, $d))) {
    echo $a, $b, $c, $d, "\n";
}
foreach ($multi as $key => list(list($a, $b), list($c, $d))) {
    echo $key, $a, $b, $c, $d, "\n";
}

$points = [
    ["x" => 1, "y" => 2],
    ["x" => 2, "y" => 1],
];
foreach ($points as list("x" => $x, "y" => $y)) {
    echo $x, ":", $y, "\n";
}

$columns = [
    "x" => [1, 2],
    "y" => [2, 1],
];
foreach ($columns as list(0 => $row1, 1 => $row2)) {
    echo $row1, ":", $row2, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1234\n5678\n01234\n15678\n1:2\n2:1\n1:2\n2:1\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_destructuring_warns_for_scalar_values_and_assigns_null() {
    let execution = run_source(
        r#"<?php
foreach ([["a", "b"], "c", 10, null] as list(, $value)) {
    var_dump($value);
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("string(1) \"b\""));
    assert!(execution
        .stdout
        .contains("Warning: Cannot use string as array"));
    assert!(execution
        .stdout
        .contains("Warning: Cannot use int as array"));
    assert_eq!(execution.stdout.matches("NULL").count(), 3);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn foreach_empty_list_and_list_key_emit_php_fatals() {
    let empty_list = run_source("<?php\nforeach ([[1]] as list()) {}\n").unwrap();
    assert_eq!(
        empty_list.stdout,
        "Fatal error: Cannot use empty list in Command line code on line 2"
    );
    assert_eq!(empty_list.exit_code, 255);

    let list_key = run_source("<?php\nforeach ([[1]] as list($key) => list($value)) {}\n").unwrap();
    assert_eq!(
        list_key.stdout,
        "Fatal error: Cannot use list as key element in Command line code on line 2"
    );
    assert_eq!(list_key.exit_code, 255);
}

#[test]
fn foreach_rejects_by_reference_destructuring() {
    let error = run_source("<?php\nforeach ([[1]] as [&$value]) {}\n").unwrap_err();

    assert!(
        error
            .message
            .contains("by-value positional destructuring loop targets"),
        "{}",
        error.message
    );
}
