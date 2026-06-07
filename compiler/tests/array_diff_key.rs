use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_diff_key_preserves_first_array_entries_with_missing_keys() {
    let source = r#"<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[-1] = "negative";
$left["drop"] = "drop";
$left[] = "next";

$right = [];
$right["name"] = "ignored";
$right["5"] = "ignored";
$right[2] = "ignored";
$right[-1] = "ignored";
$right["extra"] = "ignored";

$diffed = array_diff_key($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["02"], "|", $diffed["drop"], "|", $diffed[6], "\n";
$diffed[] = "after";
echo $diffed[7], "\n";
print_r($left);
print_r($right);

$call = "array_diff_key";
$again = $call($left, $right);
echo $again["02"], "|", $again["drop"], "|", $again[6], "\n";

$empty = array_diff_key([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff_key(["missing" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff_key(["name" => "x"], $right);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [02] => zero two\n    [drop] => drop\n    [6] => next\n)\n3\nzero two|drop|next\nafter\nArray\n(\n    [name] => Ada\n    [5] => five\n    [2] => two\n    [02] => zero two\n    [-1] => negative\n    [drop] => drop\n    [6] => next\n)\nArray\n(\n    [name] => ignored\n    [5] => ignored\n    [2] => ignored\n    [-1] => ignored\n    [extra] => ignored\n)\nzero two|drop|next\nArray\n(\n)\n0\nArray\n(\n    [missing] => x\n)\n1\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_key_compares_against_all_variadic_operands() {
    let source = r#"<?php
$base = [];
$base["name"] = "Ada";
$base[1] = "one";
$base["2"] = "two";
$base["02"] = "zero two";
$base[-1] = "negative";
$base["drop"] = "drop";
$base[8] = "eight";
$base["keep"] = "keep";
$base[] = "next";

$first = [];
$first["name"] = true;
$first["1"] = true;
$first[2] = true;
$first[-1] = true;

$second = [];
$second["02"] = true;
$second["drop"] = true;

$third = [];
$third[9] = true;

$diffed = array_diff_key($base, $first, $second, $third);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed[8], "|", $diffed["keep"], "\n";
$diffed[] = "after";
echo $diffed[9], "\n";
print_r($base);

$call = "array_diff_key";
$again = $call($base, $first, $second, $third);
echo $again[8], "|", $again["keep"], "\n";

$none = array_diff_key(["name" => "x"], $first, $second, $third);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [8] => eight\n    [keep] => keep\n)\n2\neight|keep\nafter\nArray\n(\n    [name] => Ada\n    [1] => one\n    [2] => two\n    [02] => zero two\n    [-1] => negative\n    [drop] => drop\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\neight|keep\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_key_requires_array_first_argument() {
    let execution = run_source(
        "<?php\n$right = [];\ntry { array_diff_key(42, $right); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff_key(): Argument #1 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_key_requires_array_second_argument() {
    let execution = run_source(
        "<?php\n$left = [];\ntry { array_diff_key($left, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff_key(): Argument #2 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_key_requires_array_variadic_arguments() {
    let execution = run_source(
        "<?php\n$left = [];\n$right = [];\ntry { array_diff_key($left, $right, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff_key(): Argument #3 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_diff_key_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_diff_key([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
