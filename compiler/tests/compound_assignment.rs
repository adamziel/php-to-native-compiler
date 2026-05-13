use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

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
fn compound_assignment_expression_rejects_object_property_targets() {
    let error = run_source(
        "<?php\nclass Box { public $value; }\n$box = new Box();\necho ($box->value += 2);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(error.line, 4);
    assert_eq!(error.column, 20);
    assert_eq!(
        error.message,
        "unsupported compound assignment target: only direct static variables and direct array offsets are implemented; append offsets, nested offsets, and object properties are not implemented"
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
fn emit_ir_rejects_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "compound assignment is supported by phpc run for direct static variables and direct array offsets but not LLVM IR emission yet"
    );
}

#[test]
fn emit_ir_rejects_compound_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\necho ($value += 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "compound assignment expressions are supported by phpc run for direct static variables and direct array offsets but not LLVM IR emission yet"
    );
}

#[test]
fn emit_ir_rejects_array_offset_compound_assignment_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items = 1;\n$items['count'] += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "compound assignment is supported by phpc run for direct static variables and direct array offsets but not LLVM IR emission yet"
    );
}
