use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

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
fn complex_null_coalescing_left_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\n$items = [[1]];\necho $items[0][0] ?? 'fallback';\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call ??: left operand must be a direct variable, direct array offset, or direct object property in the current subset"
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
        "null coalescing expressions are supported by phpc run for the current direct variable/array-offset/object-property subset but not LLVM IR emission yet"
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
        "null coalescing expressions are supported by phpc run for the current direct variable/array-offset/object-property subset but not LLVM IR emission yet"
    );
}

#[test]
fn emit_ir_rejects_null_coalescing_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value ??= 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "null coalescing assignment is supported by phpc run for direct variables but not LLVM IR emission yet"
    );
}
