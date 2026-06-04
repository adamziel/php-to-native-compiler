use php_compiler::run_source;

#[test]
fn gmp_integer_init_casts_and_unary_builtins_are_supported() {
    let execution = run_source(
        r#"<?php
var_dump(extension_loaded("gmp"));
var_dump(function_exists("gmp_init"));
echo gmp_strval(gmp_init("0b10011010010")), "\n";
echo gmp_strval(gmp_init("10011010010", 2)), "\n";
echo gmp_strval(gmp_init("02322")), "\n";
echo gmp_strval(gmp_init("0x4d2")), "\n";
echo gmp_strval(gmp_init("  0o16")), "\n";
$n = gmp_init(42);
echo $n, "\n";
var_dump((string) $n);
var_dump((int) $n);
var_dump((float) $n);
var_dump((bool) gmp_init(0));
echo gmp_strval(gmp_abs("-111111111111111111111")), "\n";
echo gmp_strval(gmp_neg(gmp_init("12345678901234567890"))), "\n";
var_dump(gmp_sign("-34535345345"));
try {
    gmp_init("4d2");
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_init(1, -1);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_abs(array());
} catch (TypeError $e) {
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
            "1234\n",
            "1234\n",
            "1234\n",
            "1234\n",
            "14\n",
            "42\n",
            "string(2) \"42\"\n",
            "int(42)\n",
            "float(42)\n",
            "bool(false)\n",
            "111111111111111111111\n",
            "-12345678901234567890\n",
            "int(-1)\n",
            "ValueError:gmp_init(): Argument #1 ($num) is not an integer string\n",
            "ValueError:gmp_init(): Argument #2 ($base) must be 0 or between 2 and 62\n",
            "TypeError:gmp_abs(): Argument #1 ($num) must be of type GMP|string|int, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn gmp_integer_arithmetic_helpers_are_supported() {
    let execution = run_source(
        r#"<?php
echo gmp_strval(gmp_mul("12345678901234567890", "9")), "\n";
echo gmp_strval(gmp_sub(10000, 10001)), "\n";
echo gmp_cmp("12345678900987654321", "123456789009876543211"), "\n";
echo gmp_strval(gmp_mod("-100000000", "353467")), "\n";
echo gmp_strval(gmp_div_q(1123123, 123, GMP_ROUND_PLUSINF)), "\n";
echo gmp_strval(gmp_div_r(1123123, 123, GMP_ROUND_PLUSINF)), "\n";
$qr = gmp_div_qr(1123123, 123, GMP_ROUND_ZERO);
echo gmp_strval($qr[0]), ":", gmp_strval($qr[1]), "\n";
echo gmp_strval(gmp_gcd("7623456735", "12372341234")), "\n";
echo gmp_strval(gmp_lcm(99, -77)), "\n";
echo gmp_strval(gmp_pow(-2, 11)), "\n";
echo gmp_strval(gmp_sqrt(777)), "\n";
$sr = gmp_sqrtrem(1000001);
echo gmp_strval($sr[0]), ":", gmp_strval($sr[1]), "\n";
echo gmp_strval(gmp_fact("10")), "\n";
echo gmp_strval(gmp_nextprime(100000)), "\n";
var_dump(gmp_perfect_square("1000000"));
try {
    gmp_div_q(1, 0);
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_div_q(1, 1, 10);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_pow(2, array());
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "111111110111111111010\n",
            "-1\n",
            "-1\n",
            "31161\n",
            "9132\n",
            "-113\n",
            "9131:10\n",
            "1\n",
            "693\n",
            "-2048\n",
            "27\n",
            "1000:1\n",
            "3628800\n",
            "100003\n",
            "bool(true)\n",
            "DivisionByZeroError:gmp_div_q(): Argument #2 ($num2) Division by zero\n",
            "ValueError:gmp_div_q(): Argument #3 ($rounding_mode) must be one of GMP_ROUND_ZERO, GMP_ROUND_PLUSINF, or GMP_ROUND_MINUSINF\n",
            "TypeError:gmp_pow(): Argument #2 ($exponent) must be of type int, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
