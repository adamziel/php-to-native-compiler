use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment outside lowerable direct variables, null coalescing assignment, increment/decrement, non-direct assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn direct_variable_compound_assignments_update_scalar_values() {
    let execution = run_source(
        r#"<?php
$value = 10;
$value += 5;
echo $value, "\n";
$value -= 3;
echo $value, "\n";
$value *= "2";
echo $value, "\n";
$value /= 4;
echo $value, "\n";
$text = "php";
$text .= "-native";
echo $text, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "15\n12\n24\n6\nphp-native\n");
}

#[test]
fn for_headers_accept_direct_variable_compound_assignment() {
    let execution = run_source(
        r#"<?php
$sum = 0;
for ($i = 0; $i < 5; $i += 2) {
    $sum += $i;
}
echo $sum, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "6\n");
}

#[test]
fn compound_assignment_reads_left_side_before_writing() {
    let execution = run_source(
        r#"<?php
$value = "a";
function next_value() {
    echo "rhs\n";
    return "b";
}
$value .= next_value();
echo $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "rhs\nab\n");
}

#[test]
fn compound_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$value = 10;
echo ($value += 5), ":", $value, "\n";
echo (($value *= 2) + 1), ":", $value, "\n";

$text = "php";
echo ($text .= "-native"), ":", $text, "\n";

function next_value() {
    echo "rhs\n";
    return 3;
}
echo ($value -= next_value()), ":", $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "15:15\n31:30\nphp-native:php-native\nrhs\n27:27\n"
    );
}

#[test]
fn array_offset_compound_assignments_update_values_and_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$items = ['count' => 1, 2 => 10, 'text' => 'php'];
$items['count'] += 4;
$items[2] *= 3;
$items['text'] .= '-native';
echo $items['count'], ":", $items[2], ":", $items['text'], "\n";
echo ($items['count'] -= 2), ":", $items['count'], "\n";
$key = 'count';
echo ($items[$key] /= 3), ":", $items[$key], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "5:30:php-native\n3:3\n1:1\n");
}

#[test]
fn array_offset_compound_assignment_evaluates_key_once_before_rhs() {
    let execution = run_source(
        r#"<?php
$items = ['count' => 1];
function key_name() {
    echo "key\n";
    return 'count';
}
function next_value() {
    echo "rhs\n";
    return 2;
}
$items[key_name()] += next_value();
echo $items['count'], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "key\nrhs\n3\n");
}

