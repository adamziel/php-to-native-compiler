use php_compiler::{emit_ir_source, run_source};

#[test]
fn strtr_translates_bytes_and_replace_pairs_with_longest_keys_first() {
    let execution = run_source(
        r##"<?php
$map = array("hello" => "hi", "hi" => "hello", "a" => "A", "world" => "planet");
var_dump(strtr("# hi all, I said hello world! #", $map));
echo strtr("test strtr", "test", "TEST"), "|";
echo strtr("1a2b3c", array("1" => "a", "a" => 1, "2b3c" => "b2c3", "b2c3" => "3c2b"));
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(32) \"# hello All, I sAid hi planet! #\"\nTEST STrTr|a1b2c3"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtr_preserves_binary_bytes_and_reference_backed_replacements() {
    let execution = run_source(
        r##"<?php
$foo = "foo";
$map = array("\0" => "Z", "bar" => &$foo);
echo bin2hex(strtr("a\0bar", $map)), "|";
echo strtr("abc", "abc", "12");
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "615a666f6f|12c");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtr_empty_array_keys_warn_when_subject_is_non_empty() {
    let execution = run_source(
        r##"<?php
var_dump(strtr("foo", array("" => "bar", "x" => "y")));
var_dump(strtr("", array("" => "bar")));
"##,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Warning: strtr(): Ignoring replacement of empty string"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(3) \"foo\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("string(0) \"\"\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtr_type_errors_are_catchable_for_wrong_replace_form() {
    let execution = run_source(
        r##"<?php
try {
    strtr("abc", 1);
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    strtr("abc", array(), array());
} catch (TypeError $e) {
    echo $e->getMessage();
}
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "strtr(): Argument #2 ($from) must be of type array, int given|strtr(): Argument #2 ($from) must be of type string, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strtr_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r##"<?php
$call = "strtr";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$function = new ReflectionFunction("StrTr");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|strtr:2/3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_strtr_metadata() {
    let ir = emit_ir_source(
        r##"<?php
echo function_exists("strtr") ? "1" : "0";
echo is_callable("strtr") ? "1" : "0";
"##,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
