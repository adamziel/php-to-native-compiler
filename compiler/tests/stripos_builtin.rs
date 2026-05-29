use php_compiler::{emit_ir_source, run_source};

#[test]
fn stripos_finds_ascii_case_insensitive_byte_offsets() {
    let execution = run_source(
        r#"<?php
var_dump(stripos("test string", "TEST"));
var_dump(stripos("test string", "strIng"));
var_dump(stripos("te" . chr(0) . "st", chr(0)));
var_dump(stripos("aBAbaBAbaBabAbAbaBa", "BAB", 4));
var_dump(stripos("aBAbaBAbaBabAbAbaBa", "BAB", -8));
var_dump(stripos("a", ""));
var_dump(stripos("", "a"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(0)\nint(5)\nint(2)\nint(5)\nint(11)\nint(0)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn stripos_reports_php_shaped_offset_value_errors() {
    let execution = run_source(
        r#"<?php
foreach ([12, -12] as $offset) {
    try {
        stripos("Hello World", "o", $offset);
    } catch (ValueError $exception) {
        echo $exception->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "stripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\nstripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn stripos_metadata_and_native_function_membership_are_available() {
    let execution = run_source(
        r#"<?php
$call = "stripos";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$function = new ReflectionFunction("StRiPoS");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|stripos:2/3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_stripos_function_membership() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("stripos") ? "1" : "0";
echo is_callable("stripos") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
