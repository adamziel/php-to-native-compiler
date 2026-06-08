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

#[test]
fn gmp_integer_bit_helpers_are_supported() {
    let execution = run_source(
        r#"<?php
echo gmp_strval(gmp_and("111111", "2222222")), "\n";
echo gmp_strval(gmp_or(4545, -20)), "\n";
echo gmp_strval(gmp_xor(-1, 3333)), "\n";
echo gmp_strval(gmp_com("2394876545678")), "\n";
$n = gmp_init("100000000000");
gmp_setbit($n, 23, true);
echo gmp_strval($n), "\n";
gmp_setbit($n, 23, false);
gmp_setbit($n, 3);
echo gmp_strval($n), "\n";
$m = gmp_init("238462734628347239571823641234");
gmp_clrbit($m, 3);
gmp_clrbit($m, 5);
gmp_clrbit($m, 20);
echo gmp_strval($m), "\n";
var_dump(gmp_testbit(gmp_init(-1), 1));
var_dump(gmp_scan0("434234", 1));
var_dump(gmp_scan1("1000000000", 200));
var_dump(gmp_popcount("52638927634234"));
var_dump(gmp_hamdist(gmp_init("8765434567"), gmp_init("987654445678")));
try {
    gmp_scan0("434234", -10);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_setbit("", 23);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_and(array(), 1);
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "106502\n",
            "-19\n",
            "-3334\n",
            "-2394876545679\n",
            "100008388608\n",
            "100000000008\n",
            "238462734628347239571822592658\n",
            "bool(true)\n",
            "int(2)\n",
            "int(-1)\n",
            "int(31)\n",
            "int(26)\n",
            "ValueError:gmp_scan0(): Argument #2 ($start) must be between 0 and 4096 * 8\n",
            "TypeError:gmp_setbit(): Argument #1 ($num) must be of type GMP, string given\n",
            "TypeError:gmp_and(): Argument #1 ($num1) must be of type GMP|string|int, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn gmp_operator_overloads_and_ordering_are_supported() {
    let execution = run_source(
        r#"<?php
$a = gmp_init(42);
$b = gmp_init(17);
echo gmp_strval($a / $b), "\n";
echo gmp_strval($a % $b), "\n";
echo gmp_strval($a ** "3"), "\n";
echo gmp_strval($a | $b), ":", gmp_strval($a & $b), ":", gmp_strval($a ^ $b), "\n";
echo gmp_strval($a << 2), ":", gmp_strval(-$a >> 2), "\n";
var_dump($a > null);
try {
    $a == "not-int";
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    $a + [];
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    $a ** [];
} catch (TypeError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
$values = [gmp_init(0), -3, gmp_init(2), 1];
sort($values);
foreach ($values as $value) {
    echo is_object($value) ? "G" : "I", gmp_strval($value), ",";
}
echo "\n";
$min = min(gmp_init(3), 4);
$max = max(gmp_init(3), 4);
echo is_object($min) ? "G" : "I", gmp_strval($min), "\n";
echo is_object($max) ? "G" : "I", gmp_strval($max), "\n";
var_dump(array_sum([gmp_init((string) (PHP_INT_MAX - 1)), 1]) === PHP_INT_MAX);
$a += 1;
echo gmp_strval($a), "\n";
$a -= 1;
echo gmp_strval($a), "\n";
echo gmp_strval(++$a), ":", gmp_strval($a++), ":", gmp_strval($a), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2\n",
            "8\n",
            "74088\n",
            "59:0:59\n",
            "168:-11\n",
            "bool(true)\n",
            "ValueError:Number is not an integer string\n",
            "TypeError:Number must be of type GMP|string|int, array given\n",
            "TypeError:Unsupported operand types: GMP ** array\n",
            "I-3,G0,I1,G2,\n",
            "G3\n",
            "I4\n",
            "bool(true)\n",
            "43\n",
            "42\n",
            "43:43:44\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn gmp_number_theory_helpers_are_supported() {
    let execution = run_source(
        r#"<?php
var_dump(function_exists("gmp_root"));
$r = gmp_gcdext(123, 45);
$check = gmp_add(gmp_mul(123, $r["s"]), gmp_mul(45, $r["t"]));
echo gmp_strval($r["g"]), ":", gmp_strval($check), "\n";
echo gmp_strval(gmp_invert(123123, 5467624)), "\n";
var_dump(gmp_invert(123123, "3333334345467624"));
echo gmp_jacobi(7, 23), ":", gmp_legendre(7, 23), ":", gmp_kronecker(-23, 12), "\n";
echo gmp_strval(gmp_root(1000, 3)), "\n";
$rootrem = gmp_rootrem(100, 3);
echo gmp_strval($rootrem[0]), ":", gmp_strval($rootrem[1]), "\n";
var_dump(gmp_perfect_power(gmp_init("7442665456261594668083173595997")));
var_dump(gmp_perfect_power(gmp_init("7442665456261594668083173595997") + 1));
var_dump(gmp_prob_prime(-31));
echo gmp_strval(gmp_binomial(10, 5)), ":", gmp_strval(gmp_binomial(-2, 6)), "\n";
try {
    gmp_invert(1, 0);
} catch (DivisionByZeroError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_root(-100, 4);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_binomial(5, -2);
} catch (ValueError $e) {
    echo get_class($e), ":", $e->getMessage(), "\n";
}
try {
    gmp_gcdext(array(), array());
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
            "3:3\n",
            "2293131\n",
            "bool(false)\n",
            "-1:-1:1\n",
            "10\n",
            "4:36\n",
            "bool(true)\n",
            "bool(false)\n",
            "int(2)\n",
            "252:7\n",
            "DivisionByZeroError:Division by zero\n",
            "ValueError:gmp_root(): Argument #2 ($nth) must be odd if argument #1 ($a) is negative\n",
            "ValueError:gmp_binomial(): Argument #2 ($k) must be between 0 and 4096\n",
            "TypeError:gmp_gcdext(): Argument #1 ($num1) must be of type GMP|string|int, array given\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
