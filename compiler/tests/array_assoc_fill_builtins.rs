use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_fill_builds_integer_keyed_arrays_and_clones_values() {
    let source = r#"<?php
$filled = array_fill(-2, 4, "x");
print_r($filled);
$filled[] = "tail";
echo $filled[-2], "|", $filled[2], "\n";

$nested = array_fill(1, 2, ["name" => "Ada"]);
$nested[1]["name"] = "Grace";
echo $nested[1]["name"], "|", $nested[2]["name"], "\n";

$call = "array_fill";
$empty = $call(5.9, 0.2, true);
echo count($empty), "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [-2] => x\n    [-1] => x\n    [0] => x\n    [1] => x\n)\nx|tail\nGrace|Ada\n0\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_fill_reports_php_value_error_for_negative_count() {
    let source = r#"<?php
try {
    array_fill(0, -1, "x");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "ValueError:array_fill(): Argument #2 ($count) must be greater than or equal to 0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_fill_int_max_count_uses_php_overflow_fatal() {
    let source = r#"<?php
$intMax = 2147483647;
try {
    array_fill(0, $intMax + 1, 1);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
array_fill(0, $intMax, 1);
echo "unreachable";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_fill(): Argument #2 ($count) is too large\n\nFatal error: Possible integer overflow in memory allocation (2147483647 * 32 + 32) in Command line code on line 8"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn array_auto_keys_continue_from_negative_integer_keys() {
    let execution = run_source(
        r#"<?php
$a = [];
$a[-5] = "-5";
$a[] = "after -5";
print_r($a);

$b = [-2 => true, true, true];
$d = [];
$d[-2] = true;
$d[] = true;
$d[] = true;
var_dump($b === $d);

$e = [-2 => false];
array_pop($e);
$e[] = true;
$e[] = true;
$e[] = true;
var_dump($d == $e);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n    [-5] => -5\n    [-4] => after -5\n)\nbool(true)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_assoc_and_intersect_assoc_compare_scalar_string_values_with_keys() {
    let source = r#"<?php
$left = ["a" => "green", "b" => "brown", 0 => "red", 1 => "", "2" => "two"];
$right = ["a" => "green", 0 => "yellow", 1 => "red", 2 => "two"];
$third = [0 => "red", "b" => "brown", 2 => "two"];

print_r(array_diff_assoc($left, $right));
print_r(array_diff_assoc($left, $right, $third));
print_r(array_intersect_assoc($left, $right));
print_r(array_intersect_assoc($left, $right, $third));

$call = "array_intersect_assoc";
print_r($call([0 => 1, 1 => 2.0, "x" => false], [0 => "1", 1 => "2", "x" => ""]));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [b] => brown\n    [0] => red\n    [1] => \n)\nArray\n(\n    [1] => \n)\nArray\n(\n    [a] => green\n    [2] => two\n)\nArray\n(\n    [2] => two\n)\nArray\n(\n    [0] => 1\n    [1] => 2\n    [x] => \n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn print_r_indents_nested_arrays_with_php_separators() {
    let execution = run_source(
        r#"<?php
$items = ["outer" => ["child" => "ok"], "tail" => "done"];
print_r($items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Array\n(\n    [outer] => Array\n        (\n            [child] => ok\n        )\n\n    [tail] => done\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_assoc_builtins_reject_non_array_arguments() {
    let execution = run_source(
        "<?php\ntry { array_diff_assoc([1], 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff_assoc(): Argument #2 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_assoc_and_fill_arrays_until_native_call_lowering_exists() {
    for source in [
        "<?php\necho array_fill(0, 1, \"x\")[0];\n",
        "<?php\necho array_diff_assoc([1], [2])[0];\n",
        "<?php\necho array_intersect_assoc([1], [1])[0];\n",
    ] {
        let error = emit_ir_source(source).unwrap_err();
        assert_eq!(error.phase, Phase::Codegen);
        assert!(
            error.message.contains("function calls"),
            "{}",
            error.message
        );
    }
}
