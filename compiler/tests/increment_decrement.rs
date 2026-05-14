use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_CONTROL_FLOW_REJECTION: &str = "LLVM control-flow lowering rejects if/else and elseif chains, while loops, for loops, do-while loops, switch statements, break, and continue until native PHP truthiness, branch layout, loop control flow, switch fallthrough, references/copy-on-write side effects, and exact native error behavior exist; phpc run handles current control-flow behavior";
const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn direct_variable_increment_decrement_updates_int_and_float_values() {
    let execution = run_source(
        r#"<?php
$int = 10;
++$int;
echo $int, "\n";
$int++;
echo $int, "\n";
--$int;
echo $int, "\n";
$int--;
echo $int, "\n";

$float = 1.5;
++$float;
echo $float, "\n";
$float--;
echo $float, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "11\n12\n11\n10\n2.5\n1.5\n");
}

#[test]
fn for_headers_accept_direct_variable_increment_decrement_actions() {
    let execution = run_source(
        r#"<?php
$sum = 0;
for ($i = 0; $i < 4; $i++) {
    $sum += $i;
}
echo $sum, "\n";

for (++$sum; $sum > 3; --$sum) {
    echo $sum, "\n";
}
echo "done:", $sum, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "6\n7\n6\n5\n4\ndone:3\n");
}

#[test]
fn expression_increment_decrement_returns_pre_or_post_values() {
    let execution = run_source(
        r#"<?php
$int = 10;
echo ++$int, ":", $int, "\n";
echo $int++, ":", $int, "\n";
echo --$int, ":", $int, "\n";
echo $int--, ":", $int, "\n";

$value = 2;
echo $value++ + 10, ":", $value, "\n";
echo ++$value + 10, ":", $value, "\n";
echo $value++ + $value++, ":", $value, "\n";

$side = 1;
++$side + 10;
echo "side:", $side, "\n";
$side++ + 10;
echo "side:", $side, "\n";

$float = 1.5;
echo $float++, ":", $float, "\n";
echo --$float, ":", $float, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11:11\n11:12\n11:11\n11:10\n12:3\n14:4\n9:6\nside:2\nside:3\n1.5:2.5\n1.5:1.5\n"
    );
}

#[test]
fn object_property_increment_decrement_updates_values_and_returns_pre_or_post_values() {
    let execution = run_source(
        r#"<?php
class Box {
    public $value;
    public $float;
    public $i;
    public $sum;
}

$box = new Box();
$box->value = 10;
++$box->value;
echo $box->value, "\n";
echo $box->value++, ":", $box->value, "\n";
echo --$box->value, ":", $box->value, "\n";
echo $box->value--, ":", $box->value, "\n";

$box->float = 1.5;
echo $box->float++, ":", $box->float, "\n";
echo --$box->float, ":", $box->float, "\n";

$box->value = 1;
++$box->value + 10;
echo "side:", $box->value, "\n";
$box->value++ + 10;
echo "side:", $box->value, "\n";

$box->sum = 0;
for ($box->i = 0; $box->i < 3; $box->i++) {
    $box->sum += $box->i;
}
echo $box->sum, ":", $box->i, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11\n11:12\n11:11\n11:10\n1.5:2.5\n1.5:1.5\nside:2\nside:3\n3:3\n"
    );
}

#[test]
fn array_offset_increment_decrement_updates_values_and_returns_pre_or_post_values() {
    let execution = run_source(
        r#"<?php
$items = ["value" => 10, "float" => 1.5, "i" => 0, "sum" => 0];
++$items["value"];
echo $items["value"], "\n";
echo $items["value"]++, ":", $items["value"], "\n";
echo --$items["value"], ":", $items["value"], "\n";
echo $items["value"]--, ":", $items["value"], "\n";

echo $items["float"]++, ":", $items["float"], "\n";
echo --$items["float"], ":", $items["float"], "\n";

$items["value"] = 1;
++$items["value"] + 10;
echo "side:", $items["value"], "\n";
$items["value"]++ + 10;
echo "side:", $items["value"], "\n";

for ($items["i"] = 0; $items["i"] < 3; $items["i"]++) {
    $items["sum"] += $items["i"];
}
echo $items["sum"], ":", $items["i"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "11\n11:12\n11:11\n11:10\n1.5:2.5\n1.5:1.5\nside:2\nside:3\n3:3\n"
    );
}

#[test]
fn undefined_increment_decrement_left_side_is_runtime_error() {
    let error = runtime_error("<?php\n$missing++;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn expression_undefined_increment_decrement_left_side_is_runtime_error() {
    let error = runtime_error("<?php\necho $missing++;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn for_header_undefined_increment_decrement_left_side_is_runtime_error() {
    let error = runtime_error("<?php\nfor ($missing++; false; ) {}\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn object_property_increment_decrement_reports_missing_properties() {
    let error =
        runtime_error("<?php\nclass Box { public $value; }\n$box = new Box();\n$box->missing++;\n");

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn object_property_increment_decrement_reports_non_public_properties() {
    let error = runtime_error(
        "<?php\nclass Box { private $secret; }\n$box = new Box();\n++$box->secret;\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
}

#[test]
fn object_property_increment_decrement_reports_non_object_targets() {
    let error = runtime_error("<?php\n$box = 1;\n$box->value++;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid property access: cannot read property $value from int"
    );
}

#[test]
fn array_offset_increment_decrement_reports_missing_keys() {
    let error = runtime_error("<?php\n$items = [];\n$items['missing']++;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined array key \"missing\"");
}

#[test]
fn array_offset_increment_decrement_reports_non_array_targets() {
    let error = runtime_error("<?php\n$items = 1;\n$items[0]++;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot read offset from int"
    );
}

#[test]
fn array_offset_increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error("<?php\n$items = ['value' => 'az'];\n$items['value']++;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables, array offsets, object properties, or static properties are implemented, got string"
    );
}

#[test]
fn object_property_increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error(
        "<?php\nclass Box { public $value; }\n$box = new Box();\n$box->value = 'az';\n$box->value++;\n",
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables, array offsets, object properties, or static properties are implemented, got string"
    );
}

#[test]
fn increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error("<?php\n$value = 'az';\n++$value;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables, array offsets, object properties, or static properties are implemented, got string"
    );
}

#[test]
fn expression_increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error("<?php\n$value = 'az';\necho ++$value;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables, array offsets, object properties, or static properties are implemented, got string"
    );
}

#[test]
fn for_header_increment_decrement_rejects_non_numeric_current_gap() {
    let error = runtime_error(
        "<?php\n$value = 'az';\n$go = true;\nfor (; $go; ++$value) {\n    $go = false;\n}\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 13);
    assert_eq!(
        error.message,
        "unsupported call increment/decrement: only int and float variables, array offsets, object properties, or static properties are implemented, got string"
    );
}

#[test]
fn emit_ir_rejects_increment_decrement_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\necho $value++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\n$value++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_object_property_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box->value++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_array_offset_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items[0]++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_static_property_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nCounter::$count++;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 8);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_for_header_increment_decrement_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nfor ($i = 0; $i < 3; $i++) {}\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_CONTROL_FLOW_REJECTION);
}
