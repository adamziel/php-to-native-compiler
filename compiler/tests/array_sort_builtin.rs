use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_sort_family_mutates_direct_variable_arrays_with_php_flags() {
    let execution = run_source(
        r#"<?php
$items = array("l" => "lemon", "o2" => "orange2", "O" => "Orange", "b" => "banana", "o20" => "orange20", "O3" => "Orange3");
var_dump(sort($items, SORT_NATURAL | SORT_FLAG_CASE));
echo implode(",", $items), "\n";
$keys = array("o20" => 1, "O3" => 2, "o2" => 3, "b" => 4);
ksort($keys, SORT_NATURAL | SORT_FLAG_CASE);
echo implode(",", array_keys($keys)), "\n";
$reverse = array(3, 1, 2);
rsort($reverse);
echo implode(",", $reverse), "\n";
$assoc = array("c" => 3, "a" => 1, "b" => 2);
arsort($assoc, SORT_NUMERIC);
echo implode(",", array_keys($assoc)), "|", implode(",", $assoc), "\n";
$reverseTie = array("o" => "orange", "O" => "Orange", "o2" => "orange2");
rsort($reverseTie, SORT_STRING | SORT_FLAG_CASE);
echo implode(",", $reverseTie), "\n";
$assocTie = array("o" => "orange", "O" => "Orange", "o2" => "orange2");
arsort($assocTie, SORT_STRING | SORT_FLAG_CASE);
echo implode(",", array_keys($assocTie)), "|", implode(",", $assocTie), "\n";
$keyTie = array("o" => 1, "O" => 2, "o2" => 3);
krsort($keyTie, SORT_STRING | SORT_FLAG_CASE);
echo implode(",", array_keys($keyTie)), "\n";
$natural = array("img12.png", "img10.png", "img2.png", "img1.png");
natsort($natural);
echo implode(",", array_keys($natural)), "|", implode(",", $natural);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbanana,lemon,Orange,orange2,Orange3,orange20\nb,o2,O3,o20\n3,2,1\nc,b,a|3,2,1\norange2,orange,Orange\no2,o,O|orange2,orange,Orange\no2,o,O\n3,2,1,0|img1.png,img2.png,img10.png,img12.png"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sort_family_is_available_through_string_valued_direct_calls() {
    let execution = run_source(
        r#"<?php
$call = "krsort";
$items = array("a" => 1, "c" => 3, "b" => 2);
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
var_dump($call($items, SORT_STRING));
echo implode(",", array_keys($items));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|bool(true)\nc,b,a");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sort_residuals_handle_nested_arrays_objects_shuffle_and_spaceship() {
    let execution = run_source(
        r#"<?php
class SortBox {
    public $class_value;
    function __construct($value) { $this->class_value = $value; }
    function __toString() { return (string)$this->class_value; }
}

$objects = array(new SortBox(3), new SortBox(1), new SortBox(2));
sort($objects);
foreach ($objects as $object) { echo $object->class_value; }
echo "|";

$natural = array(new SortBox("b10"), new SortBox("a"), new SortBox("b2"));
natcasesort($natural);
foreach ($natural as $key => $object) { echo $key, ":", $object->class_value, ";"; }
echo "|";

$nested = array(array(2), array(), array(1, 0), array(1));
usort($nested, function ($left, $right) {
    if ($left == $right) {
        return 0;
    }
    return $left > $right ? 1 : -1;
});
foreach ($nested as $row) {
    echo count($row), ":", (count($row) ? $row[0] : "E"), ";";
}
echo "|";

$items = array("a" => 1, "b" => 2, "c" => 3);
var_dump(shuffle($items));
echo implode(",", array_keys($items)), ":", implode(",", $items), "|";
echo (array(1) <=> array(2)), ",";
echo ("600" <=> "aaa");
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "123|1:a;2:b2;0:b10;|0:E;1:1;1:2;2:1;|bool(true)\n0,1,2:3,2,1|-1,-1"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_sort_family_rejects_forms_outside_current_subset() {
    let non_variable = run_source("<?php\nsort(array('tail'));\n").unwrap_err();
    assert_eq!(non_variable.phase, Phase::Runtime);
    assert_eq!(non_variable.line, 2);
    assert_eq!(non_variable.column, 6);
    assert_eq!(
        non_variable.message,
        "unsupported call sort(): only direct variable and direct object-property array arguments are implemented"
    );

    let unsupported_flag =
        run_source("<?php\n$items = array(3, 1, 2);\nsort($items, SORT_LOCALE_STRING);\n")
            .unwrap_err();
    assert_eq!(unsupported_flag.phase, Phase::Runtime);
    assert_eq!(unsupported_flag.line, 3);
    assert_eq!(unsupported_flag.column, 1);
    assert!(
        unsupported_flag
            .message
            .contains("sort flag parameter 5 is not supported"),
        "{}",
        unsupported_flag.message
    );
}

#[test]
fn emit_ir_folds_array_sort_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("sort") ? "1" : "0";
echo is_callable("sort") ? "1" : "0";
echo function_exists("natsort") ? "1" : "0";
echo is_callable("natsort") ? "1" : "0";
echo defined("SORT_NATURAL") ? "1" : "0";
echo defined("SORT_FLAG_CASE") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 6, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
    assert!(!ir.contains("defined"), "{ir}");

    let error = emit_ir_source("<?php\n$items = 1;\nsort($items);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
