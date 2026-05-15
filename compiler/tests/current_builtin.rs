use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn current_returns_first_ordered_array_value_or_false_for_empty_arrays() {
    let execution = run_source(
        r#"<?php
$items = array("name" => "Ada", 5 => "five", "2" => "two");
echo current($items), "|";
$items["name"] = "Grace";
echo current($items), "|";
var_dump(current(array()));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|Grace|bool(false)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn current_is_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "current";
$items = array("head", "tail");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
echo $call($items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|head");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn current_rejects_forms_outside_current_subset() {
    let error = run_source("<?php\necho current(42);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call current(): argument must be array, got int"
    );
}

#[test]
fn emit_ir_folds_current_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("current") ? "1" : "0";
echo is_callable("current") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho current([1]);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
