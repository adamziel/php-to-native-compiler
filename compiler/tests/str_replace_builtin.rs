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
fn str_ireplace_reuses_replacement_engine_case_insensitively() {
    let execution = run_source(
        r#"<?php
$count = 0;
echo str_ireplace("tt", "a", "ttttTttttttttTT", $count), "|", $count, "\n";
$count = 0;
echo str_ireplace(["%0d", "%0a"], "", "%0%0DDD%0a", $count), "|", $count, "\n";
$call = "str_ireplace";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call("wp", "php", "WP wp", $count), "|", $count, "\n";
echo call_user_func("str_ireplace", "A", "x", "aA"), "\n";
$count = 0;
echo call_user_func_array("str_ireplace", ["a", "b", "BanAna", "count" => &$count]), "|", $count;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "aaaaaaaT|7\n%0DD|2\nyes|callable|php php|2\nxx\nBbnbnb|3"
    );
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
fn str_replace_rejects_forms_outside_current_subset() {
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

    let indirect_count = runtime_error(
        r#"<?php
echo call_user_func("str_replace", "a", "b", "abc", 0);
"#,
    );
    assert_eq!(indirect_count.line, 2);
    assert_eq!(indirect_count.column, 6);
    assert_eq!(
        indirect_count.message,
        "unsupported call str_replace(): count output requires a direct str_replace() call with a direct variable in the current subset"
    );

    let array_replace = runtime_error(
        r#"<?php
echo str_replace("a", ["b"], "abc");
"#,
    );
    assert_eq!(array_replace.line, 2);
    assert_eq!(array_replace.column, 6);
    assert_eq!(
        array_replace.message,
        "unsupported call str_replace(): replacement argument arrays are not implemented in the current subset"
    );

    let nested_array_search = runtime_error(
        r#"<?php
echo str_replace([["a"]], "b", "abc");
"#,
    );
    assert_eq!(nested_array_search.line, 2);
    assert_eq!(nested_array_search.column, 6);
    assert_eq!(
        nested_array_search.message,
        "unsupported call str_replace(): search array values must be null, bool, int, float, or string in the current subset, got array"
    );

    let array_subject = runtime_error(
        r#"<?php
echo str_replace("a", "b", ["abc"]);
"#,
    );
    assert_eq!(array_subject.line, 2);
    assert_eq!(array_subject.column, 6);
    assert_eq!(
        array_subject.message,
        "unsupported call str_replace(): subject argument arrays are not implemented in the current subset"
    );
}

#[test]
fn str_ireplace_rejects_forms_outside_current_subset_with_own_label() {
    let missing = runtime_error(
        r#"<?php
echo str_ireplace("a", "b");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for str_ireplace(): expected 3 to 4 argument(s), got 2"
    );

    let count_target = runtime_error(
        r#"<?php
$counts = ["n" => 0];
echo str_ireplace("a", "b", "abc", $counts["n"]);
"#,
    );
    assert_eq!(count_target.line, 3);
    assert_eq!(count_target.column, 6);
    assert_eq!(
        count_target.message,
        "unsupported call str_ireplace(): count output must be a direct variable in the current subset"
    );

    let indirect_count = runtime_error(
        r#"<?php
echo call_user_func("str_ireplace", "a", "b", "abc", 0);
"#,
    );
    assert_eq!(indirect_count.line, 2);
    assert_eq!(indirect_count.column, 6);
    assert_eq!(
        indirect_count.message,
        "unsupported call str_ireplace(): count output requires a direct str_ireplace() call with a direct variable in the current subset"
    );

    let array_replace = runtime_error(
        r#"<?php
echo str_ireplace("a", ["b"], "abc");
"#,
    );
    assert_eq!(array_replace.line, 2);
    assert_eq!(array_replace.column, 6);
    assert_eq!(
        array_replace.message,
        "unsupported call str_ireplace(): replacement argument arrays are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_rejects_str_replace_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho str_replace('a', 'b', 'abc');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho str_ireplace('a', 'b', 'abc');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_folds_str_ireplace_function_exists_name() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_ireplace") ? "1" : "0";
echo function_exists("STR_IREPLACE") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("str_ireplace"), "{ir}");
    assert!(!ir.contains("STR_IREPLACE"), "{ir}");
}
