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
fn math_constants_use_precision_minus_one_for_string_formatting() {
    let execution = run_source(
        r#"<?php
ini_set("precision", "-1");
$constants = array(
    "M_E",
    "M_LOG2E",
    "M_LOG10E",
    "M_LN2",
    "M_LN10",
    "M_PI",
    "M_PI_2",
    "M_PI_4",
    "M_1_PI",
    "M_2_PI",
    "M_SQRTPI",
    "M_2_SQRTPI",
    "M_LNPI",
    "M_EULER",
    "M_SQRT2",
    "M_SQRT1_2",
    "M_SQRT3"
);
foreach ($constants as $constant) {
    printf("%-10s: %s\n", $constant, constant($constant));
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "M_E       : 2.718281828459045\nM_LOG2E   : 1.4426950408889634\nM_LOG10E  : 0.4342944819032518\nM_LN2     : 0.6931471805599453\nM_LN10    : 2.302585092994046\nM_PI      : 3.141592653589793\nM_PI_2    : 1.5707963267948966\nM_PI_4    : 0.7853981633974483\nM_1_PI    : 0.3183098861837907\nM_2_PI    : 0.6366197723675814\nM_SQRTPI  : 1.772453850905516\nM_2_SQRTPI: 1.1283791670955126\nM_LNPI    : 1.1447298858494002\nM_EULER   : 0.5772156649015329\nM_SQRT2   : 1.4142135623730951\nM_SQRT1_2 : 0.7071067811865476\nM_SQRT3   : 1.7320508075688772\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn serialize_uses_php_nonfinite_float_spellings() {
    let execution = run_source(
        r#"<?php
var_dump(serialize(-INF));
var_dump(unserialize(serialize(-INF)));
var_dump(serialize(INF));
var_dump(unserialize(serialize(INF)));
var_dump(serialize(NAN));
var_dump(unserialize(serialize(NAN)));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(7) \"d:-INF;\"\nfloat(-INF)\nstring(6) \"d:INF;\"\nfloat(INF)\nstring(6) \"d:NAN;\"\nfloat(NAN)\n"
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
