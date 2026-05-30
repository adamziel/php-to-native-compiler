use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn platform_and_version_functions_are_available() {
    let execution = run_source(
        r#"<?php
foreach (["a", "s", "n", "r", "v", "m"] as $mode) {
    $value = php_uname($mode);
    echo is_string($value) && strlen($value) > 0 ? $mode : "bad";
}
echo "|";
echo php_uname() === php_uname("a") ? "default" : "bad";
echo "|";
echo phpversion();
echo "|";
echo phpversion("standard");
echo "|";
var_dump(phpversion("not_loaded"));
echo "|";
foreach (["php_uname", "phpversion"] as $call) {
    echo function_exists($call) ? "1" : "0";
    echo is_callable($call) ? "1" : "0";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "asnrvm|default|8.3.0|8.3.0|bool(false)\n|1111"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_uname_invalid_modes_are_catchable_value_errors() {
    let execution = run_source(
        r#"<?php
foreach (["", "test", "z"] as $mode) {
    try {
        php_uname($mode);
    } catch (Throwable $e) {
        echo $e::class, ": ", $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError: php_uname(): Argument #1 ($mode) must be a single character\n\
ValueError: php_uname(): Argument #1 ($mode) must be a single character\n\
ValueError: php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_platform_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("php_uname") ? "1" : "0";
echo is_callable("phpversion") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho php_uname();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho phpversion();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
