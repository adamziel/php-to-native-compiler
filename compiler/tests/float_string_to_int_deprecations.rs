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

#[test]
fn lossy_floats_emit_deprecations_when_weakly_coerced_to_int() {
    let execution = run_source(
        r#"<?php
function accepts_int(int $value) { return $value; }
function accepts_int_or_string(int|string $value) { var_dump($value); }
function returns_int(): int { return 3.5; }
class Box { public int $value; }

$box = new Box();
var_dump(~1.5);
var_dump(1.5 | 3);
var_dump(6.5 % 2);
$compound = 3;
$compound <<= 1.5;
var_dump($compound);
var_dump(chr(60.5));
var_dump(accepts_int(1.5));
var_dump(returns_int());
$box->value = 1.5;
var_dump($box->value);
accepts_int_or_string(1.5);
accepts_int_or_string(fdiv(0, 0));
accepts_int_or_string(10e120);
var_dump(~1.0);
"#,
    )
    .unwrap();

    assert_eq!(
        execution
            .stdout
            .matches("Implicit conversion from float 1.5 to int loses precision")
            .count(),
        6
    );
    assert!(execution
        .stdout
        .contains("Implicit conversion from float 6.5 to int loses precision"));
    assert!(execution
        .stdout
        .contains("Implicit conversion from float 60.5 to int loses precision"));
    assert!(execution
        .stdout
        .contains("unexpected NAN value was coerced to string"));
    assert!(execution.stdout.contains("string(3) \"NAN\""));
    assert!(execution.stdout.contains("string(8) \"1.0E+121\""));
    assert!(execution.stdout.ends_with("int(-2)\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn int_float_unions_accept_floats_without_int_coercion_diagnostics() {
    let execution = run_source(
        r#"<?php
function accepts_int_or_float(int|float $value) { return $value; }
function increments(int|float $x) { return ++$x; }
function decrements(int|float $x) { return --$x; }
function returns_int_or_float(): int|float { return 3.5; }
class TypedBox { public int|float $value; }

$box = new TypedBox();
$box->value = 1.5;
var_dump(accepts_int_or_float(1.5));
var_dump(increments(1.1));
var_dump(decrements(1.1));
var_dump(returns_int_or_float());
var_dump($box->value);
"#,
    )
    .unwrap();

    assert!(!execution.stdout.contains("Implicit conversion from float"));
    assert!(!execution
        .stdout
        .contains("unexpected NAN value was coerced to string"));
    assert!(execution.stdout.contains("float(1.5)"));
    assert!(execution.stdout.contains("float(2.1)"));
    assert!(execution.stdout.contains("float(0.10000000000000009)"));
    assert!(execution.stdout.contains("float(3.5)"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
