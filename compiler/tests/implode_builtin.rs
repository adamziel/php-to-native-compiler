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
fn implode_rejects_invalid_arity_and_handles_php_shaped_boundaries() {
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

    let single = run_source(
        r#"<?php
echo implode("not-array");
"#,
    )
    .unwrap();
    assert_eq!(single.exit_code, 255);
    assert_eq!(single.stderr, "");
    assert!(
        single.stdout.contains(
            "TypeError: implode(): If argument #1 ($separator) is of type string, argument #2 ($array) must be of type array, null given"
        ),
        "{}",
        single.stdout
    );

    let separator = run_source(
        r#"<?php
echo implode(42, ["a", "b"]);
"#,
    )
    .unwrap();
    assert_eq!(separator.stdout, "a42b");
    assert_eq!(separator.exit_code, 0);
    assert_eq!(separator.stderr, "");

    let array = run_source(
        r#"<?php
echo implode(",", "not-array");
"#,
    )
    .unwrap();
    assert_eq!(array.exit_code, 255);
    assert_eq!(array.stderr, "");
    assert!(
        array.stdout.contains(
            "TypeError: implode(): Argument #2 ($array) must be of type ?array, string given"
        ),
        "{}",
        array.stdout
    );

    let value = run_source(
        r#"<?php
echo implode(",", [["nested"]]);
"#,
    )
    .unwrap();
    assert_eq!(value.exit_code, 0);
    assert_eq!(value.stderr, "");
    assert_eq!(
        value.stdout,
        "Warning: Array to string conversion in Command line code on line 2\nArray"
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
