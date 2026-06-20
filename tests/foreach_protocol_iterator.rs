use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ptn::{compile_file, CompileOptions};

#[test]
fn foreach_protocol_iterator_exceptions_rethrow_to_catch() {
    let root = temp_dir("ptn-foreach-protocol-iterator-exceptions");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-protocol-iterator-exceptions.php");
    let output = root.join("foreach-protocol-iterator-exceptions-bin");
    fs::write(
        &input,
        r#"<?php
class IT implements Iterator {
    private $n = 0;
    private $count = 0;
    private $trap = null;

    function __construct($count, $trap = null) {
        $this->count = $count;
        $this->trap = $trap;
    }

    function trap($trap) {
        if ($trap === $this->trap) {
            throw new Exception($trap);
        }
    }

    function rewind(): void  {$this->trap(__FUNCTION__); $this->n = 0;}
    function valid(): bool   {$this->trap(__FUNCTION__); return $this->n < $this->count;}
    function key(): mixed     {$this->trap(__FUNCTION__); return $this->n;}
    function current(): mixed {$this->trap(__FUNCTION__); return $this->n;}
    function next(): void    {$this->trap(__FUNCTION__); $this->n++;}
}

class Agg implements IteratorAggregate {
    function getIterator(): Traversable {
        throw new Exception('getIterator');
    }
}

foreach (['rewind', 'valid', 'key', 'current', 'next'] as $trap) {
    try {
        foreach (new IT(3, $trap) as $key => $val) {
            echo $val, "\n";
        }
    } catch (Exception $e) {
        echo $e->getMessage(), "\n";
    }
}

try {
    foreach (new Agg as $value) {
        echo $value, "\n";
    }
} catch (Exception $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: true }).unwrap();

    let execution = run_with_timeout(&output, Duration::from_secs(8));
    assert!(
        !execution.timed_out,
        "compiled foreach protocol iterator case timed out"
    );
    assert!(execution.status_success, "native process failed");
    assert_eq!(
        execution.stdout,
        concat!(
            "rewind\n",
            "valid\n",
            "key\n",
            "current\n",
            "0\n",
            "next\n",
            "getIterator\n",
        )
    );
    assert_eq!(execution.stderr, "");
}

#[test]
fn foreach_value_only_user_iterator_skips_key_call() {
    let root = temp_dir("ptn-foreach-value-only-user-iterator-key");
    fs::create_dir_all(&root).unwrap();
    let input = root.join("foreach-value-only-user-iterator-key.php");
    let output = root.join("foreach-value-only-user-iterator-key-bin");
    fs::write(
        &input,
        r#"<?php
class IT implements Iterator {
    private int $n = 0;

    function rewind(): void  { echo "rewind\n"; $this->n = 0; }
    function valid(): bool   { echo "valid\n"; return $this->n < 2; }
    function current(): mixed { echo "current\n"; return $this->n; }
    function key(): mixed     { echo "key\n"; return $this->n; }
    function next(): void    { echo "next\n"; $this->n++; }
}

class Agg implements IteratorAggregate {
    function getIterator(): Traversable {
        echo "agg-get\n";
        return new IT();
    }
}

echo "plain\n";
foreach (new IT() as $value) {
    echo "v=$value\n";
}

echo "agg\n";
foreach (new Agg() as $value) {
    echo "a=$value\n";
}

echo "keyed\n";
foreach (new IT() as $key => $value) {
    echo "kv=$key:$value\n";
}
"#,
    )
    .unwrap();

    compile_file(&input, &output, CompileOptions { emit_c: false }).unwrap();

    let execution = run_with_timeout(&output, Duration::from_secs(8));
    assert!(
        !execution.timed_out,
        "compiled foreach value-only iterator case timed out"
    );
    assert!(execution.status_success, "native process failed");
    assert_eq!(
        execution.stdout,
        concat!(
            "plain\n",
            "rewind\n",
            "valid\n",
            "current\n",
            "v=0\n",
            "next\n",
            "valid\n",
            "current\n",
            "v=1\n",
            "next\n",
            "valid\n",
            "agg\n",
            "agg-get\n",
            "rewind\n",
            "valid\n",
            "current\n",
            "a=0\n",
            "next\n",
            "valid\n",
            "current\n",
            "a=1\n",
            "next\n",
            "valid\n",
            "keyed\n",
            "rewind\n",
            "valid\n",
            "current\n",
            "key\n",
            "kv=0:0\n",
            "next\n",
            "valid\n",
            "current\n",
            "key\n",
            "kv=1:1\n",
            "next\n",
            "valid\n",
        )
    );
    assert_eq!(execution.stderr, "");
}

struct TimedOutput {
    timed_out: bool,
    status_success: bool,
    stdout: String,
    stderr: String,
}

fn run_with_timeout(binary: &PathBuf, timeout: Duration) -> TimedOutput {
    let mut child = Command::new(binary)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            child.kill().ok();
            let output = child.wait_with_output().unwrap();
            return TimedOutput {
                timed_out: true,
                status_success: output.status.success(),
                stdout: String::from_utf8(output.stdout).unwrap(),
                stderr: String::from_utf8(output.stderr).unwrap(),
            };
        }
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            return TimedOutput {
                timed_out: false,
                status_success: output.status.success(),
                stdout: String::from_utf8(output.stdout).unwrap(),
                stderr: String::from_utf8(output.stderr).unwrap(),
            };
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn temp_dir(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("{prefix}-{}-{nanos}", std::process::id()));
    path
}
