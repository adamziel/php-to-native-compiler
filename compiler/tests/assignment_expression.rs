use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn direct_variable_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
echo ($value = 10), ":", $value, "\n";
echo ($value = $value + 5), ":", $value, "\n";
echo (($text = "php") . "-native"), ":", $text, "\n";
echo ($array = ["name" => "Ada"])["name"], ":", $array["name"], "\n";

function next_value() {
    echo "rhs\n";
    return 42;
}
echo ($value = next_value()), ":", $value, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "10:10\n15:15\nphp-native:php\nAda:Ada\nrhs\n42:42\n"
    );
}

#[test]
fn assignment_expression_rhs_can_use_current_expression_subset() {
    let execution = run_source(
        r#"<?php
$fallback = "fallback";
echo ($value = $missing ?? $fallback), ":", $value, "\n";
$count = 0;
if (($count = $count + 1) === 1) {
    echo "if:", $count, "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fallback:fallback\nif:1\n");
}

#[test]
fn direct_array_offset_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$items = [];
echo ($items["name"] = "Ada"), ":", $items["name"], "\n";
echo ($items[2] = 99), ":", $items[2], "\n";

$missing["created"] = "statement";
echo ($dynamic["created"] = "expression"), ":", $dynamic["created"], "\n";
$nullable = null;
echo ($nullable["slot"] = "materialized"), ":", $nullable["slot"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Ada:Ada\n99:99\nexpression:expression\nmaterialized:materialized\n"
    );
}

#[test]
fn array_offset_assignment_expression_evaluates_key_before_rhs() {
    let execution = run_source(
        r#"<?php
function key_name() {
    echo "key\n";
    return "slot";
}
function next_value() {
    echo "rhs\n";
    return "value";
}
$items = [];
echo ($items[key_name()] = next_value()), ":", $items["slot"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "key\nrhs\nvalue:value\n");
}

#[test]
fn array_offset_assignment_expression_rejects_non_array_targets() {
    let error = run_source("<?php\n$value = 1;\necho ($value['key'] = 'x');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on int"
    );
}

#[test]
fn direct_object_property_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
class Box {
    public $name;
    public $count;
    public $result;
}

$box = new Box();
echo ($box->name = "Ada"), ":", $box->name, "\n";
echo ($box->count = 41 + 1), ":", $box->count, "\n";

function next_value() {
    echo "rhs\n";
    return "value";
}
echo ($box->result = next_value()), ":", $box->result, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada:Ada\n42:42\nrhs\nvalue:value\n");
}

#[test]
fn object_property_assignment_expression_rejects_non_object_targets_after_rhs() {
    let error = run_source(
        r#"<?php
function next_value() {
    echo "rhs\n";
    return "value";
}
$value = 1;
echo ($value->name = next_value());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "invalid property access: cannot write property $name on int"
    );
}

#[test]
fn chained_assignment_expressions_have_stable_parse_errors() {
    let cases = [
        ("<?php\n$value = $other = 1;\n", 2, 10),
        ("<?php\necho ($value = ($other = 1));\n", 2, 28),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported assignment expression: chained assignment expressions are not implemented"
        );
    }
}

#[test]
fn assignment_expression_rejects_complex_targets() {
    let cases = [
        (
            "<?php\n$items = [];\necho ($items['outer']['inner'] = 'value');\n",
            3,
            32,
        ),
        (
            "<?php\nclass Box { public $value; }\n$box = new Box();\necho (($box->value)->nested = 2);\n",
            4,
            29,
        ),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported assignment expression target: only direct static variables, direct array offsets, and direct object properties are implemented; append offsets and nested targets are not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\necho ($value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "assignment expressions are supported by phpc run for direct static variables, direct array offsets, and direct object properties but not LLVM IR emission yet"
    );
}

#[test]
fn emit_ir_rejects_array_offset_assignment_expressions_until_native_lowering_exists() {
    let error =
        emit_ir_source("<?php\n$items = 1;\necho ($items['key'] = 'value');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "assignment expressions are supported by phpc run for direct static variables, direct array offsets, and direct object properties but not LLVM IR emission yet"
    );
}

#[test]
fn emit_ir_rejects_object_property_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box = 1;\necho ($box->value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "assignment expressions are supported by phpc run for direct static variables, direct array offsets, and direct object properties but not LLVM IR emission yet"
    );
}
