use php_compiler::error::{Diagnostic, Phase};
use php_compiler::run_source;

fn parse_error(source: &str) -> Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn object_and_class_syntax_is_rejected_with_stable_parse_errors() {
    let cases = [
        (
            r#"<?php
class Box {}
"#,
            2,
            1,
            "unsupported class declaration: object/class syntax is not implemented",
        ),
        (
            r#"<?php
$box = new Box();
"#,
            2,
            8,
            "unsupported object instantiation: object/class syntax is not implemented",
        ),
        (
            r#"<?php
$box->name;
"#,
            2,
            5,
            "unsupported object access: object property and method access are not implemented",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
