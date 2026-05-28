use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn filesystem_mutation_builtins_cover_local_file_directory_flow() {
    let fixture = TempFsFixture::new("flow");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$sub = $root . "/sub";
$copy = $sub . "/copy.txt";
$renamed = $sub . "/renamed.txt";
var_dump(mkdir($sub));
echo "|";
echo file_put_contents($sub . "/data.txt", "A");
echo ":";
echo file_put_contents($sub . "/data.txt", ["B", ["nested"], "C"], FILE_APPEND | LOCK_EX);
echo ":";
echo file_get_contents($sub . "/data.txt");
echo "|";
$count = readfile($sub . "/data.txt");
echo ":" . $count;
echo "|";
var_dump(copy($sub . "/data.txt", $copy));
var_dump(rename($copy, $renamed));
echo implode(",", scandir($sub, SCANDIR_SORT_DESCENDING));
echo "|";
var_dump(unlink($sub . "/data.txt"));
var_dump(unlink($renamed));
var_dump(rmdir($sub));
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "|1:7:ABArrayC|ABArrayC:8|",
            "bool(true)\n",
            "bool(true)\n",
            "renamed.txt,data.txt,..,.|",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
    assert!(
        execution
            .stderr
            .contains("PHP Warning:  file_put_contents(): Array to string conversion"),
        "{}",
        execution.stderr
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chdir_and_scandir_constants_are_available_to_runtime_and_metadata() {
    let fixture = TempFsFixture::new("cwd");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$old = getcwd();
$root = {root};
mkdir($root . "/cwd");
chdir($root . "/cwd");
file_put_contents("relative.txt", "cwd-data");
echo file_exists("relative.txt") ? "exists" : "missing";
echo ":" . file_get_contents("relative.txt");
echo ":" . (function_exists("file_put_contents") ? "fn" : "missing");
echo ":" . FILE_APPEND . ":" . SCANDIR_SORT_DESCENDING;
unlink("relative.txt");
chdir($old);
rmdir($root . "/cwd");
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "exists:cwd-data:fn:8:1");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn filesystem_stat_string_and_resource_boundaries_cover_file_dir_phpt_cluster() {
    let fixture = TempFsFixture::new("stat");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$root = {root};
$path = $root . "/mode.txt";
$handle = fopen($path, "x+");
var_dump($handle != false);
fwrite($handle, "abc");
fclose($handle);
var_dump(strstr("w+b", "x"));
var_dump(strstr("x+b", "x"));
echo strstr("abc", "b", true) . ":" . stristr("AbC", "b");
var_dump(chmod($path, 0644));
$stat = stat($path);
echo ":" . (($stat[2] == $stat["mode"]) ? "stat" : "bad");
echo ":";
printf("%o", fileperms($path) & 0777);
$dir = $root . "/dir";
mkdir($dir, 0777);
echo ":";
printf("%o", fileperms($dir) & 0777);
$entries = scandir($root, SCANDIR_SORT_DESCENDING, stream_context_create());
echo ":" . $entries[0];
try {{
    file_put_contents($path, stream_context_create());
}} catch (TypeError $e) {{
    echo ":" . $e->getMessage();
}}
$stream = fopen($path, "r");
try {{
    file_put_contents($path, "x", 0, $stream);
}} catch (TypeError $e) {{
    echo ":" . $e->getMessage();
}}
fclose($stream);
unlink($path);
rmdir($dir);
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "bool(false)\n",
            "string(3) \"x+b\"\n",
            "a:bC",
            "bool(true)\n",
            ":stat:644:777:mode.txt",
            ":file_put_contents(): supplied resource is not a valid stream resource",
            ":file_put_contents(): supplied resource is not a valid Stream-Context resource",
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
            "phpc-standard-fs-{label}-{}-{}",
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
