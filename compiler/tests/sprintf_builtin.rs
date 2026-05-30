use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

const LLVM_FUNCTION_CALL_REJECTION: &str = "LLVM function-call lowering rejects function calls, including user functions, callable builtins outside define()/constant()/defined(), and dynamic string-valued calls, until native runtime call lookup, stack frames, arity/type diagnostics, and callback dispatch exist; phpc run handles current function-call behavior";

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn sprintf_executes_current_string_placeholder_subset() {
    let execution = run_source(
        r#"<?php
echo sprintf("Hello %s", "Ada"), "\n";
echo sprintf('%2$s:%1$s', "one", "two"), "\n";
echo sprintf('%% %s %1$s', "done"), "\n";
echo sprintf('%05d|%.2F|%5s|%-5s|%1$s|%s', "7", 12, "x", "y", "z"), "\n";
echo sprintf('%.4d|%b|%c|%u|%o|%x|%X|%.2e', -42, 5, 65, 9, 9, 255, 255, 1000), "\n";
echo sprintf('%e|%u|%o', +3e3, +2345432, +0567), "\n";
echo sprintf('%g|%.3G|%*.*f|%.*s|%d', 1.234567, 12345.0, 8, 2, 1.25, 3, "abcdef", "world"), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Hello Ada\ntwo:one\n% done done\n00007|12.00|    x|y    |7|z\n-42|101|A|9|11|ff|FF|1.00e+3\n3.000000e+3|2345432|567\n1.23457|1.23E+4|    1.25|abc|0\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_and_vsprintf_are_available_through_string_valued_calls() {
    let execution = run_source(
        r#"<?php
$call = "sprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", $call('%1$s-%2$s', "wp", "php");
echo "|";
$call = "vsprintf";
echo function_exists($call) ? "yes" : "no";
echo "|", is_callable($call) ? "callable" : "missing";
echo "|", $call("SELECT option_value FROM wp_options WHERE option_name = '%s' LIMIT %d", ["rewrite_rules", 1]);
$call = "vprintf";
echo "|", function_exists($call) ? "yes" : "no";
echo "|";
$length = $call('%2$s/%1$s', ["left", "right"]);
echo "|", $length;
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "yes|wp-php|yes|callable|SELECT option_value FROM wp_options WHERE option_name = 'rewrite_rules' LIMIT 1|yes|right/left|10"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn vprintf_outputs_formatted_string_and_returns_byte_length() {
    let execution = run_source(
        r#"<?php
$length = vprintf("%s:%04d:%x", ["id", 7, 255]);
echo "\n", $length, "\n";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "id:0007:ff\n10\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn vprintf_handles_flagged_percent_and_binary_char_output() {
    let execution = run_source(
        r#"<?php
$length = vprintf("% %%d", [1234, -5678]);
echo "\n", $length, "\n";
$length = vprintf("%c", [191]);
echo "\n", $length;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "%-5678\n6\n\u{fffd}\n1");
    assert_eq!(execution.stdout_bytes, b"%-5678\n6\n\xbf\n1".to_vec());
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn vprintf_values_argument_type_error_is_catchable() {
    let execution = run_source(
        r#"<?php
try {
    vprintf("%s", true);
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "vprintf(): Argument #2 ($values) must be of type array, true given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn php_int_size_constant_matches_runtime_integer_width() {
    let execution = run_source(
        r#"<?php
echo defined("PHP_INT_SIZE") ? "yes" : "no";
echo "|", PHP_INT_SIZE;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|8");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_unsigned_radices_wrap_to_runtime_integer_width() {
    let execution = run_source(
        r#"<?php
echo sprintf("%b|%u|%x|%X|%o", -1, -1, -2, -2, -1);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "1111111111111111111111111111111111111111111111111111111111111111|18446744073709551615|fffffffffffffffe|FFFFFFFFFFFFFFFE|1777777777777777777777"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_formatter_residuals_cover_padding_float_overflow_and_binary_var_dump() {
    let execution = run_source(
        r#"<?php
echo sprintf("%-07.2d", 1234), "\n";
echo vprintf("%10.4u", [10e20]), "\n";
var_dump(sprintf("%c", 0xd2));
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains("1234   \n"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Warning: The float 1.0E+21 is not representable as an int, cast occurred"),
        "{}",
        execution.stdout
    );
    assert!(
        execution.stdout.contains("3875820019684212736"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout_bytes
            .windows(b"string(1) \"\xd2\"\n".len())
            .any(|window| window == b"string(1) \"\xd2\"\n"),
        "{:?}",
        execution.stdout_bytes
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_left_aligned_zero_padding_matches_php_by_placeholder_kind() {
    let execution = run_source(
        r#"<?php
echo sprintf("%-05.2f|%-07.2f|%-07.2F|%+-07.2f", 3.4, -5.6, 3.4, 3.4), "\n";
echo sprintf("%-07.2d|%-07s|%-'#7.2f", 1234, "xy", 3.4);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "3.400|-5.6000|3.40000|+3.4000\n1234   |xy00000|3.40###"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn json_encode_covers_current_formatter_descriptor_values() {
    let execution = run_source(
        r#"<?php
echo json_encode(null), "|", json_encode(false), "|", json_encode(true), "\n";
echo json_encode([1, "x"]), "|", json_encode(["color" => "red"]), "|", json_encode(new stdClass), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "null|false|true\n[1,\"x\"]|{\"color\":\"red\"}|{}\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sprintf_rejects_format_forms_outside_current_subset() {
    let placeholder = run_source(
        r#"<?php
try {
    echo sprintf("%a", 4);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(placeholder.stdout, "Unknown format specifier \"a\"");
    assert_eq!(placeholder.exit_code, 0);

    let vsprintf_args = run_source(
        r#"<?php
echo vsprintf("%s", "not-array");
"#,
    );
    let vsprintf_args = vsprintf_args.unwrap();
    assert_eq!(vsprintf_args.exit_code, 255);
    assert!(
        vsprintf_args.stdout.starts_with(
            "Fatal error: Uncaught TypeError: vsprintf(): Argument #2 ($values) must be of type array, string given"
        ),
        "{}",
        vsprintf_args.stdout
    );

    let vsprintf_star = run_source(
        r#"<?php
try {
    echo vsprintf("%*2f", [1, 2]);
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(vsprintf_star.stdout, "Unknown format specifier \"*\"");
    assert_eq!(vsprintf_star.exit_code, 0);

    let vprintf_args = run_source(
        r#"<?php
echo vprintf("%s", "not-array");
"#,
    );
    let vprintf_args = vprintf_args.unwrap();
    assert_eq!(vprintf_args.exit_code, 255);
    assert!(
        vprintf_args.stdout.starts_with(
            "Fatal error: Uncaught TypeError: vprintf(): Argument #2 ($values) must be of type array, string given"
        ),
        "{}",
        vprintf_args.stdout
    );
}

#[test]
fn emit_ir_folds_sprintf_metadata_but_rejects_runtime_formatting_calls() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("sprintf") ? "1" : "0";
echo is_callable("vsprintf") ? "1" : "0";
echo function_exists("vprintf") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");

    let error = emit_ir_source(
        r#"<?php
echo sprintf("Hello %s", "Ada");
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
echo vsprintf("Hello %s", ["Ada"]);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);

    let error = emit_ir_source(
        r#"<?php
echo vprintf("Hello %s", ["Ada"]);
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, LLVM_FUNCTION_CALL_REJECTION);
}
