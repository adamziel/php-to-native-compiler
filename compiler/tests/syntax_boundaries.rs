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

#[test]
fn unset_syntax_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$value = 1;
unset($value);
"#,
            3,
            1,
        ),
        (
            r#"<?php
$items = ["name" => "Ada"];
unset($items["name"]);
"#,
            3,
            1,
        ),
        (
            r#"<?php
class Box {
    public $name;
}
$box = new Box();
unset($box->name);
"#,
            6,
            1,
        ),
        (
            r#"<?php
$value = 1;
UNSET($value);
"#,
            3,
            1,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported unset: variable, array offset, and property removal are not implemented"
        );
    }
}

#[test]
fn unsupported_foreach_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$items = ["name" => "Ada"];
foreach ($items as $key => $value) echo $value;
"#,
            3,
            25,
            "unsupported foreach: key/value iteration is not implemented; only foreach ($array as $value) is supported",
        ),
        (
            r#"<?php
$items = [1];
FOREACH ($items as &$item) {
    echo $item;
}
"#,
            3,
            20,
            "unsupported foreach: by-reference iteration is not implemented; only by-value iteration is supported",
        ),
        (
            r#"<?php
$items = [1];
echo foreach ($items as $item);
"#,
            3,
            6,
            "unsupported foreach: foreach is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn for_syntax_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
for ($i = 0; $i < 3; $i = $i + 1) {
    echo $i;
}
"#,
            2,
            1,
        ),
        (
            r#"<?php
FOR ($i = 0; $i < 3; $i = $i + 1) echo $i;
"#,
            2,
            1,
        ),
        (
            r#"<?php
echo for ($i = 0; $i < 3; $i = $i + 1);
"#,
            2,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported for: C-style loops are not implemented"
        );
    }
}

#[test]
fn do_while_syntax_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$i = 0;
do {
    echo $i;
    $i = $i + 1;
} while ($i < 3);
"#,
            3,
            1,
        ),
        (
            r#"<?php
DO echo "tick"; WHILE (false);
"#,
            2,
            1,
        ),
        (
            r#"<?php
echo do {
    echo "tick";
} while (false);
"#,
            2,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported do-while: post-condition loops are not implemented"
        );
    }
}

#[test]
fn switch_syntax_is_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
$value = 2;
switch ($value) {
    case 1:
        echo "one";
        break;
    default:
        echo "other";
}
"#,
            3,
            1,
        ),
        (
            r#"<?php
SWITCH (1) {
    CASE 1:
        echo "one";
}
"#,
            2,
            1,
        ),
        (
            r#"<?php
echo switch ($value) {
    default:
        echo "fallback";
};
"#,
            2,
            6,
        ),
    ];

    for (source, line, column) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(
            error.message,
            "unsupported switch: switch/case control flow is not implemented"
        );
    }
}

#[test]
fn unsupported_break_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    break 2;
}
"#,
            3,
            5,
            "unsupported break: loop-depth arguments are not implemented; only 'break;' for the innermost while loop is supported",
        ),
        (
            r#"<?php
echo break;
"#,
            2,
            6,
            "unsupported break: break is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}

#[test]
fn unsupported_continue_forms_are_rejected_with_stable_parse_error() {
    let cases = [
        (
            r#"<?php
while (true) {
    continue 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost while loop is supported",
        ),
        (
            r#"<?php
while (true) {
    CONTINUE 2;
}
"#,
            3,
            5,
            "unsupported continue: loop-depth arguments are not implemented; only 'continue;' for the innermost while loop is supported",
        ),
        (
            r#"<?php
echo continue;
"#,
            2,
            6,
            "unsupported continue: continue is only supported as a statement in the current subset",
        ),
    ];

    for (source, line, column, message) in cases {
        let error = parse_error(source);
        assert_eq!(error.line, line);
        assert_eq!(error.column, column);
        assert_eq!(error.message, message);
    }
}
