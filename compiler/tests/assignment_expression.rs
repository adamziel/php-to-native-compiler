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
            "<?php\n$items = [];\necho ($items['key'] = 'value');\n",
            3,
            21,
        ),
        (
            "<?php\nclass Box { public $value; }\n$box = new Box();\necho ($box->value = 2);\n",
            4,
            19,
        ),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported assignment expression target: only direct static variables are implemented; array offsets and object properties are not implemented"
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
        "assignment expressions are supported by phpc run for direct static variables but not LLVM IR emission yet"
    );
}
