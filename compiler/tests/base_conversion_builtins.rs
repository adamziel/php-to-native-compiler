use php_compiler::{emit_ir_source, run_source};

#[test]
fn decimal_base_string_builtins_execute_current_scalar_subset() {
    let execution = run_source(
        r#"<?php
echo dechex(255), "\n";
echo decbin(10), "\n";
echo decoct(64), "\n";
echo bindec("0b1010"), "\n";
echo octdec("0o70"), "\n";
echo hexdec("0x1f"), "\n";
echo dechex(-1), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ff\n1010\n100\n10\n56\n31\nffffffffffffffff\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn binary_and_octal_decoders_ignore_invalid_digits_with_deprecation() {
    let execution = run_source(
        r#"<?php
echo bindec("100002001"), "\n";
echo octdec("0x1234ABC"), "\n";
echo bindec(1111111111111111111111111111111), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Deprecated: Invalid characters passed for attempted conversion, these have been ignored in Command line code on line 2\n129\n\nDeprecated: Invalid characters passed for attempted conversion, these have been ignored in Command line code on line 3\n668\n\nDeprecated: Invalid characters passed for attempted conversion, these have been ignored in Command line code on line 4\n32766\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base_convert_executes_prefix_and_base36_subset() {
    let execution = run_source(
        r#"<?php
echo base_convert("0xFF", 16, 10), "\n";
echo base_convert("0b1010", 2, 10), "\n";
echo base_convert("0o7", 8, 10), "\n";
echo base_convert("zz", 36, 10), "\n";
echo base_convert("255", 10, 16), "\n";
echo base_convert("", 16, 10), "\n";
echo .5, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "255\n10\n7\n1295\nff\n0\n0.5\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base_convert_invalid_base_errors_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach ([[1234, 1, 10], [1234, 10, 37]] as $args) {
    try {
        base_convert($args[0], $args[1], $args[2]);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "base_convert(): Argument #2 ($from_base) must be between 2 and 36 (inclusive)\nbase_convert(): Argument #3 ($to_base) must be between 2 and 36 (inclusive)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base_conversion_float_stringification_matches_php_precision_for_conversion() {
    let execution = run_source(
        r#"<?php
echo hexdec(12.3456789000E-10), "\n";
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("1250999896553\n"));
    assert!(execution
        .stdout
        .contains("Deprecated: Invalid characters passed"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base_conversion_metadata_and_dynamic_calls_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["dechex", "decbin", "decoct", "bindec", "octdec", "hexdec", "base_convert"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
}
echo "|";
$call = "base_convert";
echo $call("ff", 16, 2);
echo "|";
$bindec = new ReflectionFunction("bindec");
$octdec = new ReflectionFunction("octdec");
echo $bindec->getName(), ":", $bindec->getNumberOfRequiredParameters(), "/", $bindec->getNumberOfParameters(), "|";
echo $octdec->getName(), ":", $octdec->getNumberOfRequiredParameters(), "/", $octdec->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11111111111111|11111111|bindec:1/1|octdec:1/1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_base_conversion_capability_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("dechex") ? "1" : "0";
echo is_callable("dechex") ? "1" : "0";
echo function_exists("decbin") ? "1" : "0";
echo is_callable("decbin") ? "1" : "0";
echo function_exists("decoct") ? "1" : "0";
echo is_callable("decoct") ? "1" : "0";
echo function_exists("bindec") ? "1" : "0";
echo is_callable("bindec") ? "1" : "0";
echo function_exists("octdec") ? "1" : "0";
echo is_callable("octdec") ? "1" : "0";
echo function_exists("hexdec") ? "1" : "0";
echo is_callable("hexdec") ? "1" : "0";
echo function_exists("base_convert") ? "1" : "0";
echo is_callable("base_convert") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 14, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
