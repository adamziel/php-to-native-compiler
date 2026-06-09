use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Category {
    Arrays,
    Strings,
    Foreach,
    Functions,
    NestedValues,
    References,
}

impl Category {
    const ALL: [Category; 6] = [
        Category::Arrays,
        Category::Strings,
        Category::Foreach,
        Category::Functions,
        Category::NestedValues,
        Category::References,
    ];

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

#[derive(Clone, Copy)]
struct CowCase {
    name: &'static str,
    category: Category,
    source: &'static str,
}

#[derive(Debug)]
struct ProcessOutput {
    timed_out: bool,
    success: bool,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
enum NativeOutput {
    CompileError(String),
    Process(ProcessOutput),
}

#[derive(Clone, Copy, Default)]
struct CategoryStats {
    total: usize,
    matched: usize,
}

#[test]
fn cow_copy_on_write_oracle_suite() {
    let root = temp_dir("ptn-cow-oracle");
    fs::create_dir_all(&root).unwrap();

    let mut stats: BTreeMap<Category, CategoryStats> = BTreeMap::new();
    let mut failures = Vec::new();

    for case in COW_CASES {
        let php_path = root.join(format!("{}.php", case.name));
        let native_path = root.join(format!("{}-bin", case.name));
        fs::write(&php_path, case.source).unwrap();

        let php = run_php(&php_path);
        assert!(
            php.success,
            "PHP oracle failed for {}:\nstdout:\n{}\nstderr:\n{}",
            case.name, php.stdout, php.stderr
        );

        let native = run_native(&php_path, &native_path);
        let matches_php = native_matches_php(&php, &native);
        let entry = stats.entry(case.category).or_default();
        entry.total += 1;
        if matches_php {
            entry.matched += 1;
        } else {
            failures.push(format_failure(case, &php, &native));
        }
    }

    let report = format_report(&stats, &failures);
    eprintln!("{report}");
    assert_baseline(&stats, &report);
}

const COW_CASES: &[CowCase] = &[
    CowCase {
        name: "array_assignment_write_detaches_copy",
        category: Category::Arrays,
        source: r#"<?php
$original = ["a" => "A", "b" => "B", 2 => "two"];
$copy = $original;
$copy["a"] = "changed";
$copy[] = "appended";
unset($copy["b"]);
$copy[2] = "replaced";
var_dump($original);
var_dump($copy);
"#,
    },
    CowCase {
        name: "array_cursor_internals_detach_copy",
        category: Category::Arrays,
        source: r#"<?php
$numbers = [1, 2, 3];
$copy = $numbers;
var_dump(array_shift($copy));
var_dump(array_push($copy, 4));
var_dump($numbers);
var_dump($copy);
"#,
    },
    CowCase {
        name: "string_offset_write_detaches_copy",
        category: Category::Strings,
        source: r#"<?php
$original = "abc";
$copy = $original;
$copy[1] = "Z";
var_dump($original);
var_dump($copy);
"#,
    },
    CowCase {
        name: "foreach_value_mutation_keeps_nested_source",
        category: Category::Foreach,
        source: r#"<?php
$items = [["x" => 1], ["x" => 2]];
foreach ($items as $item) {
    $item["x"] = 99;
    $item[] = "local";
}
var_dump($items);
"#,
    },
    CowCase {
        name: "foreach_appends_to_source_after_snapshot",
        category: Category::Foreach,
        source: r#"<?php
$items = [1, 2];
foreach ($items as $item) {
    var_dump($item);
    $items[] = $item + 10;
}
var_dump($items);
"#,
    },
    CowCase {
        name: "function_array_parameter_detaches_local_write",
        category: Category::Functions,
        source: r#"<?php
function mutate_array($value) {
    $value["k"] = "changed";
    $value[] = "tail";
    return $value;
}
$base = ["k" => "base"];
$result = mutate_array($base);
var_dump($base);
var_dump($result);
"#,
    },
    CowCase {
        name: "function_array_return_detaches_caller_write",
        category: Category::Functions,
        source: r#"<?php
function identity_array($value) {
    return $value;
}
$base = ["x" => 1, "y" => 2];
$returned = identity_array($base);
$returned["x"] = 9;
unset($returned["y"]);
var_dump($base);
var_dump($returned);
"#,
    },
    CowCase {
        name: "function_string_parameter_detaches_local_write",
        category: Category::Functions,
        source: r#"<?php
function mutate_string($value) {
    $value[0] = "Z";
    return $value;
}
$base = "abc";
$result = mutate_string($base);
var_dump($base);
var_dump($result);
"#,
    },
    CowCase {
        name: "nested_value_assignment_detaches_child_copy",
        category: Category::NestedValues,
        source: r#"<?php
$source = [[10, 20], [30, 40]];
$copy = $source[0];
var_dump(array_shift($copy));
var_dump($source[0]);
var_dump($copy);
"#,
    },
    CowCase {
        name: "nested_direct_write_detaches_outer_copy",
        category: Category::NestedValues,
        source: r#"<?php
$original = [["x" => 1]];
$copy = $original;
$copy[0]["x"] = 2;
var_dump($original);
var_dump($copy);
"#,
    },
    CowCase {
        name: "reference_assignment_shares_array_storage",
        category: Category::References,
        source: r#"<?php
$original = ["x" => 1];
$alias =& $original;
$alias["x"] = 2;
var_dump($original);
var_dump($alias);
"#,
    },
    CowCase {
        name: "foreach_reference_mutates_source_slots",
        category: Category::References,
        source: r#"<?php
$items = [1, 2];
foreach ($items as &$item) {
    $item = $item + 10;
}
unset($item);
var_dump($items);
"#,
    },
];

fn run_php(path: &Path) -> ProcessOutput {
    run_command_with_timeout(Command::new("php").arg(path))
}

fn run_native(input: &Path, output: &Path) -> NativeOutput {
    match compile_file(input, output, CompileOptions { emit_c: false }) {
        Ok(_) => {
            let mut command = Command::new(output);
            let native = run_command_with_timeout(&mut command);
            NativeOutput::Process(native)
        }
        Err(error) => NativeOutput::CompileError(error.to_string()),
    }
}

fn run_command_with_timeout(command: &mut Command) -> ProcessOutput {
    const PROCESS_TIMEOUT: Duration = Duration::from_secs(2);

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + PROCESS_TIMEOUT;
    loop {
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            return ProcessOutput {
                timed_out: false,
                success: output.status.success(),
                stdout: String::from_utf8(output.stdout).unwrap(),
                stderr: String::from_utf8(output.stderr).unwrap(),
            };
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            return ProcessOutput {
                timed_out: true,
                success: false,
                stdout: String::from_utf8(output.stdout).unwrap(),
                stderr: String::from_utf8(output.stderr).unwrap(),
            };
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn native_matches_php(php: &ProcessOutput, native: &NativeOutput) -> bool {
    match native {
        NativeOutput::CompileError(_) => false,
        NativeOutput::Process(native) => {
            !native.timed_out
                && native.success == php.success
                && native.stdout == php.stdout
                && native.stderr == php.stderr
        }
    }
}

fn format_report(stats: &BTreeMap<Category, CategoryStats>, failures: &[String]) -> String {
    let mut lines = Vec::new();
    let total_cases: usize = stats.values().map(|stat| stat.total).sum();
    let total_matched: usize = stats.values().map(|stat| stat.matched).sum();
    lines.push(format!(
        "COW oracle matched {total_matched}/{total_cases} cases against PHP"
    ));
    for category in Category::ALL {
        let stat = stats.get(&category).copied().unwrap_or_default();
        lines.push(format!(
            "{}: {}/{} matched",
            category.name(),
            stat.matched,
            stat.total
        ));
    }
    if !failures.is_empty() {
        lines.push("remaining divergences:".to_string());
        lines.extend(failures.iter().cloned());
    }
    lines.join("\n")
}

fn format_failure(case: &CowCase, php: &ProcessOutput, native: &NativeOutput) -> String {
    match native {
        NativeOutput::CompileError(error) => format!(
            "- {} [{}]: native compile error: {}",
            case.name,
            case.category.name(),
            one_line(error)
        ),
        NativeOutput::Process(native) if native.timed_out => {
            format!(
                "- {} [{}]: native timed out\n  php stdout: {}\n  native stdout before timeout: {}\n  native stderr before timeout: {}",
                case.name,
                case.category.name(),
                one_line(&php.stdout),
                one_line(&native.stdout),
                one_line(&native.stderr)
            )
        }
        NativeOutput::Process(native) => {
            format!(
                "- {} [{}]: native output mismatch\n  php stdout: {}\n  native stdout: {}\n  php stderr: {}\n  native stderr: {}",
                case.name,
                case.category.name(),
                one_line(&php.stdout),
                one_line(&native.stdout),
                one_line(&php.stderr),
                one_line(&native.stderr)
            )
        }
    }
}

fn one_line(value: &str) -> String {
    if value.is_empty() {
        return "<empty>".to_string();
    }
    const LIMIT: usize = 240;

    let escaped = value.replace('\n', "\\n");
    let length = escaped.chars().count();
    if length <= LIMIT {
        return escaped;
    }
    let prefix: String = escaped.chars().take(LIMIT).collect();
    format!("{prefix}... <{length} chars total>")
}

fn assert_baseline(stats: &BTreeMap<Category, CategoryStats>, report: &str) {
    let expected = [
        (Category::Arrays, 2, 2),
        (Category::Strings, 1, 1),
        (Category::Foreach, 2, 2),
        (Category::Functions, 3, 3),
        (Category::NestedValues, 2, 2),
        (Category::References, 2, 0),
    ];

    let mut total_matched = 0;
    for (category, total, minimum_matched) in expected {
        let actual = stats.get(&category).copied().unwrap_or_default();
        total_matched += actual.matched;
        assert_eq!(
            actual.total,
            total,
            "unexpected {} case count\n{}",
            category.name(),
            report
        );
        assert!(
            actual.matched >= minimum_matched,
            "{} COW oracle coverage regressed below {minimum_matched}/{total}\n{}",
            category.name(),
            report
        );
    }
    assert!(
        total_matched >= 10,
        "COW oracle coverage regressed below 10/12\n{}",
        report
    );
}

fn temp_dir(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
