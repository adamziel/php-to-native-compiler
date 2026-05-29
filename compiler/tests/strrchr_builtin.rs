use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strrchr_returns_tail_from_last_first_needle_byte() {
    let execution = run_source(
        r#"<?php
var_dump(strrchr("Hello, World", "World"));
var_dump(strrchr("Hello, World", "World", true));
var_dump(strrchr("Hello, World", "ooo"));
var_dump(strrchr("Hello, World", "ooo", true));
var_dump(strrchr("Hello, World", "Zzzz"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(5) \"World\"\nstring(7) \"Hello, \"\nstring(4) \"orld\"\nstring(8) \"Hello, W\"\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strrchr_treats_empty_needle_as_nul_byte() {
    let execution = run_source(
        r#"<?php
var_dump(strrchr("abc", ""));
var_dump(strrchr("Hello" . chr(0) . "World", ""));
var_dump(strrchr("Hello" . chr(0) . "World", chr(0), true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(false)\nstring(6) \"\0World\"\nstring(5) \"Hello\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strrchr_uses_php_double_quoted_escape_bytes() {
    let execution = run_source(
        r#"<?php
$value = "\escape \\tail";
var_dump(strrchr($value, "\e"));
var_dump(strrchr($value, "\\"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(12) \"\u{1b}scape \\tail\"\nstring(5) \"\\tail\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strrchr_is_available_through_metadata_and_dynamic_calls() {
    let execution = run_source(
        r#"<?php
$call = "strrchr";
echo function_exists($call) ? "fn" : "missing";
echo "|", is_callable("strrchr") ? "callable" : "missing";
echo "|", $call("abcabc", "b");
echo "|";
$function = new ReflectionFunction("StrRchr");
echo $function->getName(), ":", $function->getNumberOfParameters(), ":", $function->getNumberOfRequiredParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|bc|strrchr:3:2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strrchr_rejects_array_operands_and_bad_arity() {
    let array_haystack = run_source("<?php\nstrrchr(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call strrchr(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstrrchr('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call strrchr(): needle argument arrays are not implemented in the current subset"
    );

    let arity = run_source("<?php\nstrrchr('abc');\n").unwrap_err();
    assert_eq!(arity.phase, Phase::Runtime);
    assert_eq!(arity.line, 2);
    assert_eq!(arity.column, 1);
    assert_eq!(
        arity.message,
        "arity mismatch for strrchr(): expected 2 to 3 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_strrchr_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strrchr") ? "1" : "0";
echo is_callable("strrchr") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
