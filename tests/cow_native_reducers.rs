use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

struct CowReducerCase {
    name: &'static str,
    oracle: &'static str,
    source: &'static str,
    expected_stdout: &'static str,
}

struct CowDiagnosticReducerCase {
    name: &'static str,
    oracle: &'static str,
    source: &'static str,
    expected_diagnostic: &'static str,
}

#[test]
fn recursive_reference_diagnostic_reducers_fail_before_codegen() {
    let cases = [
        CowDiagnosticReducerCase {
            name: "recursive_array_element_literal_self",
            oracle:
                "PHP oracle: assigning a literal that references the target array creates recursion",
            source: "<?php\n$array = [];\n$array[] = [&$array];",
            expected_diagnostic: "recursive array references are unsupported",
        },
        CowDiagnosticReducerCase {
            name: "same_array_literal_element_reference",
            oracle:
                "PHP oracle: array literal value reference to an assigned array element aliases same-array state",
            source: "<?php\n$array = [&$array[0]];",
            expected_diagnostic: "same-array element references are unsupported",
        },
        CowDiagnosticReducerCase {
            name: "same_array_element_literal_element_reference",
            oracle:
                "PHP oracle: assigning a literal with a same-array element reference aliases same-array state",
            source: "<?php\n$array = [];\n$array[] = [&$array[0]];",
            expected_diagnostic: "same-array element references are unsupported",
        },
    ];

    let root = temp_dir("ptn-native-recursive-reference-diagnostic-reducers");
    fs::create_dir_all(&root).unwrap();

    let expected = cases.len();
    let mut passed = 0usize;
    for case in cases {
        let input = root.join(format!("{}.php", case.name));
        let output = root.join(format!("{}-bin", case.name));
        fs::write(&input, case.source).unwrap();

        match compile_file(&input, &output, CompileOptions { emit_c: false }) {
            Ok(_) => {
                panic!(
                    "{} ({}) compiled successfully; expected diagnostic",
                    case.name, case.oracle
                );
            }
            Err(error) => {
                let actual = error.to_string();
                if actual.contains(case.expected_diagnostic) {
                    passed += 1;
                    continue;
                }

                assert!(
                    actual.contains(case.expected_diagnostic),
                    "{} ({}) diagnostic mismatch\nexpected substring: {}\nactual: {}",
                    case.name,
                    case.oracle,
                    case.expected_diagnostic,
                    actual
                );
            }
        }
    }

    assert_eq!(
        passed, expected,
        "recursive reference diagnostic reducer pass count changed"
    );
    let failed = expected - passed;
    assert_eq!(
        failed, 0,
        "recursive reference diagnostic reducer fail count changed"
    );
}

