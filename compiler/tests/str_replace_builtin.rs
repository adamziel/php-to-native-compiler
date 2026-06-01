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
