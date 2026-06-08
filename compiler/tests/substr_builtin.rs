use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn substr_executes_current_scalar_string_subset() {
    let execution = run_source(
        r#"<?php
echo substr("abcdef", 2);
echo "|";
echo substr("abcdef", 2, 3);
echo "|";
echo substr("abcdef", -2);
echo "|";
echo substr("abcdef", 0, -1);
echo "|";
echo substr("abcdef", -4, 2);
echo "|";
echo substr("abcdef", 99) === "" ? "empty" : "nonempty";
echo "|";
echo substr("abcdef", 2, null);
echo "|";
echo substr(12345, 1, 3);
echo "|";
echo substr("abcdef", "2", "3");
echo "|";
echo substr("abcdef", 1.9, "2.8");
echo "|";
echo substr("abcdef", false, true);
echo "|";
echo substr("abcdef", null, 2);
echo "|";
echo substr("abcdef", -2.7, null);
echo "|";
echo substr("x", PHP_INT_MIN);
echo "|";
echo substr("x", 0, PHP_INT_MIN) === "" ? "empty" : "nonempty";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "cdef|cde|ef|abcde|cd|empty|cdef|234|cde|bc|a|ab|ef|x|empty"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "substr";
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call("abcdef", -3, 2);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|de");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_uses_php_string_argument_boundary() {
    let execution = run_source(
        r#"<?php
class Label {
    public function __toString() {
        return "abcdef";
    }
}

$call = "substr";
echo substr(new Label(), 1, 3), "|";
echo $call(new Label(), -2), "|";
try {
    substr(null, 0, 1);
} catch (Throwable $e) {
    echo "unexpected";
}
echo "|";
try {
    substr([], 0);
} catch (TypeError $e) {
    echo $e->getMessage(), "|";
}
try {
    substr(new stdClass(), 0);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Deprecated: substr(): Passing null to parameter #1 ($string) of type string is deprecated"
    ));
    assert!(execution.stdout.starts_with("bcd|ef|"));
    assert!(execution.stdout.ends_with(
        "|substr(): Argument #1 ($string) must be of type string, array given|substr(): Argument #1 ($string) must be of type string, stdClass given"
    ));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_rejects_forms_outside_current_subset() {
    let array_string = run_source(
        r#"<?php
try {
    substr(['abc'], 1);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array_string.stdout,
        "substr(): Argument #1 ($string) must be of type string, array given"
    );

    let bad_offset = run_source(
        r#"<?php
try {
    substr('abc', 'bad');
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_offset.stdout,
        "substr(): Argument #2 ($offset) must be of type int, string given"
    );

    let bad_length = run_source(
        r#"<?php
try {
    substr('abc', 0, 'bad');
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        bad_length.stdout,
        "substr(): Argument #3 ($length) must be of type ?int, string given"
    );

    let array_offset = run_source(
        r#"<?php
try {
    substr('abc', []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array_offset.stdout,
        "substr(): Argument #2 ($offset) must be of type int, array given"
    );

    let too_few = run_source("<?php\nsubstr('abc');\n").unwrap();
    assert!(too_few.stdout.starts_with(
        "Fatal error: Uncaught ArgumentCountError: substr() expects at least 2 arguments, 1 given"
    ));
    assert_eq!(too_few.exit_code, 255);
}

#[test]
fn emit_ir_folds_substr_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("substr") ? "1" : "0";
echo is_callable("substr") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\nsubstr('abc', 1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
