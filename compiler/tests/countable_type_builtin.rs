use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

#[test]
fn is_countable_matches_current_array_and_countable_object_subset() {
    let execution = run_source(
        r#"<?php
class Box {}
class Basket implements Countable {
    #[ReturnTypeWillChange]
    public function count() { return 3; }
}

$box = new Box();
$basket = new Basket();
$values = [null, false, true, 0, 3.5, "", [], [1], $box, $basket];
foreach ($values as $value) {
    echo is_countable($value) ? "1" : "0";
}
echo "\n";
$call = "is_countable";
echo $call([]) ? "1" : "0", $call("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0000001101\n10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn count_dispatches_current_countable_object_protocol() {
    let execution = run_source(
        r#"<?php
class Basket implements Countable {
    #[ReturnTypeWillChange]
    public function count() { return 3; }
}

$basket = new Basket();
echo is_countable($basket) ? "yes" : "no";
echo "|";
echo count($basket);
echo "|";
echo count([1, 2]);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|3|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn count_rejects_countable_object_without_count_method() {
    let error = run_source(
        r#"<?php
class Marker implements Countable {}
count(new Marker());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call Countable::count(): Countable objects must declare count() in the current subset"
    );
}

#[test]
fn count_rejects_countable_object_non_int_result() {
    let error = run_source(
        r#"<?php
class Basket implements Countable {
    #[ReturnTypeWillChange]
    public function count() { return "3"; }
}

count(new Basket());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call Countable::count(): count method must return int in the current subset, got string"
    );
}

#[test]
fn count_rejects_non_countable_objects() {
    let error = run_source(
        r#"<?php
class Box {}
count(new Box());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call count(): only arrays and Countable objects are supported"
    );
}

#[test]
fn emit_ir_folds_direct_scalar_null_is_countable_calls_to_false() {
    let ir = emit_ir_source(
        r#"<?php
echo is_countable(null) ? "1" : "0";
echo is_countable(false) ? "1" : "0";
echo is_countable(0) ? "1" : "0";
echo is_countable(3.5) ? "1" : "0";
echo is_countable("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("is_countable"), "{ir}");
}

#[test]
fn emit_ir_rejects_array_is_countable_until_native_array_lowering_exists() {
    let error = emit_ir_source("<?php\necho is_countable([]) ? 1 : 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}

#[test]
fn emit_ir_rejects_dynamic_is_countable_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
$call = "is_countable";
echo $call([]) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}

#[test]
fn emit_ir_rejects_object_is_countable_until_native_object_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
class Basket implements Countable {
    #[ReturnTypeWillChange]
    public function count() { return 3; }
}

echo is_countable(new Basket()) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}
