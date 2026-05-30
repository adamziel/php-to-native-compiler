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
fn bounded_pack_unpack_hex_and_padding_formats_preserve_bytes() {
    let execution = run_source(
        r#"<?php
echo bin2hex(pack("x")), "\n";
echo bin2hex(pack("H3", "181")), ":", unpack("H3", pack("H3", "181"))[1], "\n";
echo bin2hex(pack("H*", "a")), ":", unpack("H*", pack("H*", "a"))[1], "\n";
echo bin2hex(pack("h3", "181")), ":", unpack("h3", pack("h3", "181"))[1], "\n";
foreach (["pack", "unpack"] as $name) {
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
        "00\n1810:181\na0:a0\n8101:181\n11:1/2;11:2/3;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_residual_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["strrev", "str_rot13", "hex2bin", "pack", "unpack", "ord", "quotemeta", "nl2br", "ucfirst", "lcfirst", "ucwords"] as $name) {
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
        "11:1/1;11:1/1;11:1/1;11:1/2;11:2/3;11:1/1;11:1/1;11:1/2;11:1/1;11:1/1;11:1/2;"
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

#[test]
fn string_wordwrap_uuencode_and_natural_compare_builtins_cover_phpt_families() {
    let execution = run_source(
        r#"<?php
echo convert_uudecode(convert_uuencode("Cat\nDog")) === "Cat\nDog" ? "uu" : "bad";
echo "|", wordwrap("The quick brown fox", 9, "|");
echo "|", implode(",", str_word_count("Hello fri3nd, you're looking good", 1, "3"));
echo "|", strnatcmp("img2", "img10") < 0 ? "nat" : "bad";
echo "|", strnatcasecmp("A10", "a2") > 0 ? "case" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "uu|The quick|brown fox|Hello,fri3nd,you're,looking,good|nat|case"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_wordwrap_uuencode_and_natural_compare_metadata_is_available() {
    let execution = run_source(
        r#"<?php
$names = ["convert_uuencode", "convert_uudecode", "wordwrap", "str_word_count", "strnatcmp", "strnatcasecmp"];
foreach ($names as $name) {
    $reflection = new ReflectionFunction($name);
    echo $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "convert_uuencode:1/1;convert_uudecode:1/1;wordwrap:1/4;str_word_count:1/3;strnatcmp:2/2;strnatcasecmp:2/2;"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_word_count_matches_ascii_hyphen_boundaries() {
    let execution = run_source(
        r#"<?php
echo implode(",", str_word_count("foo--bar", 1)), "\n";
echo implode(",", str_word_count("foo - bar", 1)), "\n";
echo implode(",", str_word_count("foo- -bar", 1)), "\n";
echo implode(",", str_word_count("foo-1bar", 1)), "\n";
echo implode(",", str_word_count("-foo-", 1)), "|", implode(",", str_word_count("-foo-", 1, "-"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "foo--bar\nfoo,-,bar\nfoo-,-bar\nfoo-,bar\nfoo|-foo-"
    );
    assert_eq!(execution.exit_code, 0);
}
