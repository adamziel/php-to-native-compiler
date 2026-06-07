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
fn bcmath_number_pow_compound_and_incdec_operators_are_generalized() {
    let execution = run_source(
        r#"<?php
$pow = (new BcMath\Number("3")) ** -2;
echo $pow->value, "|", $pow->scale, "\n";
$decimalPow = (new BcMath\Number("0.01")) ** new BcMath\Number("-1");
echo $decimalPow->value, "|", $decimalPow->scale, "\n";
try {
    (new BcMath\Number("0")) ** -1;
} catch (DivisionByZeroError $e) {
    echo $e->getMessage(), "\n";
}
$num = new BcMath\Number("10");
$old = $num;
$num **= 3;
echo $num, "|", $old, "\n";
$num += "5";
$num /= new BcMath\Number("30");
echo $num->value, "|", $num->scale, "\n";
$step = new BcMath\Number("0.01");
$step++;
echo $step->value, "|", $step->scale, "\n";
$step--;
echo $step->value, "|", $step->scale, "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "0.1111111111|10\n",
            "100.00|2\n",
            "Negative power of zero\n",
            "1000|10\n",
            "33.5|1\n",
            "1.01|2\n",
            "0.01|2\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_number_default_scales_serialization_comparison_and_handles_are_generalized() {
    let execution = run_source(
        r#"<?php
foreach (["0", "0.00", "123"] as $value) {
    $tmp = new BcMath\Number($value);
    var_dump($tmp);
    unset($tmp);
}
var_dump((new BcMath\Number("1"))->div("1000"));
var_dump((new BcMath\Number("1"))->div("2000"));

$num = new BcMath\Number("100.012");
$div = $num->div("-30");
echo $div->value, "|", $div->scale, "\n";
[$quot, $rem] = $num->divmod("80.3");
echo $quot->value, "|", $quot->scale, "|", $rem->value, "|", $rem->scale, "\n";
$pow = (new BcMath\Number("12.5"))->pow("-2");
echo $pow->value, "|", $pow->scale, "\n";
$sqrt = (new BcMath\Number("15151324141414.412312232141241"))->sqrt();
echo $sqrt->value, "|", $sqrt->scale, "\n";
$zeroPow = pow(new BcMath\Number("0"), 2);
echo $zeroPow->value, "|", $zeroPow->scale, "\n";
var_dump((new BcMath\Number("100.0000")) > "99.9999");
var_dump("100.00001" > new BcMath\Number("100.0000"));
var_dump(new BcMath\Number("100.0000") == 100);
echo serialize(new BcMath\Number("0.1230")), "\n";
$copy = unserialize('O:13:"BcMath\Number":1:{s:5:"value";s:6:"0.1230";}');
echo $copy->value, "|", $copy->scale, "\n";
try {
    (new BcMath\Number(1))->__unserialize(["value" => "5"]);
} catch (Error $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    unserialize('O:13:"BcMath\Number":1:{s:5:"value";s:1:"a";}');
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    unserialize('O:13:"BcMath\Number":1:{s:5:"value";s:0:"";}');
} catch (Exception $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "object(BcMath\\Number)#1 (2) {\n",
            "  [\"value\"]=>\n",
            "  string(1) \"0\"\n",
            "  [\"scale\"]=>\n",
            "  int(0)\n",
            "}\n",
            "object(BcMath\\Number)#1 (2) {\n",
            "  [\"value\"]=>\n",
            "  string(4) \"0.00\"\n",
            "  [\"scale\"]=>\n",
            "  int(2)\n",
            "}\n",
            "object(BcMath\\Number)#1 (2) {\n",
            "  [\"value\"]=>\n",
            "  string(3) \"123\"\n",
            "  [\"scale\"]=>\n",
            "  int(0)\n",
            "}\n",
            "object(BcMath\\Number)#2 (2) {\n",
            "  [\"value\"]=>\n",
            "  string(5) \"0.001\"\n",
            "  [\"scale\"]=>\n",
            "  int(3)\n",
            "}\n",
            "object(BcMath\\Number)#1 (2) {\n",
            "  [\"value\"]=>\n",
            "  string(6) \"0.0005\"\n",
            "  [\"scale\"]=>\n",
            "  int(4)\n",
            "}\n",
            "-3.3337333333333|13\n",
            "1|0|19.712|3\n",
            "0.0064|4\n",
            "3892470.1850385973524458288799178|25\n",
            "0|0\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "O:13:\"BcMath\\Number\":1:{s:5:\"value\";s:6:\"0.1230\";}\n",
            "0.1230|4\n",
            "Error:Cannot modify readonly property BcMath\\Number::$value\n",
            "Exception:Invalid serialization data for BcMath\\Number object\n",
            "Exception:Invalid serialization data for BcMath\\Number object\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_number_invalid_operands_are_catchable_and_coerced_like_php() {
    let execution = run_source(
        r#"<?php
function show($e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}

$num = new BcMath\Number("10");
try { $num->add("not-a-number"); } catch (Throwable $e) { show($e); }
try { $num->add([]); } catch (Throwable $e) { show($e); }
try { $num->add(1, []); } catch (Throwable $e) { show($e); }
$num->add(0.1);
$num->add(null);
try { $num->div(0.1); } catch (Throwable $e) { show($e); }
try { $num + []; } catch (Throwable $e) { show($e); }
try { $num + "not-a-number"; } catch (Throwable $e) { show($e); }
$num + 1.01;
var_dump($num > null);
var_dump($num < "not-a-number");
var_dump("not-a-number" > $num);
"#,
    )
    .unwrap();

    assert!(
        execution
            .stdout
            .contains("ValueError:BcMath\\Number::add(): Argument #1 ($num) is not well-formed\n"),
        "stdout:\n{}",
        execution.stdout
    );
    assert!(execution.stdout.contains(
        "TypeError:BcMath\\Number::add(): Argument #1 ($num) must be of type int, string, or BcMath\\Number, array given\n"
    ));
    assert!(execution.stdout.contains(
        "TypeError:BcMath\\Number::add(): Argument #2 ($scale) must be of type ?int, array given\n"
    ));
    assert!(execution
        .stdout
        .contains("Deprecated: Implicit conversion from float 0.1 to int loses precision"));
    assert!(execution.stdout.contains(
        "Deprecated: BcMath\\Number::add(): Passing null to parameter #1 ($num) of type BcMath\\Number|string|int is deprecated"
    ));
    assert!(execution
        .stdout
        .contains("DivisionByZeroError:Division by zero\n"));
    assert!(execution
        .stdout
        .contains("TypeError:Unsupported operand types: BcMath\\Number + array\n"));
    assert!(execution
        .stdout
        .contains("TypeError:Right string operand cannot be converted to BcMath\\Number\n"));
    assert!(execution
        .stdout
        .contains("Deprecated: Implicit conversion from float 1.01 to int loses precision"));
    assert!(execution
        .stdout
        .ends_with("bool(true)\nbool(false)\nbool(false)\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn bcmath_number_readonly_properties_can_be_read_by_reference_as_values() {
    let execution = run_source(
        r#"<?php
$num = new BcMath\Number("1.25");
$value = &$num->value;
$scale = &$num->scale;
var_dump($value, $scale);
$value = "changed";
var_dump($num->value);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(4) \"1.25\"\nint(2)\nstring(4) \"1.25\"\n"
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
            "Error:Cannot modify protected(set) readonly property BcMath\\Number::$value from global scope\n",
            "Error:Cannot unset protected(set) readonly property BcMath\\Number::$scale from global scope\n",
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
