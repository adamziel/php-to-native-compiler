#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn touch_updates_file_and_directory_timestamps_and_rejects_missing_trailing_directory() {
    let fixture = TempFsFixture::new("timestamps");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$file = $root . "/file.txt";
$dir = $root . "/dir";
$new = $root . "/new.txt";
file_put_contents($file, "payload");
mkdir($dir);
var_dump(touch($file, 10, 11));
clearstatcache(true, $file);
echo filemtime($file), ":", fileatime($file), "\n";
echo file_get_contents($file), "\n";
var_dump(touch($dir, 20, 21));
clearstatcache(true, $dir);
echo filemtime($dir), ":", fileatime($dir), "\n";
var_dump(touch($new, 30));
clearstatcache(true, $new);
echo filemtime($new), ":", fileatime($new), "\n";
var_dump(touch($root . "/missing/"));
var_dump(file_exists($root . "/missing/"));
try {{
    touch($file, null, 40);
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
unlink($new);
unlink($file);
rmdir($dir);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution
            .stdout
            .contains("Warning: touch(): Unable to create file"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("because Is a directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.starts_with(concat!(
            "bool(true)\n",
            "10:11\n",
            "payload\n",
            "bool(true)\n",
            "20:21\n",
            "bool(true)\n",
            "30:30\n",
            "\n",
            "Warning: touch(): Unable to create file "
        )),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(&format!(
            "{}/missing/ because Is a directory",
            fixture.root.to_str().unwrap()
        )),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with(concat!(
            "bool(false)\n",
            "bool(false)\n",
            "touch(): Argument #2 ($mtime) cannot be null when argument #3 ($atime) is an integer\n"
        )),
        "{}",
        execution.stdout
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
            "phpc-standard-file-touch-{label}-{}-{}",
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
