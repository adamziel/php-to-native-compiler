use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn base64_encode_encodes_scalar_and_binary_strings() {
    let execution = run_source(
        r#"<?php
$binary = "A" . chr(0) . chr(255);
echo base64_encode(""), "|";
echo base64_encode("f"), "|";
echo base64_encode("fo"), "|";
echo base64_encode("foo"), "|";
echo base64_encode("hello world"), "|";
echo base64_encode(42042), "|";
echo bin2hex(base64_decode(base64_encode($binary), true));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "|Zg==|Zm8=|Zm9v|aGVsbG8gd29ybGQ=|NDIwNDI=|4100ff"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base64_encode_supports_callable_and_reflection_metadata() {
    let execution = run_source(
        r#"<?php
$call = "base64_encode";
echo function_exists($call) ? "yes" : "no";
echo "|", is_callable($call) ? "callable" : "missing";
echo "|", $call("php");
$fn = new ReflectionFunction("BASE64_ENCODE");
echo "|", $fn->getName();
echo ":", $fn->getNumberOfRequiredParameters();
echo "/", $fn->getNumberOfParameters();
echo ":", $fn->getReturnType()->getName();
echo ":", $fn->invoke("wp");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|cGhw|base64_encode:1/1:string:d3A="
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn base64_encode_rejects_arity_and_array_inputs() {
    let arity = run_source("<?php\nbase64_encode();\n").unwrap();
    assert_eq!(arity.exit_code, 255);
    assert!(
        arity.stdout.contains(
            "Fatal error: Uncaught TypeError: Too few arguments to function base64_encode(), 0 passed in Command line code on line 2 and exactly 1 expected"
        ),
        "{}",
        arity.stdout
    );

    let array = run_source("<?php\nbase64_encode([]);\n").unwrap_err();
    assert_eq!(array.phase, Phase::Runtime);
    assert_eq!(array.line, 2);
    assert_eq!(array.column, 1);
    assert_eq!(
        array.message,
        "unsupported call base64_encode(): string argument arrays are not implemented in the current subset"
    );
}

#[test]
fn emit_ir_folds_base64_encode_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("base64_encode") ? "1" : "0";
echo is_callable("base64_encode") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nbase64_encode('x');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
