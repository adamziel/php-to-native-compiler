use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_DYNAMIC_FUNCTION_CALL_REJECTION: &str = "LLVM dynamic function-call lowering rejects variable-call expressions such as $name(...) until native callable expression evaluation, runtime function lookup, stack frames, arity/type diagnostics, callback dispatch, and exact native callable errors exist; phpc run handles current string-valued dynamic function calls";
const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";
const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

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
fn countable_internal_interface_method_shape_is_enforced_for_concrete_classes() {
    let error = run_source(
        r#"<?php
class Marker implements Countable {}
count(new Marker());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface method Countable::count()"
    );

    let protected_error = run_source(
        r#"<?php
class Marker implements Countable {
    protected function count() { return 1; }
}
"#,
    )
    .unwrap_err();

    assert_eq!(protected_error.phase, Phase::Runtime);
    assert_eq!(protected_error.line, 2);
    assert_eq!(protected_error.column, 1);
    assert_eq!(
        protected_error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface method Countable::count()"
    );

    let static_error = run_source(
        r#"<?php
class Marker implements Countable {
    public static function count() { return 1; }
}
"#,
    )
    .unwrap_err();

    assert_eq!(static_error.phase, Phase::Runtime);
    assert_eq!(static_error.line, 2);
    assert_eq!(static_error.column, 1);
    assert_eq!(
        static_error.message,
        "unsupported class inheritance for Marker: concrete class Marker must implement internal interface method Countable::count() as non static method; found static Marker::count()"
    );

    let required_param_error = run_source(
        r#"<?php
class Marker implements Countable {
    public function count($mode) { return 1; }
}
"#,
    )
    .unwrap_err();

    assert_eq!(required_param_error.phase, Phase::Runtime);
    assert_eq!(required_param_error.line, 2);
    assert_eq!(required_param_error.column, 1);
    assert_eq!(
        required_param_error.message,
        "unsupported class inheritance for Marker: method Marker::count() cannot require parameters for internal interface method Countable::count()"
    );

    let inherited_error = run_source(
        r#"<?php
abstract class Base implements Countable {}
class Child extends Base {}
"#,
    )
    .unwrap_err();

    assert_eq!(inherited_error.phase, Phase::Runtime);
    assert_eq!(inherited_error.line, 3);
    assert_eq!(inherited_error.column, 1);
    assert_eq!(
        inherited_error.message,
        "unsupported class inheritance for Child: concrete class Child must implement internal interface method Countable::count()"
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
    let execution = run_source(
        r#"<?php
class Box {}
count(new Box());
"#,
    )
    .unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught TypeError: count(): Argument #1 ($value) must be of type Countable|array, Box given, called in Command line code:3\nStack trace:\n#0 Command line code(3): count()\n#1 {main}\n  thrown in Command line code on line 3"
    );
}

#[test]
fn count_and_sizeof_typeerrors_use_php_operand_names() {
    let execution = run_source(
        r#"<?php
class Box {}
$values = [null, "string", 123, true, false, (object) [], new Box()];
foreach ($values as $value) {
    try {
        count($value);
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}
try {
    sizeof(new Box());
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "count(): Argument #1 ($value) must be of type Countable|array, null given\ncount(): Argument #1 ($value) must be of type Countable|array, string given\ncount(): Argument #1 ($value) must be of type Countable|array, int given\ncount(): Argument #1 ($value) must be of type Countable|array, true given\ncount(): Argument #1 ($value) must be of type Countable|array, false given\ncount(): Argument #1 ($value) must be of type Countable|array, stdClass given\ncount(): Argument #1 ($value) must be of type Countable|array, Box given\nsizeof(): Argument #1 ($value) must be of type Countable|array, Box given"
    );
    assert_eq!(execution.exit_code, 0);
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
    assert_eq!(error.message, LLVM_DYNAMIC_FUNCTION_CALL_REJECTION);
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
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
