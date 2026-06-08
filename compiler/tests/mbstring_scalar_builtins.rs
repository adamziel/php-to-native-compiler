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
fn mb_strcut_uses_byte_windows_without_splitting_utf8_characters() {
    let execution = run_source(
        r#"<?php
$sample = "A日本B";
foreach ([[0, 1], [1, 2], [1, 3], [2, 2], [2, 3], [4, 2], [4, 3], [-4, 3], [0, null], [99, 1], [-99, 1]] as $case) {
    $cut = mb_strcut($sample, $case[0], $case[1], "UTF-8");
    echo base64_encode($cut), ":", strlen($cut), "\n";
}
foreach ([-3, -4, -5, -6, -999] as $length) {
    $cut = mb_strcut("Déjà vu", 1, $length, "UTF-8");
    echo base64_encode($cut), ":", strlen($cut), "\n";
}
echo bin2hex(mb_strcut("ABC", 1, 1, "8bit")), "\n";
try {
    mb_strcut("abc", 0, 1, "unknown-encoding");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
$call = "mb_strcut";
echo function_exists($call) ? "fn" : "missing";
echo is_callable($call) ? ":callable:" : ":missing:";
echo $call("foobarbaz", 6, null, "UTF-8"), "\n";
$reflection = new ReflectionFunction("mb_strcut");
echo $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "QQ==:1\n",
            ":0\n",
            "5pel:3\n",
            ":0\n",
            "5pel:3\n",
            ":0\n",
            "5pys:3\n",
            "5pys:3\n",
            "QeaXpeacrEI=:8\n",
            ":0\n",
            "QQ==:1\n",
            "w6lqw6A=:5\n",
            "w6lq:3\n",
            "w6lq:3\n",
            "w6k=:2\n",
            ":0\n",
            "42\n",
            "mb_strcut(): Argument #4 ($encoding) must be a valid encoding, \"unknown-encoding\" given\n",
            "fn:callable:baz\n",
            "mb_strcut:2/4\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_substr_count_counts_non_overlapping_ascii_and_utf8_matches() {
    let execution = run_source(
        r#"<?php
$ascii = "This is an English string. 0123456789.";
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=");
$period = base64_decode("44CC");
$phrase = base64_decode("44GT44KT44Gr44Gh44Gv44CB5LiW55WM");
var_dump(mb_substr_count($ascii, "is"));
var_dump(mb_substr_count($ascii, "hello, world"));
var_dump(mb_substr_count($japanese, $period));
var_dump(mb_substr_count($japanese, $phrase));
var_dump(mb_substr_count("abcabcabc", "abcabc"));
var_dump(mb_substr_count($japanese . $japanese, $japanese, "utf-8"));
var_dump(mb_substr_count("A" . chr(0) . "B" . chr(0), chr(0), "8bit"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(3)\nint(0)\nint(2)\nint(0)\nint(1)\nint(2)\nint(2)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn mb_substr_count_reports_empty_needle_and_encoding_errors() {
    let execution = run_source(
        r#"<?php
try {
    mb_substr_count("abc", "");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    mb_substr_count("Hello, World!", "Hello", "unknown-encoding");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
$call = "mb_substr_count";
echo function_exists($call) ? "fn" : "missing";
echo is_callable($call) ? ":callable:" : ":missing:";
echo $call("abcabcabc", "abcabc");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "mb_substr_count(): Argument #2 ($needle) must not be empty\nmb_substr_count(): Argument #3 ($encoding) must be a valid encoding, \"unknown-encoding\" given\nfn:callable:1"
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
fn mb_strstr_family_returns_before_or_after_substrings() {
    let execution = run_source(
        r#"<?php
$ascii = "abcdef zbcdyx";
$japanese = base64_decode("5pel5pys6Kqe44OG44Kt44K544OIMzTvvJXvvJbml6XmnKzoqp7jg4bjgq3jgrnjg4g=");
$japanese_needle = base64_decode("6Kqe44OG44Kt");
$greek = base64_decode("zrrOu868zr3Ovs6/z4DPgSDOus67zrzOvc6+zr/PgA==");
$greek_needle = base64_decode("zpzOnc6ezp8=");
var_dump(bin2hex(mb_strstr($ascii, "bcd", false, "ISO-8859-1")));
var_dump(bin2hex(mb_strstr($ascii, "bcd", true)));
var_dump(bin2hex(mb_strrchr($ascii, "bcd", false)));
var_dump(bin2hex(mb_strrchr($ascii, "bcd", true)));
var_dump(bin2hex(mb_strstr($japanese, $japanese_needle)));
var_dump(bin2hex(mb_strrchr($japanese, $japanese_needle, true)));
var_dump(bin2hex(mb_stristr($greek, $greek_needle)));
var_dump(bin2hex(mb_strrichr($greek, $greek_needle, true)));
var_dump(mb_strstr("abc", ""));
var_dump(mb_strrchr("abc", ""));
try {
    mb_strrichr("x", "x", false, "unknown-encoding");
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
$call = "mb_stristr";
echo function_exists($call) ? "fn" : "missing";
echo is_callable($call) ? ":callable:" : ":missing:";
echo bin2hex($call("abcDef", "BCD", true, "8bit")), "\n";
$reflection = new ReflectionFunction("mb_strrichr");
echo $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(24) \"6263646566207a6263647978\"\n",
            "string(2) \"61\"\n",
            "string(10) \"6263647978\"\n",
            "string(16) \"616263646566207a\"\n",
            "string(88) \"e8aa9ee38386e382ade382b9e383883334efbc95efbc96e697a5e69cace8aa9ee38386e382ade382b9e38388\"\n",
            "string(70) \"e697a5e69cace8aa9ee38386e382ade382b9e383883334efbc95efbc96e697a5e69cac\"\n",
            "string(54) \"cebccebdcebecebfcf80cf8120cebacebbcebccebdcebecebfcf80\"\n",
            "string(42) \"cebacebbcebccebdcebecebfcf80cf8120cebacebb\"\n",
            "string(3) \"abc\"\n",
            "string(0) \"\"\n",
            "mb_strrichr(): Argument #4 ($encoding) must be a valid encoding, \"unknown-encoding\" given\n",
            "fn:callable:61\n",
            "mb_strrichr:2/4\n",
        )
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
fn mb_scalar_helpers_accept_stringable_encoding_arguments_and_report_type_errors() {
    let execution = run_source(
        r#"<?php
class Utf8Encoding {
    public function __toString(): string {
        return "UTF-8";
    }
}
class Latin1Encoding {
    public function __toString(): string {
        return "latin1";
    }
}
class MissingEncoding {}

$utf8 = new Utf8Encoding();
$latin1 = new Latin1Encoding();
var_dump(mb_strlen("é", $utf8));
var_dump(mb_strlen("é", $latin1));
var_dump(strlen(mb_substr("éx", 0, 1, $utf8)));
var_dump(mb_strpos("éx", "x", 0, $utf8));
var_dump(mb_strpos("éx", "x", 0, $latin1));
var_dump(mb_substr_count("éé", "é", $utf8));
try {
    mb_strtoupper("x", new MissingEncoding());
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    mb_strtolower("x", []);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
ini_set("internal_encoding", "ISO-8859-1");
var_dump(mb_strlen("é", null));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(1)\n",
            "int(2)\n",
            "int(2)\n",
            "int(1)\n",
            "int(2)\n",
            "int(2)\n",
            "mb_strtoupper(): Argument #2 ($encoding) must be of type ?string, MissingEncoding given\n",
            "mb_strtolower(): Argument #2 ($encoding) must be of type ?string, array given\n",
            "int(2)\n",
        )
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
echo function_exists("mb_strcut") ? "1" : "0";
echo function_exists("mb_substr_count") ? "1" : "0";
echo is_callable("mb_stripos") ? "1" : "0";
echo is_callable("mb_strrichr") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
