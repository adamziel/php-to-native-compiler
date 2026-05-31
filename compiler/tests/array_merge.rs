use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn assert_uncaught_type_error(source: &str, message: &str, line: usize) {
    let execution = run_source(source).unwrap();
    assert_eq!(execution.exit_code, 255);
    assert!(execution.stdout.contains(message), "{}", execution.stdout);
    assert!(
        execution
            .stdout
            .contains(&format!("thrown in Command line code on line {line}")),
        "{}",
        execution.stdout
    );
}

#[test]
fn array_merge_reindexes_integer_keys_and_overwrites_string_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$merged = array_merge($left, $right);
echo count($merged), "\n";
echo $merged["name"], "|", $merged[0], "|", $merged[1], "|", $merged["02"], "|", $merged[2], "|", $merged[3], "|", $merged[4], "|", $merged["extra"], "\n";
$merged[] = "after";
echo $merged[5], "\n";
echo $left["name"], "|", $left[5], "|", $left[2], "|", $left["02"], "|", $left[6], "\n";
echo $right["name"], "|", $right[7], "|", $right["02"], "|", $right[8], "|", $right["extra"], "\n";

$call = "array_merge";
$again = $call($left, $right);
echo $again["name"], "|", $again[0], "|", $again["02"], "|", $again["extra"], "\n";

$zero = array_merge();
print_r($zero);
echo count($zero), "\n";

$single = array_merge($left);
print_r($single);
echo count($single), "\n";
$single[] = "single after";
echo $single[3], "\n";

$third = [];
$third["name"] = "Cy";
$third[10] = "ten";
$third["extra"] = "third extra";
$third[] = "third next";

$variadic = array_merge($left, $right, $third);
print_r($variadic);
echo count($variadic), "\n";
echo $variadic["name"], "|", $variadic[0], "|", $variadic[1], "|", $variadic["02"], "|", $variadic[2], "|", $variadic[3], "|", $variadic[4], "|", $variadic["extra"], "|", $variadic[5], "|", $variadic[6], "\n";

$again_three = $call($left, $right, $third);
echo $again_three["name"], "|", $again_three[5], "|", $again_three[6], "|", $again_three["extra"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "8\nBea|five|two|zero two right|left next|seven|right next|extra\nafter\nAda|five|two|zero two|left next\nBea|seven|zero two right|right next|extra\nBea|five|zero two right|extra\nArray\n(\n)\n0\nArray\n(\n    [name] => Ada\n    [0] => five\n    [1] => two\n    [02] => zero two\n    [2] => left next\n)\n5\nsingle after\nArray\n(\n    [name] => Cy\n    [0] => five\n    [1] => two\n    [02] => zero two right\n    [2] => left next\n    [3] => seven\n    [4] => right next\n    [extra] => third extra\n    [5] => ten\n    [6] => third next\n)\n10\nCy|five|two|zero two right|left next|seven|right next|third extra|ten|third next\nCy|ten|third next|third extra"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_merge_recursive_recurses_string_key_collisions_and_appends_integer_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left["meta"] = ["lang" => "php", "tags" => ["core"]];
$left[5] = "five";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right["meta"] = ["lang" => "rust", "tags" => ["native"], "extra" => "yes"];
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";

$third = [];
$third["name"] = ["Cy"];
$third["meta"] = ["tags" => ["compiler"], "extra" => ["third"]];
$third[10] = "ten";

$result = array_merge_recursive($left, $right, $third);
echo count($result), "\n";
echo $result["name"][0], "|", $result["name"][1], "|", $result["name"][2], "\n";
echo $result["meta"]["lang"][0], "|", $result["meta"]["lang"][1], "\n";
echo $result["meta"]["tags"][0], "|", $result["meta"]["tags"][1], "|", $result["meta"]["tags"][2], "\n";
echo $result["meta"]["extra"][0], "|", $result["meta"]["extra"][1], "\n";
echo $result[0], "|", $result[1], "|", $result["02"][0], "|", $result["02"][1], "|", $result[2], "|", $result[3], "|", $result[4], "\n";

$call = "array_merge_recursive";
$again = $call($left, $right);
echo $again["name"][0], "|", $again["name"][1], "|", $again["meta"]["tags"][1], "\n";

$zero = array_merge_recursive();
echo count($zero), "\n";

$single = array_merge_recursive($left);
echo count($single), "\n";
$single[] = "single after";
echo $single[2], "\n";
echo $left["name"], "|", $left["meta"]["lang"], "|", $left[5], "|", $left["02"], "|", $left[6], "\n";
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "8
Ada|Bea|Cy
php|rust
core|native|compiler
yes|third
five|left next|zero two|zero two right|seven|right next|ten
Ada|Bea|native
0
5
single after
Ada|php|five|zero two|left next
"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_merge_recursive_type_errors_use_php_argument_messages() {
    let source = r#"<?php
try {
    array_merge_recursive(42, []);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    array_merge_recursive([], false);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_merge_recursive(): Argument #1 must be of type array, int given\narray_merge_recursive(): Argument #2 must be of type array, false given\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_merge_recursive_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_merge_recursive([1], [2]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}

#[test]
fn array_merge_requires_array_first_argument() {
    assert_uncaught_type_error(
        "<?php\n$right = [];\necho array_merge(42, $right);\n",
        "Fatal error: Uncaught TypeError: array_merge(): Argument #1 must be of type array, int given",
        3,
    );
}

#[test]
fn array_merge_requires_array_second_argument() {
    assert_uncaught_type_error(
        "<?php\n$left = [];\necho array_merge($left, 42);\n",
        "Fatal error: Uncaught TypeError: array_merge(): Argument #2 must be of type array, int given",
        3,
    );
}

#[test]
fn array_merge_requires_array_variadic_arguments() {
    assert_uncaught_type_error(
        "<?php\n$left = [];\n$right = [];\necho array_merge($left, $right, 42);\n",
        "Fatal error: Uncaught TypeError: array_merge(): Argument #3 must be of type array, int given",
        4,
    );
}

#[test]
fn array_merge_preserves_reference_backed_value_slots() {
    let source = r#"<?php
$value = "foo";
$left = [&$value];
$right = ["name" => "bar"];
$merged = array_merge($left, $right);
var_dump($merged);
$value = "changed";
var_dump($merged);
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "array(2) {\n  [0]=>\n  &string(3) \"foo\"\n  [\"name\"]=>\n  string(3) \"bar\"\n}\narray(2) {\n  [0]=>\n  &string(7) \"changed\"\n  [\"name\"]=>\n  string(3) \"bar\"\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_merge_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_merge([1], [2]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
