use php_compiler::run_source;

#[test]
fn bcmath_scalar_decimal_arithmetic_and_scale_are_generalized() {
    let execution = run_source(
        r#"<?php
ini_set("bcmath.scale", "0");
var_dump(extension_loaded("bcmath"));
var_dump(function_exists("bcadd"));
echo bcadd("15151324141414.412312232141241", "-132132245132134.1515123765412", 10), "\n";
echo bcsub("141241241241241248267654747412", "0.1322135476547459213732911312", 0), "\n";
echo bcmul("123456789.987654321", "-10.01", 6), "\n";
echo bcdiv("10.99", "2", 3), "\n";
echo bccomp("-2.29", "-2.3", "2"), "\n";
echo bcceil("1.0001"), "|", bcceil("-1.0001"), "|", bcfloor("1.999"), "|", bcfloor("-1.0001"), "\n";
echo bcadd("1", "2"), "\n";
var_dump(bcscale(2));
echo bcadd("1", "2"), "\n";
ini_set("bcmath.scale", "4");
echo bcsub("2", "1"), "\n";
try {
    bcdiv("1", "0");
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    bcadd("a", "1");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "-116980920990719.7392001443\n",
            "141241241241241248267654747411\n",
            "-1235802467.776419\n",
            "5.495\n",
            "1\n",
            "2|-1|1|-2\n",
            "3\n",
            "int(0)\n",
            "3.00\n",
            "1.0000\n",
            "DivisionByZeroError:Division by zero\n",
            "ValueError:bcadd(): Argument #1 ($num1) is not well-formed\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_mod_pow_powmod_and_sqrt_are_generalized() {
    let execution = run_source(
        r#"<?php
ini_set("bcmath.scale", "0");
var_dump(function_exists("bcmod"));
echo bcmod("15", "14.14", 10), "\n";
echo bcmod("-16.60", "14.14", 10), "\n";
echo bcpow("14.14", "3", 5), "\n";
echo bcpow("10", "-5", 8), "\n";
echo bcpow("0", "1128321638", 2), "\n";
echo bcpowmod("10", "2147483648", "2047"), "\n";
echo bcpowmod("-2", "5", "7", 3), "\n";
echo bcsqrt("15151324141414.412312232141241", 10), "\n";
echo bcsqrt("0.1322135476547459213732911312", 10), "\n";
try {
    bcmod("10", "0");
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    bcpow("0", "-1");
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    bcpow("1", "1.1");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    bcpowmod("4.1", "4", "3");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    bcsqrt("-9");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "0.8600000000\n",
            "-2.4600000000\n",
            "2827.14594\n",
            "0.00001000\n",
            "0.00\n",
            "790\n",
            "-4.000\n",
            "3892470.1850385973\n",
            "0.3636118090\n",
            "DivisionByZeroError:Modulo by zero\n",
            "DivisionByZeroError:Negative power of zero\n",
            "ValueError:bcpow(): Argument #2 ($exponent) cannot have a fractional part\n",
            "ValueError:bcpowmod(): Argument #1 ($num) cannot have a fractional part\n",
            "ValueError:bcsqrt(): Argument #1 ($num) must be greater than or equal to 0\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_rounding_modes_are_generalized() {
    let execution = run_source(
        r#"<?php
foreach (RoundingMode::cases() as $mode) {
    echo $mode->name, "=", bcround("2.5", 0, $mode), "\n";
}
echo bcround("1.5"), "\n";
echo bcround("123", 1), "\n";
echo bcround("50", -2, RoundingMode::HalfEven), "\n";
echo bcround("50", -2, RoundingMode::HalfOdd), "\n";
echo bcround("3450.0001", -2, RoundingMode::HalfTowardsZero), "\n";
echo bcround("-0.0005", 3, RoundingMode::HalfAwayFromZero), "\n";
var_dump(function_exists("bcround"));
$reflection = new ReflectionFunction("bcround");
echo $reflection->getNumberOfParameters(), ":", $reflection->getNumberOfRequiredParameters(), "\n";
try {
    bcround("hoge");
} catch (Throwable $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "HalfAwayFromZero=3\n",
            "HalfTowardsZero=2\n",
            "HalfEven=2\n",
            "HalfOdd=3\n",
            "TowardsZero=2\n",
            "AwayFromZero=3\n",
            "NegativeInfinity=2\n",
            "PositiveInfinity=3\n",
            "2\n",
            "123.0\n",
            "0\n",
            "100\n",
            "3500\n",
            "-0.001\n",
            "bool(true)\n",
            "3:1\n",
            "ValueError:bcround(): Argument #1 ($num) is not well-formed\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
