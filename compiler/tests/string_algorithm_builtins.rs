use php_compiler::{emit_ir_source, run_source};

#[test]
fn string_algorithm_builtins_match_core_examples() {
    let execution = run_source(
        r#"<?php
echo crc32("foo") . "|";
echo crc32("") . "|";
echo levenshtein("111", "121", 2, 3, 2) . ":";
echo levenshtein("111", "121", 2, 9, 2) . "|";
echo soundex("Euler") . ":" . soundex("Lukasiewicz") . ":";
var_dump(soundex("Gauss") == soundex("Ghosh"));
$counts = count_chars("abca", 1);
echo $counts[97] . "," . $counts[98] . "," . $counts[99] . "|";
echo count_chars("abca", 3) . "|";
echo (int) (strlen(count_chars("abca", 4)) == 253);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2356372769|0|3:4|E460:L222:bool(true)\n2,1,1|abc|1"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_algorithm_helpers_use_php_string_argument_boundary() {
    let execution = run_source(
        r#"<?php
class TextValue {
    public function __toString() {
        return "abc";
    }
}

set_error_handler(function($_, $message) {
    echo "deprecated:", $message, "\n";
    return true;
});

echo crc32(null), "\n";
echo crc32(new TextValue) === crc32("abc") ? "crc-object\n" : "bad\n";
echo soundex(null), "\n";
echo soundex(new TextValue), "\n";
echo metaphone(null), "\n";
echo metaphone(new TextValue), "\n";
echo count_chars(null, 3), "\n";
echo count_chars(new TextValue, 3), "\n";
echo levenshtein(null, new TextValue), "\n";
$percent = -1;
echo similar_text(new TextValue, null, $percent), ":", $percent, "\n";
$call = "similar_text";
echo $call(new TextValue, "xbc"), "\n";

foreach ([
    fn() => crc32([]),
    fn() => soundex(new stdClass),
    fn() => metaphone([]),
    fn() => count_chars([]),
    fn() => levenshtein([], "a"),
    fn() => levenshtein("a", []),
    fn() => similar_text([], "a"),
    fn() => similar_text("a", []),
] as $case) {
    try {
        $case();
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "deprecated:crc32(): Passing null to parameter #1 ($string) of type string is deprecated\n",
            "0\n",
            "crc-object\n",
            "deprecated:soundex(): Passing null to parameter #1 ($string) of type string is deprecated\n",
            "0000\n",
            "A120\n",
            "deprecated:metaphone(): Passing null to parameter #1 ($string) of type string is deprecated\n",
            "\n",
            "ABK\n",
            "deprecated:count_chars(): Passing null to parameter #1 ($string) of type string is deprecated\n",
            "\n",
            "abc\n",
            "deprecated:levenshtein(): Passing null to parameter #1 ($string1) of type string is deprecated\n",
            "3\n",
            "deprecated:similar_text(): Passing null to parameter #2 ($string2) of type string is deprecated\n",
            "0:0\n",
            "2\n",
            "crc32(): Argument #1 ($string) must be of type string, array given\n",
            "soundex(): Argument #1 ($string) must be of type string, stdClass given\n",
            "metaphone(): Argument #1 ($string) must be of type string, array given\n",
            "count_chars(): Argument #1 ($string) must be of type string, array given\n",
            "levenshtein(): Argument #1 ($string1) must be of type string, array given\n",
            "levenshtein(): Argument #2 ($string2) must be of type string, array given\n",
            "similar_text(): Argument #1 ($string1) must be of type string, array given\n",
            "similar_text(): Argument #2 ($string2) must be of type string, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn metaphone_matches_public_standard_string_rows() {
    let execution = run_source(
        r#"<?php
var_dump(metaphone(""));
var_dump(metaphone(-1));
try {
    var_dump(metaphone("valid phrase", -1));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(metaphone("valid phrase", 0));
var_dump(metaphone("valid phrase", 10000));
echo metaphone('CMXFXZ'), "\n";
echo metaphone('CMXFXV'), "\n";
echo metaphone('CMXFXZXZ'), "\n";
var_dump(metaphone("scratch"));
var_dump(metaphone("scrath"));
var_dump(metaphone("scratc"));
var_dump(metaphone("The naked waste, as far as the eye could pierce, even to the distant menace of the mountains, was dappled with the fitful moonlight."));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(0) \"\"\n",
            "string(0) \"\"\n",
            "metaphone(): Argument #2 ($max_phonemes) must be greater than or equal to 0\n",
            "string(6) \"FLTFRS\"\n",
            "string(6) \"FLTFRS\"\n",
            "KMKSFKSS\n",
            "KMKSFKSF\n",
            "KMKSFKSSKSS\n",
            "string(4) \"SKRX\"\n",
            "string(4) \"SKR0\"\n",
            "string(5) \"SKRTK\"\n",
            "string(56) \"0NKTWSTSFRS0YKLTPRSFNT0TSTNTMNSF0MNTNSWSTPLTW00FTFLMNLFT\"\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn count_chars_invalid_mode_is_catchable_value_error() {
    let execution = run_source(
        r#"<?php
try {
    count_chars("abc", 5);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "count_chars(): Argument #2 ($mode) must be between 0 and 4 (inclusive)"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_algorithm_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["crc32", "levenshtein", "soundex", "metaphone", "count_chars"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1;11:2/5;11:1/1;11:1/2;11:1/2;");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_string_algorithm_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("crc32") ? "1" : "0";
echo is_callable("soundex") ? "1" : "0";
echo function_exists("metaphone") ? "1" : "0";
echo function_exists("count_chars") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("count_chars"), "{ir}");
}
