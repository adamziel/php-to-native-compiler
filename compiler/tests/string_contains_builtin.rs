use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_STRING_PREDICATE_REJECTION: &str = "LLVM string-predicate lowering rejects forms outside the reusable native string predicate contract until operands can reach byte-preserving value conversion, diagnostics, and cleanup; lowerable LLVM and generated-native C str_starts_with(), str_ends_with(), and str_contains() operands route through the shared runtime contract";

#[test]
fn str_contains_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo str_contains("128m", "m") ? "yes" : "no";
echo "|";
echo str_contains("128m", "g") ? "yes" : "no";
echo "|";
echo str_contains(42, "2") ? "yes" : "no";
echo "|";
echo str_contains(null, "") ? "yes" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no|yes|yes");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_contains_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "str_contains";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("abc", "b") ? "found" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|found");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_contains_rejects_forms_outside_current_subset() {
    let array_haystack = run_source("<?php\nstr_contains(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call str_contains(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstr_contains('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call str_contains(): needle argument arrays are not implemented in the current subset"
    );

    let too_few = run_source("<?php\nstr_contains('abc');\n").unwrap();
    assert_eq!(too_few.exit_code, 255);
    assert_eq!(too_few.stderr, "");
    assert!(
        too_few
            .stdout
            .contains("Fatal error: Uncaught TypeError: Too few arguments to function str_contains(), 1 passed"),
        "{}",
        too_few.stdout
    );
}

#[test]
fn emit_ir_routes_str_contains_metadata_and_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_contains") ? "1" : "0";
echo is_callable("str_contains") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir = emit_ir_source("<?php\necho str_contains('abc', 'b');\n").unwrap();
    assert!(
        ir.contains("call i1 @phpc_native_value_string_predicate_with_diagnostic"),
        "{ir}"
    );
    assert!(ir.contains("i8 2, ptr %"), "{ir}");
    assert!(
        !ir.contains("LLVM string-predicate lowering rejects"),
        "{ir}"
    );
}

#[test]
fn emit_ir_rejects_str_contains_unsupported_arity_at_shared_boundary() {
    for source in [
        "<?php\nstr_contains('abc');\n",
        "<?php\nstr_contains('abc', 'b', 'extra');\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, LLVM_STRING_PREDICATE_REJECTION);
    }
}
