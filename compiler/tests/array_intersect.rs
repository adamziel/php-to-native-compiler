use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn array_intersect_preserves_first_array_entries_with_present_scalar_values() {
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
$left["drop"] = "drop";
$left[] = "next";

$right = [];
$right[] = "";
$right[] = "0";
$right[] = "1";
$right[] = "10";
$right[] = "abc";
$right[] = "eight";
$right[] = "missing";

$intersected = array_intersect($left, $right);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["true"], "|", $intersected["one"], "|", $intersected["zero"], "|", $intersected["string-zero"], "|", $intersected["int-ten"], "|", $intersected["float-ten"], "|", $intersected["text"], "|", $intersected[8], "\n";
$intersected[] = "after";
echo $intersected[10], "\n";
print_r($left);
print_r($right);

$call = "array_intersect";
$again = $call($left, $right);
echo $again["true"], "|", $again["one"], "|", $again["zero"], "|", $again["string-zero"], "|", $again["int-ten"], "|", $again["float-ten"], "|", $again["text"], "|", $again[8], "\n";

$empty = array_intersect([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_intersect(["x" => "x"], ["x"]);
print_r($all);
echo count($all), "\n";

$none = array_intersect(["name" => "x"], ["y"]);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [null] => \n    [false] => \n    [empty] => \n    [true] => 1\n    [one] => 1\n    [zero] => 0\n    [string-zero] => 0\n    [int-ten] => 10\n    [float-ten] => 10\n    [text] => abc\n    [8] => eight\n)\n11\n1|1|0|0|10|10|abc|eight\nafter\nArray\n(\n    [null] => \n    [false] => \n    [empty] => \n    [true] => 1\n    [one] => 1\n    [zero] => 0\n    [string-zero] => 0\n    [int-ten] => 10\n    [float-ten] => 10\n    [string-ten-float] => 10.0\n    [text] => abc\n    [8] => eight\n    [drop] => drop\n    [9] => next\n)\nArray\n(\n    [0] => \n    [1] => 0\n    [2] => 1\n    [3] => 10\n    [4] => abc\n    [5] => eight\n    [6] => missing\n)\n1|1|0|0|10|10|abc|eight\nArray\n(\n)\n0\nArray\n(\n    [x] => x\n)\n1\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_compares_against_all_variadic_operands() {
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
$first[] = "eight";
$first[] = "next";
$first[] = "extra";

$second = [];
$second[] = "Ada";
$second[] = "10";
$second[] = "eight";
$second[] = "drop";
$second[] = "next";

$third = [];
$third[] = "Ada";
$third[] = "10";
$third[] = "eight";
$third[] = "next";

$intersected = array_intersect($base, $first, $second, $third);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["name"], "|", $intersected["ten"], "|", $intersected["float-ten"], "|", $intersected[8], "|", $intersected[9], "\n";
$intersected[] = "after";
echo $intersected[10], "\n";
print_r($base);

$call = "array_intersect";
$again = $call($base, $first, $second, $third);
echo $again["name"], "|", $again["ten"], "|", $again["float-ten"], "|", $again[8], "|", $again[9], "\n";

$none = array_intersect(["name" => "x"], $first, $second, $third);
print_r($none);
echo count($none);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [name] => Ada\n    [ten] => 10\n    [float-ten] => 10\n    [8] => eight\n    [9] => next\n)\n5\nAda|10|10|eight|next\nafter\nArray\n(\n    [name] => Ada\n    [1] => 1\n    [two] => two\n    [ten] => 10\n    [float-ten] => 10\n    [drop] => drop\n    [8] => eight\n    [keep] => keep\n    [9] => next\n)\nAda|10|10|eight|next\nArray\n(\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_requires_array_first_argument() {
    let execution = run_source(
        "<?php\n$right = [];\ntry { array_intersect(42, $right); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_intersect(): Argument #1 ($array) must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_requires_array_second_argument() {
    let execution = run_source(
        "<?php\n$left = [];\ntry { array_intersect($left, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_intersect(): Argument #2 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_requires_array_variadic_arguments() {
    let execution = run_source(
        "<?php\n$left = [];\n$right = [];\ntry { array_intersect($left, $right, 42); } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(
        execution.stdout,
        "array_intersect(): Argument #3 must be of type array, int given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_intersect_uses_php_string_comparison_for_arrays_objects_and_resources() {
    let source = r#"<?php
class Box {
    public function __toString() {
        return "box";
    }
}

$stream = fopen("php://memory", "w+");
$box = new Box();
$left = ["array" => [1], "object" => $box, "resource" => $stream, "drop" => "drop"];
$right = ["Array", "box", $stream];
print_r(array_intersect($left, $right));
print_r(array_intersect($left));
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
        "Array\n(\n    [array] => Array\n        (\n            [0] => 1\n        )\n\n    [object] => Box Object\n        (\n        )\n\n    [resource] => Resource id #"
    ));
    assert!(execution.stdout.contains("    [drop] => drop\n)\n"));
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_rejects_array_intersect_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_intersect([1], [1]);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls"),
        "{}",
        error.message
    );
}
