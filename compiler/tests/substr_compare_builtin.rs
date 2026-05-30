use php_compiler::emit_ir_source;
use php_compiler::run_source;

#[test]
fn substr_compare_matches_php_offset_length_and_case_rows() {
    let execution = run_source(
        r#"<?php
var_dump(substr_compare("abcde", "df", -2) < 0);
var_dump(substr_compare("abcde", "df", -2, null) < 0);
var_dump(substr_compare("abcde", "bc", 1, 2));
var_dump(substr_compare("abcde", "bcg", 1, 2));
var_dump(substr_compare("abcde", "BC", 1, 2, true));
var_dump(substr_compare("abcde", "bc", 1, 3) > 0);
var_dump(substr_compare("abcde", "cd", 1, 2) < 0);
var_dump(substr_compare("abcde", "abc", 5, 1));
var_dump(substr_compare("abcde", "abcdef", -10, 10) < 0);
var_dump(substr_compare("abcde", "abc", 0, 0));
try {
    substr_compare("abcde", "abc", 0, -1);
} catch (\ValueError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(substr_compare("abcde", "abc", -1, NULL, -5) > 0);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "int(0)\n",
            "int(0)\n",
            "int(0)\n",
            "bool(true)\n",
            "bool(true)\n",
            "int(-1)\n",
            "bool(true)\n",
            "int(0)\n",
            "substr_compare(): Argument #4 ($length) must be greater than or equal to 0\n",
            "bool(true)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_compare_covers_empty_and_offset_boundaries() {
    let execution = run_source(
        r#"<?php
var_dump(substr_compare('', '', 0, 0));
var_dump(substr_compare('', '', 0));
var_dump(substr_compare('abc', '', 3, 0));
var_dump(substr_compare('abc', '', 3));
var_dump(substr_compare('abc', "\0", 3));
var_dump(substr_compare('/', '/asd', 0, 4));
try {
    substr_compare("abcde", "abc", 2147483647, 2147483647);
} catch (ValueError $exception) {
    echo $exception->getMessage(), "\n";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "int(0)\n",
            "int(0)\n",
            "int(0)\n",
            "int(0)\n",
            "int(-1)\n",
            "int(-1)\n",
            "substr_compare(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn substr_compare_is_callable_and_exposes_reflection_metadata() {
    let execution = run_source(
        r#"<?php
$call = "substr_compare";
echo function_exists("substr_compare") ? "fn" : "missing";
echo "|", is_callable($call) ? "callable" : "missing";
$function = new ReflectionFunction("substr_compare");
echo "|", $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
foreach ($function->getParameters() as $parameter) {
    echo "|", $parameter->getName(), ":", $parameter->isOptional() ? "optional" : "required";
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "fn|callable|substr_compare:3/5|haystack:required|needle:required|offset:required|length:optional|case_insensitive:optional"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_substr_compare_metadata_but_rejects_direct_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("substr_compare") ? "1" : "0";
echo is_callable("substr_compare") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source("<?php\necho substr_compare('abc', 'a', 0);\n").unwrap_err();
    assert!(
        error
            .message
            .contains("LLVM function-call lowering rejects function calls"),
        "{}",
        error.message
    );
}
