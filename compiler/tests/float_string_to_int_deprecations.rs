use php_compiler::run_source;

#[test]
fn lossy_float_strings_emit_deprecations_when_weakly_coerced_to_int() {
    let execution = run_source(
        r#"<?php
function accepts_int(int $value) { return $value; }
function returns_int(): int { return "3.5"; }
class Box { public int $value; }

$box = new Box();
var_dump("1.5" | 3);
var_dump("6.5" % 2);
var_dump(3 << "1.5");
$compound = "1.5";
$compound <<= 3;
var_dump($compound);
var_dump(chr("60.5"));
var_dump(accepts_int("1.5"));
var_dump(returns_int());
$box->value = "1.5";
var_dump($box->value);
var_dump("1.0" | 3);
"#,
    )
    .unwrap();

    assert_eq!(
        execution
            .stdout
            .matches("Implicit conversion from float-string")
            .count(),
        8
    );
    assert!(!execution.stdout.contains("float-string \"1.0\""));
    assert!(execution.stdout.contains("float-string \"1.5\""));
    assert!(execution.stdout.contains("float-string \"6.5\""));
    assert!(execution.stdout.contains("float-string \"60.5\""));
    assert!(execution.stdout.contains("float-string \"3.5\""));
    assert!(execution.stdout.contains("string(1) \"<\""));
    assert!(execution.stdout.ends_with("int(3)\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
