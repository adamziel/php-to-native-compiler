use php_compiler::{emit_ir_source, run_source};

#[test]
fn str_split_splits_default_and_custom_byte_lengths() {
    let execution = run_source(
        r#"<?php
$parts = str_split("abcde", 2);
echo $parts[0], "|", $parts[1], "|", $parts[2], "\n";
$default = str_split("php");
echo $default[0], ":", $default[1], ":", $default[2];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ab|cd|e\np:h:p");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_split_preserves_binary_bytes_and_empty_string_shape() {
    let execution = run_source(
        r#"<?php
$bytes = str_split("A\0B" . chr(255), 1);
echo bin2hex($bytes[0]), ":", bin2hex($bytes[1]), ":", bin2hex($bytes[2]), ":", bin2hex($bytes[3]), "|";
$empty = str_split("");
echo isset($empty[0]) ? "not-empty" : "empty";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "41:00:42:ff|empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_split_length_errors_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach ([0, -3] as $length) {
    try {
        str_split("abc", $length);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "str_split(): Argument #2 ($length) must be greater than 0\nstr_split(): Argument #2 ($length) must be greater than 0\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_split_metadata_and_string_callable_are_available() {
    let execution = run_source(
        r#"<?php
$call = "str_split";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$parts = $call("abcd", 2);
echo $parts[0], ":", $parts[1];
echo "|";
$function = new ReflectionFunction("Str_Split");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|ab:cd|str_split:1/2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_str_split_capability_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_split") ? "1" : "0";
echo is_callable("str_split") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
