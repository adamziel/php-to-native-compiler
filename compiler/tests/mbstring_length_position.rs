use php_compiler::run_source;

#[test]
fn mb_strlen_counts_utf8_and_converted_japanese_encodings() {
    let execution = run_source(
        r#"<?php
$text = "0123この文字列は日本語です。EUC-JPを使っています。0123日本語は面倒臭い。";
$euc = mb_convert_encoding($text, "EUC-JP", "UTF-8");
$sjis = mb_convert_encoding($euc, "SJIS", "EUC-JP");
$jis = mb_convert_encoding($euc, "JIS", "EUC-JP");
$utf8 = mb_convert_encoding($euc, "UTF-8", "EUC-JP");
echo mb_strlen("abc def"), "|";
echo mb_strlen($euc, "EUC-JP"), ":", strlen($euc), "|";
echo mb_strlen($sjis, "SJIS"), ":", strlen($sjis), "|";
echo mb_strlen($jis, "JIS"), ":", strlen($jis), "|";
echo mb_strlen($utf8, "UTF-8"), ":", strlen($utf8);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "7|43:72|43:72|43:90|43:101");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strlen_recognizes_legacy_labels_and_reports_invalid_encoding() {
    let execution = run_source(
        r#"<?php
$labels = ["UCS-4", "UTF-16LE", "UTF-8", "ASCII", "EUC-JP", "SJIS-win", "CP932", "MacJapanese", "ISO-8859-1", "Windows-1251", "KOI8-R"];
foreach ($labels as $label) {
    echo $label, "=", mb_strlen("abc def", $label) ? "1" : "0", ";";
}
try {
    mb_strlen("abcdef", "unknown-encoding");
} catch (ValueError $e) {
    echo "|", $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "UCS-4=1;UTF-16LE=1;UTF-8=1;ASCII=1;EUC-JP=1;SJIS-win=1;CP932=1;MacJapanese=1;ISO-8859-1=1;Windows-1251=1;KOI8-R=1;|mb_strlen(): Argument #2 ($encoding) must be a valid encoding, \"unknown-encoding\" given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strpos_and_stripos_use_character_offsets() {
    let execution = run_source(
        r#"<?php
$text = "日本語テキストです。01234５６７８９。";
echo mb_strpos($text, "語"), "|";
echo mb_strpos($text, "", -2), "|";
echo mb_strpos("abc def", "d", 2, "ISO-8859-1"), "|";
echo mb_stripos("κλμνοπ κλΜΝΟΠ", "ΜΝΟΠ"), "|";
echo mb_stripos("abc defabc def", "DE", 6), "|";
var_dump(mb_strpos("abc", "z"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2|19|4|2|11|bool(false)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strpos_family_reports_value_errors_for_invalid_offsets_and_encodings() {
    let execution = run_source(
        r#"<?php
foreach (["mb_strpos", "mb_stripos", "mb_strrpos", "mb_strripos"] as $fn) {
    try {
        $fn("f", "bar", 3);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
try {
    mb_strpos("Hello, world", "world", 2, "unknown-encoding");
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "mb_strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n",
            "mb_stripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n",
            "mb_strrpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n",
            "mb_strripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n",
            "mb_strpos(): Argument #4 ($encoding) must be a valid encoding, \"unknown-encoding\" given",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mbstring_helpers_track_internal_encoding_subset() {
    let execution = run_source(
        r#"<?php
echo mb_internal_encoding(), "|";
var_dump(mb_internal_encoding("EUC-JP"));
echo mb_internal_encoding(), "|";
var_dump(mb_detect_order("auto"));
var_dump(mb_check_encoding("abc"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "UTF-8|bool(true)\nEUC-JP|bool(true)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strlen_deprecates_legacy_encodings_once_per_request() {
    let execution = run_source(
        r#"<?php
echo mb_strlen("abc", "BASE64"), "|";
echo mb_strlen("def", "BASE64"), "|";
echo mb_strlen("abc", "HTML-ENTITIES");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Deprecated: mb_strlen(): Handling Base64 via mbstring is deprecated; use base64_encode/base64_decode instead in Command line code on line 2\n",
            "3|3|\n",
            "Deprecated: mb_strlen(): Handling HTML entities via mbstring is deprecated; use htmlspecialchars, htmlentities, or mb_encode_numericentity/mb_decode_numericentity instead in Command line code on line 4\n",
            "3",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn print_expression_participates_in_logical_or() {
    let execution = run_source(
        r#"<?php
true or print("bad\n");
false or print("fallback\n");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fallback\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strlen_rejects_array_string_operands() {
    let execution = run_source("<?php\nmb_strlen([]);\n").unwrap();
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution.stdout.contains(
            "TypeError: mb_strlen(): Argument #1 ($string) must be of type string, array given"
        ),
        "{}",
        execution.stdout
    );
}
