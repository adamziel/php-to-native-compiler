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
fn scandir_invalid_sorting_order_is_catchable_value_error() {
    let source = r#"<?php
try {
    scandir(".", -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "scandir(): Argument #2 ($sorting_order) must be one of the SCANDIR_SORT_ASCENDING, SCANDIR_SORT_DESCENDING, or SCANDIR_SORT_NONE constants"
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
