use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn fscanf_covers_local_file_and_memory_scan_rows() {
    let fixture = TempFsFixture::new("scanf");
    let path = php_string(&fixture.root.join("data.txt"));
    let source = format!(
        r#"<?php
$path = {path};
$file = fopen($path, "w+");
fwrite($file, "alpha 0007 2.5\n");
rewind($file);
$row = fscanf($file, "%s %d %f");
var_dump($row);
rewind($file);
$assigned = fscanf($file, "%s %d %f", $word, $int, $float);
var_dump($assigned, $word, $int, $float);
fclose($file);

$memory = fopen("php://memory", "w+");
fwrite($memory, "abc-123\n");
rewind($memory);
var_dump(fscanf($memory, "%[a-z]-%d"));
"#,
        path = path
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "array(3) {\n",
            "  [0]=>\n",
            "  string(5) \"alpha\"\n",
            "  [1]=>\n",
            "  int(7)\n",
            "  [2]=>\n",
            "  float(2.5)\n",
            "}\n",
            "int(3)\n",
            "string(5) \"alpha\"\n",
            "int(7)\n",
            "float(2.5)\n",
            "array(2) {\n",
            "  [0]=>\n",
            "  string(3) \"abc\"\n",
            "  [1]=>\n",
            "  int(123)\n",
            "}\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn fscanf_reports_value_type_errors_and_eof_false() {
    let source = r#"<?php
$stream = fopen("php://memory", "w+");
fwrite($stream, "data\n");
rewind($stream);
try {
    fscanf($stream, "%s%d", $word);
} catch (ValueError $e) {
    echo $e->getMessage() . "|";
}
try {
    fscanf($stream, "%.a");
} catch (ValueError $e) {
    echo $e->getMessage() . "|";
}
fclose($stream);
try {
    fscanf($stream, "%s");
} catch (TypeError $e) {
    echo $e->getMessage() . "|";
}
$empty = fopen("php://memory", "w+");
var_dump(fscanf($empty, "%s"));
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "Different numbers of variable names and field specifiers|",
            "Bad scan conversion character \".\"|",
            "fscanf(): supplied resource is not a valid File-Handle resource|",
            "bool(false)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-standard-file-{label}-{}-{}",
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
    let escaped = path
        .display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("\"{escaped}\"")
}
