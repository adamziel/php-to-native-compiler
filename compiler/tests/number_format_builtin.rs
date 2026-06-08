use php_compiler::run_source;

#[test]
fn number_format_handles_core_scalar_and_separator_cases() {
    let execution = run_source(
        r#"<?php
$values = [1234.5678, -1234.5678, 1234.6578e4, -1234.56789e4, 0x1234CDEF, 02777777777, "123456789", "123.456789", "12.3456789e1", true, false];
foreach ($values as $value) {
    echo number_format($value), "|";
}
echo "\n";
foreach ($values as $value) {
    echo number_format($value, 2, " DECIMALS ", " THOUSAND "), "|";
}
echo "\n";
echo number_format(2020.1415, 2, "F"), "|";
echo number_format(2020.1415, 2, null, "T"), "|";
echo number_format(2020.1415, 2, "F", null), "|";
echo number_format(-1.15E-15, 2), "|";
echo number_format(-0.01, 2), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "1,235|-1,235|12,346,578|-12,345,679|305,450,479|402,653,183|123,456,789|123|123|1|0|\n",
            "1 THOUSAND 234 DECIMALS 57|-1 THOUSAND 234 DECIMALS 57|12 THOUSAND 346 THOUSAND 578 DECIMALS 00|-12 THOUSAND 345 THOUSAND 678 DECIMALS 90|305 THOUSAND 450 THOUSAND 479 DECIMALS 00|402 THOUSAND 653 THOUSAND 183 DECIMALS 00|123 THOUSAND 456 THOUSAND 789 DECIMALS 00|123 DECIMALS 46|123 DECIMALS 46|1 DECIMALS 00|0 DECIMALS 00|\n",
            "2,020F14|2T020.14|2,020F14|0.00|-0.01\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn number_format_handles_negative_decimals_and_metadata() {
    let execution = run_source(
        r#"<?php
$values = [1.5151, 15.151, 151.51, 1515.1, 15151, -15151, 999, -999];
$decimals = [0, 1, 2, -1, -2, -3, -4, -5, PHP_INT_MIN];
foreach ($values as $value) {
    foreach ($decimals as $decimal) {
        echo number_format($value, $decimal), "|";
    }
    echo "\n";
}
$large_values = [PHP_INT_MAX, PHP_INT_MIN, "9223372036854775807", 9.223372036854776E+18, 9.223372036854775E+18, -9.223372036854776E+18];
$large_decimals = [5, 0, -1, -5, -10, -19, -20, PHP_INT_MIN];
foreach ($large_values as $value) {
    foreach ($large_decimals as $decimal) {
        echo number_format($value, $decimal), "|";
    }
    echo "\n";
}
echo function_exists("number_format") ? "exists" : "missing";
echo "|";
echo is_callable("number_format") ? "callable" : "missing";
echo "|";
$call = "number_format";
echo $call(1234.5678, 2), "|";
$reflection = new ReflectionFunction("number_format");
echo $reflection->getName(), "|", $reflection->invoke(1234.5678, 2);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "2|1.5|1.52|0|0|0|0|0|0|\n",
            "15|15.2|15.15|20|0|0|0|0|0|\n",
            "152|151.5|151.51|150|200|0|0|0|0|\n",
            "1,515|1,515.1|1,515.10|1,520|1,500|2,000|0|0|0|\n",
            "15,151|15,151.0|15,151.00|15,150|15,200|15,000|20,000|0|0|\n",
            "-15,151|-15,151.0|-15,151.00|-15,150|-15,200|-15,000|-20,000|0|0|\n",
            "999|999.0|999.00|1,000|1,000|1,000|0|0|0|\n",
            "-999|-999.0|-999.00|-1,000|-1,000|-1,000|0|0|0|\n",
            "9,223,372,036,854,775,807.00000|9,223,372,036,854,775,807|9,223,372,036,854,775,810|9,223,372,036,854,800,000|9,223,372,040,000,000,000|10,000,000,000,000,000,000|0|0|\n",
            "-9,223,372,036,854,775,808.00000|-9,223,372,036,854,775,808|-9,223,372,036,854,775,810|-9,223,372,036,854,800,000|-9,223,372,040,000,000,000|-10,000,000,000,000,000,000|0|0|\n",
            "9,223,372,036,854,775,807.00000|9,223,372,036,854,775,807|9,223,372,036,854,775,810|9,223,372,036,854,800,000|9,223,372,040,000,000,000|10,000,000,000,000,000,000|0|0|\n",
            "9,223,372,036,854,775,808.00000|9,223,372,036,854,775,808|9,223,372,036,854,775,808|9,223,372,036,854,800,384|9,223,372,040,000,000,000|10,000,000,000,000,000,000|0|0|\n",
            "9,223,372,036,854,774,784.00000|9,223,372,036,854,774,784|9,223,372,036,854,774,780|9,223,372,036,854,800,000|9,223,372,040,000,000,000|10,000,000,000,000,000,000|0|0|\n",
            "-9,223,372,036,854,775,808.00000|-9,223,372,036,854,775,808|-9,223,372,036,854,775,810|-9,223,372,036,854,800,000|-9,223,372,040,000,000,000|-10,000,000,000,000,000,000|0|0|\n",
            "exists|callable|1,234.57|number_format|1,234.57",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn number_format_rejects_trailing_junk_numeric_strings() {
    let execution = run_source(
        r#"<?php
foreach (["123abc", "12.3e1x", "123 abc"] as $value) {
    try {
        $formatted = number_format($value, 2);
        echo $value, "=>", $formatted, "\n";
    } catch (TypeError $e) {
        echo $value, "=>TypeError\n";
    }
}
echo "trim=>", number_format("  123  ", 2), "\n";
echo "exp=>", number_format("+12.3e1", 2), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "123abc=>TypeError\n",
            "12.3e1x=>TypeError\n",
            "123 abc=>TypeError\n",
            "trim=>123.00\n",
            "exp=>123.00\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn number_format_rejects_currently_unsupported_shapes() {
    let too_few = run_source("<?php\nnumber_format();\n").unwrap_err();
    assert_eq!(
        too_few.message,
        "arity mismatch for number_format(): expected 1 to 4 argument(s), got 0"
    );

    let too_many = run_source("<?php\nnumber_format(1, 2, '.', ',', true);\n").unwrap_err();
    assert_eq!(
        too_many.message,
        "arity mismatch for number_format(): expected 1 to 4 argument(s), got 5"
    );

    let array_arg = run_source("<?php\nnumber_format([]);\n").unwrap();
    assert_eq!(array_arg.exit_code, 255);
    assert!(array_arg.stdout.contains("number_format()"));
    assert!(array_arg.stdout.contains("array given"));
}
