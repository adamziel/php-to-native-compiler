use php_compiler::run_source;

#[test]
fn json_decode_scalars_arrays_and_objects() {
    let execution = run_source(
        r#"<?php
var_dump(json_decode('0'));
var_dump(json_decode('true'));
var_dump(json_decode('"abc"'));
var_dump(json_decode('[1,2,3]'));
var_dump(json_decode('{"name":"Ada","count":2}'));
var_dump(json_decode('{"name":"Ada","count":2}', true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(0)\n",
            "bool(true)\n",
            "string(3) \"abc\"\n",
            "array(3) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  int(2)\n",
            "  [2]=>\n",
            "  int(3)\n",
            "}\n",
            "object(stdClass)#1 (2) {\n",
            "  [\"name\"]=>\n",
            "  string(3) \"Ada\"\n",
            "  [\"count\"]=>\n",
            "  int(2)\n",
            "}\n",
            "array(2) {\n",
            "  [\"name\"]=>\n",
            "  string(3) \"Ada\"\n",
            "  [\"count\"]=>\n",
            "  int(2)\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_error_state_tracks_decode_and_encode_failures() {
    let execution = run_source(
        r#"<?php
var_dump(json_decode('[1}'));
var_dump(json_last_error());
var_dump(json_last_error_msg());
json_decode('[1]');
var_dump(json_last_error(), json_last_error_msg());
$fp = fopen(__FILE__, "r");
var_dump(json_encode($fp));
var_dump(json_last_error(), json_last_error_msg());
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("NULL\nint(2)\n"));
    assert!(execution.stdout.contains("State mismatch"));
    assert!(execution.stdout.contains("int(0)\nstring(8) \"No error\""));
    assert!(execution.stdout.contains("bool(false)\nint(8)\n"));
    assert!(execution.stdout.contains("Type is not supported"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_flags_cover_core_string_and_shape_options() {
    let execution = run_source(
        r#"<?php
echo json_encode(["<foo>", "'bar'", '"baz"', "&blong&"], JSON_HEX_TAG|JSON_HEX_APOS|JSON_HEX_QUOT|JSON_HEX_AMP), "\n";
echo json_encode(["руссиш", "1.25", "abc"], JSON_NUMERIC_CHECK), "\n";
echo json_encode([["x"]], JSON_FORCE_OBJECT), "\n";
echo json_encode([12.0, 0.0], JSON_PRESERVE_ZERO_FRACTION), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "[\"\\u003Cfoo\\u003E\",\"\\u0027bar\\u0027\",\"\\u0022baz\\u0022\",\"\\u0026blong\\u0026\"]\n[\"\\u0440\\u0443\\u0441\\u0441\\u0438\\u0448\",1.25,\"abc\"]\n{\"0\":{\"0\":\"x\"}}\n[12.0,0.0]\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_decode_bigint_and_object_as_array_flags() {
    let execution = run_source(
        r#"<?php
$json = '{"largenum":123456789012345678901234567890}';
var_dump(json_decode($json, true, 512, JSON_BIGINT_AS_STRING));
var_dump(json_decode('{"foo":"bar"}', null, 512, JSON_OBJECT_AS_ARRAY));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(1) {\n",
            "  [\"largenum\"]=>\n",
            "  string(30) \"123456789012345678901234567890\"\n",
            "}\n",
            "array(1) {\n",
            "  [\"foo\"]=>\n",
            "  string(3) \"bar\"\n",
            "}\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_decode_depth_and_error_locations_match_core_rows() {
    let execution = run_source(
        r#"<?php
var_dump(json_decode("[[1]]", false, 2));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_decode("[1}"));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_decode('["' . chr(0) . 'abcd"]'));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_decode("[1"));
var_dump(json_last_error(), json_last_error_msg());
try {
    json_decode('"abc"', true, -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "NULL\n",
            "int(1)\n",
            "string(46) \"Maximum stack depth exceeded near location 1:2\"\n",
            "NULL\n",
            "int(2)\n",
            "string(60) \"State mismatch (invalid or malformed JSON) near location 1:3\"\n",
            "NULL\n",
            "int(3)\n",
            "string(71) \"Control character error, possibly incorrectly encoded near location 1:2\"\n",
            "NULL\n",
            "int(4)\n",
            "string(30) \"Syntax error near location 1:3\"\n",
            "json_decode(): Argument #3 ($depth) must be greater than 0\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_decode_invalid_utf8_flags_repair_binary_string_tokens() {
    let execution = run_source(
        r#"<?php
$one = "\"a\xb0b\"";
var_dump(json_decode($one));
var_dump(json_last_error(), json_last_error_msg());
var_dump(json_decode($one, true, 512, JSON_INVALID_UTF8_IGNORE));
var_dump(json_last_error(), json_last_error_msg());
echo bin2hex(json_decode($one, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)), "\n";
var_dump(json_last_error(), json_last_error_msg());

$overlong = "\"\x61\xf0\x80\x80\x41\"";
echo bin2hex(json_decode($overlong, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$array = json_decode("[\"\xc1\xc1\",\"a\"]", true, 512, JSON_INVALID_UTF8_IGNORE);
var_dump($array);
$substituted = json_decode("[\"\xc1\xc1\",\"a\"]", true, 512, JSON_INVALID_UTF8_SUBSTITUTE);
echo bin2hex($substituted[0]), "|", bin2hex($substituted[1]), "\n";
echo bin2hex(json_decode($one, true, 512, JSON_INVALID_UTF8_IGNORE | JSON_INVALID_UTF8_SUBSTITUTE)), "\n";

$outside = "[" . "\xb0" . "]";
var_dump(json_decode($outside, true, 512, JSON_INVALID_UTF8_IGNORE));
echo json_last_error(), "|", json_last_error_msg();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "NULL\n",
            "int(5)\n",
            "string(56) \"Malformed UTF-8 characters, possibly incorrectly encoded\"\n",
            "string(2) \"ab\"\n",
            "int(0)\n",
            "string(8) \"No error\"\n",
            "61efbfbd62\n",
            "int(0)\n",
            "string(8) \"No error\"\n",
            "61efbfbdefbfbdefbfbd41\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  string(0) \"\"\n",
            "  [1]=>\n",
            "  string(1) \"a\"\n",
            "}\n",
            "efbfbdefbfbd|61\n",
            "61efbfbd62\n",
            "NULL\n",
            "5|Malformed UTF-8 characters, possibly incorrectly encoded",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_validate_tracks_state_depth_flags_and_utf8() {
    let execution = run_source(
        "<?php\n\
var_dump(json_validate('{\"ok\":true}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate('-'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate('', -1));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
try { json_validate('-', 0); } catch (Error $e) { echo $e->getCode(), '|', $e->getMessage(), \"\\n\"; }\n\
var_dump(json_last_error(), json_last_error_msg());\n\
try { json_validate('-', 512, JSON_BIGINT_AS_STRING); } catch (Error $e) { echo $e->getCode(), '|', $e->getMessage(), \"\\n\"; }\n\
$bad = \"\\\"a\\xb0b\\\"\";\n\
var_dump(json_validate($bad));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate($bad, 512, JSON_INVALID_UTF8_IGNORE));\n\
var_dump(json_last_error(), json_last_error_msg());\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "int(0)\n",
            "string(8) \"No error\"\n",
            "bool(false)\n",
            "int(4)\n",
            "string(30) \"Syntax error near location 1:1\"\n",
            "bool(false)\n",
            "int(4)\n",
            "string(12) \"Syntax error\"\n",
            "0|json_validate(): Argument #2 ($depth) must be greater than 0\n",
            "int(4)\n",
            "string(12) \"Syntax error\"\n",
            "0|json_validate(): Argument #3 ($flags) must be a valid flag (allowed flags: JSON_INVALID_UTF8_IGNORE)\n",
            "bool(false)\n",
            "int(5)\n",
            "string(74) \"Malformed UTF-8 characters, possibly incorrectly encoded near location 1:1\"\n",
            "bool(true)\n",
            "int(0)\n",
            "string(8) \"No error\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_error_diagnostics_are_catchable_for_phpt_rows() {
    let execution = run_source(
        r#"<?php
try {
    json_decode('"abc"', true, -1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(json_last_error());
try {
    json_last_error(true);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "json_decode(): Argument #3 ($depth) must be greater than 0\n\
int(0)\n\
json_last_error() expects exactly 0 arguments, 1 given\n"
    );
    assert_eq!(execution.exit_code, 0);
}
