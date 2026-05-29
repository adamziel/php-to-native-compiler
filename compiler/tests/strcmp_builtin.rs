use php_compiler::emit_ir_source;
use php_compiler::run_source;

#[test]
fn strcmp_compares_byte_strings_and_coerces_scalars() {
    let execution = run_source(
        r#"<?php
echo strcmp("abc", "abc") === 0 ? "eq" : "no";
echo "\n";
echo strcmp("abc", "abd") < 0 ? "lt" : "no";
echo "\n";
echo strcmp("abe", "abd") > 0 ? "gt" : "no";
echo "\n";
echo strcmp("A".chr(0)."B", "A".chr(0)."C") < 0 ? "nul" : "no";
echo "\n";
echo strcmp(123, "123") === 0 ? "coerced" : "no";
echo "\n";
echo function_exists("strcmp") ? "fn" : "missing";
echo "\n";
echo is_callable("strcmp") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "eq\nlt\ngt\nnul\ncoerced\nfn\ncallable");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strcmp_rejects_current_unsupported_argument_shapes() {
    let error = run_source("<?php\nstrcmp(['a'], 'a');\n").unwrap_err();
    assert!(
        error
            .message
            .contains("unsupported call strcmp(): first argument arrays are not implemented"),
        "{}",
        error.message
    );
}

#[test]
fn strcmp_native_metadata_already_routes_through_string_int_contract() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strcmp") ? "1" : "0";
echo is_callable("strcmp") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
