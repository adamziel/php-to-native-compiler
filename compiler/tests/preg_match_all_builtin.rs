use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn preg_match_all_preserves_named_capture_keys_and_offsets() {
    let execution = run_source(
        r#"<?php
preg_match('/(?P<word>[a-z]+)-(?P<num>\d+)/', 'ab-12', $one, PREG_OFFSET_CAPTURE);
echo implode(",", array_keys($one)), "|", $one['word'][0], ":", $one['word'][1], "|", $one['num'][0], ":", $one['num'][1], "\n";
$count = preg_match_all('/(?P<word>[a-z]+)-(?P<num>\d+)/', 'ab-12 cd-34', $matches, PREG_PATTERN_ORDER | PREG_OFFSET_CAPTURE);
echo $count, "|", implode(",", array_keys($matches)), "\n";
echo $matches['word'][1][0], ":", $matches['word'][1][1], "|";
echo $matches[1][0][0], ":", $matches[1][0][1], "|";
echo $matches['num'][0][0], ":", $matches['num'][0][1], "\n";
preg_match_all('/(?P<a>a)(?P<b>b)?/', 'a ab', $set, PREG_SET_ORDER);
echo implode(",", array_keys($set[0])), "|", implode(",", array_keys($set[1])), "\n";
echo array_key_exists('b', $set[0]) ? "b" : "no-b";
echo "|", array_key_exists('a', $set[1]) ? "a" : "no-a";
echo "|", $set[1]['b'];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0,word,1,num,2|ab:0|12:3\n2|0,word,1,num,2\ncd:6|ab:0|12:3\n0,a,1|0,a,1,b,2\nno-b|a|b"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_preg_match_all_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("preg_match_all") ? "1" : "0";
echo is_callable("preg_match_all") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error =
        emit_ir_source("<?php\npreg_match_all('/(?P<word>[a-z]+)/', 'ab cd', $m);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
