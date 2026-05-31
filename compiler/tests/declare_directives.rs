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
fn declare_strict_types_enforces_scalar_parameters_returns_and_properties() {
    let strict = run_source(
        r#"<?php
declare(strict_types=1);
function strict_param(int $value) { var_dump($value); }
function strict_return(): int { return "42"; }
class StrictBox { public int $id; }

try {
    strict_param("42");
} catch (Throwable $error) {
    echo "param:", get_class($error), "\n";
}

try {
    var_dump(strict_return());
} catch (Throwable $error) {
    echo "return:", get_class($error), "\n";
}

try {
    $box = new StrictBox();
    $box->id = "42";
    var_dump($box->id);
} catch (Throwable $error) {
    echo "prop:", get_class($error), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        strict.stdout,
        "param:TypeError\nreturn:TypeError\nprop:TypeError\n"
    );
    assert_eq!(strict.stderr, "");
    assert_eq!(strict.exit_code, 0);

    let weak = run_source(
        r#"<?php
declare(strict_types=0);
function weak_param(int $value) { var_dump($value); }
function weak_return(): int { return "42"; }
class WeakBox { public int $id; }

weak_param("42");
var_dump(weak_return());
$box = new WeakBox();
$box->id = "42";
var_dump($box->id);
"#,
    )
    .unwrap();

    assert_eq!(weak.stdout, "int(42)\nint(42)\nint(42)\n");
    assert_eq!(weak.stderr, "");
    assert_eq!(weak.exit_code, 0);
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