#[test]
fn for_headers_accept_direct_array_offset_compound_assignment() {
    let execution = run_source(
        r#"<?php
$items = ['i' => 0, 'sum' => 0];
for ($items['i'] = 0; $items['i'] < 3; $items['i'] += 1) {
    $items['sum'] += $items['i'];
}
echo $items['sum'], ":", $items['i'], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3:3\n");
}

#[test]
fn object_property_compound_assignments_update_values_and_return_assigned_values() {
    let execution = run_source(
        r#"<?php
class Box {
    public $value;
    public $text;
    public $i;
    public $sum;
}

$box = new Box();
$box->value = 10;
$box->text = "php";
$box->value += 5;
$box->value *= "2";
$box->text .= "-native";
echo $box->value, ":", $box->text, "\n";
echo ($box->value -= 4), ":", $box->value, "\n";
echo ($box->value /= 2), ":", $box->value, "\n";

function next_value() {
    echo "rhs\n";
    return 3;
}
echo ($box->value += next_value()), ":", $box->value, "\n";

$box->sum = 0;
for ($box->i = 0; $box->i < 3; $box->i += 1) {
    $box->sum += $box->i;
}
echo $box->sum, ":", $box->i, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "30:php-native\n26:26\n13:13\nrhs\n16:16\n3:3\n"
    );
}

#[test]
fn object_property_array_offset_compound_assignments_update_nested_values() {
    let execution = run_source(
        r#"<?php
class Store {
    public $cache;
}

$store = new Store();
$store->cache = ['group' => ['key' => 4]];
$group = 'group';
$key = 'key';
$store->cache[$group][$key] += 6;
echo $store->cache['group']['key'], "\n";
echo ($store->cache[$group][$key] *= 2), ":", $store->cache['group']['key'], "\n";

function group_key() {
    echo "group\n";
    return 'group';
}
function item_key() {
    echo "key\n";
    return 'key';
}
function next_value() {
    echo "rhs\n";
    return 5;
}
$store->cache[group_key()][item_key()] += next_value();
echo $store->cache['group']['key'];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10\n20:20\ngroup\nkey\nrhs\n25");
}

#[test]
fn bitwise_and_shift_compound_assignments_update_supported_targets() {
    let execution = run_source(
        r#"<?php
$value = 14;
$value &= 11;
echo $value, "\n";
$value |= 1;
echo $value, "\n";
$value ^= 3;
echo $value, "\n";
$value <<= 2;
echo $value, "\n";
$value >>= 3;
echo $value, "\n";

$text = "ab";
$text &= "AB";
echo $text, "\n";
$text |= " !";
echo $text, "\n";

$items = ['bits' => 6, 'shift' => 2];
echo ($items['bits'] &= 3), ":", $items['bits'], "\n";
echo ($items['bits'] |= 8), ":", $items['bits'], "\n";
echo ($items['shift'] <<= 3), ":", $items['shift'], "\n";

class Box {
    public $mask;
}

$box = new Box();
$box->mask = 5;
echo ($box->mask ^= 3), ":", $box->mask, "\n";
echo ($box->mask >>= 1), ":", $box->mask, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "10\n11\n8\n32\n4\nAB\nac\n2:2\n10:10\n16:16\n6:6\n3:3\n"
    );
}

#[test]
fn for_headers_accept_direct_bitwise_and_shift_compound_assignment() {
    let execution = run_source(
        r#"<?php
for ($i = 1; $i < 16; $i <<= 1) {
    echo $i;
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1248");
}

#[test]
fn bitwise_compound_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$value = 14;
echo ($value &= 11), ":", $value, "\n";
echo (($value ^= 3) + 1), ":", $value, "\n";
echo ($value <<= 2), ":", $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "10:10\n10:9\n36:36\n");
}

#[test]
fn modulo_compound_assignments_update_supported_targets() {
    let execution = run_source(
        r#"<?php
$value = 29;
$value %= 5;
echo $value, "\n";
echo ($value %= 3), ":", $value, "\n";

$items = ['count' => 22];
echo ($items['count'] %= 6), ":", $items['count'], "\n";

class Box {
    public $value;
}

$box = new Box();
$box->value = 17;
echo ($box->value %= 5), ":", $box->value, "\n";

for ($i = 35; $i > 5; $i %= 8) {
    echo $i, ":";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "4\n1:1\n4:4\n2:2\n35:");
}

#[test]
fn array_offset_compound_assignment_reports_missing_keys() {
    let error = runtime_error("<?php\n$items = [];\n$items['missing'] += 1;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined array key \"missing\"");
}

#[test]
fn array_offset_compound_assignment_reports_non_array_targets() {
    let error = runtime_error("<?php\n$items = 1;\n$items['count'] += 1;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot read offset from int"
    );
}

#[test]
fn object_property_compound_assignment_reports_missing_properties() {
    let error = runtime_error(
        "<?php\nclass Box { public $value; }\n$box = new Box();\n$box->missing += 2;\n",
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn object_property_compound_assignment_reports_non_public_properties() {
    let execution = run_source(
        "<?php\nclass Box { private $secret; }\n$box = new Box();\n$box->secret += 2;\n",
    )
    .unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught Error: Cannot access private property Box::$secret in Command line code:4\nStack trace:\n#0 {main}\n  thrown in Command line code on line 4"
    );
}

#[test]
fn object_property_compound_assignment_reports_non_object_targets() {
    let error = runtime_error("<?php\n$box = 1;\n$box->value += 2;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid property access: cannot read property $value from int"
    );
}

#[test]
fn undefined_compound_assignment_left_side_is_runtime_error() {
    let error = runtime_error("<?php\n$missing += 1;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn compound_assignment_reuses_arithmetic_diagnostics() {
    let error = runtime_error("<?php\n$value = 'abc';\n$value += 1;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid arithmetic for +: string is not numeric"
    );
}

#[test]
fn undefined_compound_assignment_expression_left_side_is_runtime_error() {
    let error = runtime_error("<?php\necho ($missing += 1);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn compound_assignment_expression_reuses_arithmetic_diagnostics() {
    let error = runtime_error("<?php\n$value = 'abc';\necho ($value += 1);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "invalid arithmetic for +: string is not numeric"
    );
}

#[test]
fn bitwise_compound_assignment_reuses_bitwise_diagnostics() {
    let error = runtime_error("<?php\n$value = 'abc';\n$value &= 1;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid arithmetic for &: string is not numeric"
    );
}

#[test]
fn shift_compound_assignment_reuses_negative_shift_diagnostics() {
    let execution = run_source("<?php\n$value = 8;\n$value <<= -1;\n").unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught ArithmeticError: Bit shift by negative number in Command line code:3\nStack trace:\n#0 {main}\n  thrown in Command line code on line 3"
    );
}

#[test]
fn modulo_compound_assignment_reuses_modulo_diagnostics() {
    let execution = run_source("<?php\n$value = 10;\n$value %= 0;\n").unwrap();

    assert_eq!(execution.exit_code, 255);
    assert_eq!(
        execution.stdout,
        "Fatal error: Uncaught DivisionByZeroError: Modulo by zero in Command line code:3\nStack trace:\n#0 {main}\n  thrown in Command line code on line 3"
    );
}

#[test]
fn emit_ir_lowers_direct_variable_arithmetic_compound_assignments() {
    let ir = emit_ir_source(
        "<?php\n$value = 1;\n$delta = 2;\n$value += $delta;\necho $value;\n$float = 1.5;\n$factor = 2.0;\n$float *= $factor;\necho $float;\n",
    )
    .unwrap();

    assert!(
        ir.contains(" = add i64 1, 2") && ir.contains(" = fmul double 1.5, 2.0"),
        "direct variable arithmetic compound assignments should reuse LLVM binary lowering:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "lowerable direct variable compound assignments should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_direct_variable_bitwise_shift_and_modulo_compound_assignments() {
    let ir = emit_ir_source(
        "<?php\n$value = 13;\n$mask = 7;\n$value &= $mask;\n$shift = 1;\n$value <<= $shift;\n$mod = 4;\n$value %= $mod;\necho $value;\n",
    )
    .unwrap();

    assert!(
        ir.contains(" = and i64 13, 7") && ir.contains(" = srem i64"),
        "direct variable bitwise and modulo compound assignments should reuse integer binary lowering:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "lowerable direct variable compound assignments should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_lowers_direct_variable_compound_assignment_expressions() {
    let ir = emit_ir_source("<?php\n$value = 2;\n$delta = 3;\necho ($value += $delta), $value;\n")
        .unwrap();

    assert!(
        ir.contains(" = add i64 2, 3"),
        "compound assignment expressions should read the left value and evaluate the RHS through existing value semantics:\n{ir}"
    );
    assert!(
        ir.matches("phpc_native_diagnostic_result_report_stderr_echo_stdout_list_and_free")
            .count()
            >= 2,
        "compound assignment expression result and later variable read should both echo the stored result:\n{ir}"
    );
    assert!(
        !ir.contains(LLVM_MUTATION_REJECTION),
        "direct variable compound assignment expressions should not fall through the mutation blocker:\n{ir}"
    );
}

#[test]
fn emit_ir_rejects_undefined_direct_variable_compound_assignment_until_diagnostics_exist() {
    let error = emit_ir_source("<?php\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_array_offset_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items = 1;\n$items['count'] += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_object_property_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box->value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_static_property_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\nCounter::$count += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 8);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}
