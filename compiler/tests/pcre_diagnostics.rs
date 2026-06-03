use php_compiler::run_source;

#[test]
fn pcre_match_family_reports_pattern_warnings_and_type_errors() {
    let execution = run_source(
        r#"<?php
$bad_patterns = [
    'abcdef',
    '/[a-zA-Z]',
    '[a-zA-Z]/',
    '/[a-zA-Z]/F',
];

foreach ($bad_patterns as $pattern) {
    var_dump(preg_match($pattern, 'this is a test'));
}

foreach ([[], new stdClass()] as $pattern) {
    try {
        var_dump(preg_match($pattern, 'this is a test'));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}

var_dump(preg_match_all('abcdef', 'test', $matches));
var_dump($matches);

$matches = null;
var_dump(preg_match_all('/[a-zA-Z]/', 'test', $matches));
var_dump($matches[0][3]);
"#,
    )
    .unwrap();

    assert_eq!(execution.exit_code, 0);
    assert!(
        execution.stdout.contains(
            "Warning: preg_match(): Delimiter must not be alphanumeric, backslash, or NUL byte"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_match(): No ending delimiter '/' found"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_match(): Unknown modifier '/'"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_match(): Unknown modifier 'F'"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("preg_match(): Argument #1 ($pattern) must be of type string, array given"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "preg_match(): Argument #1 ($pattern) must be of type string, stdClass given"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Warning: preg_match_all(): Delimiter must not be alphanumeric, backslash, or NUL byte"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("bool(false)\nNULL"),
        "{}",
        execution.stdout
    );
    assert!(
        !execution.stdout.contains("Undefined variable"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("int(4)"), "{}", execution.stdout);
    assert!(
        execution.stdout.contains("string(1) \"t\""),
        "{}",
        execution.stdout
    );
}

#[test]
fn pcre_replace_family_reports_invalid_patterns_and_replacement_type_errors() {
    let execution = run_source(
        r#"<?php
$regex_array = [
    'abcdef',
    '/[a-zA-Z]',
    '[a-zA-Z]/',
    '/[a-zA-Z]/F',
    [
        '[a-z]',
        '[A-Z]',
        '[0-9]',
    ],
    '/[a-zA-Z]/',
];

foreach ($regex_array as $regex_value) {
    var_dump(preg_replace($regex_value, 1, 'a'));
}

$callback_regex_array = [
    'abcdef',
    '/[a-zA-Z]',
    '[a-zA-Z]/',
    '/[a-zA-Z]/F',
    [
        '[a-z]',
        '[A-Z]',
        '[0-9]',
    ],
    '/[0-9]/',
];

$replacement = [
    'zero',
    'one',
    'two',
    'three',
    'four',
    'five',
    'six',
    'seven',
    'eight',
    'nine',
];
function integer_word($matches) {
    global $replacement;
    return $replacement[$matches[0]];
}

foreach ($callback_regex_array as $regex_value) {
    var_dump(preg_replace_callback($regex_value, 'integer_word', 'number 1.'));
}

foreach (['this is a string', ['this is', 'a subarray'], new stdClass()] as $value) {
    try {
        var_dump(preg_replace('/[a-zA-Z]/', $value, 'test'));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    )
    .unwrap();

    assert_eq!(execution.exit_code, 0);
    assert!(
        execution.stdout.contains(
            "Warning: preg_replace(): Delimiter must not be alphanumeric, backslash, or NUL byte"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_replace(): No ending delimiter '/' found"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_replace(): Unknown modifier '/'"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: preg_replace(): Unknown modifier 'F'"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "Warning: preg_replace_callback(): Delimiter must not be alphanumeric, backslash, or NUL byte"
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(1) \"a\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(1) \"1\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(9) \"number 1.\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("string(11) \"number one.\""),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "string(64) \"this is a stringthis is a stringthis is a stringthis is a string\""
        ),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("preg_replace(): Argument #1 ($pattern) must be of type array when argument #2 ($replacement) is an array, string given"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains(
            "preg_replace(): Argument #2 ($replacement) must be of type array|string, stdClass given"
        ),
        "{}",
        execution.stdout
    );
}
