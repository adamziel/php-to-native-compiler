use php_compiler::{emit_ir_source, run_source};

#[test]
fn string_byte_residuals_transform_and_escape_bytes() {
    let execution = run_source(
        r##"<?php
echo strrev("Hello\0World"), "|";
echo str_rot13("Nopqrstuvwxyz 0123"), "|";
echo quotemeta("\+*?[^]($)"), "|";
echo bin2hex(hex2bin("414200ff")), "|";
echo nl2br("A\r\nB\n\rC\rD\n"), "|";
echo ucfirst("hello"), ":", lcfirst("HELLO"), ":";
echo ucwords("testing\twords\rand\nmore"), "|";
echo ucwords("test(braced)words", "()");
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "dlroW\0olleH|Abcdefghijklm 0123|\\\\\\+\\*\\?\\[\\^\\]\\(\\$\\)|414200ff|A<br />\r\nB<br />\n\rC<br />\rD<br />\n|Hello:hELLO:Testing\tWords\rAnd\nMore|Test(Braced)Words"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ord_and_hex2bin_emit_php_shaped_warnings() {
    let execution = run_source(
        r#"<?php
var_dump(ord(""));
var_dump(ord("Hello"));
var_dump(hex2bin("AH"));
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("Deprecated: ord(): Providing an empty string is deprecated"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Deprecated: ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: hex2bin(): Input string must be hexadecimal string"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("int(0)"), "{}", execution.stdout);
    assert!(execution.stdout.contains("int(72)"), "{}", execution.stdout);
    assert!(
        execution.stdout.ends_with("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uuencode_builtins_round_trip_php_byte_strings() {
    let execution = run_source(
        r#"<?php
$values = array(
    "",
    "123",
    "abc",
    "1a2b3c",
    "Here is a simple string to test convert_uuencode/decode",
    "\t This String contains \t\t some control characters\r\n",
    "\x90\x91\x00\x93\x94\x90\x91\x95\x96\x97\x98\x99\x9a\x9b\x9c\x9d\x9e\x9f",
    '\t This String contains \t\t some control characters\r\n',
);
foreach ($values as $value) {
    $encoded = convert_uuencode($value);
    echo bin2hex(convert_uudecode($encoded)) === bin2hex($value) ? "1" : "0";
}
echo "\n";
echo convert_uuencode("123");
echo convert_uuencode("");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111111\n#,3(S\n`\n`\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn uudecode_invalid_inputs_emit_php_shaped_warnings() {
    let execution = run_source(
        r#"<?php
$encoded = convert_uuencode("not very sophisticated");
var_dump(convert_uudecode("!@#$%^YUGFDFGHJKLUYTFBNMLOYT"));
var_dump(convert_uudecode(""));
var_dump(convert_uudecode(substr($encoded, 0, -10)));
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains("string(1) \""),
        "{}",
        execution.stdout
    );
    assert_eq!(
        execution
            .stdout
            .matches(
                "Warning: convert_uudecode(): Argument #1 ($data) is not a valid uuencoded string"
            )
            .count(),
        2,
        "{}",
        execution.stdout
    );
    assert_eq!(
        execution.stdout.matches("bool(false)").count(),
        2,
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_residual_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["strrev", "str_rot13", "hex2bin", "ord", "convert_uuencode", "convert_uudecode", "quotemeta", "nl2br", "ucfirst", "lcfirst", "ucwords"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11:1/1;11:1/1;11:1/1;11:1/1;11:1/1;11:1/1;11:1/1;11:1/2;11:1/1;11:1/1;11:1/2;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_string_residual_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strrev") ? "1" : "0";
echo function_exists("hex2bin") ? "1" : "0";
echo function_exists("convert_uuencode") ? "1" : "0";
echo is_callable("convert_uudecode") ? "1" : "0";
echo is_callable("ucwords") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("ucwords"), "{ir}");
}
