use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

struct Case {
    name: &'static str,
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

#[test]
fn foreach_by_reference_cow_oracle_suite() {
    let root = temp_dir("ptn-foreach-by-ref-cow");
    fs::create_dir_all(&root).unwrap();

    let mut matched = 0;
    let mut failures = Vec::new();
    let mut categories = BTreeMap::new();

    for case in CASES {
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
        if native_matches_php(&php, &native) {
            matched += 1;
        } else {
            failures.push(format_failure(case.name, &php, &native));
        }
        let category = case
            .name
            .split_once('_')
            .map(|(category, _)| category)
            .unwrap();
        let entry = categories.entry(category).or_insert((0, 0));
        entry.0 += 1;
        if native_matches_php(&php, &native) {
            entry.1 += 1;
        }
    }

    let report = format_report(matched, CASES.len(), &categories, &failures);
    eprintln!("{report}");
    assert_eq!(
        matched,
        CASES.len(),
        "by-reference foreach COW oracle regressions\n{report}"
    );
}

const CASES: &[Case] = &[
    Case {
        name: "direct_mutates_source_slots",
        source: r#"<?php
$items = [1, 2];
foreach ($items as &$item) {
    $item += 10;
}
unset($item);
var_dump($items);
"#,
    },
    Case {
        name: "direct_key_value_mutates_source_slots",
        source: r#"<?php
$items = ["a" => 1, "b" => 2];
foreach ($items as $key => &$item) {
    echo $key, "=", $item, "\n";
    $item += 10;
}
unset($item);
var_dump($items);
"#,
    },
    Case {
        name: "cow_detaches_shared_source",
        source: r#"<?php
$items = [1, 2];
$copy = $items;
foreach ($items as &$item) {
    $item += 10;
}
unset($item);
var_dump($copy);
var_dump($items);
"#,
    },
    Case {
        name: "live_appends_are_visited",
        source: r#"<?php
$items = [1, 2];
$seen = 0;
foreach ($items as &$item) {
    echo $item, "\n";
    if ($seen < 2) {
        $items[] = $item + 10;
    }
    $seen += 1;
}
unset($item);
var_dump($items);
"#,
    },
    Case {
        name: "live_unset_shifts_iteration",
        source: r#"<?php
$items = [1, 2, 3];
foreach ($items as $key => &$item) {
    echo $key, "=", $item, "\n";
    if ($item === 1) {
        unset($items[1]);
    }
}
unset($item);
var_dump($items);
"#,
    },
    Case {
        name: "nested_array_dim_iterable_mutates_source",
        source: r#"<?php
$outer = [[1, 2]];
foreach ($outer[0] as &$item) {
    $item += 10;
}
unset($item);
echo $outer[0][0], ":", $outer[0][1], "\n";
"#,
    },
    Case {
        name: "temporary_literal_keeps_last_reference_value",
        source: r#"<?php
foreach ([1, 2] as &$item) {
    $item += 10;
}
var_dump($item);
$item = 99;
var_dump($item);
"#,
    },
    Case {
        name: "alias_unset_loop_variable_breaks_last_alias",
        source: r#"<?php
$items = [1, 2];
foreach ($items as &$item) {
}
unset($item);
$item = 99;
var_dump($items);
var_dump($item);
"#,
    },
    Case {
        name: "alias_without_unset_keeps_last_alias",
        source: r#"<?php
$items = [1, 2];
foreach ($items as &$item) {
}
$item = 99;
var_dump($items);
"#,
    },
    Case {
        name: "reference_preexisting_element_alias_survives",
        source: r#"<?php
$items = [1, 2];
$ref =& $items[0];
foreach ($items as &$item) {
    $item += 5;
}
unset($item);
var_dump($items[0], $ref, $items[1]);
"#,
    },
    Case {
        name: "nested_shared_rows_detach_through_reference",
        source: r#"<?php
$items = [["v" => 1], ["v" => 2]];
$copy = $items;
foreach ($items as &$row) {
    $row["v"] += 10;
}
unset($row);
var_dump($items);
var_dump($copy);
"#,
    },
    Case {
        name: "nested_unset_current_and_future_by_ref_iterators",
        source: r#"<?php
$a = [0, 1, 2, 3];
foreach ($a as &$x) {
    foreach ($a as &$y) {
        echo $x, " - ", $y, "\n";
        if ($x == 0 && $y == 1) {
            unset($a[2]);
            unset($a[1]);
        }
    }
}
"#,
    },
    Case {
        name: "function_child_rekey_by_reference",
        source: r#"<?php
$arr = [
    "a" => [
        "a" => "apple",
        "b" => "banana",
        "c" => "cranberry",
        "d" => "mango",
        "e" => "pineapple",
    ],
    "b" => [
        "a" => "apple",
        "b" => "banana",
        "c" => "cranberry",
        "d" => "mango",
        "e" => "pineapple",
    ],
    "c" => "cranberry",
    "d" => "mango",
    "e" => "pineapple",
];

function test_child_rekey(&$child) {
    $i = 1;
    foreach ($child as $key => $fruit) {
        if (!is_numeric($key)) {
            $child[$i] = $fruit;
            unset($child[$key]);
            $i++;
        }
    }
}

$i = 1;
foreach ($arr as $key => $fruit) {
    $arr[$i] = $fruit;
    if (is_array($fruit)) {
        test_child_rekey($arr[$i]);
    }
    unset($arr[$key]);
    $i++;
}

var_dump($arr);
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
            NativeOutput::Process(run_command_with_timeout(&mut command))
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

fn format_report(
    matched: usize,
    total: usize,
    categories: &BTreeMap<&str, (usize, usize)>,
    failures: &[String],
) -> String {
    let mut lines = vec![format!(
        "by-reference foreach COW oracle matched {matched}/{total} cases against PHP"
    )];
    for (category, (total, matched)) in categories {
        lines.push(format!("{category}: {matched}/{total} matched"));
    }
    if !failures.is_empty() {
        lines.push("remaining divergences:".to_string());
        lines.extend(failures.iter().cloned());
    }
    lines.join("\n")
}

fn format_failure(name: &str, php: &ProcessOutput, native: &NativeOutput) -> String {
    match native {
        NativeOutput::CompileError(error) => {
            format!("- {name}: native compile error: {}", one_line(error))
        }
        NativeOutput::Process(native) if native.timed_out => {
            format!(
                "- {name}: native timed out\n  php stdout: {}\n  native stdout before timeout: {}\n  native stderr before timeout: {}",
                one_line(&php.stdout),
                one_line(&native.stdout),
                one_line(&native.stderr)
            )
        }
        NativeOutput::Process(native) => {
            format!(
                "- {name}: native output mismatch\n  php stdout: {}\n  native stdout: {}\n  php stderr: {}\n  native stderr: {}",
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
    let escaped = value.replace('\n', "\\n");
    if escaped.len() <= 240 {
        escaped
    } else {
        format!("{}... <{} bytes total>", &escaped[..240], escaped.len())
    }
}

fn temp_dir(name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{}-{now}", std::process::id()))
}
