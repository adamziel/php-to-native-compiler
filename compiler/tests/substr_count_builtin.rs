use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_STRING_INT_OPERATION_REJECTION: &str = "LLVM string-int builtin lowering rejects strcasecmp(), substr_count(), ord(), and crc32() until native PHP string conversion, byte-preserving argument/result ownership, warning recovery, references/copy-on-write, and exact native builtin diagnostics exist; generated-native C routes lowerable string-int operands through the shared runtime contract";

#[test]
fn substr_count_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo substr_count("db:3306", ":");
echo "|";
echo substr_count("2001:db8::1", ":");
echo "|";
echo substr_count("aaaa", "aa");
echo "|";
echo substr_count("abcabc", "a", 1);
echo "|";
echo substr_count("abcabc", "a", 0, 3);
echo "|";
echo substr_count("abcabc", "a", 0, -1);
echo "|";
echo substr_count("abc", "c", -1);
echo "|";
echo substr_count("abc", "needle", 3);
echo "|";
echo substr_count(12121, 21);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|3|2|1|1|2|1|0|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_count_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "substr_count";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("a:b:c", ":");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_count_rejects_forms_outside_current_subset() {
    let empty_needle = run_source("<?php\nsubstr_count('abc', '');\n").unwrap_err();
    assert_eq!(empty_needle.phase, Phase::Runtime);
    assert_eq!(empty_needle.line, 2);
    assert_eq!(empty_needle.column, 1);
    assert_eq!(
        empty_needle.message,
        "unsupported call substr_count(): empty needles are not supported in the current subset"
    );

    let array_haystack = run_source("<?php\nsubstr_count(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call substr_count(): haystack argument arrays are not implemented in the current subset"
    );

    let bad_offset = run_source("<?php\nsubstr_count('abc', 'a', '1');\n").unwrap_err();
    assert_eq!(bad_offset.phase, Phase::Runtime);
    assert_eq!(bad_offset.line, 2);
    assert_eq!(bad_offset.column, 1);
    assert_eq!(
        bad_offset.message,
        "unsupported call substr_count(): offset argument must be int in the current subset, got string"
    );

    let bad_length = run_source("<?php\nsubstr_count('abc', 'a', 0, '2');\n").unwrap_err();
    assert_eq!(bad_length.phase, Phase::Runtime);
    assert_eq!(bad_length.line, 2);
    assert_eq!(bad_length.column, 1);
    assert_eq!(
        bad_length.message,
        "unsupported call substr_count(): length argument must be int in the current subset, got string"
    );

    let out_of_bounds = run_source("<?php\nsubstr_count('abc', 'a', 1, 5);\n").unwrap_err();
    assert_eq!(out_of_bounds.phase, Phase::Runtime);
    assert_eq!(out_of_bounds.line, 2);
    assert_eq!(out_of_bounds.column, 1);
    assert_eq!(
        out_of_bounds.message,
        "unsupported call substr_count(): length must keep the searched slice within the haystack bounds in the current subset"
    );

    let too_few = run_source("<?php\nsubstr_count('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for substr_count(): expected 2 to 4 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_substr_count_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("substr_count") ? "1" : "0";
echo is_callable("substr_count") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nsubstr_count('abc', 'b');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_STRING_INT_OPERATION_REJECTION);
}
