use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn long_array_syntax_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = array();
"#,
            2,
            10,
        ),
        (
            r#"<?php
echo array("a" => 1);
"#,
            2,
            6,
        ),
        (
            r#"<?php
$items = ARRAY("a", "b");
"#,
            2,
            10,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported long array syntax: array(...) literals are not implemented; use short [] literals in the current subset"
        );
    }
}
