use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn fflush_and_ftruncate_cover_local_and_memory_streams() {
    let fixture = TempFsFixture::new("flush-truncate");
    let path = php_string(&fixture.root.join("stream.txt"));
    let source = format!(
        r#"<?php
$path = {path};
$h = fopen($path, "w+");
echo fwrite($h, "Hello");
echo ":" . (fflush($h) ? "flushed" : "bad");
echo ":" . file_get_contents($path);
echo ":" . (ftruncate($h, 2) ? "truncated" : "bad");
echo ":" . ftell($h);
echo ":" . fwrite($h, "World");
fclose($h);
echo ":" . bin2hex(file_get_contents($path));
$r = fopen($path, "r");
echo ":" . (ftruncate($r, 1) ? "bad" : "readonly");
fclose($r);
$m = fopen("php://memory", "w+");
fwrite($m, "Hello");
echo ":" . (ftruncate($m, 2) ? "mem" : "bad");
echo ":" . ftell($m);
fwrite($m, "World");
rewind($m);
echo ":" . stream_get_contents($m);
fclose($m);
try {{
    ftruncate(fopen("php://memory", "w+"), -1);
}} catch (ValueError $e) {{
    echo ":" . $e->getMessage();
}}
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
            "5:flushed:Hello:truncated:5:5:4865000000576f726c64",
            ":readonly:mem:2:HeWorld",
            ":ftruncate(): Argument #2 ($size) must be greater than or equal to 0",
        )
    );
}

#[test]
fn directory_separator_constant_matches_host_paths() {
    let execution = run_source(
        r#"<?php
echo DIRECTORY_SEPARATOR, "|", defined("DIRECTORY_SEPARATOR") ? "defined" : "missing";
echo "|", constant("DIRECTORY_SEPARATOR");
"#,
    )
    .unwrap();

    let separator = std::path::MAIN_SEPARATOR.to_string();
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert_eq!(execution.stdout, format!("{separator}|defined|{separator}"));
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
    let value = path.to_str().expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}
