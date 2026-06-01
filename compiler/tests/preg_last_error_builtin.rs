use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn preg_last_error_msg_reports_current_pcre_error_message() {
    let execution = run_source(
        r#"<?php
echo preg_last_error_msg(), "\n";
preg_match('/a/', 'a', $m, 0, 99);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
ini_set('pcre.backtrack_limit', '1');
preg_match('/(?:\D+|<\d+>)*[!?]/', 'foobar foobar foobar');
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
$text = json_decode('"\u2019"');
preg_match('/\b/u', $text, $m, 0, 1);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
$text = "VA\xff";
$text .= "LID";
preg_match('/\b/u', $text, $m, 0, 0);
echo preg_last_error(), "|", preg_last_error_msg(), "\n";
preg_match('/a/', 'a');
echo preg_last_error(), "|", preg_last_error_msg();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "No error\n1|Internal error\n2|Backtrack limit exhausted\n5|The offset did not correspond to the beginning of a valid UTF-8 code point\n4|Malformed UTF-8 characters, possibly incorrectly encoded\n0|No error"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn preg_last_error_functions_are_callable_and_expose_reflection_metadata() {
    let execution = run_source(
        r#"<?php
foreach (["preg_last_error", "preg_last_error_msg"] as $name) {
    echo function_exists($name) ? "fn" : "missing";
    echo is_callable($name) ? ":callable" : ":missing";
    $function = new ReflectionFunction($name);
    echo "|", $function->getName(), ":";
    echo $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters(), ":";
    echo $function->getReturnType()->getName(), ";";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn:callable|preg_last_error:0/0:int;fn:callable|preg_last_error_msg:0/0:string;"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_preg_last_error_msg_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_last_error_msg") ? "1" : "0";
echo is_callable("preg_last_error_msg") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\npreg_last_error_msg();\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
