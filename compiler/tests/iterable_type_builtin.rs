use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

#[test]
fn is_iterable_matches_current_array_and_iterator_object_subset() {
    let execution = run_source(
        r#"<?php
class Box {}
class CustomIterator implements Iterator {
    #[ReturnTypeWillChange]
    public function current() { return null; }
    #[ReturnTypeWillChange]
    public function key() { return null; }
    #[ReturnTypeWillChange]
    public function next() { return null; }
    #[ReturnTypeWillChange]
    public function rewind() { return null; }
    #[ReturnTypeWillChange]
    public function valid() { return false; }
}
class Aggregate implements IteratorAggregate {
    #[ReturnTypeWillChange]
    public function getIterator() { return null; }
}

$box = new Box();
$iterator = new CustomIterator();
$aggregate = new Aggregate();
$values = [null, false, true, 0, 3.5, "", [], [1], $box, $iterator, $aggregate];
foreach ($values as $value) {
    echo is_iterable($value) ? "1" : "0";
}
echo "\n";
$call = "is_iterable";
echo $call([]) ? "1" : "0", $call("x") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "00000011011\n10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn iterable_internal_interface_method_shape_is_enforced_for_concrete_classes() {
    let iterator_error = run_source(
        r#"<?php
class Marker implements Iterator {}
"#,
    )
    .unwrap_err();

    assert_eq!(iterator_error.phase, Phase::Runtime);
    assert_eq!(iterator_error.line, 2);
    assert_eq!(iterator_error.column, 1);
    assert_eq!(
        iterator_error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface methods Iterator::current(), Iterator::key(), Iterator::next(), Iterator::rewind(), Iterator::valid()"
    );

    let aggregate_error = run_source(
        r#"<?php
class Marker implements IteratorAggregate {}
"#,
    )
    .unwrap_err();

    assert_eq!(aggregate_error.phase, Phase::Runtime);
    assert_eq!(aggregate_error.line, 2);
    assert_eq!(aggregate_error.column, 1);
    assert_eq!(
        aggregate_error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface method IteratorAggregate::getIterator()"
    );

    let static_error = run_source(
        r#"<?php
class Marker implements Iterator {
    public static function current() {}
    public function key() {}
    public function next() {}
    public function rewind() {}
    public function valid() {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(static_error.phase, Phase::Runtime);
    assert_eq!(static_error.line, 2);
    assert_eq!(static_error.column, 1);
    assert_eq!(
        static_error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface method Iterator::current() as non static method; found static Marker::current()"
    );

    let required_param_error = run_source(
        r#"<?php
class Marker implements IteratorAggregate {
    public function getIterator($mode) {}
}
"#,
    )
    .unwrap_err();

    assert_eq!(required_param_error.phase, Phase::Runtime);
    assert_eq!(required_param_error.line, 2);
    assert_eq!(required_param_error.column, 1);
    assert_eq!(
        required_param_error.message,
        "unsupported class inheritance for Marker: method Marker::getIterator() cannot require parameters for internal interface method IteratorAggregate::getIterator()"
    );

    let inherited_error = run_source(
        r#"<?php
abstract class Base implements Iterator {}
class Child extends Base {}
"#,
    )
    .unwrap_err();

    assert_eq!(inherited_error.phase, Phase::Runtime);
    assert_eq!(inherited_error.line, 3);
    assert_eq!(inherited_error.column, 1);
    assert_eq!(
        inherited_error.message,
        "unsupported class inheritance for Child: concrete class Child must implement internal interface methods Iterator::current(), Iterator::key(), Iterator::next(), Iterator::rewind(), Iterator::valid()"
    );
}

#[test]
fn direct_traversable_implementation_is_a_stable_runtime_boundary() {
    let error = run_source(
        r#"<?php
class Marker implements Traversable {}
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Marker: concrete class Marker cannot directly implement internal interface Traversable; implement Iterator or IteratorAggregate in the current subset"
    );
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
class CustomIterator implements Iterator {
    #[ReturnTypeWillChange]
    public function current() { return null; }
    #[ReturnTypeWillChange]
    public function key() { return null; }
    #[ReturnTypeWillChange]
    public function next() { return null; }
    #[ReturnTypeWillChange]
    public function rewind() { return null; }
    #[ReturnTypeWillChange]
    public function valid() { return false; }
}
echo is_iterable(new CustomIterator()) ? 1 : 0;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
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
