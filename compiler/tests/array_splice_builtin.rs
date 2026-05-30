use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_ARRAY_REJECTION: &str = "LLVM array lowering rejects arrays, array literals, array indexing, array assignment, foreach array iteration, array offset unset, and array builtin function calls until native array storage layout, key normalization, copy-on-write, references, callbacks, and exact native error behavior exist; phpc run handles current array behavior";

#[test]
fn array_splice_mutates_direct_variable_arrays_and_reindexes_integer_keys() {
    let execution = run_source(
        r#"<?php
$items = array("a" => 1, 2 => 2, "b" => 3, 4 => 4, 5 => 5);
$removed = array_splice($items, 1, 2, array("x" => 9, 7 => 8));
echo implode(",", array_keys($removed)), "|", implode(",", $removed), "\n";
echo implode(",", array_keys($items)), "|", implode(",", $items), "\n";
$tail = array_splice($items, -2, null, "z");
echo implode(",", array_keys($tail)), "|", implode(",", $tail), "\n";
echo implode(",", array_keys($items)), "|", implode(",", $items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "0,b|2,3\na,0,1,2,3|1,9,8,4,5\n0,1|4,5\na,0,1,2|1,9,8,z"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_handles_negative_offsets_lengths_and_null_replacement() {
    let execution = run_source(
        r#"<?php
$items = array("a", "b", "c", "d");
$removed = array_splice($items, 2, -1);
echo implode(",", $removed), "|", implode(",", $items), "\n";
$assoc = array("x" => 1, "y" => 2, "z" => 3);
$removed = array_splice($assoc, 1, 1, null);
echo implode(",", array_keys($removed)), "|", implode(",", $removed), "\n";
echo implode(",", array_keys($assoc)), "|", implode(",", $assoc);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "c|a,b,d\ny|2\nx,z|1,3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_is_available_through_string_valued_direct_calls() {
    let execution = run_source(
        r#"<?php
$call = "array_splice";
$items = array("a", "b", "c");
echo function_exists($call) ? "yes" : "no";
echo "|";
echo is_callable($call) ? "callable" : "missing";
echo "|";
$removed = $call($items, 1, 1, "x");
echo implode(",", $removed), "|", implode(",", $items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|callable|b|a,x,c");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_mutates_direct_array_offsets_and_preserves_replacement_references() {
    let execution = run_source(
        r#"<?php
$input = array(array(1, 2));
$input[] =& $input[0];
$removed = array_splice($input[0], 1, 1);
echo implode(",", $removed), "|", implode(",", $input[1]), "\n";
var_dump($input);
$three = 3;
$four = 4;
$items = array(0, 1, 2);
$replacement = array(&$three, "fourkey" => &$four);
array_splice($items, -1, 1, $replacement);
$three = 30;
$four = 40;
echo implode(",", $items);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2|1\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  &array(1) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "  }\n",
            "  [1]=>\n",
            "  &array(1) {\n",
            "    [0]=>\n",
            "    int(1)\n",
            "  }\n",
            "}\n",
            "0,1,30,40",
        )
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_callback_reference_arguments_mutate_and_warn_for_values() {
    let execution = run_source(
        r#"<?php
$items = array("a", "b", "c");
$args = array(&$items, 1, 1, "x");
$removed = call_user_func_array("array_splice", $args);
echo implode(",", $removed), "|", implode(",", $items), "\n";
function warn_ref($errno, $errstr) {
    echo str_contains($errstr, "must be passed by reference") ? "warning" : "other";
    echo "|";
    return true;
}
set_error_handler("warn_ref", E_WARNING);
$items = array("a", "b");
$removed = call_user_func("array_splice", $items, 0, 1);
echo implode(",", $removed), "|", implode(",", $items);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b|a,x,c\nwarning|a|a,b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_reflection_reports_empty_array_replacement_default() {
    let execution = run_source(
        r#"<?php
$function = new ReflectionFunction("array_splice");
$replacement = $function->getParameters()[3];
echo $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters(), "\n";
echo $replacement->getName(), "|", ($replacement->isOptional() ? "1" : "0"), "|", ($replacement->isDefaultValueAvailable() ? "1" : "0"), "|", $replacement->getType()->getName(), "\n";
$default = $replacement->getDefaultValue();
echo gettype($default), "|", count($default), "\n";
var_dump($default);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2/4\nreplacement|1|1|mixed\narray|0\narray(0) {\n}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_splice_rejects_forms_outside_current_subset() {
    let non_variable = run_source("<?php\narray_splice(['tail'], 0, 0, 'head');\n").unwrap_err();
    assert_eq!(non_variable.phase, Phase::Runtime);
    assert_eq!(non_variable.line, 2);
    assert_eq!(non_variable.column, 1);
    assert_eq!(
        non_variable.message,
        "unsupported call array_splice(): first argument must be a direct variable array path in the current subset"
    );

    let non_array = run_source("<?php\n$value = 1;\narray_splice($value, 0);\n").unwrap_err();
    assert_eq!(non_array.phase, Phase::Runtime);
    assert_eq!(non_array.line, 3);
    assert_eq!(non_array.column, 1);
    assert_eq!(
        non_array.message,
        "unsupported call array_splice(): argument must be array, got int"
    );

    let non_int_offset =
        run_source("<?php\n$items = array('a');\narray_splice($items, '0');\n").unwrap_err();
    assert_eq!(non_int_offset.phase, Phase::Runtime);
    assert_eq!(non_int_offset.line, 3);
    assert_eq!(non_int_offset.column, 1);
    assert_eq!(
        non_int_offset.message,
        "unsupported call array_splice(): offset argument must be int in the current subset, got string"
    );

    let non_int_length =
        run_source("<?php\n$items = array('a');\narray_splice($items, 0, false);\n").unwrap_err();
    assert_eq!(non_int_length.phase, Phase::Runtime);
    assert_eq!(non_int_length.line, 3);
    assert_eq!(non_int_length.column, 1);
    assert_eq!(
        non_int_length.message,
        "unsupported call array_splice(): length argument must be int or null in the current subset, got bool"
    );
}

#[test]
fn emit_ir_folds_array_splice_function_exists_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("array_splice") ? "1" : "0";
echo is_callable("array_splice") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\n$items = 1;\narray_splice($items, 0);\n").unwrap_err();
    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, LLVM_ARRAY_REJECTION);
}
