use php_compiler::{emit_ir_source, run_source};

#[test]
fn quoted_printable_encode_decode_match_binary_and_object_smoke() {
    let execution = run_source(
        r#"<?php
var_dump(quoted_printable_encode(""));
var_dump(quoted_printable_encode("test"));
var_dump(quoted_printable_encode(1));
var_dump(quoted_printable_encode(false));
$encoded = quoted_printable_encode(str_repeat("\0", 26));
echo strlen($encoded), "|", substr_count($encoded, "=\r\n"), "|", bin2hex(substr($encoded, -3)), "\n";
echo bin2hex(quoted_printable_decode("=FAwow-factor=C1=d0=0A=\r\n=20done")), "\n";
class Foo {
    function __toString() {
        return "this is a foo";
    }
}
var_dump(quoted_printable_encode(new Foo));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(0) \"\"\nstring(4) \"test\"\nstring(1) \"1\"\nstring(0) \"\"\n81|1|3d3030\nfa776f772d666163746f72c1d00a20646f6e65\nstring(13) \"this is a foo\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn quoted_printable_decode_accepts_php_soft_break_variants() {
    let execution = run_source(
        r#"<?php
foreach ([
    "abc=",
    "abc= \r\ndef",
    "abc=\rdef",
    "abc=\ndef",
    "abc=\t\ndef",
    "abc= \t",
] as $value) {
    echo bin2hex(quoted_printable_decode($value)), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "616263\n616263646566\n616263646566\n616263646566\n616263646566\n616263\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn quoted_printable_encode_matches_php_newline_space_and_tab_edges() {
    let execution = run_source(
        r#"<?php
foreach ([
    "\n",
    "\r",
    "\r\n",
    " ",
    "hello ",
    "hello \n",
    "\t",
    "a\tb",
    " \r\n",
] as $value) {
    echo bin2hex(quoted_printable_encode($value)), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3d3041\n3d3044\n0d0a\n20\n68656c6c6f20\n68656c6c6f203d3041\n3d3039\n613d303962\n3d32300d0a\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn quoted_printable_encode_uses_php_byte_class_wrap_limits() {
    let execution = run_source(
        r#"<?php
foreach ([
    str_repeat("\0", 26),
    str_repeat("\xA9", 25),
    str_repeat("\xE2\x82\xAC", 13),
    str_repeat("\xF0\x9F\x98\x80", 7),
] as $value) {
    $encoded = quoted_printable_encode($value);
    $parts = explode("=\r\n", $encoded);
    echo strlen($parts[0]), ":", substr_count($encoded, "=\r\n"), ":", strlen($encoded), ":", bin2hex(substr($encoded, -3)), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "75:1:81:3d3030\n72:1:78:3d4139\n72:1:120:3d4143\n72:1:87:3d3830\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn quoted_printable_string_parameter_boundary_matches_php_types() {
    let execution = run_source(
        r#"<?php
foreach ([["x"], fopen("php://memory", "r")] as $value) {
    foreach (["quoted_printable_encode", "quoted_printable_decode"] as $function) {
        try {
            var_dump($function($value));
        } catch (TypeError $e) {
            echo get_class($e), ":", $e->getMessage(), "\n";
        }
    }
}
class StringableQuotedPrintable {
    function __toString() {
        return "ok";
    }
}
class PlainQuotedPrintable {}
var_dump(quoted_printable_encode(new StringableQuotedPrintable));
var_dump(quoted_printable_decode(new StringableQuotedPrintable));
foreach (["quoted_printable_encode", "quoted_printable_decode"] as $function) {
    try {
        var_dump($function(new PlainQuotedPrintable));
    } catch (TypeError $e) {
        echo get_class($e), ":", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "TypeError:quoted_printable_encode(): Argument #1 ($string) must be of type string, array given\n",
            "TypeError:quoted_printable_decode(): Argument #1 ($string) must be of type string, array given\n",
            "TypeError:quoted_printable_encode(): Argument #1 ($string) must be of type string, resource given\n",
            "TypeError:quoted_printable_decode(): Argument #1 ($string) must be of type string, resource given\n",
            "string(2) \"ok\"\n",
            "string(2) \"ok\"\n",
            "TypeError:quoted_printable_encode(): Argument #1 ($string) must be of type string, PlainQuotedPrintable given\n",
            "TypeError:quoted_printable_decode(): Argument #1 ($string) must be of type string, PlainQuotedPrintable given\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn quoted_printable_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["quoted_printable_decode", "quoted_printable_encode"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11:1/1;11:1/1;");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_quoted_printable_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("quoted_printable_decode") ? "1" : "0";
echo is_callable("quoted_printable_encode") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("quoted_printable"), "{ir}");
}
