use php_compiler::run_source;

#[test]
fn ctype_builtins_classify_c_locale_ascii_strings() {
    let execution = run_source(
        r#"<?php
$checks = [
    ["ctype_alnum", "abcXYZ123", "abc-123"],
    ["ctype_alpha", "abcXYZ", "abc123"],
    ["ctype_cntrl", chr(0) . chr(31) . chr(127), "x"],
    ["ctype_digit", "0123456789", "12 34"],
    ["ctype_graph", "!A9~", "A B"],
    ["ctype_lower", "abcxyz", "abcXYZ"],
    ["ctype_print", " A9~", "A" . chr(127)],
    ["ctype_punct", "!@#", "A!"],
    ["ctype_space", " \t\r\n", " x"],
    ["ctype_upper", "ABCXYZ", "ABCxyz"],
    ["ctype_xdigit", "0129aAfF", "012g"],
];

foreach ($checks as $check) {
    $fn = $check[0];
    echo $fn, ":", $fn($check[1]) ? "1" : "0", "/", $fn($check[2]) ? "1" : "0", "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "ctype_alnum:1/0\n",
            "ctype_alpha:1/0\n",
            "ctype_cntrl:1/0\n",
            "ctype_digit:1/0\n",
            "ctype_graph:1/0\n",
            "ctype_lower:1/0\n",
            "ctype_print:1/0\n",
            "ctype_punct:1/0\n",
            "ctype_space:1/0\n",
            "ctype_upper:1/0\n",
            "ctype_xdigit:1/0\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ctype_integer_arguments_match_current_ascii_code_boundary() {
    let execution = run_source(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
var_dump(ctype_digit(48));
var_dump(ctype_digit(394829384));
var_dump(ctype_space(32));
var_dump(ctype_alpha(65));
var_dump(ctype_alpha(256));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base64_decode_feeds_binary_strings_to_ctype_checks() {
    let execution = run_source(
        r#"<?php
$decoded = base64_decode("w4DDoMOHw6fDiMOo");
var_dump(strlen($decoded));
var_dump(ctype_alnum($decoded));
var_dump(base64_decode("a GVs   bG8gd29ybGQh", true));
var_dump(base64_decode("aGVsbG8gd29ybGQh*", true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(12)\nbool(false)\nstring(12) \"hello world!\"\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base64_decode_strict_flag_uses_php_bool_boundary() {
    let execution = run_source(
        r#"<?php
$call = "base64_decode";
var_dump(base64_decode("aGVsbG8*", "0"));
var_dump(base64_decode("aGVsbG8*", "1"));
var_dump($call("aGVsbG8*", 0));
var_dump($call("aGVsbG8*", 1));
foreach ([[], new stdClass, fopen("php://memory", "r")] as $value) {
    try {
        var_dump(base64_decode("aGVsbG8*", $value));
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
            "string(5) \"hello\"\n",
            "bool(false)\n",
            "string(5) \"hello\"\n",
            "bool(false)\n",
            "base64_decode(): Argument #2 ($strict) must be of type bool, array given\n",
            "base64_decode(): Argument #2 ($strict) must be of type bool, stdClass given\n",
            "base64_decode(): Argument #2 ($strict) must be of type bool, resource given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base64_encode_round_trips_binary_strings_and_exposes_metadata() {
    let execution = run_source(
        r#"<?php
$payload = "Hello World" . chr(0) . chr(255);
$encoded = base64_encode($payload);
echo $encoded, "\n";
echo base64_decode($encoded) === $payload ? "roundtrip" : "bad", "\n";
echo base64_encode("f"), "|", base64_encode("fo"), "|", base64_encode("foo"), "\n";
echo function_exists("base64_encode") ? "fn" : "missing";
echo is_callable("base64_encode") ? ":callable:" : ":missing:";
$reflection = new ReflectionFunction("base64_encode");
echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters(), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "SGVsbG8gV29ybGQA/w==\nroundtrip\nZg==|Zm8=|Zm9v\nfn:callable:1/1\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ctype_non_string_non_int_arguments_emit_deprecation_and_return_false() {
    let execution = run_source(
        r#"<?php
var_dump(ctype_digit(true));
var_dump(ctype_digit(null));
var_dump(ctype_digit([]));
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains(
            "Deprecated: ctype_digit(): Argument of type bool will be interpreted as string in the future"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Deprecated: ctype_digit(): Argument of type null will be interpreted as string in the future"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Deprecated: ctype_digit(): Argument of type array will be interpreted as string in the future"
        ),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stdout.matches("bool(false)").count(), 3);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ctype_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
foreach (["ctype_alnum", "ctype_digit", "ctype_xdigit"] as $name) {
    echo function_exists($name) ? "1" : "0";
    echo is_callable($name) ? "1" : "0";
    $fn = new ReflectionFunction($name);
    echo ":", $fn->getName(), "/", $fn->getExtensionName(), "/";
    echo $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11:ctype_alnum/ctype/1/1;11:ctype_digit/ctype/1/1;11:ctype_xdigit/ctype/1/1;"
    );
    assert_eq!(execution.exit_code, 0);
}
