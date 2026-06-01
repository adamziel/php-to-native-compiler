use php_compiler::run_source;

#[test]
fn request_parse_body_validates_option_keys() {
    let execution = run_source(
        r#"<?php
try {
    request_parse_body(options: ['foo' => 1]);
} catch (Error $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}

try {
    request_parse_body(options: [42 => 1]);
} catch (Error $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError: Invalid key \"foo\" in $options argument\nValueError: Invalid integer key in $options argument\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_parse_body_validates_option_values() {
    let execution = run_source(
        r#"<?php
try {
    request_parse_body(options: ['max_input_vars' => []]);
} catch (Error $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ValueError: Invalid array value in $options argument\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_parse_body_reports_missing_content_type_after_valid_options() {
    let execution = run_source(
        r#"<?php
$options = ['post_max_size' => '128M'];
foreach ($options as $k => &$v) {
}

try {
    request_parse_body($options);
} catch (Throwable $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "RequestParseBodyException: Request does not provide a content type\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn request_parse_body_emits_quantity_warnings_before_missing_content_type() {
    let execution = run_source(
        r#"<?php
try {
    request_parse_body(options: ['upload_max_filesize' => '1GB']);
} catch (Throwable $e) {
    echo get_class($e), ': ', $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "Warning: Invalid quantity \"1GB\": unknown multiplier \"B\", interpreting as \"1\" for backwards compatibility"
    ));
    assert!(execution
        .stdout
        .contains("RequestParseBodyException: Request does not provide a content type\n"));
    assert_eq!(execution.exit_code, 0);
}
