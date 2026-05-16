use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_OBJECT_CLASS_REJECTION: &str = "LLVM object/class lowering rejects class declarations, inheritance metadata, object instantiation, constructor dispatch, public property reads/writes, instance method calls, and object metadata builtins until native object layout, handles, visibility, method dispatch, and exact native error behavior exist; phpc run handles current object/class behavior";

#[test]
fn is_iterable_matches_current_array_and_traversable_object_subset() {
    let execution = run_source(
        r#"<?php
class Box {}
class DirectTraversable implements Traversable {}
class CustomIterator implements Iterator {}
class Aggregate implements IteratorAggregate {}

$box = new Box();
$direct = new DirectTraversable();
$iterator = new CustomIterator();
$aggregate = new Aggregate();
$values = [null, false, true, 0, 3.5, "", [], [1], $box, $direct, $iterator, $aggregate];
foreach ($values as $value) {
    echo is_iterable($value) ? "1" : "0";
}
echo "\n";
$call = "is_iterable";
echo $call([]) ? "1" : "0", $call("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "000000110111\n10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_direct_scalar_null_is_iterable_calls_to_false() {
    let ir = emit_ir_source(
        r#"<?php
echo is_iterable(null) ? "1" : "0";
echo is_iterable(false) ? "1" : "0";
echo is_iterable(0) ? "1" : "0";
echo is_iterable(3.5) ? "1" : "0";
echo is_iterable("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"0\\00\"").count(), 5, "{ir}");
    assert!(!ir.contains("is_iterable"), "{ir}");
}

#[test]
fn emit_ir_rejects_array_is_iterable_until_native_array_lowering_exists() {
    let error = emit_ir_source("<?php\necho is_iterable([]) ? 1 : 0;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}

#[test]
fn emit_ir_rejects_object_is_iterable_until_native_object_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
class CustomIterator implements Iterator {}
echo is_iterable(new CustomIterator()) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_OBJECT_CLASS_REJECTION);
}

#[test]
fn emit_ir_rejects_dynamic_is_iterable_until_native_runtime_call_lowering_exists() {
    let error = emit_ir_source(
        r#"<?php
$call = "is_iterable";
echo $call([]) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
}
