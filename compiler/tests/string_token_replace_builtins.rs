use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn strtok_tracks_saved_tokenization_state() {
    let execution = run_source(
        r#"<?php
$str = "This testcase test strtok() function.";
$token = " ().";
var_dump(strtok($str, $token));
var_dump(strtok($token));
var_dump(strtok($token));
var_dump(strtok($token));
var_dump(strtok($token));
var_dump(strtok($token));
var_dump(strtok($token));
var_dump(strtok("\0", "\0"));
var_dump(strtok("\0"));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(4) \"This\"\n",
            "string(8) \"testcase\"\n",
            "string(4) \"test\"\n",
            "string(6) \"strtok\"\n",
            "string(8) \"function\"\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
            "\n",
            "Warning: strtok(): Both arguments must be provided when starting tokenization in Command line code on line 12\n",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_replace_handles_string_and_array_subjects() {
    let execution = run_source(
        r#"<?php
var_dump(substr_replace("try this", "bala ", 2));
var_dump(substr_replace("try this", "bala ", 2, 3));
var_dump(substr_replace("try this", ["bala "], 4, 3));
print_r(substr_replace(["abc" => "llsskdkk", "def" => "llsskjkkdd", 4 => "hello", 42 => "world"], "zzz", 0, -2));
print_r(substr_replace(["1 string", "2 string"], ["A", 2 => "B"], 0));
print_r(substr_replace(["ala portokala", "try this"], ["bala "], [4], 3));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(7) \"trbala \"\n",
            "string(10) \"trbala his\"\n",
            "string(10) \"try bala s\"\n",
            "Array\n",
            "(\n",
            "    [abc] => zzzkk\n",
            "    [def] => zzzdd\n",
            "    [4] => zzzlo\n",
            "    [42] => zzzld\n",
            ")\n",
            "Array\n",
            "(\n",
            "    [0] => A\n",
            "    [1] => B\n",
            ")\n",
            "Array\n",
            "(\n",
            "    [0] => ala bala tokala\n",
            "    [1] =>  this\n",
            ")\n",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_replace_rejects_string_subject_array_offsets() {
    let error = run_source(
        r#"<?php
try {
    substr_replace("Good morning", "evening", [5]);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    substr_replace("Good morning", "evening", 5, [1]);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        error.stdout,
        concat!(
            "substr_replace(): Argument #3 ($offset) cannot be an array when working on a single string\n",
            "substr_replace(): Argument #4 ($length) cannot be an array when working on a single string\n",
        )
    );
}

#[test]
fn emit_ir_rejects_stateful_string_builtins_until_native_runtime_calls_exist() {
    let error = emit_ir_source("<?php\necho strtok('a b', ' ');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source("<?php\necho substr_replace('abc', 'x', 1);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
