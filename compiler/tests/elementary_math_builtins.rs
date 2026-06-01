use php_compiler::run_source;

#[test]
fn elementary_math_builtins_cover_trig_and_angle_conversions() {
    let execution = run_source(
        r#"<?php
$sixty = M_PI / 3.0;
$thirty = M_PI / 6.0;
$ninety = M_PI / 2.0;
$oneeighty = M_PI;
$twoseventy = M_PI * 1.5;
$threesixty = M_PI * 2.0;

echo (sin($thirty) > 0.49 && sin($thirty) < 0.51) ? "sin30" : "bad", "\n";
echo (cos($sixty) > 0.49 && cos($sixty) < 0.51) ? "cos60" : "bad", "\n";
echo (tan($thirty) > 0.57 && tan($thirty) < 0.58) ? "tan30" : "bad", "\n";
echo (asin(0.5) > 0.52 && asin(0.5) < 0.53) ? "asin" : "bad", "\n";
echo (acos(0.5) > 1.04 && acos(0.5) < 1.05) ? "acos" : "bad", "\n";
echo (atan(1.7320508075689) > 1.04 && atan(1.7320508075689) < 1.05) ? "atan" : "bad", "\n";
echo (sin($ninety) > 0.99 && cos($oneeighty) < -0.99 && sin($twoseventy) < -0.99 && cos($threesixty) > 0.99) ? "quadrants" : "bad", "\n";
echo (deg2rad(180.0) > 3.14 && deg2rad(180.0) < 3.15) ? "deg" : "bad", "\n";
echo (rad2deg(M_PI) > 179.99 && rad2deg(M_PI) < 180.01) ? "rad" : "bad", "\n";
echo function_exists("sin") ? "exists" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "sin30\ncos60\ntan30\nasin\nacos\natan\nquadrants\ndeg\nrad\nexists"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn angle_conversion_matches_php_formula_order_for_phpt_edges() {
    let execution = run_source(
        r#"<?php
var_dump(deg2rad(23));
var_dump(deg2rad("23.45"));
var_dump(deg2rad("1000"));
var_dump(rad2deg(9223372034707292160));
var_dump(rad2deg(-2147483649));
var_dump(rad2deg(4294967295));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "float(0.40142572795869574)\nfloat(0.40927970959267024)\nfloat(17.453292519943293)\nfloat(5.284602904677184E+20)\nfloat(-123041749661.05348)\nfloat(246083499150.21957)\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn elementary_math_builtins_cover_hyperbolic_and_log10() {
    let execution = run_source(
        r#"<?php
echo (sinh(0.5) > 0.52 && sinh(0.5) < 0.53) ? "sinh" : "bad", "\n";
echo (cosh(-0.5) > 1.12 && cosh(-0.5) < 1.13) ? "cosh" : "bad", "\n";
echo (tanh(3.0) > 0.99 && tanh(3.0) < 1.0) ? "tanh" : "bad", "\n";
echo (asinh(10.0) > 2.99 && asinh(10.0) < 3.0) ? "asinh" : "bad", "\n";
echo (acosh(10.0) > 2.99 && acosh(10.0) < 3.0) ? "acosh" : "bad", "\n";
echo (atanh(0.5) > 0.54 && atanh(0.5) < 0.55) ? "atanh" : "bad", "\n";
echo log10(1.0), ":", log10(10.0), ":", log10(100.0), "\n";
echo sin("3.141592653589793") < 0.000000000000001 ? "numeric-string" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "sinh\ncosh\ntanh\nasinh\nacosh\natanh\n0:1:2\nnumeric-string"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn elementary_math_builtins_cover_binary_and_nonfinite_functions() {
    let execution = run_source(
        r#"<?php
echo (atan2(1, 1) > 0.78 && atan2(1, 1) < 0.79) ? "atan2" : "bad", "\n";
echo fmod(234.5, "2.3"), "\n";
echo hypot(3, 4), "\n";
echo is_finite(234.5) ? "finite" : "bad", "\n";
echo is_infinite(pow(0, -2)) ? "infinite" : "bad", "\n";
echo is_nan(acos(1.01)) ? "nan" : "bad", "\n";
echo exp(10) > 22026 && exp(10) < 22027 ? "exp" : "bad", "\n";
echo expm1(10) > 22025 && expm1(10) < 22026 ? "expm1" : "bad";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "atan2\n2.2\n5\nfinite\n\nDeprecated: Power of base 0 and negative exponent is deprecated in Command line code on line 6\ninfinite\nnan\nexp\nexpm1"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn elementary_math_builtins_cover_sqrt_ceil_and_floor() {
    let execution = run_source(
        r#"<?php
echo sqrt(9.0), "\n";
echo ceil(0.5), ":", ceil("-10.5"), ":", ceil("3.95E3"), "\n";
echo floor(-0.5), ":", floor("10.5"), ":", floor("039"), "\n";
echo function_exists("ceil") && function_exists("floor") && function_exists("sqrt") ? "exists" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3\n1:-10:3950\n-1:10:39\nexists");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
