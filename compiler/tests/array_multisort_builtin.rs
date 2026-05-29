use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_multisort_sorts_multiple_arrays_with_flags() {
    let execution = run_source(
        r#"<?php
$ar1 = array("row1" => 2, "row2" => 1, "row3" => 1);
$ar2 = array("row1" => 2, "row2" => "aa", "row3" => "1");
var_dump(array_multisort($ar1, SORT_ASC, SORT_REGULAR, $ar2, SORT_DESC, SORT_STRING));
echo implode(",", array_keys($ar1)), "|", implode(",", $ar1), "\n";
echo implode(",", array_keys($ar2)), "|", implode(",", $ar2), "\n";
var_dump(array_multisort($ar2));
echo implode(",", array_keys($ar2)), "|", implode(",", $ar2), "\n";

$names = array("Second", "First.1", "First.2", "First.3", "Twentieth", "Tenth", "Third");
$keys = array("2 a", "1 bb 1", "1 bB 2", "1 BB 3", "20 c", "10 d", "3 e");
array_multisort($keys, SORT_NATURAL | SORT_FLAG_CASE, $names);
echo implode(",", $names), "\n";
echo implode(",", $keys);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nrow2,row3,row1|1,1,2\nrow2,row3,row1|aa,1,2\nbool(true)\nrow3,row1,row2|1,2,aa\nFirst.1,First.2,First.3,Second,Third,Tenth,Twentieth\n1 bb 1,1 bB 2,1 BB 3,2 a,3 e,10 d,20 c"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_multisort_reindexes_numeric_keys_and_accepts_literals() {
    let execution = run_source(
        r#"<?php
$left = array("strkey" => 2, 1, 9 => 1);
$right = array(2, "aa", "1");
var_dump(array_multisort($left, SORT_ASC, SORT_REGULAR, $right, SORT_ASC, SORT_NUMERIC));
echo implode(",", array_keys($left)), "|", implode(",", $left), "\n";
echo implode(",", array_keys($right)), "|", implode(",", $right), "\n";

$first = array(4, 3, 3, 3);
$second = array(9, 3, 2, 2);
$third = array(9, 9, 2, 1);
var_dump(array_multisort($first, $second, $third));
echo implode(",", $first), "|", implode(",", $second), "|", implode(",", $third), "\n";

var_dump(array_multisort(array(1, 3, 2, 4)));
var_dump(array_multisort(array()));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\n0,1,strkey|1,1,2\n0,1,2|aa,1,2\nbool(true)\n3,3,3,4|2,2,3,9|1,2,9,9\nbool(true)\nbool(true)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_multisort_reports_php_shaped_errors() {
    let execution = run_source(
        r#"<?php
$items = array(1);
try {
    array_multisort($items, SORT_ASC, SORT_ASC);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    array_multisort($items, SORT_STRING, SORT_NUMERIC);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    array_multisort($items, 999);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    array_multisort($items, array(1, 2));
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "array_multisort(): Argument #3 must be an array or a sort flag that has not already been specified\narray_multisort(): Argument #3 must be an array or a sort flag that has not already been specified\narray_multisort(): Argument #2 must be a valid sort flag\nArray sizes are inconsistent\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_multisort_reflection_metadata_matches_php_shape() {
    let execution = run_source(
        r#"<?php
$function = new ReflectionFunction("array_multisort");
foreach ($function->getParameters() as $param) {
    echo $param->getName(), "|",
        ($param->isPassedByReference() ? "1" : "0"), "|",
        ($param->isVariadic() ? "1" : "0"), "|",
        ($param->hasType() ? "1" : "0"), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "array|1|0|0\nrest|1|1|0\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_array_multisort_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("array_multisort") ? "1" : "0";
echo is_callable("array_multisort") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$items = [2, 1];\narray_multisort($items);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
