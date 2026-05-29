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
fn string_residual_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["strrev", "str_rot13", "hex2bin", "ord", "quotemeta", "nl2br", "ucfirst", "lcfirst", "ucwords"] as $name) {
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
        "11:1/1;11:1/1;11:1/1;11:1/1;11:1/1;11:1/2;11:1/1;11:1/1;11:1/2;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_string_residual_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strrev") ? "1" : "0";
echo function_exists("hex2bin") ? "1" : "0";
echo is_callable("ucwords") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("ucwords"), "{ir}");
}
