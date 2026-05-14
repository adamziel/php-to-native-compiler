use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn assert_builtin_accepts_truthy_assertions() {
    let execution = run_source(
        r#"<?php
class ParagonIE_Sodium_Compat {}
echo assert(true) ? "1" : "0";
echo assert(1, "ok") ? "1" : "0";
echo assert("false") ? "1" : "0";
echo assert(class_exists("ParagonIE_Sodium_Compat"), "Possible filesystem/autoloader bug?") ? "1" : "0";
$call = "assert";
echo $call(true, null) ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11111");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assert_builtin_evaluates_arguments_left_to_right() {
    let execution = run_source(
        r#"<?php
function mark($label) {
    echo $label;
    return true;
}

assert(mark("A"), mark("B"));
echo "C";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ABC");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn assert_builtin_reports_current_failure_boundary() {
    let error = run_source(
        r#"<?php
assert(false, "boom");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call assert(): assertion failures are not implemented in the current subset"
    );
}

#[test]
fn assert_builtin_rejects_unsupported_description_values() {
    let error = run_source(
        r#"<?php
assert(true, []);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call assert(): description argument must be null, bool, int, float, or string in the current subset, got array"
    );
}

#[test]
fn emit_ir_rejects_direct_assert_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\nassert(true);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
