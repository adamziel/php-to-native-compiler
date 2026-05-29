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
foreach (["crc32", "levenshtein", "soundex", "count_chars"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1;11:2/5;11:1/1;11:1/2;");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_string_algorithm_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("crc32") ? "1" : "0";
echo is_callable("soundex") ? "1" : "0";
echo function_exists("count_chars") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("count_chars"), "{ir}");
}
