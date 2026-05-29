use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strrpos_executes_reverse_byte_search_with_offsets() {
    let execution = run_source(
        r#"<?php
echo strrpos("Hello, World", "Hello") === 0 ? "start" : "bad";
echo "|";
echo strrpos("Hello, World", "o") === 8 ? "last-o" : "bad";
echo "|";
echo strrpos("Hello, World", "o", 10) === false ? "offset-miss" : "bad";
echo "|";
echo strrpos("haystack", "a", -3) === 5 ? "neg-a" : "bad";
echo "|";
echo strrpos("haystack", "a", -4) === 1 ? "neg-early" : "bad";
echo "|";
echo strrpos("te" . chr(0) . "st", chr(0)) === 2 ? "nul" : "bad";
echo "|";
echo strrpos(1234512345, 345) === 7 ? "coerced" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "start|last-o|offset-miss|neg-a|neg-early|nul|coerced"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strripos_executes_ascii_case_insensitive_reverse_search() {
    let execution = run_source(
        r#"<?php
echo strripos("Hello, World", "HELLO") === 0 ? "start" : "bad";
echo "|";
echo strripos("Hello, World", "O") === 8 ? "last-o" : "bad";
echo "|";
echo strripos("HAYSTACK", "a", -3) === 5 ? "neg-a" : "bad";
echo "|";
echo strripos("ababababAbaBa", "aba", -1) === 10 ? "last-prefix" : "bad";
echo "|";
echo strripos("abc", "Z") === false ? "miss" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "start|last-o|neg-a|last-prefix|miss");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reverse_position_builtins_handle_empty_needles_and_catchable_offset_errors() {
    let execution = run_source(
        r#"<?php
echo strrpos("abc", "") === 3 ? "empty-end" : "bad";
echo "|";
echo strrpos("abc", "", -1) === 2 ? "empty-neg" : "bad";
echo "|";
try {
    strrpos("haystack", "h", -9);
} catch (ValueError $e) {
    echo $e->getMessage();
}
echo "|";
try {
    strripos("t", "t", PHP_INT_MAX + 1);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "empty-end|empty-neg|strrpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)|strripos(): Argument #3 ($offset) must be of type int, float given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reverse_position_builtins_are_callable_and_reflectable() {
    let execution = run_source(
        r#"<?php
$last = "strrpos";
$ilast = "strripos";
echo function_exists($last) ? "yes" : "no";
echo "|";
echo is_callable($ilast) ? "callable" : "missing";
echo "|";
echo $last("abcabc", "b") === 4 ? "found" : "missing";
echo "|";
echo $ilast("ABCabc", "a") === 3 ? "ifound" : "missing";
echo "|";
$function = new ReflectionFunction("StrRIPOS");
$params = $function->getParameters();
$returnNames = array();
foreach ($function->getReturnType()->getTypes() as $part) {
    $returnNames[] = $part->getName();
}
echo $function->getName(), ":";
echo implode("|", $returnNames), ":";
echo $params[0]->getName(), "/", $params[1]->getName(), "/", $params[2]->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|found|ifound|strripos:int|false:haystack/needle/offset"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn reverse_position_builtins_reject_current_unsupported_forms() {
    let array_haystack = run_source("<?php\nstrrpos(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_haystack.phase, Phase::Runtime);
    assert_eq!(
        array_haystack.message,
        "unsupported call strrpos(): haystack argument arrays are not implemented in the current subset"
    );

    let array_needle = run_source("<?php\nstrripos('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_needle.phase, Phase::Runtime);
    assert_eq!(
        array_needle.message,
        "unsupported call strripos(): needle argument arrays are not implemented in the current subset"
    );

    let too_few = run_source("<?php\nstrrpos('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(
        too_few.message,
        "arity mismatch for strrpos(): expected 2 to 3 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_reverse_position_metadata_but_rejects_native_lowering() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strrpos") ? "1" : "0";
echo is_callable("strripos") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let direct = emit_ir_source("<?php\necho strrpos('abc', 'b');\n").unwrap_err();
    assert_eq!(direct.phase, Phase::Codegen);
    assert!(
        direct
            .message
            .contains("function-call lowering rejects function calls"),
        "{}",
        direct.message
    );
}
