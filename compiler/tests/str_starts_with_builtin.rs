use php_compiler::emit_asm_source;
use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_STRING_PREDICATE_REJECTION: &str = "LLVM string-predicate lowering rejects forms outside the reusable native string predicate contract until operands can reach byte-preserving value conversion, diagnostics, and cleanup; lowerable LLVM and generated-native C str_starts_with(), str_ends_with(), and str_contains() operands route through the shared runtime contract";

#[test]
fn str_starts_with_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo str_starts_with("wp-admin/admin-ajax.php", "wp-admin") ? "yes" : "no";
echo "|";
echo str_starts_with("index.php", "php") ? "yes" : "no";
echo "|";
echo str_starts_with("index.php", "") ? "empty" : "no";
echo "|";
echo str_starts_with(42, "4") ? "coerced" : "no";
echo "|";
echo str_starts_with(null, "") ? "null-empty" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no|empty|coerced|null-empty");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_starts_with_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "str_starts_with";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("wp-content/plugins/example.php", "wp-content") ? "prefix" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|prefix");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_starts_with_rejects_forms_outside_current_subset() {
    let array_haystack = run_source("<?php\nstr_starts_with(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(array_haystack.line, 2);
    assert_eq!(array_haystack.column, 1);
    assert_eq!(
        array_haystack.message,
        "unsupported call str_starts_with(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstr_starts_with('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(array_needle.line, 2);
    assert_eq!(array_needle.column, 1);
    assert_eq!(
        array_needle.message,
        "unsupported call str_starts_with(): needle argument arrays are not implemented in the current subset"
    );

    let too_few = run_source("<?php\nstr_starts_with('abc');\n").unwrap();
    assert!(
        too_few.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function str_starts_with(), 1 passed"
        ),
        "{}",
        too_few.stdout
    );
    assert!(
        too_few.stdout.contains("exactly 2 expected"),
        "{}",
        too_few.stdout
    );
    assert_eq!(too_few.stderr, "");
    assert_eq!(too_few.exit_code, 255);
}

#[test]
fn emit_ir_routes_str_starts_with_metadata_and_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_starts_with") ? "1" : "0";
echo is_callable("str_starts_with") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let ir = emit_ir_source("<?php\necho str_starts_with('abc', 'a');\n").unwrap();
    assert!(
        ir.contains("call i1 @phpc_native_value_string_predicate_with_diagnostic"),
        "{ir}"
    );
    assert!(ir.contains("i8 0, ptr %"), "{ir}");
    assert!(
        !ir.contains("LLVM string-predicate lowering rejects"),
        "{ir}"
    );
}

#[test]
fn emit_ir_rejects_str_starts_with_before_lowering_arguments() {
    for source in [
        "<?php\nstr_starts_with('abc');\n",
        "<?php\nstr_starts_with('abc', 'a', 'extra');\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();

        assert_eq!(error.phase, Phase::Codegen);
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 1);
        assert_eq!(error.message, LLVM_STRING_PREDICATE_REJECTION);
    }
}

#[test]
fn emit_asm_routes_str_starts_with_through_shared_predicate_abi() {
    let asm = emit_asm_source("<?php\necho str_starts_with('abc', 'a');\n").unwrap();

    assert!(
        asm.contains("call\tphpc_native_value_string_predicate_with_diagnostic"),
        "{asm}"
    );
    assert!(asm.contains("movl\t$0, %edx"), "{asm}");
    assert!(
        !asm.contains("assembly string-predicate lowering rejects"),
        "{asm}"
    );
}
