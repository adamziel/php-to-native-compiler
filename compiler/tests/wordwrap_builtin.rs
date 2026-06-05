use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn wordwrap_wraps_on_ascii_spaces_and_optional_long_word_cuts() {
    let execution = run_source(
        r#"<?php
echo wordwrap("The quick brown fox", 10, "|"), "\n";
echo wordwrap("abcdefghijk", 4, "|", false), "\n";
echo wordwrap("abcdefghijk", 4, "|", true), "\n";
echo wordwrap("abc  def", 4, "|"), "\n";
echo wordwrap("ab cd", 4, "|");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "The quick|brown fox\nabcdefghijk\nabcd|efgh|ijk\nabc |def\nab|cd"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn wordwrap_preserves_newlines_and_accepts_scalar_subset_arguments() {
    let execution = run_source(
        r#"<?php
echo bin2hex(wordwrap("ab\r\ncd ef", 4, "|")), "\n";
echo wordwrap(42042, 2, "|", true), "\n";
echo wordwrap("abc def", "3", "--", false);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "61620d0a63647c6566\n42|04|2\nabc--def");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn wordwrap_metadata_and_string_callable_are_available() {
    let execution = run_source(
        r#"<?php
$call = "wordwrap";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
echo $call("abc def", 3, "|");
echo "|";
$function = new ReflectionFunction("WordWrap");
$params = $function->getParameters();
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
echo "|", $params[0]->getName(), ":", $params[0]->getType()->getName();
echo "|", $params[1]->getName(), ":", $params[1]->getType()->getName();
echo "|", $params[2]->getName(), ":", $params[2]->getType()->getName();
echo "|", $params[3]->getName(), ":", $params[3]->getType()->getName();
echo "|", $function->getReturnType()->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|abc|def|wordwrap:1/4|string:string|width:int|break:string|cut_long_words:bool|string"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn wordwrap_rejects_unsupported_argument_shapes() {
    let empty_break = run_source(
        r#"<?php
try {
    wordwrap("abc", 2, "");
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        empty_break.stdout,
        "wordwrap(): Argument #3 ($break) cannot be empty"
    );

    let array_string = run_source("<?php\nwordwrap([], 2, '|');\n").unwrap_err();
    assert_eq!(array_string.phase, Phase::Runtime);
    assert_eq!(array_string.line, 2);
    assert_eq!(array_string.column, 1);
    assert_eq!(
        array_string.message,
        "unsupported call wordwrap(): string argument arrays are not implemented in the current subset"
    );

    let array_cut = run_source(
        r#"<?php
try {
    wordwrap('abc', 2, '|', []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        array_cut.stdout,
        "wordwrap(): Argument #4 ($cut_long_words) must be of type bool, array given"
    );
}

#[test]
fn emit_ir_folds_wordwrap_capability_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("wordwrap") ? "1" : "0";
echo is_callable("wordwrap") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho wordwrap('abc def', 3, '|');\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
