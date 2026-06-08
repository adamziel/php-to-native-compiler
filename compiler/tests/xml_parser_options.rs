use php_compiler::{emit_ir_source, run_source};

#[test]
fn xml_parser_option_state_matches_basic_get_set_slice() {
    let execution = run_source(
        r#"<?php
$parser = xml_parser_create_ns();

var_dump(xml_parser_get_option($parser, XML_OPTION_CASE_FOLDING));
var_dump(xml_parser_get_option($parser, XML_OPTION_TARGET_ENCODING));

var_dump(xml_parser_set_option($parser, XML_OPTION_CASE_FOLDING, 1));
var_dump(xml_parser_set_option($parser, XML_OPTION_TARGET_ENCODING, "ISO-8859-1"));

var_dump(xml_parser_get_option($parser, XML_OPTION_CASE_FOLDING));
var_dump(xml_parser_get_option($parser, XML_OPTION_TARGET_ENCODING));

var_dump(xml_parser_set_option($parser, XML_OPTION_CASE_FOLDING, 0));
var_dump(xml_parser_set_option($parser, XML_OPTION_TARGET_ENCODING, "utf-8"));

var_dump(xml_parser_get_option($parser, XML_OPTION_CASE_FOLDING));
var_dump(xml_parser_get_option($parser, XML_OPTION_TARGET_ENCODING));

var_dump(xml_parser_set_option($parser, XML_OPTION_TARGET_ENCODING, "us-ascii"));
var_dump(xml_parser_get_option($parser, XML_OPTION_TARGET_ENCODING));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nstring(5) \"UTF-8\"\nbool(true)\nbool(true)\nbool(true)\nstring(10) \"ISO-8859-1\"\nbool(true)\nbool(true)\nbool(false)\nstring(5) \"UTF-8\"\nbool(true)\nstring(8) \"US-ASCII\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xml_parser_invalid_option_is_catchable_value_error() {
    let execution = run_source(
        r#"<?php
$parser = xml_parser_create();

try {
    xml_parser_get_option($parser, 42);
} catch (ValueError $exception) {
    echo $exception->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "xml_parser_get_option(): Argument #2 ($option) must be a XML_OPTION_* constant"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xml_parser_metadata_is_available_without_claiming_extension_loaded() {
    let execution = run_source(
        r#"<?php
foreach ([
    "xml_parser_create",
    "xml_parser_create_ns",
    "xml_parser_get_option",
    "xml_parser_set_option",
] as $name) {
    echo function_exists($name) ? "1" : "0";
}
echo "|";
echo defined("XML_OPTION_CASE_FOLDING") ? XML_OPTION_CASE_FOLDING : "missing";
echo ":";
echo defined("XML_OPTION_TARGET_ENCODING") ? XML_OPTION_TARGET_ENCODING : "missing";
echo ":";
echo extension_loaded("xml") ? "loaded" : "not-loaded";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1111|1:2:not-loaded");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn xml_parser_create_arguments_remain_outside_current_subset() {
    let error = run_source("<?php\nxml_parser_create_ns('ISO-8859-1');\n").unwrap_err();

    assert_eq!(
        error.message,
        "arity mismatch for xml_parser_create_ns(): expected 0 argument(s), got 1"
    );
}

#[test]
fn native_lowering_still_rejects_xml_parser_runtime_state() {
    let error = emit_ir_source("<?php\n$parser = xml_parser_create();\n").unwrap_err();

    assert!(
        error
            .message
            .contains("function-call lowering rejects function calls"),
        "{}",
        error.message
    );
}
