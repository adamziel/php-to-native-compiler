use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Category {
    Arrays,
    Strings,
    Foreach,
    Functions,
    NestedValues,
    References,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Category::Arrays => "arrays",
            Category::Strings => "strings",
            Category::Foreach => "foreach",
            Category::Functions => "functions",
            Category::NestedValues => "nested values",
            Category::References => "references",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedPtnStatus {
    MatchesPhp,
    DiffersFromPhp,
    CompileBlocked,
}

struct CowCase {
    name: &'static str,
    category: Category,
    source: &'static str,
    expected: ExpectedPtnStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProcessOutcome {
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PtnOutcome {
    Ran(ProcessOutcome),
    CompileError(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CategoryTally {
    total: usize,
    matches_php: usize,
    differs_from_php: usize,
    compile_blocked: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Summary {
    total: usize,
    matches_php: usize,
    differs_from_php: usize,
    compile_blocked: usize,
}

#[test]
fn cow_copy_on_write_oracle_suite() {
    let cases = cow_cases();
    let root = temp_dir("ptn-cow-oracle");
    fs::create_dir_all(&root).unwrap();

    let mut summary = Summary::default();
    let mut by_category = BTreeMap::<&'static str, CategoryTally>::new();

    for case in cases {
        let input = root.join(format!("{}.php", case.name));
        let output = root.join(format!("{}-bin", case.name));
        fs::write(&input, format!("<?php\n{}\n", case.source)).unwrap();

        let php = run_php(&input);
        let ptn = run_ptn(&input, &output);
        let actual = classify_ptn_status(&php, &ptn);

        assert_eq!(
            actual, case.expected,
            "COW case `{}` changed status.\nPHP: {php:?}\nPTN: {ptn:?}",
            case.name
        );

        summary.total += 1;
        let tally = by_category.entry(case.category.name()).or_default();
        tally.total += 1;
        match actual {
            ExpectedPtnStatus::MatchesPhp => {
                summary.matches_php += 1;
                tally.matches_php += 1;
            }
            ExpectedPtnStatus::DiffersFromPhp => {
                summary.differs_from_php += 1;
                tally.differs_from_php += 1;
            }
            ExpectedPtnStatus::CompileBlocked => {
                summary.compile_blocked += 1;
                tally.compile_blocked += 1;
            }
        }
    }

    assert_eq!(
        summary,
        Summary {
            total: 12,
            matches_php: 6,
            differs_from_php: 3,
            compile_blocked: 3,
        }
    );

    assert_eq!(
        by_category.get("arrays"),
        Some(&CategoryTally {
            total: 2,
            matches_php: 2,
            differs_from_php: 0,
            compile_blocked: 0,
        })
    );
    assert_eq!(
        by_category.get("strings"),
        Some(&CategoryTally {
            total: 1,
            matches_php: 0,
            differs_from_php: 1,
            compile_blocked: 0,
        })
    );
    assert_eq!(
        by_category.get("foreach"),
        Some(&CategoryTally {
            total: 2,
            matches_php: 1,
            differs_from_php: 1,
            compile_blocked: 0,
        })
    );
    assert_eq!(
        by_category.get("functions"),
        Some(&CategoryTally {
            total: 3,
            matches_php: 2,
            differs_from_php: 1,
            compile_blocked: 0,
        })
    );
    assert_eq!(
        by_category.get("nested values"),
        Some(&CategoryTally {
            total: 2,
            matches_php: 1,
            differs_from_php: 0,
            compile_blocked: 1,
        })
    );
    assert_eq!(
        by_category.get("references"),
        Some(&CategoryTally {
            total: 2,
            matches_php: 0,
            differs_from_php: 0,
            compile_blocked: 2,
        })
    );
}

fn cow_cases() -> &'static [CowCase] {
    &[
        CowCase {
            name: "array_assignment_write",
            category: Category::Arrays,
            source: r#"$a = [1, 2]; $b = $a; $b[0] = 9; var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "array_append_after_copy",
            category: Category::Arrays,
            source: r#"$a = ["x" => 1]; $b = $a; $b[] = 2; var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "string_offset_write_after_copy",
            category: Category::Strings,
            source: r#"$s = "abc"; $t = $s; $t[1] = "Z"; var_dump($s, $t);"#,
            expected: ExpectedPtnStatus::DiffersFromPhp,
        },
        CowCase {
            name: "foreach_source_mutation_snapshot",
            category: Category::Foreach,
            source: r#"$a = [1, 2]; $i = 0; foreach ($a as $v) { echo $v, "\n"; $a[] = 9; $i++; if ($i == 3) break; } var_dump($a);"#,
            expected: ExpectedPtnStatus::DiffersFromPhp,
        },
        CowCase {
            name: "foreach_value_copy_of_nested_arrays",
            category: Category::Foreach,
            source: r#"$items = [[1], [2]]; foreach ($items as $value) { $value[] = 9; } var_dump($items);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "function_parameter_array_copy",
            category: Category::Functions,
            source: r#"function mutate($x) { $x[0] = 9; return $x; } $a = [1, 2]; $b = mutate($a); var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "function_return_array_copy",
            category: Category::Functions,
            source: r#"function make_items() { $x = [1]; return $x; } $a = make_items(); $b = $a; $b[0] = 2; var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "function_parameter_string_offset_write",
            category: Category::Functions,
            source: r#"function mutate_string($s) { $s[1] = "Z"; return $s; } $a = "abc"; $b = mutate_string($a); var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::DiffersFromPhp,
        },
        CowCase {
            name: "nested_value_read_copy",
            category: Category::NestedValues,
            source: r#"$root = ["child" => [1, 2]]; $child = $root["child"]; $child[0] = 9; var_dump($root["child"], $child);"#,
            expected: ExpectedPtnStatus::MatchesPhp,
        },
        CowCase {
            name: "nested_direct_write_after_copy",
            category: Category::NestedValues,
            source: r#"$a = [["x" => 1]]; $b = $a; $b[0]["x"] = 2; var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::CompileBlocked,
        },
        CowCase {
            name: "reference_assignment_alias",
            category: Category::References,
            source: r#"$a = [1]; $b =& $a; $b[0] = 2; var_dump($a, $b);"#,
            expected: ExpectedPtnStatus::CompileBlocked,
        },
        CowCase {
            name: "reference_parameter_alias",
            category: Category::References,
            source: r#"function touch_ref(&$x) { $x[0] = 2; } $a = [1]; touch_ref($a); var_dump($a);"#,
            expected: ExpectedPtnStatus::CompileBlocked,
        },
    ]
}

fn run_php(input: &Path) -> ProcessOutcome {
    let output = Command::new("php")
        .args(["-d", "error_reporting=E_ALL"])
        .arg(input)
        .output()
        .unwrap();
    ProcessOutcome {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn run_ptn(input: &Path, output: &Path) -> PtnOutcome {
    match compile_file(input, output, CompileOptions { emit_c: false }) {
        Ok(_) => {
            let execution = Command::new(output).output().unwrap();
            PtnOutcome::Ran(ProcessOutcome {
                code: execution.status.code(),
                stdout: execution.stdout,
                stderr: execution.stderr,
            })
        }
        Err(error) => PtnOutcome::CompileError(error.to_string()),
    }
}

fn classify_ptn_status(php: &ProcessOutcome, ptn: &PtnOutcome) -> ExpectedPtnStatus {
    match ptn {
        PtnOutcome::Ran(outcome) if outcome == php => ExpectedPtnStatus::MatchesPhp,
        PtnOutcome::Ran(_) => ExpectedPtnStatus::DiffersFromPhp,
        PtnOutcome::CompileError(_) => ExpectedPtnStatus::CompileBlocked,
    }
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
