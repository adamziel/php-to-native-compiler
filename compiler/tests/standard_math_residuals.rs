use php_compiler::run_source;

#[test]
fn standard_math_constants_and_basic_helpers_execute() {
    let execution = run_source(
        r#"<?php
printf("%.4f|%.4f|%.4f\n", M_E, M_PI_2, M_SQRT3);
echo pi() === M_PI ? "pi" : "bad", "\n";
echo getrandmax() > 0 ? "randmax" : "bad", "\n";
echo log(100, 10), "|", log1p(0), "\n";
var_dump(fdiv(10., 2.));
var_dump(fdiv(10, 0));
var_dump(fdiv(-0.0, INF));
var_dump(fpow(0, -1));
var_dump(intdiv(-3, 2));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "2.7183|1.5708|1.7321\npi\nrandmax\n2|0\nfloat(5)\nfloat(INF)\nfloat(-0)\nfloat(INF)\nint(-1)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn set_time_limit_accepts_supported_cli_noop() {
    let execution = run_source(
        r#"<?php
var_dump(set_time_limit(1));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "bool(true)\n");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn nonfinite_float_serialization_uses_php_math_bug_tokens() {
    let execution = run_source(
        r#"<?php
foreach ([-INF, INF, NAN] as $value) {
    var_dump(serialize($value));
    var_dump(unserialize(serialize($value)));
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(7) \"d:-INF;\"\n\
float(-INF)\n\
string(6) \"d:INF;\"\n\
float(INF)\n\
string(6) \"d:NAN;\"\n\
float(NAN)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn round_supports_legacy_and_enum_modes() {
    let execution = run_source(
        r#"<?php
echo round(2.5, 0, PHP_ROUND_HALF_UP), "\n";
echo round(2.5, 0, PHP_ROUND_HALF_DOWN), "\n";
echo round(2.5, 0, PHP_ROUND_HALF_EVEN), "\n";
echo round(2.5, 0, PHP_ROUND_HALF_ODD), "\n";
echo round(-1.2, 0, RoundingMode::NegativeInfinity), "\n";
echo round(-1.2, 0, RoundingMode::PositiveInfinity), "\n";
var_dump(round(-0.0001, 2));
try {
    round(1.5, mode: 1234);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3\n2\n2\n3\n-2\n-1\nfloat(-0)\nround(): Argument #3 ($mode) must be a valid rounding mode (RoundingMode::*)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn clamp_uses_php_comparison_and_reports_invalid_bounds() {
    let execution = run_source(
        r#"<?php
var_dump(clamp(2, 1, 3));
var_dump(clamp(0, 1, 3));
var_dump(clamp("d", "c", "g"));
var_dump(clamp(null, -1, 1));
try {
    clamp(4, NAN, 6);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
try {
    clamp(1, 3, 2);
} catch (ValueError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(2)\nint(1)\nstring(1) \"d\"\nint(-1)\nclamp(): Argument #2 ($min) must not be NAN\nclamp(): Argument #2 ($min) must be smaller than or equal to argument #3 ($max)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
