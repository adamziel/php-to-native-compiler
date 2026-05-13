use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn long_array_literals_execute_as_short_array_aliases() {
    let execution = run_source(
        r#"<?php
$items = array(
    "first",
    2 => "two",
    "2" => "two updated",
    "02" => "zero two",
    "name" => "Ada",
    1 + 2 => "three",
);
$upper = ARRAY("a", "b");
echo count($items), "\n";
echo $items[0], "|", $items[2], "|", $items["02"], "|", $items["name"], "|", $items[3], "\n";
echo $upper[1], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5\nfirst|two updated|zero two|Ada|three\nb\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unsupported_array_item_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$values = [1, 2];
$items = array(...$values);
"#,
            3,
            16,
            "unsupported array spread: spread elements are not implemented",
        ),
        (
            r#"<?php
$value = "Ada";
$items = array(&$value);
"#,
            3,
            16,
            "unsupported array reference element: references are not implemented",
        ),
        (
            r#"<?php
$values = [1, 2];
$items = [...$values];
"#,
            3,
            11,
            "unsupported array spread: spread elements are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_unset_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = [[1]];
unset($items[0][0]);
"#,
            3,
            16,
        ),
        (
            r#"<?php
$items = [];
UNSET($items[]);
"#,
            3,
            13,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported unset: only direct variables like unset($name) and direct array offset removal like unset($array[$key]) are implemented; property, append, and nested unset forms are not implemented"
        );
    }
}

#[test]
fn object_property_unset_has_stable_parse_boundary() {
    let error = parse_error(
        r#"<?php
class Box {
    public $name;
}
$box = new Box();
unset($box->name);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled"
    );
}

#[test]
fn emit_ir_rejects_object_property_unset_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\nclass Box { public $name; }\n$box = new Box();\nunset($box->name);\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported unset: object property unset is not implemented; property uninitialization, magic methods, and typed property semantics are not modeled"
    );
}

#[test]
fn unsupported_exception_syntax_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\nthrow new Exception('boom');\n",
            2,
            1,
            "unsupported throw: exception objects and stack unwinding are not implemented",
        ),
        (
            "<?php\n$value = throw new Exception('boom');\n",
            2,
            10,
            "unsupported throw: exception objects and stack unwinding are not implemented",
        ),
        (
            "<?php\ntry {\n    echo 'work';\n} catch (Exception $e) {\n    echo 'caught';\n} finally {\n    echo 'done';\n}\n",
            2,
            1,
            "unsupported try/catch/finally: exception handling and stack unwinding are not implemented",
        ),
        (
            "<?php\nCATCH (Exception $e) {\n    echo 'caught';\n}\n",
            2,
            1,
            "unsupported try/catch/finally: exception handling and stack unwinding are not implemented",
        ),
        (
            "<?php\nFINALLY {\n    echo 'done';\n}\n",
            2,
            1,
            "unsupported try/catch/finally: exception handling and stack unwinding are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn emit_ir_rejects_exception_syntax_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\nthrow new Exception('boom');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported throw: exception objects and stack unwinding are not implemented"
    );
}

#[test]
fn unsupported_match_expression_has_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$value = match ($status) {\n    200 => 'ok',\n    default => 'other',\n};\n",
            2,
            10,
        ),
        (
            "<?php\nMATCH ($status) {\n    200 => 'ok',\n    default => 'other',\n};\n",
            2,
            1,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported match expression: expression-form branching is not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_match_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source(
        "<?php\n$value = match ($status) {\n    default => 'other',\n};\n",
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported match expression: expression-form branching is not implemented"
    );
}

#[test]
fn unsupported_ternary_expressions_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$condition = true;\n$result = $condition ? 'yes' : 'no';\n",
            3,
            22,
        ),
        (
            "<?php\n$value = '';\n$result = $value ?: 'fallback';\n",
            3,
            18,
        ),
        ("<?php\necho $ok ? 'yes' : 'no';\n", 2, 10),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported ternary expression: expression-form branching is not implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_ternary_expression_at_parse_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$result = $condition ? 'yes' : 'no';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported ternary expression: expression-form branching is not implemented"
    );
}

#[test]
fn unsupported_chained_null_coalescing_has_stable_parse_error() {
    let error = parse_error("<?php\n$first = null;\n$result = $first ?? $second ?? 'fallback';\n");
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 29);
    assert_eq!(
        error.message,
        "unsupported null coalescing expression: null-aware expression-form branching is not implemented"
    );
}

#[test]
fn unsupported_null_coalescing_assignment_targets_have_stable_parse_errors() {
    let cases = [("<?php\n$items[] ??= 'fallback';\n", 2, 10)];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported null coalescing assignment: only direct variable, direct array-offset, and direct object-property targets are implemented"
        );
    }
}

#[test]
fn emit_ir_rejects_null_coalescing_expression_at_codegen_boundary() {
    let error =
        php_compiler::emit_ir_source("<?php\n$result = $value ?? 'fallback';\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "null coalescing expressions are supported by phpc run for the current direct variable/array-offset/object-property subset but not LLVM IR emission yet"
    );
}

