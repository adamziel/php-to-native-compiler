use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn spl_autoload_register_accepts_current_callback_shapes_without_invoking_them() {
    let execution = run_source(
        r#"<?php
$called = "no";
echo spl_autoload_register(function ($class) use ($called) {
    echo "called";
    return false;
}) ? "1" : "0";
echo "|", $called, "\n";

$arrow_called = "no";
echo spl_autoload_register(fn ($class) => false) ? "1" : "0";
echo "|", $arrow_called, "\n";

$call = "spl_autoload_register";
function MissingAutoloader($class) {
    return false;
}
echo $call("MissingAutoloader", true, false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|no\n1|no\n1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn spl_autoload_register_reports_current_argument_boundaries() {
    let non_callable = run_source("<?php\nspl_autoload_register(42);\n").unwrap_err();
    assert_eq!(non_callable.phase, Phase::Runtime);
    assert_eq!(non_callable.line, 2);
    assert_eq!(non_callable.column, 1);
    assert_eq!(
        non_callable.message,
        "unsupported call spl_autoload_register(): callback argument must be closure or string in the current subset, got int"
    );

    let non_bool_throw = run_source("<?php\nspl_autoload_register('Loader', 1);\n").unwrap_err();
    assert_eq!(non_bool_throw.phase, Phase::Runtime);
    assert_eq!(non_bool_throw.line, 2);
    assert_eq!(non_bool_throw.column, 1);
    assert_eq!(
        non_bool_throw.message,
        "unsupported call spl_autoload_register(): argument #2 must be bool in the current subset, got int"
    );
}

#[test]
fn emit_ir_rejects_direct_spl_autoload_register_until_native_autoloading_exists() {
    let error = emit_ir_source("<?php\nspl_autoload_register('MissingAutoloader');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
