use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use php_compiler::run_source;

#[test]
fn open_basedir_dot_denies_relative_parent_escapes_for_metadata_helpers() {
    let _cwd_guard = cwd_guard();
    let fixture = OpenBaseDirFixture::new("metadata");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$old = getcwd();
$root = {root};
chdir($root . "/test/ok");
ini_set("open_basedir", ".");

$basedir = 0;
$other = 0;
function capture_open_basedir_warning($errno, $errstr) {{
    global $basedir, $other;
    if (str_contains($errstr, "open_basedir restriction in effect")) {{
        $basedir++;
    }} else {{
        $other++;
    }}
    return true;
}}
set_error_handler("capture_open_basedir_warning", E_WARNING);

$denied = [
    "../bad",
    "../bad/bad.txt",
    "..",
    "../",
    "/",
    "../bad/.",
    $root . "/test/bad/bad.txt",
    $root . "/test/bad/../bad/bad.txt",
    "./../.",
];
$allowed = [
    "../ok",
    "ok.txt",
    "../ok/ok.txt",
    $root . "/test/ok/ok.txt",
    $root . "/test/ok/../ok/ok.txt",
];
$functions = [
    "file_exists",
    "filesize",
    "is_dir",
    "is_file",
    "is_readable",
    "is_writable",
    "is_link",
];

$false = 0;
$unexpected = 0;
foreach ($functions as $fn) {{
    foreach ($denied as $path) {{
        $value = $fn($path);
        if ($value === false) {{
            $false++;
        }} else {{
            $unexpected++;
        }}
    }}
    foreach ($allowed as $path) {{
        $fn($path);
    }}
}}

restore_error_handler();
ini_set("open_basedir", "");
chdir($old);

echo "false=$false;unexpected=$unexpected;basedir=$basedir;other=$other";
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(execution.stdout, "false=63;unexpected=0;basedir=63;other=0");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn open_basedir_denied_stream_and_directory_opens_emit_followup_warnings() {
    let _cwd_guard = cwd_guard();
    let fixture = OpenBaseDirFixture::new("opens");
    let root = php_string(&fixture.root);
    let source = format!(
        r#"<?php
$old = getcwd();
$root = {root};
chdir($root . "/test/ok");
ini_set("open_basedir", ".");

$basedir = 0;
$stream = 0;
$directory = 0;
$errno = 0;
$other = 0;
function capture_open_basedir_open_warning($errno_value, $errstr) {{
    global $basedir, $stream, $directory, $errno, $other;
    if (str_contains($errstr, "open_basedir restriction in effect")) {{
        $basedir++;
    }} elseif (str_contains($errstr, "Failed to open stream")) {{
        $stream++;
    }} elseif (str_contains($errstr, "Failed to open directory")) {{
        $directory++;
    }} elseif (str_contains($errstr, "(errno 1)")) {{
        $errno++;
    }} else {{
        $other++;
    }}
    return true;
}}
set_error_handler("capture_open_basedir_open_warning", E_WARNING);

$false = 0;
$false += file_get_contents("../bad/bad.txt") === false ? 1 : 0;
$false += file_put_contents("../bad/new.txt", "payload") === false ? 1 : 0;
$false += fopen("../bad/bad.txt", "r") === false ? 1 : 0;
$false += opendir("../bad") === false ? 1 : 0;
$false += scandir("../bad") === false ? 1 : 0;

$allowed = 0;
$allowed += file_get_contents("ok.txt") === "Hello World!" ? 1 : 0;
$stream_handle = fopen("../ok/ok.txt", "r");
$allowed += is_resource($stream_handle) ? 1 : 0;
if (is_resource($stream_handle)) {{
    fclose($stream_handle);
}}
$dir_handle = opendir("../ok");
$allowed += is_resource($dir_handle) ? 1 : 0;
if (is_resource($dir_handle)) {{
    closedir($dir_handle);
}}
$entries = scandir($root . "/test/ok/../ok");
$allowed += is_array($entries) && in_array("ok.txt", $entries, true) ? 1 : 0;

restore_error_handler();
ini_set("open_basedir", "");
chdir($old);

echo "false=$false;allowed=$allowed;basedir=$basedir;stream=$stream;directory=$directory;errno=$errno;other=$other";
"#,
        root = root
    );

    let execution = run_source(&source).unwrap();

    assert_eq!(
        execution.stdout,
        "false=5;allowed=4;basedir=5;stream=3;directory=2;errno=1;other=0"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

struct OpenBaseDirFixture {
    root: PathBuf,
}

impl OpenBaseDirFixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phpc-open-basedir-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time is after Unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("test/ok")).expect("ok directory is created");
        fs::create_dir_all(root.join("test/bad")).expect("bad directory is created");
        fs::write(root.join("test/ok/ok.txt"), "Hello World!").expect("ok file is written");
        fs::write(root.join("test/bad/bad.txt"), "Nope").expect("bad file is written");
        Self { root }
    }
}

impl Drop for OpenBaseDirFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn php_string(path: &Path) -> String {
    let value = path.to_str().expect("temporary path is valid UTF-8");
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

struct CwdGuard {
    _lock: MutexGuard<'static, ()>,
    original: PathBuf,
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

fn cwd_guard() -> CwdGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cwd lock is not poisoned");
    let original = std::env::current_dir().expect("current directory is available");
    CwdGuard {
        _lock: lock,
        original,
    }
}
