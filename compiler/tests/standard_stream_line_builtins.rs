use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn fgetc_and_fgets_cover_local_file_and_memory_line_reads() {
    let fixture = TempFsFixture::new("line-reads");
    let path = php_string(&fixture.root.join("lines.txt"));
    let source = format!(
        r#"<?php
$path = {path};
file_put_contents($path, "alpha\nbeta\ngamma");
$handle = fopen($path, "r");
echo fgetc($handle);
echo fgetc($handle);
echo "\n";
echo fgets($handle);
echo fgets($handle, 3) . "\n";
echo fgets($handle);
echo fgets($handle) . "\n";
var_dump(fgets($handle));
fclose($handle);
$memory = fopen("php://memory", "w+");
fwrite($memory, "x\ny");
rewind($memory);
echo fgetc($memory) . "\n";
echo fgets($memory);
echo fgets($memory);
var_dump(fgets($memory));
fclose($memory);
unlink($path);
"#,
        path = path
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(
        execution.stdout,
        concat!(
            "al\n",
            "pha\n",
            "be\n",
            "ta\n",
            "gamma\n",
            "bool(false)\n",
            "x\n",
            "\n",
            "y",
            "bool(false)\n",
        )
    );
}

#[test]
fn fgetcsv_parses_memory_and_file_records_with_named_escape() {
    let fixture = TempFsFixture::new("csv-line-reads");
    let path = php_string(&fixture.root.join("records.csv"));
    let source = format!(
        r#"<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "\"water\",fruit\n=water===fruit=\n-water---fruit---air-\n");
rewind($memory);
$row = fgetcsv($memory, 1024, ",", "\"", escape: "\\");
echo $row[0] . ":" . $row[1] . ":" . ftell($memory) . ":" . (feof($memory) ? "eof" : "more") . "\n";
$row = fgetcsv($memory, 1024, "=", "=", escape: "\\");
echo $row[0] . ":" . $row[1] . ":" . ftell($memory) . "\n";
$row = fgetcsv($memory, 1024, "-", "-", escape: "\\");
echo $row[0] . ":" . $row[1] . ":" . $row[2] . ":" . ftell($memory) . "\n";
var_dump(fgetcsv($memory));
fclose($memory);
$escaped = fopen("php://memory", "w+");
fwrite($escaped, "\"a\\\"b\",tail\n");
rewind($escaped);
$row = fgetcsv($escaped, 0, ",", "\"", escape: "\\");
echo $row[0] . ":" . $row[1] . ":" . ftell($escaped) . "\n";
fclose($escaped);
$path = {path};
file_put_contents($path, "^alpha^ ^beta^\n");
$file = fopen($path, "r");
$row = fgetcsv($file, 1024, " ", "^", escape: "\\");
echo $row[0] . ":" . $row[1] . ":" . ftell($file) . "\n";
fclose($file);
unlink($path);
"#,
        path = path
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(
        execution.stdout,
        concat!(
            "water:fruit:14:more\n",
            "water=fruit::30\n",
            "water-fruit:air::52\n",
            "bool(false)\n",
            "a\\\"b:tail:12\n",
            "alpha:beta:15\n",
        )
    );
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-standard-stream-{label}-{}-{}",
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