#[test]
fn unsupported_expression_position_assignments_have_stable_parse_errors() {
    let cases = [
        (
            "<?php\n$value = 1;\necho ($value = 2);\n",
            3,
            14,
            "unsupported assignment expression: assignment expressions are not implemented; use statement-level assignment in the current subset",
        ),
        (
            "<?php\n$value = null;\necho ($value ??= 'fallback');\n",
            3,
            14,
            "unsupported assignment expression: assignment expressions are not implemented; use statement-level assignment in the current subset",
        ),
        (
            "<?php\n$items = [];\necho ($items['key'] = 'value');\n",
            3,
            21,
            "unsupported assignment expression: assignment expressions are not implemented; use statement-level assignment in the current subset",
        ),
        (
            "<?php\n$items = [];\necho ($items['key'] ??= 'value');\n",
            3,
            21,
            "unsupported assignment expression: assignment expressions are not implemented; use statement-level assignment in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn emit_ir_rejects_assignment_expression_at_parse_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$result = ($value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Parse);
    assert_eq!(
        error.message,
        "unsupported assignment expression: assignment expressions are not implemented; use statement-level assignment in the current subset"
    );
}

#[test]
fn unsupported_compound_assignments_have_stable_parse_errors() {
    let cases = [
        ("<?php\n$items = [];\n$items['key'] += 2;\n", 3, 1),
        (
            "<?php\nclass Box { public $value; }\n$box = new Box();\n$box->value += 2;\n",
            4,
            1,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported compound assignment target: only direct static variables are implemented; array offsets and object properties are not implemented"
        );
    }
}

#[test]
fn compound_assignment_expressions_have_stable_parse_errors() {
    let error = parse_error("<?php\n$value = 1;\necho ($value += 2);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 14);
    assert_eq!(
        error.message,
        "unsupported compound assignment expression: compound assignments are only implemented as direct-variable statements in the current subset"
    );
}

#[test]
fn emit_ir_rejects_compound_assignment_at_codegen_boundary() {
    let error = php_compiler::emit_ir_source("<?php\n$value += 2;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "compound assignment is supported by phpc run for direct static variables but not LLVM IR emission yet"
    );
}

#[test]
fn unsupported_foreach_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = [1];
FOREACH ($items as &$item) {
    echo $item;
}
"#,
            3,
            20,
            "unsupported foreach: by-reference iteration is not implemented; only by-value iteration is supported",
        ),
        (
            r#"<?php
$items = [[1]];
foreach ($items as [$item]) {
    echo $item;
}
"#,
            3,
            20,
            "unsupported foreach: destructuring loop targets are not implemented",
        ),
        (
            r#"<?php
$items = [[1]];
foreach ($items as $key => [$item]) {
    echo $item;
}
"#,
            3,
            28,
            "unsupported foreach: destructuring loop targets are not implemented",
        ),
        (
            r#"<?php
$items = [1];
echo foreach ($items as $item);
"#,
            3,
            6,
            "unsupported foreach: foreach is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_for_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
for ($i = 0, $j = 0; $i < 3; $i = $i + 1) {
    echo $i;
}
"#,
            2,
            12,
            "unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot",
        ),
        (
            r#"<?php
for ($i = 0; $i < 3; $i = $i + 1, $j = $j + 1) {
    echo $i;
}
"#,
            2,
            33,
            "unsupported for: comma-separated initializer, condition, or increment expression lists are not implemented; use at most one assignment or expression per header slot",
        ),
        (
            r#"<?php
echo for ($i = 0; $i < 3; $i = $i + 1);
"#,
            2,
            6,
            "unsupported for: for loops are only supported as statements in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn do_while_expression_form_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
echo do {
    echo "tick";
} while (false);
"#,
            2,
            6,
        ),
        (
            r#"<?php
echo DO echo "tick"; WHILE (false);
"#,
            2,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported do-while: do-while loops are only supported as statements in the current subset"
        );
    }
}

#[test]
fn unsupported_switch_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$value = 2;
switch ($value):
    case 1:
        echo "one";
        break;
    default:
        echo "other";
endswitch;
"#,
            3,
            16,
            "unsupported switch: alternate colon/endswitch syntax is not implemented; use brace switch blocks",
        ),
        (
            r#"<?php
switch (1) {
    case 1;
        echo "one";
}
"#,
            3,
            11,
            "expected ':' after switch case",
        ),
        (
            r#"<?php
echo switch ($value) {
    default:
        echo "fallback";
};
"#,
            2,
            6,
            "unsupported switch: switch is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_alternate_if_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
if ($value):
    echo "yes";
endif;
"#,
            2,
            12,
        ),
        (
            r#"<?php
$value = 2;
if ($value == 1) {
    echo "one";
} elseif ($value == 2):
    echo "two";
endif;
"#,
            5,
            23,
        ),
        (
            r#"<?php
if ($value) {
    echo "yes";
} ELSE:
    echo "no";
endif;
"#,
            4,
            7,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported if: alternate if/elseif/else colon/endif syntax is not implemented; use brace blocks or single-statement bodies"
        );
    }
}

#[test]
fn unsupported_break_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    break 2;
}
"#,
            3,
            5,
            "unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost loop is supported",
        ),
        (
            r#"<?php
echo break;
"#,
            2,
            6,
            "unsupported break: break is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_continue_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    continue 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported",
        ),
        (
            r#"<?php
while (true) {
    CONTINUE 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost loop is supported",
        ),
        (
            r#"<?php
echo continue;
"#,
            2,
            6,
            "unsupported continue: continue is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
