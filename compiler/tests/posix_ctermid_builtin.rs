#![cfg(unix)]

use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn posix_ctermid_returns_terminal_path_and_metadata() {
    let execution = run_source(
        r#"<?php
$path = posix_ctermid();
echo function_exists("posix_ctermid") ? "fn" : "missing";
echo "|";
echo is_callable("posix_ctermid") ? "callable" : "not-callable";
echo "|";
echo extension_loaded("posix") ? "posix" : "no-posix";
echo "|";
echo is_string($path) && strlen($path) > 0 ? "path" : gettype($path);
echo "|";
$call = "posix_ctermid";
echo $call() === $path ? "dynamic" : "different";
echo "|";
$reflection = new ReflectionFunction("posix_ctermid");
echo $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
echo "/", $reflection->getExtensionName();
echo "/", ($reflection->hasReturnType() ? "return" : "no-return");
echo "|";
$viaInvoke = $reflection->invoke();
echo is_string($viaInvoke) && strlen($viaInvoke) > 0 ? "invoke" : "bad-invoke";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|posix|path|dynamic|0/0/posix/return|invoke"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn posix_ctermid_rejects_arguments() {
    let error = run_source("<?php\nposix_ctermid(1);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "arity mismatch for posix_ctermid(): expected 0 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_posix_ctermid_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("posix_ctermid") ? "1" : "0";
echo is_callable("posix_ctermid") ? "1" : "0";
echo extension_loaded("posix") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("extension_loaded"), "{ir}");

    let error = emit_ir_source("<?php\necho posix_ctermid();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
