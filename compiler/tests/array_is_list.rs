use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_is_list_detects_zero_based_ordered_integer_keys() {
    let source = r#"<?php
$empty = [];
var_dump(array_is_list($empty));

$list = ["zero", "one"];
$list[] = "two";
var_dump(array_is_list($list));

$normalized = [];
$normalized["0"] = "zero";
$normalized["1"] = "one";
var_dump(array_is_list($normalized));

$out_of_order = [];
$out_of_order[1] = "one";
$out_of_order[0] = "zero";
var_dump(array_is_list($out_of_order));

$gap = [];
$gap[0] = "zero";
$gap[2] = "two";
var_dump(array_is_list($gap));

$string_key = [];
$string_key[0] = "zero";
$string_key["01"] = "one";
var_dump(array_is_list($string_key));

$negative = [];
$negative[-1] = "negative";
$negative[0] = "zero";
var_dump(array_is_list($negative));

$after_unset = [0 => "zero", 1 => "one", 2 => "two"];
unset($after_unset[1]);
var_dump(array_is_list($after_unset));

$reindexed = array_values($after_unset);
var_dump(array_is_list($reindexed));

$call = "array_is_list";
var_dump($call([0 => "a", 1 => "b"]));
var_dump($call([1 => "b", 0 => "a"]));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_is_list_requires_array_argument() {
    let error = runtime_error("<?php\necho array_is_list(42);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_is_list(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_array_is_list_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_is_list([1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
