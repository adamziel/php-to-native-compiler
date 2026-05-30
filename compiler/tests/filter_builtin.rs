use php_compiler::run_source;

#[test]
fn filter_metadata_lists_ids_and_absent_inputs() {
    let execution = run_source(
        r#"<?php
$filters = filter_list();
echo count($filters), "|", $filters[0], "|", $filters[20], "\n";
var_dump(filter_id("stripped"));
var_dump(filter_id("string"));
var_dump(filter_id("url"));
var_dump(filter_id("int"));
var_dump(filter_id("none"));
var_dump(filter_id(-1));
var_dump(filter_input(INPUT_GET, "missing"));
var_dump(filter_input(INPUT_GET, "missing", FILTER_DEFAULT, FILTER_NULL_ON_FAILURE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "21|int|callback\nint(513)\nint(513)\nint(518)\nint(257)\nbool(false)\nbool(false)\nNULL\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_validates_scalar_arrays_and_null_on_failure() {
    let execution = run_source(
        r#"<?php
$ints = filter_var(array(1, "1", "", "-23234", "text", "asdf234asdfgs", array()), FILTER_VALIDATE_INT, FILTER_REQUIRE_ARRAY);
echo $ints[0], "|", $ints[1], "|", $ints[3], "\n";
var_dump($ints[2], $ints[4], $ints[5], $ints[6]);
$floats = filter_var(array(1.2, "1.7", "", "-23234.123", "text", "asdf234.2asdfgs", array()), FILTER_VALIDATE_FLOAT, FILTER_REQUIRE_ARRAY);
echo $floats[0], "|", $floats[1], "|", $floats[3], "\n";
var_dump($floats[2], $floats[4], $floats[5], $floats[6]);
var_dump(filter_var("invalid", FILTER_VALIDATE_BOOL, FILTER_NULL_ON_FAILURE));
var_dump(filter_var("invalid", FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1|1|-23234\nbool(false)\nbool(false)\nbool(false)\narray(0) {\n}\n1.2|1.7|-23234.123\nbool(false)\nbool(false)\nbool(false)\narray(0) {\n}\nNULL\nNULL\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filter_var_sanitizes_scalars_and_warns_for_unknown_filters() {
    let execution = run_source(
        r#"<?php
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, 1));
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, 0));
var_dump(filter_var(1, FILTER_SANITIZE_SPECIAL_CHARS, array()));
var_dump(filter_var("<>&\"'plain", FILTER_SANITIZE_SPECIAL_CHARS));
var_dump(filter_var(array("<tag>", "safe"), FILTER_SANITIZE_SPECIAL_CHARS, FILTER_REQUIRE_ARRAY));
var_dump(filter_var(1, -1, array(123)));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("string(1) \"1\"\nstring(1) \"1\"\nstring(1) \"1\""));
    assert!(
        execution
            .stdout
            .contains("string(30) \"&#60;&#62;&#38;&#34;&#39;plain\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(13) \"&#60;tag&#62;\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: filter_var(): Unknown filter with ID -1"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}
