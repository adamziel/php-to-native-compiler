use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn strcasecmp_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo strcasecmp("UTF-8", "utf-8") === 0 ? "same" : "diff";
echo "\n";
echo strcasecmp("abc", "ABD") < 0 ? "lt" : "not";
echo "\n";
echo strcasecmp("beta", "ALPHA") > 0 ? "gt" : "not";
echo "\n";
echo strcasecmp(123, "123") === 0 ? "coerced" : "no";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "same\nlt\ngt\ncoerced");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strcasecmp_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "strcasecmp";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call("UTF8", "utf8") === 0 ? "same" : "diff";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|same");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strcasecmp_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo strcasecmp("a");
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for strcasecmp(): expected 2 argument(s), got 1"
    );

    let array_left = runtime_error(
        r#"<?php
echo strcasecmp(["a"], "a");
"#,
    );
    assert_eq!(array_left.line, 2);
    assert_eq!(array_left.column, 6);
    assert_eq!(
        array_left.message,
        "unsupported call strcasecmp(): first argument arrays are not implemented in the current subset"
    );

    let array_right = runtime_error(
        r#"<?php
echo strcasecmp("a", ["a"]);
"#,
    );
    assert_eq!(array_right.line, 2);
    assert_eq!(array_right.column, 6);
    assert_eq!(
        array_right.message,
        "unsupported call strcasecmp(): second argument arrays are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_routes_strcasecmp_through_native_string_int_contract() {
    let ir = emit_ir_source("<?php\necho strcasecmp('A', 'a');\n").unwrap();

    assert!(
        ir.contains("declare i64 @phpc_native_value_string_int_operation_with_diagnostic"),
        "{ir}"
    );
    assert!(
        ir.contains("call i64 @phpc_native_value_string_int_operation_with_diagnostic"),
        "{ir}"
    );
    assert!(ir.contains("i8 0, ptr %"), "{ir}");
    assert!(
        !ir.contains("LLVM string-int builtin lowering rejects"),
        "{ir}"
    );
}
