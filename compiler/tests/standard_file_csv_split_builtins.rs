use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn file_reads_local_lines_with_flags_and_fputs_alias() {
    let fixture = TempFsFixture::new("file-lines");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$path = $root . "/lines.txt";
$h = fopen($path, "w+");
fputs($h, "alpha\n\nbeta\nlast");
fclose($h);
$plain = file($path);
$trimmed = file($path, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
echo count($plain) . ":" . $plain[0] . ":" . $plain[1] . ":" . $plain[3];
echo "|" . count($trimmed) . ":" . implode(",", $trimmed);
unlink($path);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "4:alpha\n:\n:last|3:alpha,beta,last");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fputcsv_writes_quoted_records_with_named_options() {
    let fixture = TempFsFixture::new("fputcsv");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$path = $root . "/output.csv";
$w = fopen($path, "w+");
echo fputcsv($w, array("a,b", "c\"d", "tail"), escape: "") . "|";
rewind($w);
echo stream_get_contents($w);
fclose($w);
unlink($path);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "18|\"a,b\",\"c\"\"d\",tail\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fputcsv_encloses_fields_containing_the_active_escape_character() {
    let execution = run_source(
        r##"<?php
$h = fopen("php://memory", "w+");
echo fputcsv($h, array("a\\b")) . "|";
rewind($h);
echo stream_get_contents($h);
$custom = fopen("php://memory", "w+");
echo "|";
echo fputcsv($custom, array("a#b"), escape: "#") . "|";
rewind($custom);
echo stream_get_contents($custom);
$disabled = fopen("php://memory", "w+");
echo "|";
echo fputcsv($disabled, array("a\\b"), escape: "") . "|";
rewind($disabled);
echo stream_get_contents($disabled);
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "6|\"a\\b\"\n|6|\"a#b\"\n|4|a\\b\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fputcsv_memory_stream_sparse_writes_pad_with_nul_bytes() {
    let execution = run_source(
        r#"<?php
$h = fopen("php://memory", "w+");
fwrite($h, "A");
fseek($h, 4);
fputcsv($h, array("Z"));
rewind($h);
echo bin2hex(stream_get_contents($h));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "410000005a0a");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-standard-file-csv-split-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is after Unix epoch")
                .as_nanos()
        ));
        fs::create_dir(&root).expect("temporary filesystem fixture root is created");
        Self { root }
    }
}

impl Drop for TempFsFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn php_string(path: &Path) -> String {
    let value = path.to_str().expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
