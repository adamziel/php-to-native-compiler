use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_MUTATION_REJECTION: &str = "LLVM mutation lowering rejects compound assignment, null coalescing assignment, increment/decrement, assignment expressions, direct variable unset, object property unset, static property unset, and multiple-operand unset until native read-modify-write ordering, null-aware mutation, unset symbol-table effects, references/copy-on-write, and exact native error behavior exist; phpc run handles current mutation behavior";

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
fn nested_array_offset_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$items = [];
echo ($items["outer"]["inner"] = "Ada"), ":", $items["outer"]["inner"], "\n";

$created["a"]["b"] = "made";
echo $created["a"]["b"], "\n";

$nullable = null;
echo ($nullable["x"]["y"] = 7), ":", $nullable["x"]["y"], "\n";

$deep = [];
echo ($deep["a"]["b"]["c"] = "deep"), ":", $deep["a"]["b"]["c"], "\n";

$existing = ["outer" => ["keep" => "yes"]];
echo ($existing["outer"]["new"] = "new"), ":", $existing["outer"]["keep"], ":", $existing["outer"]["new"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Ada:Ada\nmade\n7:7\ndeep:deep\nnew:yes:new\n"
    );
}

#[test]
fn nested_array_offset_assignment_evaluates_keys_before_rhs() {
    let execution = run_source(
        r#"<?php
function first_key() {
    echo "first-key\n";
    return "outer";
}
function second_key() {
    echo "second-key\n";
    return "inner";
}
function next_value() {
    echo "rhs\n";
    return "value";
}
$items = [];
echo ($items[first_key()][second_key()] = next_value()), ":", $items["outer"]["inner"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "first-key\nsecond-key\nrhs\nvalue:value\n"
    );
}

#[test]
fn nested_array_offset_assignment_rejects_non_array_intermediate_values() {
    let error = run_source(
        r#"<?php
$items = ["outer" => 1];
echo ($items["outer"]["inner"] = "x");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on int"
    );
}

#[test]
fn direct_append_offset_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$items = [];
echo ($items[] = "first"), ":", $items[0], "\n";
echo ($items[] = 42), ":", $items[1], "\n";

echo ($created[] = "made"), ":", $created[0], "\n";
$nullable = null;
echo ($nullable[] = "null-made"), ":", $nullable[0], "\n";

function rhs_value() {
    echo "rhs\n";
    return "value";
}
echo ($items[] = rhs_value()), ":", $items[2], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "first:first\n42:42\nmade:made\nnull-made:null-made\nrhs\nvalue:value\n"
    );
}

