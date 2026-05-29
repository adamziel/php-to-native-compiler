use php_compiler::emit_ir_source;
use php_compiler::run_source;

#[test]
fn slash_escape_builtins_round_trip_byte_strings() {
    let execution = run_source(
        r#"<?php
$input = '';
for ($i = 0; $i < 128; $i++) {
    $input .= chr($i);
}
echo bin2hex($input) === bin2hex(stripslashes(addslashes($input))) ? "round-trip" : "broken";
echo "\n";
echo bin2hex(addslashes("A".chr(0)."B\"\\'")), "\n";
echo bin2hex(stripslashes("\\0\\\"\\\\\\'\\q\\")), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "round-trip\n415c30425c225c5c5c27\n00225c2771\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn cslashes_builtins_cover_masks_ranges_and_c_escapes() {
    let execution = run_source(
        r#"<?php
echo bin2hex("\v\f\e"), "\n";
echo addcslashes("foobarbaz", "bar"), "\n";
echo addcslashes("foo[ ]", "A..z"), "\n";
echo addcslashes("abcdefghijklmnopqrstuvwxyz", "a\145..\160z"), "\n";
echo bin2hex(addcslashes("A".chr(0)."\n\r\t".chr(11).chr(12).chr(7)."B", "\0..\37")), "\n";
echo bin2hex(stripcslashes("\\n\\r\\t\\v\\f\\a\\b\\065\\x64")), "\n";
echo stripcslashes("\\H\\e\\l\\l\\o \\W\\or\\l\\d"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "0b0c1b\n",
            "foo\\b\\a\\r\\b\\az\n",
            "\\f\\o\\o\\[ \\]\n",
            "\\abcd\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\pqrstuvwxy\\z\n",
            "415c3030305c6e5c725c745c765c665c6142\n",
            "0a0d090b0c07083564\n",
            "Hello World\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn cslashes_builtins_use_php_string_argument_boundaries() {
    let execution = run_source(
        r#"<?php
class StringableSubject {
    public function __toString() {
        return "Object";
    }
}

echo addcslashes(new StringableSubject(), "b"), "\n";
try {
    addcslashes("value", array());
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    stripslashes(array("value"));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "O\\bject\n",
            "addcslashes(): Argument #2 ($characters) must be of type string, array given\n",
            "stripslashes(): Argument #1 ($string) must be of type string, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn addcslashes_reports_invalid_descending_ranges() {
    let execution =
        run_source("<?php\necho addcslashes(\"zoo['.']\", \"z..A\"), \"\\n\";\n").unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Warning: addcslashes(): Invalid '..'-range, '..'-range needs to be incrementing ",
            "in Command line code on line 2\n",
            "\\zoo['\\.']\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn escape_builtins_are_available_through_string_valued_calls_and_native_metadata() {
    let execution = run_source(
        r#"<?php
$calls = array("addslashes", "stripslashes", "addcslashes", "stripcslashes");
foreach ($calls as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
}
echo "|", bin2hex($calls[0]("'"));
echo "|", bin2hex($calls[1]("\\0"));
echo "|", $calls[2]("abc", "b");
echo "|", bin2hex($calls[3]("\\x41"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111111|5c27|00|a\\bc|41");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);

    let ir = emit_ir_source(
        r#"<?php
echo function_exists("addslashes") ? "1" : "0";
echo is_callable("stripslashes") ? "1" : "0";
echo function_exists("addcslashes") ? "1" : "0";
echo is_callable("stripcslashes") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
