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
fn implode_executes_current_scalar_array_subset() {
    let execution = run_source(
        r#"<?php
echo implode(["a", "b", "c"]), "\n";
echo implode("|", ["a", 2, null, true, false, 3.5]), "\n";
echo implode(" ", ["first" => "wp", 10 => "php"]);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "abc\na|2||1||3.5\nwp php");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn implode_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "implode";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|", $call("<br>", ["one", "two"]);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|one<br>two");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn implode_rejects_forms_outside_current_subset() {
    let missing = runtime_error(
        r#"<?php
echo implode();
"#,
    );
    assert_eq!(missing.line, 2);
    assert_eq!(missing.column, 6);
    assert_eq!(
        missing.message,
        "arity mismatch for implode(): expected 1 to 2 argument(s), got 0"
    );

    let single = runtime_error(
        r#"<?php
echo implode("not-array");
"#,
    );
    assert_eq!(single.line, 2);
    assert_eq!(single.column, 6);
    assert_eq!(
        single.message,
        "unsupported call implode(): single argument must be array in the current subset, got string"
    );

    let separator = runtime_error(
        r#"<?php
echo implode(42, ["a"]);
"#,
    );
    assert_eq!(separator.line, 2);
    assert_eq!(separator.column, 6);
    assert_eq!(
        separator.message,
        "unsupported call implode(): separator argument must be string in the current subset, got int"
    );

    let array = runtime_error(
        r#"<?php
echo implode(",", "not-array");
"#,
    );
    assert_eq!(array.line, 2);
    assert_eq!(array.column, 6);
    assert_eq!(
        array.message,
        "unsupported call implode(): array argument must be array in the current subset, got string"
    );

    let value = runtime_error(
        r#"<?php
echo implode(",", [["nested"]]);
"#,
    );
    assert_eq!(value.line, 2);
    assert_eq!(value.column, 6);
    assert_eq!(
        value.message,
        "unsupported call implode(): array values must be null, bool, int, float, or string in the current subset, got array"
    );
}

#[test]
fn emit_ir_rejects_implode_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
echo implode(",", ["a", "b"]);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_includes_implode_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("implode") ? "1" : "0";
echo is_callable("implode") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
