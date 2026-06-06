use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strpos_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo strpos("db:3306", ":") === 2 ? "colon" : "missing";
echo "|";
echo strpos("db:3306", "x") === false ? "false" : "found";
echo "|";
echo strpos("abc", "") === 0 ? "empty0" : "no";
echo "|";
echo strpos("abc", "", 2) === 2 ? "empty2" : "no";
echo "|";
echo strpos("abcabc", "b", 2) === 4 ? "offset" : "no";
echo "|";
echo strpos("abc", "c", -1) === 2 ? "negative" : "no";
echo "|";
echo strpos(12345, 34) === 2 ? "coerced" : "no";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "colon|false|empty0|empty2|offset|negative|coerced"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strpos_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strpos";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("abc", "b") === 1 ? "found" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|found");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strpos_rejects_forms_outside_current_subset() {
    let array_haystack = run_source("<?php\nstrpos(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call strpos(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstrpos('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call strpos(): needle argument arrays are not implemented in the current subset"
    );

    let bad_offset = run_source("<?php\nstrpos('abc', 'a', '1');\n").unwrap_err();
    assert_eq!(bad_offset.phase, Phase::Runtime);
    assert_eq!(bad_offset.line, 2);
    assert_eq!(bad_offset.column, 1);
    assert_eq!(
        bad_offset.message,
        "unsupported call strpos(): offset argument must be int in the current subset, got string"
    );

    let out_of_bounds = run_source("<?php\nstrpos('abc', 'a', 4);\n").unwrap_err();
    assert_eq!(out_of_bounds.phase, Phase::Runtime);
    assert_eq!(out_of_bounds.line, 2);
    assert_eq!(out_of_bounds.column, 1);
    assert_eq!(
        out_of_bounds.message,
        "unsupported call strpos(): offset must be within the haystack bounds in the current subset"
    );

    let too_few = run_source("<?php\nstrpos('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for strpos(): expected 2 to 3 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_strpos_metadata_and_routes_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strpos") ? "1" : "0";
echo is_callable("strpos") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let routed =
        emit_ir_source("<?php\necho strpos('abc', 'b'); echo '|'; echo strpos('abc', 'x');\n")
            .unwrap();
    assert!(
        routed.contains("phpc_native_value_string_search_result_with_diagnostic"),
        "{routed}"
    );
    assert!(routed.contains("i8 0, ptr %"), "{routed}");
    assert!(
        routed.contains("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free"),
        "{routed}"
    );
    assert!(
        !routed.contains("function-call lowering rejects function calls"),
        "{routed}"
    );
}
