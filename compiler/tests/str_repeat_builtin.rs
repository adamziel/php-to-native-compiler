use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn str_repeat_repeats_php_scalar_string_bytes() {
    let execution = run_source(
        r#"<?php
echo str_repeat("ab", 3), "|";
echo str_repeat("x", 0), "|";
echo str_repeat(42, 2), "|";
echo str_repeat("ha", "3"), "|";
echo strlen(str_repeat("é", 2));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "ababab||4242|hahaha|4");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_repeat_is_available_through_builtin_callability() {
    let execution = run_source(
        r#"<?php
$call = "str_repeat";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("ha", 2);
echo "|";
echo function_exists("chr") && function_exists("bin2hex") ? "bytes" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|haha|bytes");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_repeat_exposes_reflection_metadata() {
    let execution = run_source(
        r#"<?php
$function = new ReflectionFunction("Str_Repeat");
$params = $function->getParameters();
echo $function->getName();
echo "|";
echo $function->getNumberOfParameters(), "/", $function->getNumberOfRequiredParameters();
echo "|";
echo $params[0]->getName(), ":", $params[0]->getType()->getName();
echo "|";
echo $params[1]->getName(), ":", $params[1]->getType()->getName();
echo "|";
echo $function->getReturnType()->getName();
echo "|";
echo $function->invoke("ha", 3);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "str_repeat|2/2|string:string|times:int|string|hahaha"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn byte_string_helpers_support_binary_chr_and_bin2hex_inputs() {
    let execution = run_source(
        r#"<?php
echo bin2hex(chr(0)), "|";
echo bin2hex(chr(128)), "|";
echo bin2hex(chr(255)), "|";
echo bin2hex(chr(1)), "|";
$binary = chr(0) . chr(255);
echo bin2hex(str_repeat($binary, 2));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "00|80|ff|01|00ff00ff");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chr_out_of_range_values_emit_php_deprecation_and_wrap_modulo_256() {
    let execution = run_source(
        r#"<?php
var_dump("\xFF" == chr(-1));
var_dump("\0" == chr(256));
"#,
    )
    .unwrap();

    let message = "Deprecated: chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256";
    assert_eq!(
        execution.stdout.matches(message).count(),
        2,
        "{:?}",
        execution.stdout
    );
    assert_eq!(
        execution.stdout.matches("bool(true)").count(),
        2,
        "{:?}",
        execution.stdout
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn str_repeat_rejects_forms_outside_current_subset() {
    let negative = run_source(
        r#"<?php
try {
    str_repeat('x', -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        negative.stdout,
        "str_repeat(): Argument #2 ($times) must be greater than or equal to 0"
    );
    assert_eq!(negative.exit_code, 0);

    let non_int_times = run_source("<?php\nstr_repeat('x', 'abc');\n").unwrap_err();
    assert_eq!(non_int_times.phase, Phase::Runtime);
    assert_eq!(non_int_times.line, 2);
    assert_eq!(non_int_times.column, 1);
    assert_eq!(
        non_int_times.message,
        "unsupported call str_repeat(): times argument must be int-compatible in the current subset, got string"
    );

    let array_string = run_source("<?php\nstr_repeat([], 2);\n").unwrap_err();
    assert_eq!(array_string.phase, Phase::Runtime);
    assert_eq!(array_string.line, 2);
    assert_eq!(array_string.column, 1);
    assert_eq!(
        array_string.message,
        "unsupported call str_repeat(): string argument arrays are not supported"
    );
}

#[test]
fn emit_ir_folds_str_repeat_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("str_repeat") ? "1" : "0";
echo is_callable("str_repeat") ? "1" : "0";
echo function_exists("chr") ? "1" : "0";
echo function_exists("bin2hex") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 4, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho str_repeat('x', 3);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
