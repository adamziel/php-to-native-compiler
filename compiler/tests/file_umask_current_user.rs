#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn umask_tracks_request_local_mask_and_applies_to_created_paths() {
    let fixture = TempFsFixture::new("umask");
    let source = format!(
        r#"<?php
function mode_tail($path) {{
    return substr(sprintf("%o", fileperms($path)), -3);
}}

$root = {root};
$first = umask(0022);
echo gettype($first), "\n";
echo umask(), "\n";

umask(0077);
$file = $root . "/created-by-fopen.txt";
$dir = $root . "/created-dir";
$fp = fopen($file, "w");
fclose($fp);
mkdir($dir);
echo mode_tail($file), "|", mode_tail($dir), "\n";

umask(0002);
$file2 = $root . "/created-by-file-put-contents.txt";
$dir2 = $root . "/created-dir-two";
file_put_contents($file2, "payload");
mkdir($dir2, 0777);
echo mode_tail($file2), "|", mode_tail($dir2), "\n";

$call = "umask";
echo $call(), "\n";
echo (new ReflectionFunction("umask"))->getParameters()[0]->getName();
"#,
        root = php_string(&fixture.root),
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "integer\n18\n600|700\n664|775\n2\nmask");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

struct TempFsFixture {
    root: PathBuf,
}

impl TempFsFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-file-{label}-{}-{}",
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
