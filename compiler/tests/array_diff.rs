use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_diff_preserves_first_array_entries_with_absent_scalar_values() {
    let source = r#"<?php
$left = [];
$left["null"] = null;
$left["false"] = false;
$left["empty"] = "";
$left["true"] = true;
$left["one"] = 1;
$left["zero"] = 0;
$left["string-zero"] = "0";
$left["int-ten"] = 10;
$left["float-ten"] = 10.0;
$left["string-ten-float"] = "10.0";
$left["text"] = "abc";
$left[8] = "eight";
$left["keep"] = "keep";
$left[] = "next";

$right = [];
$right[] = "";
$right[] = "0";
$right[] = "1";
$right[] = "10";
$right[] = "abc";
$right[] = "missing";

$diffed = array_diff($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["string-ten-float"], "|", $diffed[8], "|", $diffed["keep"], "|", $diffed[9], "\n";
$diffed[] = "after";
echo $diffed[10], "\n";
print_r($left);
print_r($right);

$call = "array_diff";
$again = $call($left, $right);
echo $again["string-ten-float"], "|", $again[8], "|", $again["keep"], "|", $again[9], "\n";

$empty = array_diff([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff(["x" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff(["name" => "x"], ["x"]);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [string-ten-float] => 10.0\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\n4\n10.0|eight|keep|next\nafter\nArray\n(\n    [null] => \n    [false] => \n    [empty] => \n    [true] => 1\n    [one] => 1\n    [zero] => 0\n    [string-zero] => 0\n    [int-ten] => 10\n    [float-ten] => 10\n    [string-ten-float] => 10.0\n    [text] => abc\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\nArray\n(\n    [0] => \n    [1] => 0\n    [2] => 1\n    [3] => 10\n    [4] => abc\n    [5] => missing\n)\n10.0|eight|keep|next\nArray\n(\n)\n0\nArray\n(\n    [x] => x\n)\n1\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_requires_array_first_argument() {
    let execution = run_source(
        "<?php\n$right = [];\ntry { array_diff(42, $right); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff(): Argument #1 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_requires_array_second_argument() {
    let execution = run_source(
        "<?php\n$left = [];\ntry { array_diff($left, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff(): Argument #2 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_compares_against_all_variadic_operands() {
    let source = r#"<?php
$base = [];
$base["name"] = "Ada";
$base[1] = "1";
$base["two"] = "two";
$base["ten"] = 10;
$base["float-ten"] = 10.0;
$base["drop"] = "drop";
$base[8] = "eight";
$base["keep"] = "keep";
$base[] = "next";

$first = [];
$first[] = "Ada";
$first[] = "1";
$first[] = "10";
$first[] = "extra";

$second = [];
$second[] = "drop";
$second[] = "eight";

$third = [];
$third[] = "two";

$diffed = array_diff($base, $first, $second, $third);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["keep"], "|", $diffed[9], "\n";
$diffed[] = "after";
echo $diffed[10], "\n";
print_r($base);

$call = "array_diff";
$again = $call($base, $first, $second, $third);
echo $again["keep"], "|", $again[9], "\n";

$none = array_diff(["name" => "Ada"], $first, $second, $third);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [keep] => keep\n    [9] => next\n)\n2\nkeep|next\nafter\nArray\n(\n    [name] => Ada\n    [1] => 1\n    [two] => two\n    [ten] => 10\n    [float-ten] => 10\n    [drop] => drop\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\nkeep|next\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_requires_array_variadic_arguments() {
    let execution = run_source(
        "<?php\n$left = [];\n$right = [];\ntry { array_diff($left, $right, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_diff(): Argument #3 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_diff_uses_php_string_comparison_for_arrays_objects_and_resources() {
    let source = r#"<?php
class Box {
    public function __toString() {
        return "box";
    }
}

$stream = fopen("php://memory", "w+");
$box = new Box();
$left = ["array" => [1], "object" => $box, "resource" => $stream, "keep" => "keep"];
$right = ["Array", "box", $stream];
print_r(array_diff($left, $right));
print_r(array_diff($left));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution
            .stdout
            .matches("Warning: Array to string conversion")
            .count(),
        1
    );
    assert!(execution.stdout.contains(
        "Array\n(\n    [keep] => keep\n)\nArray\n(\n    [array] => Array\n        (\n            [0] => 1\n        )\n\n    [object] => Box Object\n        (\n        )\n\n    [resource] => Resource id #"
    ));
    assert!(execution.stdout.contains("    [keep] => keep\n)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_diff_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_diff([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
