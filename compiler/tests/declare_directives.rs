use php_compiler::{run_source, run_source_with_source_file};

#[test]
fn declare_strict_types_and_encoding_statements_are_noop() {
    let execution = run_source(
        r#"<?php
declare(strict_types=1);
declare(encoding="ISO-8859-1");
namespace Demo;
var_dump(strlen("abc"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "int(3)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn declare_strict_types_block_mode_reports_php_fatal() {
    let execution = run_source_with_source_file(
        r#"<?php
declare(strict_types=1) {
    var_dump(strlen("abc"));
}
"#,
        "/tmp/declare-strict-block.php",
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Fatal error: strict_types declaration must not use block mode in /tmp/declare-strict-block.php on line 2"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}
