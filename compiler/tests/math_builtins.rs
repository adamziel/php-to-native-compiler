use php_compiler::run_source;

#[test]
fn inverse_hyperbolic_math_builtins_cover_current_scalar_rows() {
    let execution = run_source(
        r#"<?php
$acosh = acosh(1.1276259652064);
$asinh = asinh(0.52109530549375);
$call = "atanh";
$atanh = $call("0.46211715726001");

echo ($acosh > 0.4999999999 && $acosh < 0.5000000001) ? "acosh" : "bad", "\n";
echo ($asinh > 0.4999999999 && $asinh < 0.5000000001) ? "asinh" : "bad", "\n";
echo ($atanh > 0.4999999999 && $atanh < 0.5000000001) ? "atanh" : "bad", "\n";
echo function_exists("acosh") ? "exists" : "missing";
echo "|";
echo is_callable("asinh") ? "callable" : "missing";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "acosh\nasinh\natanh\nexists|callable");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sqrt_and_intdiv_cover_current_scalar_rows() {
    let execution = run_source(
        r#"<?php
$sqrt = sqrt(9.0);
$call = "sqrt";
$dynamic = $call("16");

echo ($sqrt > 2.9999999999 && $sqrt < 3.0000000001) ? "sqrt" : "bad", "\n";
echo ($dynamic > 3.9999999999 && $dynamic < 4.0000000001) ? "sqrt-dynamic" : "bad", "\n";
echo intdiv(3, 2), "|", intdiv(3, -2), "\n";

try {
    intdiv(PHP_INT_MIN, -1);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

try {
    intdiv(1, 0);
} catch (Throwable $e) {
    echo $e->getMessage(), "\n";
}

$reflection = new ReflectionFunction("intdiv");
echo function_exists("sqrt") ? "exists" : "missing";
echo "|";
echo is_callable("intdiv") ? "callable" : "missing";
echo "|";
echo $reflection->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "sqrt\nsqrt-dynamic\n1|-1\nDivision of PHP_INT_MIN by -1 is not an integer\nDivision by zero\nexists|callable|2"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
