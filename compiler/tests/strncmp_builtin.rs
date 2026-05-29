use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strncmp_executes_binary_safe_prefix_comparisons() {
    let execution = run_source(
        r#"<?php
echo strncmp("Hello", "Hello", 5), "|";
echo strncmp("Hello", "Hi", 5) < 0 ? "lt" : "bad";
echo "|";
echo strncmp("Hi", "Hello", 5) > 0 ? "gt" : "bad";
echo "|";
echo strncmp("abc", "abcd", 4), "|";
echo strncmp("Hello" . chr(0) . "world", "Hello" . chr(0), 12), "|";
echo strncmp(12345, "12399", "3");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|lt|gt|-1|1|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strncasecmp_executes_ascii_case_insensitive_prefix_comparisons() {
    let execution = run_source(
        r#"<?php
echo strncasecmp("ABC", "abc", 3), "|";
echo strncasecmp("Hello", "Hi", 5) < 0 ? "lt" : "bad";
echo "|";
echo strncasecmp("Hi", "Hello", 5) > 0 ? "gt" : "bad";
echo "|";
echo strncasecmp("Hello," . chr(0) . "world", "Hello,world", 12), "|";
echo strncasecmp("ABCDEF", "abcdxy", "4");
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "0|lt|gt|-119|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_prefix_compare_builtins_are_callable_and_reflectable() {
    let execution = run_source(
        r#"<?php
$cmp = "strncmp";
$casecmp = "strncasecmp";
echo function_exists($cmp) ? "yes" : "no";
echo "|";
echo is_callable($casecmp) ? "callable" : "missing";
echo "|";
echo $cmp("abc", "abd", 3) < 0 ? "lt" : "bad";
echo "|";
echo $casecmp("ABC", "abc", 3) === 0 ? "same" : "bad";
echo "|";
$function = new ReflectionFunction("StrNCmp");
$params = $function->getParameters();
echo $function->getName(), ":";
echo $function->getReturnType()->getName(), ":";
echo $params[0]->getName(), "/", $params[1]->getName(), "/", $params[2]->getName();
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|callable|lt|same|strncmp:int:string1/string2/length"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn string_prefix_compare_builtins_reject_current_unsupported_forms() {
    let negative = run_source(
        r#"<?php
try {
    strncmp("a", "a", -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        negative.stdout,
        "strncmp(): Argument #3 ($length) must be greater than or equal to 0"
    );
    assert_eq!(negative.exit_code, 0);

    let negative_case = run_source(
        r#"<?php
try {
    strncasecmp("a", "a", -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        negative_case.stdout,
        "strncasecmp(): Argument #3 ($length) must be greater than or equal to 0"
    );
    assert_eq!(negative_case.exit_code, 0);

    let array_left = run_source("<?php\nstrncmp([\"a\"], \"a\", 1);\n").unwrap_err();
    assert_eq!(array_left.phase, Phase::Runtime);
    assert_eq!(
        array_left.message,
        "unsupported call strncmp(): string1 argument arrays are not implemented in the current subset"
    );

    let bad_length = run_source("<?php\nstrncasecmp(\"a\", \"a\", []);\n").unwrap();
    assert_eq!(bad_length.exit_code, 255);
    assert!(bad_length.stdout.contains(
        "Fatal error: Uncaught TypeError: strncasecmp(): Argument #3 ($length) must be of type int, array given"
    ));
}

#[test]
fn emit_ir_routes_string_prefix_compare_builtins_through_native_contract() {
    let ir = emit_ir_source(
        r#"<?php
echo strncmp("abc", "abd", 3);
echo strncasecmp("ABC", "abc", 3);
"#,
    )
    .unwrap();

    assert!(
        ir.contains("declare i64 @phpc_native_value_string_int_operation_with_diagnostic"),
        "{ir}"
    );
    assert_eq!(
        ir.matches("call i64 @phpc_native_value_string_int_operation_with_diagnostic")
            .count(),
        2,
        "{ir}"
    );
    assert!(ir.contains("i8 3, ptr %"), "{ir}");
    assert!(ir.contains("i8 4, ptr %"), "{ir}");
}
