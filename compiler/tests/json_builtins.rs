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
