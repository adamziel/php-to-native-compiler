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
