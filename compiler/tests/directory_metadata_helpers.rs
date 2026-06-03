use std::fs;
use std::path::{Path, PathBuf};

use php_compiler::run_source;

#[test]
fn dir_and_opendir_failed_opens_emit_display_warnings_and_false() {
    let fixture = TempFsFixture::new("failed-opens");
    let file_path = fixture.root.join("plain.txt");
    fs::write(&file_path, "not a directory").expect("fixture file is written");
    let missing_path = fixture.root.join("missing");

    let source = format!(
        r#"<?php
echo "file\n";
var_dump(dir({file_path}));
echo "missing\n";
var_dump(opendir({missing_path}));
"#,
        file_path = php_string(&file_path),
        missing_path = php_string(&missing_path)
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution.stdout.contains("Warning: dir(")
            && execution
                .stdout
                .contains("): Failed to open directory: Not a directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("Warning: opendir(")
            && execution
                .stdout
                .contains("): Failed to open directory: No such file or directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("file\n\nWarning:")
            && execution
                .stdout
                .contains("bool(false)\nmissing\n\nWarning:")
            && execution.stdout.ends_with("bool(false)\n"),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn directory_core_class_rejects_direct_clone_serialize_and_readonly_mutation() {
    let fixture = TempFsFixture::new("core-class-lifecycle");
    let source = format!(
        r#"<?php
$d = dir({dir_path});

try {{
    new Directory();
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}

try {{
    clone $d;
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}

try {{
    serialize($d);
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}

try {{
    $d->path = "Havoc!";
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}
var_dump($d->path === {dir_path});

$ao = new ArrayObject($d);
$ao['handle'] = STDERR;
try {{
    $d->read();
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}
unset($ao['handle']);
try {{
    var_dump($d->handle);
}} catch (Throwable $e) {{
    echo $e::class, ": ", $e->getMessage(), "\n";
}}
"#,
        dir_path = php_string(&fixture.root)
    );

    let execution = run_source(&source).unwrap();

    assert!(
        execution
            .stdout
            .contains("Error: Cannot directly construct Directory, use dir() instead"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Error: Trying to clone an uncloneable object of class Directory"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Exception: Serialization of 'Directory' is not allowed"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Error: Cannot modify readonly property Directory::$path"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("bool(true)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Deprecated: ArrayObject::__construct(): Using an object as a backing array for ArrayObject is deprecated"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Error: Internal directory stream has been altered"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Error: Typed property Directory::$handle must not be accessed before initialization"
        ),
        "{}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn directory_reflection_metadata_and_constructor_bypass_match_internal_shape() {
    let execution = run_source(
        r#"<?php
$rc = new ReflectionClass("Directory");
var_dump($rc->isFinal());
var_dump($rc->isCloneable());
var_dump($rc->isInstantiable());
echo $rc;
$d = $rc->newInstanceWithoutConstructor();
var_dump($d);
try {
    $d->read();
} catch (Throwable $e) {
    echo $e::class, ": ", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .starts_with("bool(true)\nbool(false)\nbool(true)\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Class [ <internal:standard> final class Directory ] {"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Property [ public protected(set) readonly string $path ]"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("      - Return [ string|false ]\n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("uninitialized(string)\n  [\"handle\"]=>\n  uninitialized(mixed)"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Error: Internal directory stream has been altered"),
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
            "phpc-directory-metadata-{label}-{}-{}",
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
