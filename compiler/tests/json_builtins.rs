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
fn json_decode_error_locations_and_invalid_utf8_options() {
    let execution = run_source(
        "<?php\n\
var_dump(json_decode('[[1]]', false, 2));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_decode('[1}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_decode('[\"' . chr(0) . 'abcd\"]'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_decode('[1'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_decode(\"\\\"a\\xb0b\\\"\"));\n\
var_dump(json_decode(\"\\\"a\\xb0b\\\"\", true, 512, JSON_INVALID_UTF8_IGNORE));\n\
var_dump(bin2hex(json_decode(\"\\\"a\\xb0b\\\"\", true, 512, JSON_INVALID_UTF8_SUBSTITUTE)));\n\
var_dump(bin2hex(json_decode(\"\\\"\\x61\\xf0\\x80\\x80\\x41\\\"\", true, 512, JSON_INVALID_UTF8_SUBSTITUTE)));\n",
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
            "NULL\n",
            "string(2) \"ab\"\n",
            "string(10) \"61efbfbd62\"\n",
            "string(22) \"61efbfbdefbfbdefbfbd41\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_invalid_utf8_line_terminator_and_partial_numbers() {
    let execution = run_source(
        "<?php\n\
var_dump(json_encode(\"a\\xb0b\"));\n\
var_dump(json_encode(\"a\\xb0b\", JSON_INVALID_UTF8_IGNORE));\n\
var_dump(json_encode(\"a\\xb0b\", JSON_INVALID_UTF8_SUBSTITUTE));\n\
var_dump(bin2hex(json_encode(\"a\\xb0b\", JSON_UNESCAPED_UNICODE | JSON_INVALID_UTF8_SUBSTITUTE)));\n\
var_dump(json_encode(\"\\x61\\xf0\\x80\\x80\\x41\", JSON_INVALID_UTF8_IGNORE));\n\
var_dump(json_encode(\"\\x61\\xf0\\x80\\x80\\x41\", JSON_INVALID_UTF8_SUBSTITUTE));\n\
var_dump(json_encode(\"a\\xE2\\x80\\xA8b\", JSON_UNESCAPED_UNICODE));\n\
var_dump(json_encode(\"a\\xE2\\x80\\xA8b\", JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS));\n\
var_dump(json_encode(INF, JSON_PARTIAL_OUTPUT_ON_ERROR));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_encode(NAN, JSON_PARTIAL_OUTPUT_ON_ERROR));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_encode(array(\"\\x80\" => 1), JSON_PARTIAL_OUTPUT_ON_ERROR));\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "string(4) \"\"ab\"\"\n",
            "string(10) \"\"a\\ufffdb\"\"\n",
            "string(14) \"2261efbfbd6222\"\n",
            "string(4) \"\"aA\"\"\n",
            "string(10) \"\"a\\ufffdA\"\"\n",
            "string(10) \"\"a\\u2028b\"\"\n",
            "string(7) \"\"a\u{2028}b\"\"\n",
            "string(1) \"0\"\n",
            "int(7)\n",
            "string(34) \"Inf and NaN cannot be JSON encoded\"\n",
            "string(1) \"0\"\n",
            "int(7)\n",
            "string(34) \"Inf and NaN cannot be JSON encoded\"\n",
            "string(6) \"{\"\":1}\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_decode_invalid_property_names_propagate_to_root_with_location() {
    let execution = run_source(
        r#"<?php
var_dump(json_decode('{"key": {"\u0000": "aa"}}'));
var_dump(json_last_error() === JSON_ERROR_INVALID_PROPERTY_NAME);
var_dump(json_decode('[{"key1": 0, "\u1234": 1, "\u0000": 1}]'));
var_dump(json_last_error() === JSON_ERROR_INVALID_PROPERTY_NAME);
var_dump(json_last_error_msg());
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "NULL\n",
            "bool(true)\n",
            "NULL\n",
            "bool(true)\n",
            "string(55) \"The decoded property name is invalid near location 1:27\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_validate_ascii_location_edges() {
    let execution = run_source(
        "<?php\n\
var_dump(json_validate('{\"name\": \"value}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate('{\"name\" \"value\"}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate('{\"test\": \"\\\\x\"}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n\
var_dump(json_validate('{\"test\": \"' . \"\\n\" . '\"}'));\n\
var_dump(json_last_error(), json_last_error_msg());\n",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "int(3)\n",
            "string(72) \"Control character error, possibly incorrectly encoded near location 1:10\"\n",
            "bool(false)\n",
            "int(4)\n",
            "string(30) \"Syntax error near location 1:9\"\n",
            "bool(false)\n",
            "int(4)\n",
            "string(31) \"Syntax error near location 1:10\"\n",
            "bool(false)\n",
            "int(3)\n",
            "string(72) \"Control character error, possibly incorrectly encoded near location 1:10\"\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}