#[test]
fn append_at_depth_assignment_expressions_return_assigned_values() {
    let execution = run_source(
        r#"<?php
$submenu = [];
echo ($submenu['themes.php'][] = 'widgets'), ":", $submenu['themes.php'][0], "\n";
echo ($submenu['themes.php'][] = 'customize'), ":", $submenu['themes.php'][1], "\n";

$created = [];
echo ($created['outer']['inner'][] = 'made'), ":", $created['outer']['inner'][0], "\n";

$nullable = ['slot' => null];
echo ($nullable['slot'][] = 'null-made'), ":", $nullable['slot'][0], "\n";

function path_key() {
    echo "key\n";
    return "path";
}
function rhs_value() {
    echo "rhs\n";
    return "value";
}
$items = [];
echo ($items[path_key()][] = rhs_value()), ":", $items["path"][0], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "widgets:widgets\ncustomize:customize\nmade:made\nnull-made:null-made\nkey\nrhs\nvalue:value\n"
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
fn append_offset_assignment_expression_rejects_non_array_targets() {
    let error = run_source(
        r#"<?php
function rhs_value() {
    echo "rhs\n";
    return "value";
}
$value = 1;
echo ($value[] = rhs_value());
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 7);
    assert_eq!(error.column, 7);
    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on int"
    );
}

#[test]
fn append_at_depth_assignment_expression_rejects_non_array_intermediate_values() {
    let error = run_source(
        r#"<?php
$items = ["outer" => 1];
echo ($items["outer"][] = "x");
"#,
    )
    .unwrap_err();

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
fn chained_assignment_expressions_assign_right_to_left() {
    let execution = run_source(
        r#"<?php
$left = $right = 10;
echo $left, ":", $right, "\n";

echo ($outer = $inner = 20), ":", $outer, ":", $inner, "\n";

$items = [];
echo ($copy = $items["name"] = "Ada"), ":", $copy, ":", $items["name"], "\n";

class Box {
    public $value;
}
$box = new Box();
echo ($same = $box->value = "stored"), ":", $same, ":", $box->value, "\n";

function rhs_value() {
    echo "rhs\n";
    return 42;
}
echo ($a = $b = rhs_value()), ":", $a, ":", $b, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "10:10\n20:20:20\nAda:Ada:Ada\nstored:stored:stored\nrhs\n42:42:42\n"
    );
}

#[test]
fn chained_assignment_allows_compound_and_null_coalescing_rhs_values() {
    let execution = run_source(
        r#"<?php
$value = 2;
$copy = ($value += 3);
echo $copy, ":", $value, "\n";

$items = ["count" => 4];
echo ($array_copy = ($items["count"] *= 2)), ":", $array_copy, ":", $items["count"], "\n";

class Box {
    public $count;
}
$box = new Box();
$box->count = 5;
echo ($property_copy = ($box->count -= 1)), ":", $property_copy, ":", $box->count, "\n";

echo ($fallback_copy = ($missing ??= "fallback")), ":", $fallback_copy, ":", $missing, "\n";

function should_not_run() {
    echo "rhs\n";
    return "new";
}
$kept = "old";
echo ($kept_copy = ($kept ??= should_not_run())), ":", $kept_copy, ":", $kept, "\n";

$slots = [];
echo ($slot_copy = ($slots["name"] ??= "Ada")), ":", $slot_copy, ":", $slots["name"], "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "5:5\n8:8:8\n4:4:4\nfallback:fallback:fallback\nold:old:old\nAda:Ada:Ada\n"
    );
}

#[test]
fn assignment_expressions_work_in_non_echo_value_contexts() {
    let execution = run_source(
        r#"<?php
function capture($left, $right) {
    echo "capture:", $left, ":", $right, "\n";
    return $left . "|" . $right;
}

$arg = "start";
echo capture(($arg = "call"), ($arg .= "-arg")), ":", $arg, "\n";

$array = [
    ($key = "name") => ($value = "Ada"),
    ($next = 2) => ($value = $value . "-Lovelace"),
];
echo "array:", $key, ":", $next, ":", $array["name"], ":", $array[2], ":", $value, "\n";

if (($condition = strlen(($text = "php"))) === 3) {
    echo "if:", $condition, ":", $text, "\n";
}

echo "coalesce:", strlen(($maybe ??= "seed")), ":", $maybe, "\n";

$loop = 0;
while (($loop += 1) < 3) {
    echo "while:", $loop, "\n";
}
echo "after-while:", $loop, "\n";

for ($i = 0; ($gate = $i < 2); $i = $i + 1) {
    echo "for:", $i, ":", $gate, "\n";
}

$items = [];
echo "builtin:",
    array_key_exists(($lookup = "slot"), ($items = ["slot" => "yes"])),
    ":", $lookup,
    ":", $items["slot"],
    ":", count(($copy = [1, 2, 3])),
    ":", count($copy),
    "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "capture:call:call-arg\ncall|call-arg:call-arg\narray:name:2:Ada:Ada-Lovelace:Ada-Lovelace\nif:3:php\ncoalesce:4:seed\nwhile:1\nwhile:2\nafter-while:3\nfor:0:1\nfor:1:1\nbuiltin:1:slot:yes:3:3\n"
    );
}

#[test]
fn chained_assignment_rejects_append_offset_targets() {
    let cases = [
        ("<?php\n$items = [];\n$value = $items[] = 1;\n", 3, 10),
        (
            "<?php\n$items = [];\necho ($items[] = $value = 1);\n",
            3,
            18,
        ),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported assignment expression: this chained assignment form is not implemented in the current subset"
        );
    }
}

#[test]
fn assignment_expression_rejects_complex_targets() {
    let cases = [
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
            "unsupported assignment expression target: only direct static variables, direct array offsets, direct append offsets, nested array offsets, append-at-depth targets, and direct object properties are implemented"
        );
    }
}

#[test]
fn append_offsets_remain_unsupported_as_reads() {
    let cases = [
        ("<?php\n$items = [];\necho $items[];\n", 3, 6),
        ("<?php\n$items = [];\necho ($target = $items[]);\n", 3, 17),
    ];

    for (source, line, column) in cases {
        let error = run_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Parse);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "cannot use [] for reading; append syntax is only supported in assignments"
        );
    }
}

#[test]
fn emit_ir_rejects_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$value = 1;\necho ($value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);

    let error = emit_ir_source("<?php\n$items = 1;\necho ($items['outer']['inner'] = 'value');\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);

    let error =
        emit_ir_source("<?php\n$items = 1;\necho ($items['outer'][] = 'value');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_chained_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$left = $right = 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 9);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_array_offset_assignment_expressions_until_native_lowering_exists() {
    let error =
        emit_ir_source("<?php\n$items = 1;\necho ($items['key'] = 'value');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_append_offset_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$items = 1;\necho ($items[] = 'value');\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}

#[test]
fn emit_ir_rejects_object_property_assignment_expressions_until_native_lowering_exists() {
    let error = emit_ir_source("<?php\n$box = 1;\necho ($box->value = 2);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 7);
    assert_eq!(error.message, LLVM_MUTATION_REJECTION);
}
