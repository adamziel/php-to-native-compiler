use php_compiler::run_source;

#[test]
fn preg_quote_escapes_nul_as_pcre_octal_literal() {
    let execution = run_source(
        r#"<?php
$str = "a\000b";
$quoted = preg_quote($str);
echo strlen($quoted), "|", $quoted, "|";
echo preg_match("!{$quoted}!", $str);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "6|a\\000b|1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_quote_escaped_hash_matches_under_extended_modifier() {
    let execution = run_source(
        r##"<?php
echo preg_quote("#"), "|";
echo preg_match("~^(" . preg_quote("hello#world", "~") . ")\z~x", "hello#world", $matches);
echo "|", $matches[1];
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "\\#|1|hello#world");
    assert_eq!(execution.exit_code, 0);
}
