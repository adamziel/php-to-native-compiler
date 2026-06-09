use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CowCategory {
    Arrays,
    Strings,
    Foreach,
    Functions,
    NestedValues,
    References,
}

impl CowCategory {
    const ALL: [CowCategory; 6] = [
        CowCategory::Arrays,
        CowCategory::Strings,
        CowCategory::Foreach,
        CowCategory::Functions,
        CowCategory::NestedValues,
        CowCategory::References,
    ];

    fn index(self) -> usize {
        match self {
            CowCategory::Arrays => 0,
            CowCategory::Strings => 1,
            CowCategory::Foreach => 2,
            CowCategory::Functions => 3,
            CowCategory::NestedValues => 4,
            CowCategory::References => 5,
        }
    }

    fn label(self) -> &'static str {
        match self {
            CowCategory::Arrays => "arrays",
            CowCategory::Strings => "strings",
            CowCategory::Foreach => "foreach",
            CowCategory::Functions => "functions",
            CowCategory::NestedValues => "nested values",
            CowCategory::References => "references",
        }
    }
}

struct CowCase {
    name: &'static str,
    category: CowCategory,
    source: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
struct ProcessEvidence {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
enum NativeEvidence {
    Executed(ProcessEvidence),
    CompileBlocker(String),
}

#[derive(Default)]
struct CategoryEvidence {
    total: usize,
    matches: usize,
    runtime_mismatches: usize,
    compile_blockers: usize,
}

#[test]
fn cow_copy_on_write_oracle_suite() {
    let root = temp_dir("ptn-cow-oracle-suite");
    fs::create_dir_all(&root).unwrap();
    let mut categories: [CategoryEvidence; 6] = Default::default();
    let mut gaps = Vec::new();

    for case in cow_cases() {
        let category = &mut categories[case.category.index()];
        category.total += 1;

        let input = root.join(format!("{}.php", case.name));
        let output = root.join(format!("{}-bin", case.name));
        fs::write(&input, case.source).unwrap();

        let php = run_php(&input);
        assert_eq!(
            php.code,
            Some(0),
            "PHP oracle failed for {}:\nstdout:\n{}\nstderr:\n{}",
            case.name,
            php.stdout,
            php.stderr
        );

        match run_native(&input, &output) {
            NativeEvidence::Executed(native) if native == php => {
                category.matches += 1;
            }
            NativeEvidence::Executed(native) => {
                category.runtime_mismatches += 1;
                gaps.push(format!(
                    "[{}] {}: runtime mismatch\n  PHP stdout: {:?}\n  native stdout: {:?}\n  native stderr: {:?}",
                    case.category.label(),
                    case.name,
                    php.stdout,
                    native.stdout,
                    native.stderr
                ));
            }
            NativeEvidence::CompileBlocker(error) => {
                category.compile_blockers += 1;
                gaps.push(format!(
                    "[{}] {}: compile blocker: {}",
                    case.category.label(),
                    case.name,
                    error
                ));
            }
        }
    }

    let total: usize = categories.iter().map(|category| category.total).sum();
    let matches: usize = categories.iter().map(|category| category.matches).sum();
    let runtime_mismatches: usize = categories
        .iter()
        .map(|category| category.runtime_mismatches)
        .sum();
    let compile_blockers: usize = categories
        .iter()
        .map(|category| category.compile_blockers)
        .sum();

    println!(
        "COW oracle evidence: {matches}/{total} cases match PHP; {runtime_mismatches} runtime mismatches; {compile_blockers} compile blockers"
    );
    for category in CowCategory::ALL {
        let evidence = &categories[category.index()];
        println!(
            "  {}: {}/{} match, {} runtime mismatches, {} compile blockers",
            category.label(),
            evidence.matches,
            evidence.total,
            evidence.runtime_mismatches,
            evidence.compile_blockers
        );
    }
    for gap in &gaps {
        println!("  - {gap}");
    }

    assert_eq!(total, 13);
    assert_eq!(matches, 9);
    assert_eq!(runtime_mismatches, 1);
    assert_eq!(compile_blockers, 3);
    assert_category(&categories, CowCategory::Arrays, 2, 2, 0, 0);
    assert_category(&categories, CowCategory::Strings, 2, 2, 0, 0);
    assert_category(&categories, CowCategory::Foreach, 2, 1, 1, 0);
    assert_category(&categories, CowCategory::Functions, 3, 3, 0, 0);
    assert_category(&categories, CowCategory::NestedValues, 2, 1, 0, 1);
    assert_category(&categories, CowCategory::References, 2, 0, 0, 2);
}

fn cow_cases() -> Vec<CowCase> {
    vec![
        CowCase {
            name: "array_assignment_sharing",
            category: CowCategory::Arrays,
            source: r#"<?php
$original = ["a" => "A", "b" => "B"];
$copy = $original;
$copy["a"] = "changed";
$copy[] = "tail";
var_dump($original);
var_dump($copy);
"#,
        },
        CowCase {
            name: "array_cursor_mutation_detach",
            category: CowCategory::Arrays,
            source: r#"<?php
$items = [1, 2, 3];
$copy = $items;
var_dump(array_pop($copy));
var_dump(array_push($copy, 4));
var_dump($items);
var_dump($copy);
"#,
        },
        CowCase {
            name: "string_assignment_sharing",
            category: CowCategory::Strings,
            source: r#"<?php
$original = "seed";
$copy = $original;
$copy .= "-copy";
var_dump($original);
var_dump($copy);
"#,
        },
        CowCase {
            name: "string_offset_write_sharing",
            category: CowCategory::Strings,
            source: r#"<?php
$original = "abcd";
$copy = $original;
$copy[1] = "X";
var_dump($original);
var_dump($copy);
"#,
        },
        CowCase {
            name: "foreach_value_local_mutation",
            category: CowCategory::Foreach,
            source: r#"<?php
$items = [1, 2, 3];
foreach ($items as $value) {
    $value *= 10;
}
var_dump($items);
"#,
        },
        CowCase {
            name: "foreach_source_mutation_snapshot",
            category: CowCategory::Foreach,
            source: r#"<?php
$items = [1, 2, 3];
foreach ($items as $value) {
    echo $value;
    if ($value === 1) {
        $items[] = 4;
        unset($items[1]);
    }
}
echo "\n";
var_dump($items);
"#,
        },
        CowCase {
            name: "function_parameter_array_boundary",
            category: CowCategory::Functions,
            source: r#"<?php
function mutate_array($arr) {
    $arr[0] = "changed";
    return $arr;
}
$base = ["original"];
$result = mutate_array($base);
var_dump($base);
var_dump($result);
"#,
        },
        CowCase {
            name: "function_return_array_boundary",
            category: CowCategory::Functions,
            source: r#"<?php
function identity_array($arr) {
    return $arr;
}
$base = ["k" => "v"];
$returned = identity_array($base);
$returned["k"] = "changed";
var_dump($base);
var_dump($returned);
"#,
        },
        CowCase {
            name: "function_parameter_string_boundary",
            category: CowCategory::Functions,
            source: r#"<?php
function mutate_string($s) {
    $s[0] = "Z";
    return $s;
}
$original = "abcd";
$result = mutate_string($original);
var_dump($original);
var_dump($result);
"#,
        },
        CowCase {
            name: "nested_array_value_local_copy",
            category: CowCategory::NestedValues,
            source: r#"<?php
$source = [["x" => 1], ["x" => 2]];
$copy = $source[0];
$copy["x"] = 9;
var_dump($source);
var_dump($copy);
"#,
        },
        CowCase {
            name: "nested_direct_write_target",
            category: CowCategory::NestedValues,
            source: r#"<?php
$source = [["x" => 1], ["x" => 2]];
$copy = $source;
$copy[0]["x"] = 9;
var_dump($source);
var_dump($copy);
"#,
        },
        CowCase {
            name: "reference_assignment_alias",
            category: CowCategory::References,
            source: r#"<?php
$value = [1];
$alias =& $value;
$alias[0] = 2;
var_dump($value);
var_dump($alias);
"#,
        },
        CowCase {
            name: "reference_foreach_value",
            category: CowCategory::References,
            source: r#"<?php
$items = [1, 2];
foreach ($items as &$value) {
    $value *= 10;
}
var_dump($items);
"#,
        },
    ]
}

fn run_php(input: &Path) -> ProcessEvidence {
    let output = Command::new("php")
        .arg(input)
        .output()
        .expect("php executable is required for the COW oracle suite");
    process_evidence(output)
}

fn run_native(input: &Path, output: &Path) -> NativeEvidence {
    match compile_file(input, output, CompileOptions { emit_c: false }) {
        Ok(_) => {
            let output = Command::new(output).output().unwrap();
            NativeEvidence::Executed(process_evidence(output))
        }
        Err(error) => NativeEvidence::CompileBlocker(error.to_string()),
    }
}

fn process_evidence(output: std::process::Output) -> ProcessEvidence {
    ProcessEvidence {
        code: output.status.code(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn assert_category(
    categories: &[CategoryEvidence; 6],
    category: CowCategory,
    total: usize,
    matches: usize,
    runtime_mismatches: usize,
    compile_blockers: usize,
) {
    let evidence = &categories[category.index()];
    assert_eq!(
        (
            evidence.total,
            evidence.matches,
            evidence.runtime_mismatches,
            evidence.compile_blockers
        ),
        (total, matches, runtime_mismatches, compile_blockers),
        "unexpected COW oracle counts for {}",
        category.label()
    );
}

fn temp_dir(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
