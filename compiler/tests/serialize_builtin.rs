use php_compiler::run_source;

#[test]
fn unserialize_warns_on_extra_data_and_returns_valid_prefix() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('i:5;i:6;'));
var_dump(unserialize('N;i:6;'));
var_dump(unserialize('b:1;i:6;'));
var_dump(unserialize('a:1:{s:3:"foo";b:1;}i:6;'));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 4 of 8 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 2 of 6 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 20 of 24 bytes"));
    assert!(execution.stdout.contains("int(5)\n"));
    assert!(execution.stdout.contains("NULL\n"));
    assert!(execution.stdout.contains("bool(true)\n"));
    assert!(execution
        .stdout
        .contains("array(1) {\n  [\"foo\"]=>\n  bool(true)\n}\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unserialize_rejects_signed_lengths_and_reports_parser_offset() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('s:+1:"x";'));
var_dump(unserialize('a:-0:{}'));
var_dump(unserialize('a:1:{i:0;r:+1;}'));
var_dump(unserialize('a:1:{i:0;R:-1;}'));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Error at offset 0 of 9 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Error at offset 0 of 7 bytes"));
    assert!(
        execution
            .stdout
            .matches("Warning: unserialize(): Error at offset 9 of 15 bytes")
            .count()
            >= 2
    );
    assert_eq!(execution.stdout.matches("bool(false)\n").count(), 4);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
