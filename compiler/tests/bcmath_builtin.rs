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

#[test]
fn bcmath_number_methods_and_divmod_are_generalized() {
    let execution = run_source(
        r#"<?php
ini_set("bcmath.scale", "0");
$num = new BcMath\Number("100.012");
echo (string) $num, "|", $num->value, "|", $num->scale, "\n";
var_dump(class_exists("BcMath\\Number"));
var_dump(method_exists("BcMath\\Number", "powmod"));
echo $num->add("0.01", 4)->compare(bcadd("100.012", "0.01", 4)), "\n";
echo $num->sub("-0.40", 4)->compare(bcsub("100.012", "-0.40", 4)), "\n";
echo $num->mul("80.3", 5)->compare(bcmul("100.012", "80.3", 5)), "\n";
echo $num->div("-50.6", 6)->compare(bcdiv("100.012", "-50.6", 6)), "\n";
echo $num->mod("80.3", 3)->compare(bcmod("100.012", "80.3", 3)), "\n";
echo (new BcMath\Number("12.5"))->pow("-2", 4)->compare(bcpow("12.5", "-2", 4)), "\n";
echo (new BcMath\Number("12"))->powmod("3", "5", 2)->compare(bcpowmod("12", "3", "5", 2)), "\n";
echo (new BcMath\Number("15151324141414.412312232141241"))->sqrt(10)->compare(bcsqrt("15151324141414.412312232141241", 10)), "\n";
echo (new BcMath\Number("2.5"))->round(0, RoundingMode::HalfEven)->compare(bcround("2.5", 0, RoundingMode::HalfEven)), "\n";
echo (new BcMath\Number("-1.0001"))->ceil()->compare(bcceil("-1.0001"), 0), "\n";
echo (new BcMath\Number("-1.0001"))->floor()->compare(bcfloor("-1.0001")), "\n";
[$quot, $rem] = bcdivmod("15", "14.14", 10);
echo $quot, "|", $rem, "\n";
[$mquot, $mrem] = $num->divmod("80.3", 3);
echo $mquot->compare(bcdiv("100.012", "80.3", 0)), "|", $mrem->compare(bcmod("100.012", "80.3", 3)), "\n";
echo PHP_INT_MIN, "\n";
try {
    bcdivmod("1", "0");
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "100.012|100.012|3\n",
            "bool(true)\n",
            "bool(true)\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "0\n",
            "1|0.8600000000\n",
            "0|0\n",
            "-9223372036854775808\n",
            "DivisionByZeroError:Division by zero\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_number_object_cast_properties_and_constructor_diagnostics_are_generalized() {
    let execution = run_source(
        r#"<?php
$zero = new BcMath\Number("0.0");
$nonzero = new BcMath\Number("-0.125");
var_dump((bool) $zero);
var_dump(boolval($nonzero));
echo (string) $zero, "|", $zero->value, "|", $zero->scale, "\n";
print_r((array) $nonzero);
var_dump($zero->missing);
try {
    $zero->value = "3";
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    unset($zero->scale);
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    $zero->dynamic = "no";
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    $zero->__construct("1");
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    new BcMath\Number("not-a-number");
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
$float = new BcMath\Number(0.1234);
echo (string) $float, "|", $float->scale, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(false)\n",
            "bool(true)\n",
            "0.0|0.0|1\n",
            "Array\n",
            "(\n",
            "    [value] => -0.125\n",
            "    [scale] => 3\n",
            ")\n",
            "\n",
            "Warning: Undefined property: BcMath\\Number::$missing in Command line code on line 8\n",
            "NULL\n",
            "Error:Cannot modify readonly property BcMath\\Number::$value\n",
            "Error:Cannot unset readonly property BcMath\\Number::$scale\n",
            "Error:Cannot create dynamic property BcMath\\Number::$dynamic\n",
            "Error:Cannot modify readonly property BcMath\\Number::$value\n",
            "ValueError:BcMath\\Number::__construct(): Argument #1 ($num) is not well-formed\n",
            "\n",
            "Deprecated: Implicit conversion from float 0.1234 to int loses precision in Command line code on line 34\n",
            "0|0\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
