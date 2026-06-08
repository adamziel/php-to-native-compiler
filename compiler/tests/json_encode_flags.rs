use php_compiler::run_source;

#[test]
fn json_encode_supports_reached_flags_and_unicode_escaping() {
    let execution = run_source(
        r#"<?php
echo json_encode("a/b"), "\n";
echo json_encode("a/b", JSON_UNESCAPED_SLASHES), "\n";
echo json_encode(array('<foo>', "'bar'", '"baz"', '&blong&')), "\n";
echo json_encode(array('<foo>', "'bar'", '"baz"', '&blong&'), JSON_HEX_TAG | JSON_HEX_APOS | JSON_HEX_QUOT | JSON_HEX_AMP), "\n";
echo json_encode(array(array(1)), JSON_FORCE_OBJECT), "\n";
echo json_encode(array(), JSON_FORCE_OBJECT), "\n";
echo json_encode("\xD1\x80\xD1\x83\xD1\x81\xD1\x81\xD0\xB8\xD1\x88"), "\n";
echo json_encode(base64_decode("5pel5pys6Kqe44OG44Kt44K544OI44Gn44GZ44CCMDEyMzTvvJXvvJbvvJfvvJjvvJnjgII=")), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "\"a\\/b\"\n\
\"a/b\"\n\
[\"<foo>\",\"'bar'\",\"\\\"baz\\\"\",\"&blong&\"]\n\
[\"\\u003Cfoo\\u003E\",\"\\u0027bar\\u0027\",\"\\u0022baz\\u0022\",\"\\u0026blong\\u0026\"]\n\
{\"0\":{\"0\":1}}\n\
{}\n\
\"\\u0440\\u0443\\u0441\\u0441\\u0438\\u0448\"\n\
\"\\u65e5\\u672c\\u8a9e\\u30c6\\u30ad\\u30b9\\u30c8\\u3067\\u3059\\u300201234\\uff15\\uff16\\uff17\\uff18\\uff19\\u3002\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_flag_constants_and_metadata_are_available() {
    let execution = run_source(
        r#"<?php
echo JSON_HEX_TAG, ":", JSON_HEX_AMP, ":", JSON_HEX_APOS, ":", JSON_HEX_QUOT, ":", JSON_FORCE_OBJECT, ":", JSON_NUMERIC_CHECK, ":", JSON_UNESCAPED_SLASHES;
echo "|", function_exists("json_encode") ? "1" : "0";
echo is_callable("json_encode") ? "1" : "0";
$fn = new ReflectionFunction("json_encode");
echo ":", $fn->getExtensionName(), ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1:2:4:8:16:32:64|11:json:1/2");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_numeric_check_converts_well_formed_numeric_strings() {
    let execution = run_source(
        r#"<?php
var_dump(
    json_encode("1", JSON_NUMERIC_CHECK),
    json_encode("9.4324", JSON_NUMERIC_CHECK),
    json_encode(array("122321", "3232595.33423"), JSON_NUMERIC_CHECK),
    json_encode("1"),
    json_encode("9.4324"),
    json_encode(array("122321", "3232595.33423"))
);
$object = new stdClass;
$object->{"1"} = "5";
var_dump(json_encode($object, JSON_NUMERIC_CHECK));
var_dump(json_encode(array("test" => "123343e871700"), JSON_NUMERIC_CHECK));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        r#"string(1) "1"
string(6) "9.4324"
string(22) "[122321,3232595.33423]"
string(3) ""1""
string(8) ""9.4324""
string(26) "["122321","3232595.33423"]"
string(7) "{"1":5}"
string(24) "{"test":"123343e871700"}"
"#
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
