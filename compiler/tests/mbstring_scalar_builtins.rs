use php_compiler::{emit_ir_source, run_source};

#[test]
fn mb_strlen_counts_utf8_codepoints_and_single_byte_strings() {
    let execution = run_source(
        r#"<?php
$ascii = "abc def";
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=");
var_dump(mb_strlen($ascii));
var_dump(mb_strlen($japanese, "UTF-8"));
var_dump(mb_strlen($ascii, "ISO-8859-1"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(7)\nint(21)\nint(7)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strpos_and_mbirpos_use_character_offsets() {
    let execution = run_source(
        r#"<?php
$ascii = "abc def";
$japanese = "日本語テキストです。01234５６７８９。";
var_dump(mb_strpos($ascii, "d", 2, "ISO-8859-1"));
var_dump(mb_strpos($japanese, "テキスト"));
var_dump(mb_strpos($japanese, "", -2));
try {
    mb_strpos($japanese, "x", -150);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(4)\nint(3)\nint(19)\nmb_strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_substr_uses_character_windows_and_internal_encoding_default() {
    let execution = run_source(
        r#"<?php
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=");
var_dump(base64_encode(mb_substr($japanese, 2, 7, "UTF-8")));
var_dump(base64_encode(mb_substr($japanese, -10, 4, "UTF-8")));
var_dump(base64_encode(mb_substr($japanese, 1, -10, "UTF-8")));
ini_set("internal_encoding", "ISO-8859-1");
var_dump(base64_encode(mb_substr($japanese, 2, 7)));
try {
    mb_substr("abc", 0, 1, "unknown-encoding");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(28) \"6Kqe44OG44Kt44K544OI44Gn44GZ\"\nstring(8) \"MTIzNA==\"\nstring(40) \"5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMA==\"\nstring(12) \"peacrOiqng==\"\nmb_substr(): Argument #4 ($encoding) must be a valid encoding, \"unknown-encoding\" given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_stripos_casefolds_utf8_and_reports_offset_errors_for_reverse_variants() {
    let execution = run_source(
        r#"<?php
$greek = base64_decode("zrrOu868zr3Ovs6/z4DPgSDOus67zrzOvc6+zr/PgA==");
$needle = base64_decode("zpzOnc6ezp8=");
var_dump(mb_stripos($greek, $needle));
var_dump(mb_stripos($greek, $needle, 4));
foreach (["mb_stripos", "mb_strrpos", "mb_strripos"] as $fn) {
    try {
        $fn("f", "bar", 3);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(2)\nint(11)\nmb_stripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\nmb_strrpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\nmb_strripos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_strcase_supports_unicode_case_mapping_and_final_sigma_boundary() {
    let execution = run_source(
        r#"<?php
$greek_upper = base64_decode("zpHOks6TzpTOlc6WzpfOmM6ZzprOm86czp3Ons6fzqDOoc6jzqTOpc6mzqfOqM6p");
$greek_lower = base64_decode("zrHOss6zzrTOtc62zrfOuM65zrrOu868zr3Ovs6/z4DPgc+Dz4TPhc+Gz4fPiM+J");
var_dump(base64_encode(mb_strtolower($greek_upper, "UTF-8")));
var_dump(base64_encode(mb_strtoupper($greek_lower, "UTF-8")));
var_dump(mb_strtolower("aΣ", "UTF-8"));
var_dump(mb_strtolower("aΣb", "UTF-8"));
var_dump(mb_strtolower("a" . str_repeat(".", 63) . "Σ", "UTF-8"));
var_dump(mb_strtolower("a" . str_repeat(".", 64) . "Σ", "UTF-8"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(64) \"zrHOss6zzrTOtc62zrfOuM65zrrOu868zr3Ovs6/z4DPgc+Dz4TPhc+Gz4fPiM+J\"\nstring(64) \"zpHOks6TzpTOlc6WzpfOmM6ZzprOm86czp3Ons6fzqDOoc6jzqTOpc6mzqfOqM6p\"\nstring(3) \"aς\"\nstring(4) \"aσb\"\nstring(66) \"a...............................................................ς\"\nstring(67) \"a................................................................σ\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mbstring_metadata_and_encoding_value_errors_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["mb_strlen", "mb_substr", "mb_strpos", "mb_strtolower"] as $fn) {
    echo function_exists($fn) ? "1" : "0";
    echo is_callable($fn) ? "1" : "0";
}
$reflection = new ReflectionFunction("mb_stripos");
echo "|", $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "\n";
foreach ([
    fn() => mb_strlen("x", "unknown-encoding"),
    fn() => mb_strpos("x", "x", 0, "unknown-encoding"),
    fn() => mb_strtoupper("x", "unknown-encoding"),
] as $call) {
    try {
        $call();
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11111111|mb_stripos:2/4\nmb_strlen(): Argument #2 ($encoding) must be a valid encoding, \"unknown-encoding\" given\nmb_strpos(): Argument #4 ($encoding) must be a valid encoding, \"unknown-encoding\" given\nmb_strtoupper(): Argument #2 ($encoding) must be a valid encoding, \"unknown-encoding\" given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_mbstring_function_membership() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("mb_strlen") ? "1" : "0";
echo function_exists("mb_substr") ? "1" : "0";
echo is_callable("mb_stripos") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
