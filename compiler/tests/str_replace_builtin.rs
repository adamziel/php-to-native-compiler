use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn str_replace_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo str_replace("wp", "php", "wp wp"), "\n";
echo str_replace("-", "_", "post-type"), "\n";
echo str_replace("", "x", "abc"), "\n";
echo str_replace(1, "one", true);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "php php\npost_type\nabc\none");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "str_replace";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call(" ", "_", "hello world");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|hello_world");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_writes_count_to_direct_variable() {
    let execution = run_source(
        r#"<?php
$count = 99;
echo str_replace("na", "", "banana", $count), "|", $count, "\n";
$count = 99;
echo str_replace("z", "x", "banana", $count), "|", $count, "\n";
$count = 99;
echo str_replace("", "x", "banana", $count), "|", $count;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ba|2\nbanana|0\nbanana|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_executes_current_search_array_subset() {
    let execution = run_source(
        r#"<?php
$count = 0;
echo str_replace(["%0D", "%0A"], "", "%0%0DDD%0A", $count), "|", $count, "\n";
function deep_replace($search, $subject) {
    $count = 1;
    while ($count) {
        $subject = str_replace($search, "", $subject, $count);
    }
    return $subject;
}
echo deep_replace(["%0D", "%0A"], "%0%0DDD%0A");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "%0DD|2\nD");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_valued_str_replace_writes_count_to_direct_variable() {
    let execution = run_source(
        r#"<?php
$call = "str_replace";
$count = 0;
echo $call("a", "b", "a-a", $count), "|", $count;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b-b|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_ireplace_reuses_replacement_engine_case_insensitively() {
    let execution = run_source(
        r#"<?php
echo str_ireplace("tt", "a", "ttttTttttttttTT", $count), "|", $count, "\n";
$result = str_ireplace(
    array("tt", "y"),
    array("aaa", "bbb"),
    array("key" => "ttttTttttttttTT", "test" => "aayyaayasdayYahsdYYY"),
    $array_count
);
echo $result["key"], "\n";
echo $result["test"], "\n";
echo $array_count;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "aaaaaaaT|7\naaaaaaaaaaaaaaaaaaaaaT\naabbbbbbaabbbasdabbbbbbahsdbbbbbbbbb\n15"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_resource_type_error_preserves_count_variable() {
    let execution = run_source(
        r#"<?php
$fp = fopen("php://memory", "w+");
$fp_copy = $fp;
try {
    var_dump(str_replace($fp_copy, $fp_copy, $fp_copy, $fp_copy));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
var_dump($fp_copy);
fclose($fp);
"#,
    )
    .unwrap();

    let mut lines = execution.stdout.lines();
    assert_eq!(
        lines.next(),
        Some("str_replace(): Argument #1 ($search) must be of type array|string, resource given")
    );
    let resource_line = lines.next().expect("resource dump should be present");
    assert!(resource_line.starts_with("resource("), "{resource_line}");
    assert!(
        resource_line.ends_with(") of type (stream)"),
        "{resource_line}"
    );
    assert_eq!(lines.next(), None);
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_array_string_boundary_accepts_stringables_and_reports_type_errors() {
    let execution = run_source(
        r#"<?php
class SearchNeedle { public function __toString() { return "a"; } }
class Replacement { public function __toString() { return "x"; } }
class Subject { public function __toString() { return "abc"; } }

$fp = fopen("php://memory", "w+");
echo str_replace(new SearchNeedle, "z", "abc"), "|";
echo str_replace("b", new Replacement, "abc"), "|";
echo str_replace("b", "z", new Subject), "|";
$call = "str_ireplace";
echo $call(new SearchNeedle, "q", "ABC"), "|";

try {
    str_replace("a", $fp, "abc");
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    str_replace("a", "x", $fp);
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    str_replace(new stdClass, "x", "abc");
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    $call("a", new stdClass, "abc");
} catch (TypeError $e) {
    echo $e->getMessage();
}
fclose($fp);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "zbc|axc|azc|qBC|str_replace(): Argument #2 ($replace) must be of type array|string, resource given|str_replace(): Argument #3 ($subject) must be of type array|string, resource given|str_replace(): Argument #1 ($search) must be of type array|string, stdClass given|str_ireplace(): Argument #2 ($replace) must be of type array|string, stdClass given"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_warns_for_top_level_null_search_only() {
    let execution = run_source(
        r#"<?php
var_dump(str_replace(null, "x", "abc", $count));
var_dump($count);
var_dump(str_replace([null], "x", ["", "abc"], $array_count));
var_dump($array_count);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Deprecated: str_replace(): Passing null to parameter #1 ($search) of type array|string is deprecated in Command line code on line 2\n",
            "string(3) \"abc\"\n",
            "int(0)\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  string(0) \"\"\n",
            "  [1]=>\n",
            "  string(3) \"abc\"\n",
            "}\n",
            "int(0)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_replace_covers_array_forms_and_rejects_remaining_boundaries() {
    let missing = runtime_error(
        r#"<?php
echo str_replace("a", "b");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for str_replace(): expected 3 to 4 argument(s), got 2"
    );

    let count_target = runtime_error(
        r#"<?php
$counts = ["n" => 0];
echo str_replace("a", "b", "abc", $counts["n"]);
"#,
    );
    assert_eq!(count_target.line, 3);
    assert_eq!(count_target.column, 6);
    assert_eq!(
        count_target.message,
        "unsupported call str_replace(): count output must be a direct variable in the current subset"
    );

    let indirect_count = run_source(
        r#"<?php
echo call_user_func("str_replace", "a", "b", "abc", 0);
"#,
    )
    .unwrap();
    assert_eq!(
        indirect_count.stdout,
        "Warning: str_replace(): Argument #4 ($count) must be passed by reference, value given in Command line code on line 2\nbbc"
    );
    assert_eq!(indirect_count.stderr, "");
    assert_eq!(indirect_count.exit_code, 0);

    let array_replace = run_source(
        r#"<?php
echo str_replace("a", ["b"], "abc");
"#,
    )
    .unwrap();
    assert_eq!(array_replace.stdout, "bbc");
    assert_eq!(array_replace.stderr, "");
    assert_eq!(array_replace.exit_code, 0);

    let nested_array_search = run_source(
        r#"<?php
echo str_replace([["a"]], "b", "abc");
"#,
    )
    .unwrap();
    assert_eq!(
        nested_array_search.stdout,
        "Warning: Array to string conversion in Command line code on line 2\nabc"
    );
    assert_eq!(nested_array_search.stderr, "");
    assert_eq!(nested_array_search.exit_code, 0);

    let array_subject = run_source(
        r#"<?php
var_dump(str_replace("a", "b", ["abc"]));
"#,
    )
    .unwrap();
    assert_eq!(
        array_subject.stdout,
        "array(1) {\n  [0]=>\n  string(3) \"bbc\"\n}\n"
    );
    assert_eq!(array_subject.stderr, "");
    assert_eq!(array_subject.exit_code, 0);
}

#[test]
fn emit_ir_rejects_str_replace_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho str_replace('a', 'b', 'abc');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
