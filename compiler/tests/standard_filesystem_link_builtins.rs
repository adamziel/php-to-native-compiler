#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn symlink_readlink_linkinfo_and_hardlink_cover_standard_file_cluster() {
    let fixture = TempFsFixture::new("links");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$target = $root . "/target.txt";
$soft = $root . "/soft-link.txt";
$hard = $root . "/hard-link.txt";
file_put_contents($target, "payload");
var_dump(symlink($target, $soft));
echo (is_link($soft) ? "is-link" : "not-link") . "\n";
echo (readlink($soft) === $target ? "readlink" : "bad-readlink") . "\n";
echo gettype(linkinfo($soft)) . "\n";
var_dump(link($target, $hard));
echo file_get_contents($hard);
unlink($soft);
unlink($hard);
unlink($target);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "is-link\n",
            "readlink\n",
            "integer\n",
            "bool(true)\n",
            "payload",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filesystem_link_failures_emit_inline_warnings_and_prerequisites() {
    let fixture = TempFsFixture::new("warnings");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$file = $root . "/file.txt";
$link = $root . "/link.txt";
var_dump(touch($file));
var_dump(symlink($file, $link));
echo readlink($link) === $file ? "readlink-ok\n" : "readlink-bad\n";
unlink($link);
var_dump(readlink($link));
var_dump(linkinfo($link));
touch($link);
var_dump(symlink($file, $link));
unlink($link);
unlink($file);
var_dump(link($file, $link));
var_dump(link(false, $link));
var_dump(is_link(false));
try {{
    linkinfo("");
}} catch (ValueError $e) {{
    echo $e->getMessage(), "\n";
}}
var_dump(readlink(12.5));
@unlink($link);
@rmdir($root . "/missing-dir");
echo "suppressed-cleanup\n";
var_dump(sleep(0));
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution
            .stdout
            .contains("bool(true)\nbool(true)\nreadlink-ok\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: readlink(): No such file or directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: linkinfo(): No such file or directory"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("int(-1)"), "{}", execution.stdout);
    assert!(
        execution.stdout.contains("suppressed-cleanup\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("Warning: symlink(): File exists"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: link(): No such file or directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("linkinfo(): Argument #1 ($path) must not be empty"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.ends_with("int(0)\n"),
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
            "phpc-standard-file-link-{label}-{}-{}",
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
