use php_compiler::{emit_ir_source, run_source};

#[test]
fn str_pad_supports_default_left_right_and_both_padding() {
    let execution = run_source(
        r#"<?php
echo str_pad("php", 6), "|";
echo str_pad("php", 6, "0", STR_PAD_LEFT), "|";
echo str_pad("php", 8, "ab", STR_PAD_BOTH), "|";
echo str_pad("already", 3, "*", STR_PAD_LEFT);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "php   |000php|abphpaba|already");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_pad_truncates_repeated_pad_string_to_exact_byte_length() {
    let execution = run_source(
        r#"<?php
echo str_pad("x", 6, "abc", STR_PAD_RIGHT), "|";
echo str_pad("x", 6, "abc", STR_PAD_LEFT), "|";
echo strlen(str_pad("é", 5, "xy", STR_PAD_RIGHT));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "xabcab|abcabx|5");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_pad_metadata_and_constants_are_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
$call = "str_pad";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
echo STR_PAD_LEFT, ":", STR_PAD_RIGHT, ":", STR_PAD_BOTH;
echo "|";
$function = new ReflectionFunction("Str_Pad");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|0:1:2|str_pad:2/4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_pad_rejects_empty_pad_string_and_bad_pad_type() {
    let empty = run_source(
        r#"<?php
try {
    str_pad("x", 3, "");
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        empty.stdout,
        "str_pad(): Argument #3 ($pad_string) must not be empty"
    );

    let bad_type = run_source(
        r#"<?php
try {
    str_pad("x", 3, " ", 99);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_type.stdout,
        "str_pad(): Argument #4 ($pad_type) must be STR_PAD_LEFT, STR_PAD_RIGHT, or STR_PAD_BOTH"
    );
}

#[test]
fn str_pad_large_lengths_raise_php_shaped_type_and_memory_errors() {
    let type_error = run_source(
        r#"<?php
try {
    str_pad("x", PHP_INT_MAX * 5);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        type_error.stdout,
        "str_pad(): Argument #2 ($length) must be of type int, float given"
    );

    let fatal = run_source(
        r#"<?php
echo "before\n";
str_pad("x", PHP_INT_MAX);
"#,
    )
    .unwrap();

    assert!(fatal.stdout.contains(
        "Fatal error: Allowed memory size of 134217728 bytes exhausted (tried to allocate"
    ));
    assert_eq!(fatal.exit_code, 255);
}

#[test]
fn str_pad_long_output_feeds_setlocale_length_warning() {
    let execution = run_source(
        r#"<?php
echo "locale";
var_dump(setlocale(LC_ALL, str_pad("", 255, "A")));
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Warning: setlocale(): Specified locale name is too long in Command line code on line"
    ));
    assert!(execution.stdout.ends_with("bool(false)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_str_pad_metadata_and_pad_constants() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_pad") ? "1" : "0";
echo is_callable("str_pad") ? "1" : "0";
echo defined("STR_PAD_LEFT") ? "1" : "0";
echo defined("STR_PAD_RIGHT") ? "1" : "0";
echo defined("STR_PAD_BOTH") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("STR_PAD_LEFT"), "{ir}");
    assert!(!ir.contains("STR_PAD_RIGHT"), "{ir}");
    assert!(!ir.contains("STR_PAD_BOTH"), "{ir}");
}
