use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn null_coalescing_handles_direct_variables_and_array_offsets() {
    let source = r#"<?php
$defined = "value";
$nullable = null;
$false = false;
$zero = 0;
$empty = "";
$items = [];
$items["present"] = "array-value";
$items["null"] = null;
$items["false"] = false;
$items["zero"] = 0;
$items["empty"] = "";
$items["2"] = "two";
$key = "present";

echo ($defined ?? $undefined_fallback), "\n";
echo ($missing ?? "missing-fallback"), "\n";
echo ($nullable ?? "null-fallback"), "\n";
if (($false ?? true) === false) {
    echo "false-kept\n";
}
if (($zero ?? 9) === 0) {
    echo "zero-kept\n";
}
if (($empty ?? "fallback") === "") {
    echo "empty-string-kept\n";
}
echo ($items[$key] ?? $undefined_offset_fallback), "\n";
echo ($items["missing"] ?? "missing-key-fallback"), "\n";
echo ($items["null"] ?? "null-key-fallback"), "\n";
if (($items["false"] ?? true) === false) {
    echo "array-false-kept\n";
}
if (($items["zero"] ?? 9) === 0) {
    echo "array-zero-kept\n";
}
if (($items["empty"] ?? "fallback") === "") {
    echo "array-empty-string-kept\n";
}
echo ($items[2] ?? "normalized-missing"), "\n";
echo ($undefined_items["any"] ?? "undefined-array-fallback"), "\n";
$number = 42;
echo ($number["any"] ?? "scalar-target-fallback");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "value\nmissing-fallback\nnull-fallback\nfalse-kept\nzero-kept\nempty-string-kept\narray-value\nmissing-key-fallback\nnull-key-fallback\narray-false-kept\narray-zero-kept\narray-empty-string-kept\ntwo\nundefined-array-fallback\nscalar-target-fallback"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_handles_direct_object_properties() {
    let source = r#"<?php
class Box {
    public $value;
    public $nullable;
    public $flag;
    public $zero;
    public $empty;
}

$box = new Box();
$box->value = "object-value";
$box->flag = false;
$box->zero = 0;
$box->empty = "";

echo ($box->value ?? $undefined_object_fallback), "\n";
echo ($box->nullable ?? "null-property-fallback"), "\n";
echo ($box->missing ?? "missing-property-fallback"), "\n";
if (($box->flag ?? true) === false) {
    echo "object-false-kept\n";
}
if (($box->zero ?? 9) === 0) {
    echo "object-zero-kept\n";
}
if (($box->empty ?? "fallback") === "") {
    echo "object-empty-string-kept\n";
}
echo ($missing_box->value ?? "undefined-object-fallback"), "\n";
$number = 42;
echo ($number->value ?? "non-object-fallback");
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "object-value\nnull-property-fallback\nmissing-property-fallback\nobject-false-kept\nobject-zero-kept\nobject-empty-string-kept\nundefined-object-fallback\nnon-object-fallback"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_assignment_handles_direct_variables_lazily() {
    let source = r#"<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$missing ??= fallback("missing-called", "missing-value");
echo $missing, "\n";

$nullable = null;
$nullable ??= fallback("null-called", "null-value");
echo $nullable, "\n";

$kept = "kept-value";
$kept ??= fallback("kept-called", "replacement");
echo $kept, "\n";

$false = false;
$false ??= fallback("false-called", true);
if ($false === false) {
    echo "false-kept\n";
}

$zero = 0;
$zero ??= fallback("zero-called", 9);
if ($zero === 0) {
    echo "zero-kept\n";
}

$empty = "";
$empty ??= fallback("empty-called", "replacement");
if ($empty === "") {
    echo "empty-string-kept";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "missing-called\nmissing-value\nnull-called\nnull-value\nkept-value\nfalse-kept\nzero-kept\nempty-string-kept"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_assignment_handles_direct_array_offsets_lazily() {
    let source = r#"<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$items = [];
$items["missing"] ??= fallback("missing-called", "missing-value");
echo $items["missing"], "\n";

$items["null"] = null;
$items["null"] ??= fallback("null-called", "null-value");
echo $items["null"], "\n";

$items["kept"] = "kept-value";
$items["kept"] ??= fallback("kept-called", "replacement");
echo $items["kept"], "\n";

$items["false"] = false;
$items["false"] ??= fallback("false-called", true);
if ($items["false"] === false) {
    echo "false-kept\n";
}

$items["zero"] = 0;
$items["zero"] ??= fallback("zero-called", 9);
if ($items["zero"] === 0) {
    echo "zero-kept\n";
}

$items["empty"] = "";
$items["empty"] ??= fallback("empty-called", "replacement");
if ($items["empty"] === "") {
    echo "empty-string-kept\n";
}

$undefined_items["created"] ??= fallback("undefined-array-called", "created-value");
echo $undefined_items["created"], "\n";

$nullable_items = null;
$nullable_items["created"] ??= fallback("null-array-called", "null-created-value");
echo $nullable_items["created"], "\n";

$numeric_keys["2"] ??= fallback("numeric-key-called", "two");
echo $numeric_keys[2];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "missing-called\nmissing-value\nnull-called\nnull-value\nkept-value\nfalse-kept\nzero-kept\nempty-string-kept\nundefined-array-called\ncreated-value\nnull-array-called\nnull-created-value\nnumeric-key-called\ntwo"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_assignment_handles_direct_object_properties_lazily() {
    let source = r#"<?php
class Box {
    public $value;
    public $nullable;
    public $kept;
    public $flag;
    public $zero;
    public $empty;
}

function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

$box = new Box();
$box->value ??= fallback("value-called", "object-value");
echo $box->value, "\n";

$box->nullable = null;
$box->nullable ??= fallback("null-called", "null-value");
echo $box->nullable, "\n";

$box->kept = "kept-value";
$box->kept ??= fallback("kept-called", "replacement");
echo $box->kept, "\n";

$box->flag = false;
$box->flag ??= fallback("false-called", true);
if ($box->flag === false) {
    echo "false-kept\n";
}

$box->zero = 0;
$box->zero ??= fallback("zero-called", 9);
if ($box->zero === 0) {
    echo "zero-kept\n";
}

$box->empty = "";
$box->empty ??= fallback("empty-called", "replacement");
if ($box->empty === "") {
    echo "empty-string-kept";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "value-called\nobject-value\nnull-called\nnull-value\nkept-value\nfalse-kept\nzero-kept\nempty-string-kept"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_assignment_expressions_return_assigned_or_existing_values() {
    let source = r#"<?php
function fallback($label, $value) {
    echo $label, "\n";
    return $value;
}

echo ($missing ??= fallback("missing-called", "missing-value")), ":", $missing, "\n";

$nullable = null;
echo ($nullable ??= fallback("null-called", "null-value")), ":", $nullable, "\n";

$kept = "kept-value";
echo ($kept ??= fallback("kept-called", "replacement")), ":", $kept, "\n";

$false = false;
if (($false ??= fallback("false-called", true)) === false) {
    echo "false-result-kept\n";
}

$items = [];
echo ($items["missing"] ??= fallback("array-missing-called", "array-missing")), ":", $items["missing"], "\n";
$items["null"] = null;
echo ($items["null"] ??= fallback("array-null-called", "array-null")), ":", $items["null"], "\n";
$items["kept"] = "array-kept";
echo ($items["kept"] ??= fallback("array-kept-called", "replacement")), ":", $items["kept"], "\n";

class Box {
    public $value;
    public $kept;
    public $flag;
}

$box = new Box();
echo ($box->value ??= fallback("object-null-called", "object-value")), ":", $box->value, "\n";
$box->kept = "object-kept";
echo ($box->kept ??= fallback("object-kept-called", "replacement")), ":", $box->kept, "\n";
$box->flag = false;
if (($box->flag ??= fallback("object-false-called", true)) === false) {
    echo "object-false-result-kept";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "missing-called\nmissing-value:missing-value\nnull-called\nnull-value:null-value\nkept-value:kept-value\nfalse-result-kept\narray-missing-called\narray-missing:array-missing\narray-null-called\narray-null:array-null\narray-kept:array-kept\nobject-null-called\nobject-value:object-value\nobject-kept:object-kept\nobject-false-result-kept"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn null_coalescing_assignment_rejects_missing_object_properties_after_lazy_fallback() {
    let error = runtime_error(
        r#"<?php
class Box {
    public $value;
}
function fallback() {
    echo "missing-called\n";
    return "fallback";
}
$box = new Box();
$box->missing ??= fallback();
"#,
    );

    assert_eq!(error.line, 10);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn null_coalescing_assignment_rejects_undefined_object_targets_after_lazy_fallback() {
    let error = runtime_error(
        r#"<?php
function fallback() {
    echo "undefined-target-called\n";
    return "fallback";
}
$missing_box->value ??= fallback();
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined variable '$missing_box'");
}

#[test]
fn null_coalescing_assignment_rejects_non_object_property_targets_after_lazy_fallback() {
    let error = runtime_error(
        r#"<?php
function fallback() {
    echo "non-object-called\n";
    return "fallback";
}
$number = 42;
$number->value ??= fallback();
"#,
    );

    assert_eq!(error.line, 7);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid property access: cannot write property $value on int"
    );
}

#[test]
fn null_coalescing_assignment_rejects_non_array_offset_targets() {
    let error = runtime_error("<?php\n$value = 42;\n$value[\"key\"] ??= 'fallback';\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on int"
    );
}

#[test]
fn null_coalescing_rejects_external_non_public_object_properties() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
echo $box->secret ?? "fallback";
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
}

#[test]
fn null_coalescing_assignment_rejects_external_non_public_object_properties() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
$box->secret ??= "fallback";
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
}

#[test]
fn complex_null_coalescing_left_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\n$items = [[1]];\necho $items[0][0] ?? 'fallback';\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call ??: left operand must be a direct variable, direct array offset, direct object property, or supported static property in the current subset"
    );
}

#[test]
fn emit_ir_rejects_null_coalescing_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = $missing ?? 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior"
    );
}

#[test]
fn emit_ir_rejects_object_property_null_coalescing_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = $box->name ?? 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 10);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior"
    );
}

#[test]
fn emit_ir_rejects_static_property_null_coalescing_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = Counter::$count ?? 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 17);
    assert_eq!(
        error.message,
        "LLVM conditional lowering rejects unsupported conditional expressions or operands until native PHP truthiness, null-aware lookup, branch side-effect ordering, and exact native error behavior exist; phpc run handles current conditional expression behavior"
    );
}

#[test]
fn emit_ir_rejects_null_coalescing_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value ??= 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_array_offset_null_coalescing_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items[\"key\"] ??= 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_object_property_null_coalescing_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box->name ??= 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_static_property_null_coalescing_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nCounter::$count ??= 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 8);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_null_coalescing_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\necho ($value ??= 'fallback');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}
