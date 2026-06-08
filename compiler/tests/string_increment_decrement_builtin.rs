use php_compiler::run_source;

#[test]
fn str_increment_matches_ascii_carry_cases() {
    let execution = run_source(
        r#"<?php
$values = ["Az", "aZ", "A9", "a9", "Zz", "zZ", "9z", "9Z", "5e6", "5E9", "d", "D", "4"];
foreach ($values as $value) {
    echo str_increment($value), "|", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Ba|Az\nbA|aZ\nB0|A9\nb0|a9\nAAa|Zz\naaA|zZ\n10a|9z\n10A|9Z\n5e7|5e6\n5F0|5E9\ne|d\nE|D\n5|4\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_decrement_matches_ascii_borrow_and_underflow_cases() {
    let execution = run_source(
        r#"<?php
$values = ["Az", "aZ", "A9", "a9", "Za", "zA", "Z0", "z0", "Aa", "aA", "A0", "a0", "10", "1A", "1a", "10a", "5e6", "d", "D", "4", "1"];
foreach ($values as $value) {
    echo str_decrement($value), "|", $value, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Ay|Az\naY|aZ\nA8|A9\na8|a9\nYz|Za\nyZ|zA\nY9|Z0\ny9|z0\nz|Aa\nZ|aA\n9|A0\n9|a0\n9|10\nZ|1A\nz|1a\n9z|10a\n5e5|5e6\nc|d\nC|D\n3|4\n0|1\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_increment_and_decrement_value_errors_are_catchable() {
    let execution = run_source(
        r#"<?php
foreach (["", "-cc", "foo1.txt"] as $value) {
    try {
        str_increment($value);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
foreach (["", "0", "a", "A", "00", "0a", "0A", "01", "09", "0B", "0b", "0Z", "0z", "α"] as $value) {
    try {
        str_decrement($value);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "str_increment(): Argument #1 ($string) must not be empty\n\
str_increment(): Argument #1 ($string) must be composed only of alphanumeric ASCII characters\n\
str_increment(): Argument #1 ($string) must be composed only of alphanumeric ASCII characters\n\
str_decrement(): Argument #1 ($string) must not be empty\n\
str_decrement(): Argument #1 ($string) \"0\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"a\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"A\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"00\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0a\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0A\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"01\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"09\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0B\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0b\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0Z\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) \"0z\" is out of decrement range\n\
str_decrement(): Argument #1 ($string) must be composed only of alphanumeric ASCII characters\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_increment_and_decrement_metadata_is_available() {
    let execution = run_source(
        r#"<?php
foreach (["str_increment", "str_decrement"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ":";
    echo $name === "str_increment" ? $name("A9") : $name("A9");
    echo ";";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1:B0;11:1/1:A8;");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
