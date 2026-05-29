use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn directory_handles_track_last_opened_closed_state_and_dir_object_methods() {
    let fixture = TempFsFixture::new("dir");
    let root = php_string(&fixture.root);
    fs::write(fixture.root.join("alpha.txt"), "alpha").expect("alpha file is written");
    fs::write(fixture.root.join("beta.txt"), "beta").expect("beta file is written");

    let source = format!(
        r#"<?php
error_reporting(-1);
$root = {root};
$dh = opendir($root);
echo function_exists("dir") ? "dir-fn" : "missing-dir";
echo ":" . (function_exists("glob") ? "glob-fn" : "missing-glob");
echo ":" . (is_resource($dh) ? "open" : "closed");
echo ":" . readdir($dh);
echo ":" . readdir();
rewinddir();
echo ":" . readdir();
closedir();
echo ":" . (is_resource($dh) ? "open" : "closed");
var_dump($dh);
try {{
    closedir($dh);
}} catch (TypeError $e) {{
    echo "ERR:" . $e->getMessage();
}}
$dir = dir($root);
echo "|obj:" . get_class($dir);
echo ":" . ($dir->read() !== false ? "read" : "eof");
var_dump($dir->rewind());
echo ":" . ($dir->read() !== false ? "rewound" : "eof");
var_dump($dir->close());
var_dump($dir);
try {{
    $dir->read();
}} catch (TypeError $e) {{
    echo "OBJERR:" . $e->getMessage();
}}
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.contains("dir-fn:glob-fn:open:.:"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(":closedresource(")
            || execution.stdout.contains(":closed\nresource("),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("of type (Unknown)"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("ERR:closedir(): Argument #1 ($dir_handle) must be an open stream resource"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("|obj:Directory:readNULL\n:rewoundNULL\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "OBJERR:Directory::read(): cannot use Directory resource after it has been closed"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stderr
            .contains("PHP Deprecated:  readdir: readdir(): Passing null is deprecated"),
        "{}",
        execution.stderr
    );
    assert!(
        execution
            .stderr
            .contains("PHP Deprecated:  rewinddir: rewinddir(): Passing null is deprecated"),
        "{}",
        execution.stderr
    );
    assert!(
        execution
            .stderr
            .contains("PHP Deprecated:  closedir: closedir(): Passing null is deprecated"),
        "{}",
        execution.stderr
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn glob_matches_local_patterns_relative_paths_flags_and_open_basedir_filtering() {
    let fixture = TempFsFixture::new("glob");
    let root = php_string(&fixture.root);
    let glob_dir = fixture.root.join("glob_test");
    fs::create_dir(&glob_dir).expect("glob fixture directory is created");
    fs::write(glob_dir.join("file.text"), "file").expect("file.text is written");
    fs::write(glob_dir.join("wonder.txt"), "wonder").expect("wonder.txt is written");
    fs::write(glob_dir.join("wonder12345"), "digits").expect("wonder12345 is written");
    fs::create_dir(fixture.root.join("ok")).expect("ok directory is created");
    fs::create_dir(fixture.root.join("bad")).expect("bad directory is created");

    let source = format!(
        r#"<?php
$root = {root};
$dir = $root . "/glob_test";
$all = glob($dir . "/*");
sort($all);
echo count($all);
echo ":" . (in_array($dir . "/file.text", $all, true) ? "file" : "missing");
echo ":" . (in_array($dir . "/wonder.txt", $all, true) ? "txt" : "missing");
echo ":" . (in_array($dir . "/wonder12345", $all, true) ? "digits" : "missing");
echo ":" . count(glob($dir . "/*.txt"));
echo ":" . count(glob($dir . "/*.t?t"));
echo ":" . count(glob($dir . "/*.t*t"));
echo ":" . count(glob($dir . "/*.?"));
echo ":" . count(glob($dir . "/*.none"));
$marked = glob($dir, GLOB_MARK);
echo ":" . $marked[0];
$old = getcwd();
chdir($dir);
$relative = glob("./*");
echo ":" . $relative[0];
chdir($root . "/ok");
ini_set("open_basedir", ".");
var_dump(glob("../bad"));
$filtered = glob("../*");
sort($filtered);
var_dump($filtered);
ini_set("open_basedir", "");
chdir($old);
try {{
    glob($dir . "/*", 8);
}} catch (ValueError $e) {{
    echo "ERR:" . $e->getMessage();
}}
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.contains("3:file:txt:digits:1:1:2:0:0:"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("/glob_test/:./file.text"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(5) \"../ok\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("ERR:glob(): Argument #2 ($flags) must be a valid flag value"),
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
            "phpc-standard-dir-glob-{label}-{}-{}",
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