#[test]
fn compile_focused_cow_reducers_to_native_binary() {
    let cases = [
        CowReducerCase {
            name: "assignment_alias_key_write",
            oracle: "Zend/tests/assign_to_var_001.phpt",
            source: "<?php\n\
$a = [\"k\" => \"orig\", \"z\" => \"keep\"];\n\
$b = $a;\n\
$b[\"k\"] = \"copy\";\n\
echo $a[\"k\"], \":\", $b[\"k\"], \":\", $a[\"z\"], \"\\n\";",
            expected_stdout: "orig:copy:keep\n",
        },
        CowReducerCase {
            name: "array_append_shared_alias",
            oracle: "Zend/tests/array_append_COW.phpt",
            source: "<?php\n\
$a = [1, 2];\n\
$b = $a;\n\
$b[] = 3;\n\
echo count($a), \":\", count($b), \":\", $b[2], \"\\n\";",
            expected_stdout: "2:3:3\n",
        },
        CowReducerCase {
            name: "array_unset_shared_alias",
            oracle: "Zend/tests/unset/bug34518.phpt",
            source: "<?php\n\
$a = [\"x\" => 1, \"y\" => 2];\n\
$b = $a;\n\
unset($b[\"x\"]);\n\
echo count($a), \":\", count($b), \":\", $a[\"x\"], \":\", $b[\"y\"], \"\\n\";",
            expected_stdout: "2:1:1:2\n",
        },
        CowReducerCase {
            name: "array_dim_compound_shared_alias",
            oracle: "Zend/tests/assign_dim_op_same_var.phpt",
            source: "<?php\n\
$a = [\"n\" => 1];\n\
$b = $a;\n\
$b[\"n\"] += 4;\n\
echo $a[\"n\"], \":\", $b[\"n\"], \"\\n\";",
            expected_stdout: "1:5\n",
        },
        CowReducerCase {
            name: "same_array_append_element_reference",
            oracle: "PHP oracle: appending a reference to an existing slot aliases same-array entries",
            source: "<?php\n\
$array = [1];\n\
$array[] =& $array[0];\n\
$array[0] = 3;\n\
echo $array[0], \":\", $array[1], \"\\n\";",
            expected_stdout: "3:3\n",
        },
        CowReducerCase {
            name: "same_array_element_reference_assignment",
            oracle: "PHP oracle: assigning one slot by reference to another aliases same-array entries",
            source: "<?php\n\
$array = [1, 2];\n\
$array[0] =& $array[1];\n\
$array[1] = 5;\n\
echo $array[0], \":\", $array[1], \"\\n\";",
            expected_stdout: "5:5\n",
        },
        CowReducerCase {
            name: "call_result_array_dim_assignment_return_reference",
            oracle: "Zend/tests/dereference/dereference_006.phpt",
            source: "<?php\n\
function &slot(&$arg) { return $arg; }\n\
$items = [1];\n\
slot($items)[0] = 2;\n\
slot($items)[] = 3;\n\
echo count($items), \":\", $items[0], \":\", $items[1], \"\\n\";",
            expected_stdout: "2:2:3\n",
        },
        CowReducerCase {
            name: "dynamic_method_reference_source",
            oracle: "Zend/tests/dereference/dereference_008.phpt",
            source: "<?php\n\
class Box {\n\
    public $items = [1];\n\
    public function &items() { return $this->items; }\n\
}\n\
$box = new Box;\n\
$method = \"items\";\n\
$ref =& $box->$method();\n\
$ref[] = 2;\n\
$out = $box->$method();\n\
echo count($out), \":\", $out[0], \":\", $out[1], \"\\n\";",
            expected_stdout: "2:1:2\n",
        },
        CowReducerCase {
            name: "nested_extracted_child_write",
            oracle: "Zend/tests/bug35163.phpt",
            source: "<?php\n\
$a = [[\"x\" => 1], [\"x\" => 2]];\n\
$b = $a[0];\n\
$b[\"x\"] = 9;\n\
echo $a[0][\"x\"], \":\", $b[\"x\"], \":\", count($a), \"\\n\";",
            expected_stdout: "1:9:2\n",
        },
        CowReducerCase {
            name: "nested_copy_reinsert_child",
            oracle: "Zend/tests/bug38469.phpt",
            source: "<?php\n\
$a = [[\"n\" => 1]];\n\
$b = $a;\n\
$c = $b[0];\n\
$c[\"n\"] = 7;\n\
$b[0] = $c;\n\
echo $a[0][\"n\"], \":\", $b[0][\"n\"], \"\\n\";",
            expected_stdout: "1:7\n",
        },
        CowReducerCase {
            name: "foreach_value_nested_mutation",
            oracle: "Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt",
            source: "<?php\n\
$a = [[\"v\" => 1], [\"v\" => 2]];\n\
foreach ($a as $row) { $row[\"v\"] += 10; }\n\
echo $a[0][\"v\"], \":\", $a[1][\"v\"], \"\\n\";",
            expected_stdout: "1:2\n",
        },
        CowReducerCase {
            name: "foreach_mutate_copied_array",
            oracle: "Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt",
            source: "<?php\n\
$a = [\"x\" => 1, \"y\" => 2, \"z\" => 3];\n\
$b = $a;\n\
foreach ($b as $k => $v) { if ($k === \"y\") { $b[\"w\"] = 4; } }\n\
echo count($a), \":\", count($b), \":\", $b[\"w\"], \"\\n\";",
            expected_stdout: "3:4:4\n",
        },
        CowReducerCase {
            name: "array_splice_discarded_result_destructor_mutation",
            oracle: "ext/standard/tests/array/gh16649/array_splice_uaf_add_elements.phpt",
            source: "<?php\n\
class C { function __destruct() { global $arr; $arr[] = 0; } }\n\
$arr = [\"1\", new C, \"2\"];\n\
try {\n\
    array_splice($arr, 1, 2);\n\
    echo \"not thrown\\n\";\n\
} catch (Error $e) {\n\
    echo $e->getMessage(), \"\\n\";\n\
}",
            expected_stdout: "Array was modified during array_splice operation\n",
        },
        CowReducerCase {
            name: "array_walk_callback_global_swap",
            oracle: "ext/standard/tests/array/array_walk/bug69068_2.phpt",
            source: "<?php\n\
function walk_swap(&$value, $key) {\n\
    var_dump($value);\n\
    if ($value == 2) { $GLOBALS[\"array\"] = $GLOBALS[\"array2\"]; }\n\
    $value *= 10;\n\
}\n\
$array = [1, 2, 3];\n\
$array2 = [4, 5];\n\
array_walk($array, \"walk_swap\");\n\
var_dump($array, $array2);",
            expected_stdout: "int(1)\nint(2)\nint(4)\nint(5)\narray(2) {\n  [0]=>\n  int(40)\n  [1]=>\n  int(50)\n}\narray(2) {\n  [0]=>\n  int(4)\n  [1]=>\n  int(5)\n}\n",
        },
        CowReducerCase {
            name: "array_walk_closure_use_capture_global_swap",
            oracle: "ext/standard/tests/array/array_walk/bug69068_2.phpt closure-use row",
            source: "<?php\n\
$array = [1, 2, 3];\n\
$array2 = [4, 5];\n\
array_walk($array, function (&$value, $key) use ($array2) {\n\
    var_dump($value);\n\
    if ($value == 2) { $GLOBALS[\"array\"] = $array2; }\n\
    $value *= 10;\n\
});\n\
var_dump($array, $array2);",
            expected_stdout: "int(1)\nint(2)\nint(4)\nint(5)\narray(2) {\n  [0]=>\n  int(40)\n  [1]=>\n  int(50)\n}\narray(2) {\n  [0]=>\n  int(4)\n  [1]=>\n  int(5)\n}\n",
        },
        CowReducerCase {
            name: "function_argument_array_mutation",
            oracle: "ext/standard/tests/array/array_reduce_accumulator_refcount.phpt",
            source: "<?php\n\
function mutate($x) { $x[\"v\"] = \"changed\"; $x[] = \"tail\"; return $x; }\n\
$a = [\"v\" => \"base\"];\n\
$b = mutate($a);\n\
echo $a[\"v\"], \":\", $b[\"v\"], \":\", count($a), \":\", count($b), \"\\n\";",
            expected_stdout: "base:changed:1:2\n",
        },
        CowReducerCase {
            name: "array_reduce_named_callback_accumulator_refcount",
            oracle: "ext/standard/tests/array/array_reduce_accumulator_refcount.phpt",
            source: "<?php\n\
function reduce_accumulator($acc, $val) { debug_zval_dump($acc); $acc[] = $val; return $acc; }\n\
$result = array_reduce([1, 2, 3], \"reduce_accumulator\", []);\n\
debug_zval_dump($result);",
            expected_stdout: "array(0) interned {\n}\narray(1) packed refcount(2){\n  [0]=>\n  int(1)\n}\narray(2) packed refcount(2){\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\narray(3) packed refcount(2){\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n}\n",
        },
        CowReducerCase {
            name: "array_reduce_named_callback_by_ref_return",
            oracle: "ext/standard/tests/array/array_reduce_return_by_ref.phpt",
            source: "<?php\n\
function &pick_reduce_value($carry, $value) { return $value; }\n\
$array = [1, 2];\n\
var_dump(array_reduce($array, \"pick_reduce_value\", 0));",
            expected_stdout: "int(2)\n",
        },
        CowReducerCase {
            name: "call_user_func_array_reference_element_identity",
            oracle: "ext/standard/tests/general_functions/call_user_func_array_variation_001.phpt",
            source: "<?php\n\
function by_val($arg) { $arg = \"changed\"; }\n\
function by_ref(&$arg) { $arg = \"changed\"; }\n\
$items = [\"original\"];\n\
call_user_func_array(\"by_val\", $items);\n\
var_dump($items);\n\
$ref =& $items[0];\n\
call_user_func_array(\"by_val\", $items);\n\
var_dump($items);\n\
call_user_func_array(\"by_ref\", $items);\n\
var_dump($items);",
            expected_stdout: "array(1) {\n  [0]=>\n  string(8) \"original\"\n}\narray(1) {\n  [0]=>\n  &string(8) \"original\"\n}\narray(1) {\n  [0]=>\n  &string(7) \"changed\"\n}\n",
        },
        CowReducerCase {
            name: "function_return_array_then_write",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function make_copy($x) { return $x; }\n\
$a = [\"v\" => 1];\n\
$b = make_copy($a);\n\
$b[\"v\"] = 2;\n\
echo $a[\"v\"], \":\", $b[\"v\"], \"\\n\";",
            expected_stdout: "1:2\n",
        },
        CowReducerCase {
            name: "by_ref_assignment_from_call_result_assigns_value",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function make_array() { return [\"v\" => 1]; }\n\
$value =& make_array();\n\
$value[\"v\"] = 2;\n\
echo $value[\"v\"], \"\\n\";",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 3\n2\n",
        },
        CowReducerCase {
            name: "by_ref_assignment_from_function_result_keeps_result_alive",
            oracle: "Zend/tests/assign_ref_func_leak.phpt",
            source: "<?php\n\
function make_array() { return [0]; }\n\
$x = $y =& make_array();\n\
var_dump($x, $y);",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 3\narray(1) {\n  [0]=>\n  int(0)\n}\narray(1) {\n  [0]=>\n  int(0)\n}\n",
        },
        CowReducerCase {
            name: "array_slot_by_ref_assignment_from_call_result_assigns_value",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function make_array() { return [\"v\" => 1]; }\n\
$items = [[\"v\" => 0]];\n\
$items[0] =& make_array();\n\
$items[0][\"v\"] = 2;\n\
echo $items[0][\"v\"], \"\\n\";",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 4\n2\n",
        },
        CowReducerCase {
            name: "array_slot_by_ref_assignment_from_null_call_result",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function returnsVal() {}\n\
$items = [\"slot\" => \"before\"];\n\
var_dump($items[\"slot\"] =& returnsVal());\n\
var_dump($items[\"slot\"]);",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 4\nNULL\nNULL\n",
        },
        CowReducerCase {
            name: "recursive_array_literal_slot_replaced_by_call_result",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function returnsVal() {}\n\
$array = [&$array];\n\
var_dump($array[0] =& returnsVal());\n\
var_dump($array);",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 4\nNULL\nNULL\n",
        },
        CowReducerCase {
            name: "keyed_recursive_array_literal_slot_replaced_by_call_result",
            oracle: "PHP oracle: keyed array literal reference to assigned variable creates recursion",
            source: "<?php\n\
function returnsVal() {}\n\
$array = [\"self\" => &$array];\n\
var_dump($array[\"self\"] =& returnsVal());\n\
var_dump($array);",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 4\nNULL\nNULL\n",
        },
        CowReducerCase {
            name: "nested_recursive_array_literal_value_write_replaces_self",
            oracle: "PHP oracle: nested array literal reference to assigned variable creates recursion",
            source: "<?php\n\
$array = [[&$array]];\n\
$array[0][0] = 7;\n\
var_dump($array);",
            expected_stdout: "int(7)\n",
        },
        CowReducerCase {
            name: "by_ref_assignment_from_copied_call_result_detaches",
            oracle: "Zend/tests/assign_by_val_function_by_ref_return_value.phpt",
            source: "<?php\n\
function make_copy($x) { return $x; }\n\
$base = [\"v\" => 1];\n\
$alias = $base;\n\
$slot =& make_copy($base);\n\
$slot[\"v\"] = 9;\n\
echo $base[\"v\"], \":\", $alias[\"v\"], \":\", $slot[\"v\"], \"\\n\";",
            expected_stdout: "\nNotice: Only variables should be assigned by reference in {source_path} on line 5\n1:1:9\n",
        },
        CowReducerCase {
            name: "cursor_mutation_shared_alias",
            oracle: "PHP oracle: array cursor mutation must detach shared aliases",
            source: "<?php\n\
$a = [\"a\" => \"A\", \"b\" => \"B\"];\n\
$b = $a;\n\
echo next($b), \":\", key($b), \":\", key($a), \":\", current($a), \"\\n\";",
            expected_stdout: "B:b:a:A\n",
        },
        CowReducerCase {
            name: "array_shift_shared_alias",
            oracle: "PHP oracle: array_shift() must detach shared aliases",
            source: "<?php\n\
$a = [1, 2, 3];\n\
$b = $a;\n\
echo array_shift($b), \":\", count($a), \":\", count($b), \":\", $a[0], \":\", $b[0], \"\\n\";",
            expected_stdout: "1:3:2:1:2\n",
        },
        CowReducerCase {
            name: "array_unshift_shared_alias",
            oracle: "ext/standard/tests/array/array_unshift_basic1.phpt",
            source: "<?php\n\
$a = [1, 2];\n\
$b = $a;\n\
echo array_unshift($b, 10), \":\", count($a), \":\", count($b), \":\", $a[0], \":\", $b[0], \":\", $b[2], \"\\n\";",
            expected_stdout: "3:2:3:1:10:2\n",
        },
        CowReducerCase {
            name: "array_reindexing_internals_unwrap_single_owner_refs",
            oracle: "Zend/tests/foreach/foreach_reference.phpt",
            source: "<?php\n\
$items = [\"a\", \"b\", \"c\"];\n\
foreach ($items as &$value) {}\n\
var_dump(array_values($items));\n\
var_dump(array_reverse($items));",
            expected_stdout: "array(3) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  &string(1) \"c\"\n}\narray(3) {\n  [0]=>\n  &string(1) \"c\"\n  [1]=>\n  string(1) \"b\"\n  [2]=>\n  string(1) \"a\"\n}\n",
        },
        CowReducerCase {
            name: "string_offset_shared_alias",
            oracle: "Zend/tests/str_offset_001.phpt",
            source: "<?php\n\
$s = \"abcd\";\n\
$t = $s;\n\
$t[1] = \"X\";\n\
echo $s, \":\", $t, \"\\n\";",
            expected_stdout: "abcd:aXcd\n",
        },
        CowReducerCase {
            name: "string_offset_padding_shared_alias",
            oracle: "Zend/tests/str_offset_002.phpt",
            source: "<?php\n\
$s = \"ab\";\n\
$t = $s;\n\
$t[4] = \"Z\";\n\
echo bin2hex($s), \":\", bin2hex($t), \"\\n\";",
            expected_stdout: "6162:616220205a\n",
        },
        CowReducerCase {
            name: "function_argument_string_offset_write",
            oracle: "Zend/tests/string_offset_optimization.phpt",
            source: "<?php\n\
function poke($x) { $x[0] = \"Q\"; return $x; }\n\
$s = \"abc\";\n\
$t = poke($s);\n\
echo $s, \":\", $t, \"\\n\";",
            expected_stdout: "abc:Qbc\n",
        },
        CowReducerCase {
            name: "string_compound_shared_alias",
            oracle: "PHP oracle: scalar string compound assignment must keep aliases separate",
            source: "<?php\n\
$s = \"abcd\";\n\
$t = $s;\n\
$t &= \"WXYZ\";\n\
echo bin2hex($s), \":\", bin2hex($t), \"\\n\";",
            expected_stdout: "61626364:41404140\n",
        },
    ];

    let root = temp_dir("ptn-native-cow-reducers");
    fs::create_dir_all(&root).unwrap();

    let mut passed = 0usize;
    let mut failed = 0usize;
    for case in cases {
        let input = root.join(format!("{}.php", case.name));
        let output = root.join(format!("{}-bin", case.name));
        fs::write(&input, case.source).unwrap();
        let expected_stdout = case
            .expected_stdout
            .replace("{source_path}", &input.display().to_string());

        compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap_or_else(|error| {
            panic!("{} ({}) compile failed: {error}", case.name, case.oracle)
        });
        let execution = Command::new(&output)
            .output()
            .unwrap_or_else(|error| panic!("{} ({}) run failed: {error}", case.name, case.oracle));

        if execution.status.success()
            && execution.stdout == expected_stdout.as_bytes()
            && execution.stderr.is_empty()
        {
            passed += 1;
            continue;
        }

        failed += 1;
        assert!(
            execution.status.success(),
            "{} ({}) exited with {:?}",
            case.name,
            case.oracle,
            execution.status.code()
        );
        assert_eq!(
            String::from_utf8(execution.stdout).unwrap(),
            expected_stdout,
            "{} ({}) stdout mismatch",
            case.name,
            case.oracle
        );
        assert_eq!(
            String::from_utf8(execution.stderr).unwrap(),
            "",
            "{} ({}) stderr mismatch",
            case.name,
            case.oracle
        );
    }

    assert_eq!(passed, 35, "COW reducer pass count changed");
    assert_eq!(failed, 0, "COW reducer fail count changed");
}

#[test]
fn compile_usort_callback_mutation_snapshots_to_native_binary() {
    let root = temp_dir("ptn-native-usort-callback-mutation-snapshots");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("usort-callback-mutation-snapshots.php");
    let output = root.join("usort-callback-mutation-snapshots-bin");
    fs::write(
        &input,
        "<?php\n\
function usercompare($a, $b) {\n\
    unset($GLOBALS[\"my_var\"][2]);\n\
    if ($a == $b) { return 0; }\n\
    return $a < $b ? -1 : 1;\n\
}\n\
$my_var = [\n\
    1 => \"entry_1\",\n\
    2 => \"entry_2\",\n\
    3 => \"entry_3\",\n\
    4 => \"entry_4\",\n\
    5 => \"entry_5\",\n\
];\n\
usort($my_var, \"usercompare\");\n\
var_dump($my_var);\n\
\n\
$array = [\n\
    1 => \"entry_1\",\n\
    2 => \"entry_2\",\n\
    3 => \"entry_3\",\n\
    4 => \"entry_4\",\n\
    5 => \"entry_5\",\n\
];\n\
usort($array, function($a, $b) use (&$array, &$ref) {\n\
    unset($array[2]);\n\
    $ref = $array;\n\
    if ($a == $b) { return 0; }\n\
    return $a < $b ? -1 : 1;\n\
});\n\
var_dump($array);",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    let expected_array = "array(5) {\n  [0]=>\n  string(7) \"entry_1\"\n  [1]=>\n  string(7) \"entry_2\"\n  [2]=>\n  string(7) \"entry_3\"\n  [3]=>\n  string(7) \"entry_4\"\n  [4]=>\n  string(7) \"entry_5\"\n}\n";
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        format!("{expected_array}{expected_array}")
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_destructor_mutated_assignment_reducers_to_native_binary() {
    let root = temp_dir("ptn-native-destructor-mutated-assignment-reducers");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("destructor-mutated-assignment-reducers.php");
    let output = root.join("destructor-mutated-assignment-reducers-bin");
    fs::write(
        &input,
        "<?php\n\
class PlainBox { public ?PlainT $value; }\n\
class PlainT { public function __destruct() { global $plainBox; $plainBox->value = null; } }\n\
function plain_test($box) {\n\
    $result = ($box->value = new PlainT);\n\
    echo is_object($result) ? \"object\" : \"not-object\";\n\
    echo \":\", $box->value === null ? \"null\" : \"set\", \"\\n\";\n\
}\n\
$plainBox = new PlainBox();\n\
$plainBox->value = new PlainT;\n\
plain_test($plainBox);\n\
plain_test($plainBox);\n\
\n\
class PropAliasAssignBox { public ?PropAliasAssignT $value; }\n\
class PropAliasAssignT {\n\
    static ?PropAliasAssignT $keep;\n\
    public function __destruct() { global $propAliasAssignBox; $propAliasAssignBox->value = null; }\n\
}\n\
function prop_alias_assign_test($box) { return spl_object_id($box->value = new PropAliasAssignT); }\n\
$propAliasAssignBox = new PropAliasAssignBox();\n\
$propAliasAssignBox->value = new PropAliasAssignT;\n\
PropAliasAssignT::$keep =& $propAliasAssignBox->value;\n\
$propAliasAssignFirst = prop_alias_assign_test($propAliasAssignBox);\n\
$propAliasAssignSecond = prop_alias_assign_test($propAliasAssignBox);\n\
echo $propAliasAssignFirst === $propAliasAssignSecond ? \"prop-alias-assign:same\\n\" : \"prop-alias-assign:different\\n\";\n\
\n\
class VarRefT { public function __destruct() { $GLOBALS[\"varRefA\"] = null; } }\n\
$varRefA = new VarRefT;\n\
$varRefTmp = new VarRefT;\n\
$varRefResult = ($varRefA =& $varRefTmp);\n\
echo is_null($varRefResult) ? \"var-ref:null\\n\" : \"var-ref:object\\n\";\n\
echo is_null($varRefTmp) ? \"var-ref-tmp:null\\n\" : \"var-ref-tmp:object\\n\";\n\
\n\
class RefBox { public ?RefT $value; }\n\
class RefT { public function __destruct() { global $refBox; $refBox->value = null; } }\n\
function ref_test($box) {\n\
    $tmp = new RefT;\n\
    $result = ($box->value =& $tmp);\n\
    echo is_null($result) ? \"prop-ref:null\" : \"prop-ref:object\";\n\
    echo is_null($tmp) ? \":tmp-null\" : \":tmp-object\";\n\
    echo $box->value === null ? \":slot-null\\n\" : \":slot-set\\n\";\n\
}\n\
$refBox = new RefBox();\n\
$refBox->value = new RefT;\n\
ref_test($refBox);\n\
ref_test($refBox);\n\
\n\
class AliasBox { public ?AliasT $value; }\n\
class AliasT {\n\
    static ?AliasT $keep;\n\
    public function __destruct() { global $aliasBox; $aliasBox->value = null; }\n\
}\n\
function alias_ref_test($box) {\n\
    $tmp = new AliasT;\n\
    $result = ($box->value =& $tmp);\n\
    echo is_null($result) ? \"alias-prop-ref:null\" : \"alias-prop-ref:object\";\n\
    echo is_null($tmp) ? \":tmp-null\" : \":tmp-object\";\n\
    echo $box->value === null ? \":slot-null\\n\" : \":slot-set\\n\";\n\
}\n\
$aliasBox = new AliasBox();\n\
$aliasBox->value = new AliasT;\n\
AliasT::$keep =& $aliasBox->value;\n\
alias_ref_test($aliasBox);\n\
alias_ref_test($aliasBox);\n\
\n\
class StaticRefT {\n\
    static ?StaticRefT $test;\n\
    public function __destruct() { StaticRefT::$test = null; }\n\
}\n\
StaticRefT::$test = new StaticRefT;\n\
$staticRefTmp = new StaticRefT;\n\
$staticRefResult = (StaticRefT::$test =& $staticRefTmp);\n\
echo is_null($staticRefResult) ? \"static-ref:null\\n\" : \"static-ref:object\\n\";\n\
echo is_null($staticRefTmp) ? \"static-ref-tmp:null\\n\" : \"static-ref-tmp:object\\n\";\n",
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = Command::new(&output).output().unwrap();
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8(execution.stdout).unwrap(),
        concat!(
            "object:null\n",
            "object:set\n",
            "prop-alias-assign:same\n",
            "var-ref:null\n",
            "var-ref-tmp:null\n",
            "prop-ref:null:tmp-null:slot-null\n",
            "prop-ref:object:tmp-object:slot-set\n",
            "alias-prop-ref:object:tmp-object:slot-set\n",
            "alias-prop-ref:null:tmp-null:slot-null\n",
            "static-ref:null\n",
            "static-ref-tmp:null\n",
        )
    );
    assert_eq!(String::from_utf8(execution.stderr).unwrap(), "");
}

#[test]
fn compile_dynamic_temporary_cow_reducers_match_php_oracle() {
    let root = temp_dir("ptn-native-dynamic-temporary-cow-reducers");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("dynamic-temporary-cow-reducers.php");
    let output = root.join("dynamic-temporary-cow-reducers-bin");
    fs::write(
        &input,
        "<?php\n\
function record_case($name, $ok) {\n\
    if (!$ok) {\n\
        echo \"FAIL \", $name, \"\\n\";\n\
    }\n\
    return $ok;\n\
}\n\
function make_pair($base) { return [$base, $base + 1]; }\n\
function make_nested($base) { return [[\"v\" => $base], [\"v\" => $base + 1]]; }\n\
function identity_value($value) { return $value; }\n\
function pick_slot($items, $key) { return $items[$key]; }\n\
function snapshot_arg($value) {\n\
    $arg = func_get_arg(0);\n\
    $copy = $arg;\n\
    $copy[] = \"copy\";\n\
    return [$value, $arg, $copy];\n\
}\n\
function append_marker(&$value) { $value[] = \"callee\"; return count($value); }\n\
function make_text($seed) { return $seed . \"bc\"; }\n\
$pass = 0;\n\
$fail = 0;\n\
$tmp = make_pair(10);\n\
$tmp_copy = $tmp;\n\
$tmp_copy[] = 12;\n\
if (record_case(\"call_result_array_append\", count($tmp) === 2 && count($tmp_copy) === 3 && $tmp[0] === 10 && $tmp_copy[2] === 12)) { $pass++; } else { $fail++; }\n\
$base = [\"drop\" => \"gone\", \"keep\" => \"base\"];\n\
$result = identity_value($base);\n\
$result_copy = $result;\n\
unset($result_copy[\"drop\"]);\n\
if (record_case(\"call_result_array_unset\", array_key_exists(\"drop\", $base) && array_key_exists(\"drop\", $result) && !array_key_exists(\"drop\", $result_copy))) { $pass++; } else { $fail++; }\n\
$matrix = make_nested(20);\n\
$key = 1;\n\
$row = $matrix[$key];\n\
$row_copy = $row;\n\
$row_copy[\"v\"] = 99;\n\
$row_copy[] = \"tail\";\n\
if (record_case(\"dynamic_array_read_slot\", $matrix[1][\"v\"] === 21 && $row[\"v\"] === 21 && $row_copy[\"v\"] === 99 && count($row_copy) === 2)) { $pass++; } else { $fail++; }\n\
$called_slot = make_nested(30)[0];\n\
$called_slot_copy = $called_slot;\n\
$called_slot_copy[\"v\"] = 77;\n\
if (record_case(\"call_result_read_slot\", $called_slot[\"v\"] === 30 && $called_slot_copy[\"v\"] === 77)) { $pass++; } else { $fail++; }\n\
$text = make_text(\"a\");\n\
$text_copy = $text;\n\
$text_copy[1] = \"Z\";\n\
if (record_case(\"call_result_string_offset\", $text === \"abc\" && $text_copy === \"aZc\")) { $pass++; } else { $fail++; }\n\
$strings = [\"ab\", str_rot13(\"no\")];\n\
$string_key = 1;\n\
$slot = $strings[$string_key];\n\
$slot_copy = $slot;\n\
$slot_copy[0] = \"Z\";\n\
if (record_case(\"dynamic_string_read_slot\", $strings[1] === \"ab\" && $slot === \"ab\" && $slot_copy === \"Zb\")) { $pass++; } else { $fail++; }\n\
$letters = [\"name\" => \"ptn\"];\n\
$char = $letters[\"name\"][1];\n\
$char_copy = $char;\n\
$char_copy[0] = \"T\";\n\
if (record_case(\"string_offset_read_result\", $letters[\"name\"] === \"ptn\" && $char === \"t\" && $char_copy === \"T\")) { $pass++; } else { $fail++; }\n\
$values = array_values([\"x\" => 1, \"y\" => 2]);\n\
$values_copy = $values;\n\
$shifted = array_shift($values_copy);\n\
if (record_case(\"array_values_call_result\", $shifted === 1 && count($values) === 2 && count($values_copy) === 1 && $values_copy[0] === 2)) { $pass++; } else { $fail++; }\n\
$source_slots = make_nested(40);\n\
$picked = pick_slot($source_slots, 0);\n\
$picked_copy = $picked;\n\
$picked_copy[\"v\"] += 5;\n\
if (record_case(\"function_returned_read_slot\", $source_slots[0][\"v\"] === 40 && $picked[\"v\"] === 40 && $picked_copy[\"v\"] === 45)) { $pass++; } else { $fail++; }\n\
$arg_source = [\"seed\"];\n\
$arg_result = snapshot_arg($arg_source);\n\
if (record_case(\"func_get_arg_result\", count($arg_source) === 1 && count($arg_result[0]) === 1 && count($arg_result[1]) === 1 && count($arg_result[2]) === 2 && $arg_result[2][1] === \"copy\")) { $pass++; } else { $fail++; }\n\
$dynamic_user = \"append_marker\";\n\
$dynamic_source = [\"seed\"];\n\
$dynamic_copy = $dynamic_source;\n\
$dynamic_count = $dynamic_user($dynamic_copy);\n\
if (record_case(\"dynamic_user_call_arg_cow\", $dynamic_count === 2 && count($dynamic_source) === 1 && count($dynamic_copy) === 2 && $dynamic_copy[1] === \"callee\")) { $pass++; } else { $fail++; }\n\
$dynamic_shift = \"array_shift\";\n\
$shift_source = [10, 20, 30];\n\
$shift_copy = $shift_source;\n\
$shifted = $dynamic_shift($shift_copy);\n\
if (record_case(\"dynamic_array_shift_detaches_arg\", $shifted === 10 && count($shift_source) === 3 && count($shift_copy) === 2 && $shift_source[0] === 10 && $shift_copy[0] === 20)) { $pass++; } else { $fail++; }\n\
echo \"dynamic temporary COW: pass=\", $pass, \" fail=\", $fail, \"\\n\";",
    )
    .unwrap();

    let php = Command::new("php")
        .arg(&input)
        .output()
        .expect("php oracle should run");
    assert!(
        php.status.success(),
        "PHP oracle exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        php.status.code(),
        String::from_utf8_lossy(&php.stdout),
        String::from_utf8_lossy(&php.stderr)
    );

    let compiled = compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();
    let native = Command::new(&output).output().unwrap();
    assert!(
        native.status.success(),
        "native exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        native.status.code(),
        String::from_utf8_lossy(&native.stdout),
        String::from_utf8_lossy(&native.stderr)
    );
    assert_eq!(
        native.stdout, php.stdout,
        "native stdout diverged from PHP oracle"
    );
    assert_eq!(
        native.stderr, php.stderr,
        "native stderr diverged from PHP oracle"
    );
    assert_eq!(
        String::from_utf8(native.stdout).unwrap(),
        "dynamic temporary COW: pass=12 fail=0\n"
    );

    let c_source = fs::read_to_string(compiled.c_source.unwrap()).unwrap();
    assert!(c_source.contains("ptn_value_share("));
    assert!(c_source.contains("ptn_value_drop(&ptn_tmp_"));
    assert!(c_source.contains("ptn_dynamic_function_name("));
    assert!(c_source.contains("ptn_call_callable(&runtime"));
    assert!(c_source.contains("ptn_call_dynamic_function_name(runtime"));
    assert!(c_source.contains("ptn_runtime_reference_for_variable(&runtime"));
    assert!(c_source.contains("ptn_dynamic_call_detach_first_reference_argument"));
    assert!(
        c_source.contains("static PTN_UNUSED PtnArray *ptn_value_detach_array(PtnValue *value)")
    );
    assert!(c_source.contains("ptn_array_detach_value(entry_value);"));
    assert!(c_source.contains("ptn_runtime_string_offset_set"));
    assert!(c_source.contains("ptn_runtime_array_path_set"));
    assert!(c_source.contains("ptn_runtime_array_shift_variable"));
    assert!(c_source.contains("ptn_array_read(&runtime"));
}

#[test]
fn compile_nested_array_cow_reducers_match_php_oracle() {
    let cases = [
        CowReducerCase {
            name: "recursive_replace_unwraps_nested_reference",
            oracle: "ext/standard/tests/array/array_merge_replace_recursive_refs.phpt",
            source: "<?php\n\
$x = 24;\n\
$arr1 = [[42]];\n\
$arr2 = [[&$x]];\n\
unset($x);\n\
$arr3 = array_replace_recursive($arr1, $arr2);\n\
$arr2[0][0] = 12;\n\
echo $arr3[0][0], \":\", $arr2[0][0], \"\\n\";",
            expected_stdout: "24:12\n",
        },
        CowReducerCase {
            name: "recursive_merge_unwraps_reference_append",
            oracle: "ext/standard/tests/array/array_merge_replace_recursive_refs.phpt",
            source: "<?php\n\
$x = 24;\n\
$arr1 = [42];\n\
$arr2 = [&$x];\n\
unset($x);\n\
$arr3 = array_merge_recursive($arr1, $arr2);\n\
$arr2[0] = 12;\n\
echo $arr3[0], \":\", $arr3[1], \":\", $arr2[0], \"\\n\";",
            expected_stdout: "42:24:12\n",
        },
        CowReducerCase {
            name: "recursive_replace_detaches_nested_sources",
            oracle: "PHP oracle: array_replace_recursive recursively detaches nested arrays",
            source: "<?php\n\
$left = [\"k\" => [\"x\" => 1, \"same\" => \"left\"]];\n\
$right = [\"k\" => [\"same\" => \"right\", \"y\" => 2]];\n\
$out = array_replace_recursive($left, $right);\n\
$out[\"k\"][\"x\"] = 9;\n\
$right[\"k\"][\"same\"] = \"changed\";\n\
echo $left[\"k\"][\"x\"], \":\", $left[\"k\"][\"same\"], \":\", $out[\"k\"][\"x\"], \":\", $out[\"k\"][\"same\"], \":\", $out[\"k\"][\"y\"], \":\", $right[\"k\"][\"same\"], \"\\n\";",
            expected_stdout: "1:left:9:right:2:changed\n",
        },
        CowReducerCase {
            name: "recursive_replace_numeric_keys_replace",
            oracle: "PHP oracle: array_replace_recursive replaces numeric keys instead of appending",
            source: "<?php\n\
$left = [[\"x\" => 1], \"old\"];\n\
$right = [0 => [\"y\" => 2], 1 => \"new\"];\n\
$out = array_replace_recursive($left, $right);\n\
echo count($out), \":\", $out[0][\"x\"], \":\", $out[0][\"y\"], \":\", $out[1], \":\", count($left[0]), \"\\n\";",
            expected_stdout: "2:1:2:new:1\n",
        },
    ];

    let root = temp_dir("ptn-native-nested-cow-reducers");
    fs::create_dir_all(&root).unwrap();

    let mut passed = 0usize;
    let mut failed = 0usize;
    for case in cases {
        let input = root.join(format!("{}.php", case.name));
        let output = root.join(format!("{}-bin", case.name));
        fs::write(&input, case.source).unwrap();

        let php = Command::new("php")
            .arg(&input)
            .output()
            .unwrap_or_else(|error| {
                panic!("{} ({}) PHP oracle failed: {error}", case.name, case.oracle)
            });
        assert!(
            php.status.success(),
            "{} ({}) PHP oracle exited with {:?}\nstdout:\n{}\nstderr:\n{}",
            case.name,
            case.oracle,
            php.status.code(),
            String::from_utf8_lossy(&php.stdout),
            String::from_utf8_lossy(&php.stderr)
        );
        assert_eq!(
            String::from_utf8(php.stdout.clone()).unwrap(),
            case.expected_stdout,
            "{} ({}) PHP oracle stdout changed",
            case.name,
            case.oracle
        );
        assert_eq!(
            String::from_utf8(php.stderr.clone()).unwrap(),
            "",
            "{} ({}) PHP oracle stderr changed",
            case.name,
            case.oracle
        );

        compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap_or_else(|error| {
            panic!("{} ({}) compile failed: {error}", case.name, case.oracle)
        });
        let native = Command::new(&output).output().unwrap_or_else(|error| {
            panic!("{} ({}) native run failed: {error}", case.name, case.oracle)
        });

        if native.status.success() && native.stdout == php.stdout && native.stderr == php.stderr {
            passed += 1;
            continue;
        }

        failed += 1;
        assert!(
            native.status.success(),
            "{} ({}) native exited with {:?}",
            case.name,
            case.oracle,
            native.status.code()
        );
        assert_eq!(
            String::from_utf8(native.stdout).unwrap(),
            String::from_utf8(php.stdout).unwrap(),
            "{} ({}) stdout mismatch",
            case.name,
            case.oracle
        );
        assert_eq!(
            String::from_utf8(native.stderr).unwrap(),
            String::from_utf8(php.stderr).unwrap(),
            "{} ({}) stderr mismatch",
            case.name,
            case.oracle
        );
    }

    assert_eq!(passed, 4, "nested COW reducer pass count changed");
    assert_eq!(failed, 0, "nested COW reducer fail count changed");
}

fn temp_dir(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("{name}-{}-{unique}", std::process::id()));
    path
}
