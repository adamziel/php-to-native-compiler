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
fn compound_assignment_expression_rejects_complex_targets() {
    let cases = [
        (
            "<?php\n$items = ['count' => 1];\necho ($items['count'] += 1);\n",
            3,
            24,
        ),
        (
            "<?php\nclass Box { public $value; }\n$box = new Box();\necho ($box->value += 2);\n",
            4,
            20,
        ),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported compound assignment target: only direct static variables are implemented; array offsets and object properties are not implemented"
        );
    }
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
        "compound assignment is supported by phpc run for direct static variables but not LLVM IR emission yet"
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
        "compound assignment expressions are supported by phpc run for direct static variables but not LLVM IR emission yet"
    );
}
