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

        compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap_or_else(|error| {
            panic!("{} ({}) compile failed: {error}", case.name, case.oracle)
        });
        let execution = Command::new(&output)
            .output()
            .unwrap_or_else(|error| panic!("{} ({}) run failed: {error}", case.name, case.oracle));

        if execution.status.success()
            && execution.stdout == case.expected_stdout.as_bytes()
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
            case.expected_stdout,
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

    assert_eq!(passed, 16, "COW reducer pass count changed");
    assert_eq!(failed, 0, "COW reducer fail count changed");
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
