use php_compiler::error::Phase;
use php_compiler::interpreter::MAX_USER_FUNCTION_CALL_DEPTH;
use php_compiler::{run_source, run_source_with_execution_step_limit};
use std::thread;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

fn execution(source: &str) -> php_compiler::interpreter::Execution {
    let execution = run_source(source).unwrap();
    assert_eq!(execution.stderr, "");
    execution
}

fn with_large_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(f)
        .expect("large-stack test thread should spawn")
        .join()
        .expect("large-stack test thread should not panic")
}

#[test]
fn undefined_variable_read_emits_php_warning() {
    let execution = execution("<?php\necho $missing;\n");

    assert_eq!(
        execution.stdout,
        "Warning: Undefined variable $missing in Command line code on line 2\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn user_function_arity_mismatch_reports_php_fatal() {
    let execution =
        execution("<?php\nfunction identity($value) {\n    return $value;\n}\necho identity();\n");

    assert!(execution.stdout.starts_with(
        "Fatal error: Uncaught TypeError: Too few arguments to function identity(), 0 passed"
    ));
    assert!(execution.stdout.contains("exactly 1 expected"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn unsupported_builtin_call_reports_php_fatal() {
    let execution = execution("<?php\necho count(1);\n");

    assert!(execution.stdout.starts_with(
        "Fatal error: Uncaught TypeError: count(): Argument #1 ($value) must be of type Countable|array, int given"
    ));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn boolean_array_keys_are_normalized_in_short_arrays() {
    let execution = run_source(
        "<?php\n$items = [true => \"yes\", false => \"no\"];\necho $items[1], \"|\", $items[0];\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn boolean_array_keys_are_normalized_in_long_arrays() {
    let execution = run_source(
        "<?php\n$items = array(true => \"yes\", false => \"no\");\necho $items[1], \"|\", $items[0];\n",
    )
    .unwrap();

    assert_eq!(execution.stdout, "yes|no");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_array_key_read_emits_php_warning() {
    let execution = execution("<?php\n$items = [];\necho $items[0];\n");

    assert_eq!(
        execution.stdout,
        "Warning: Undefined array key 0 in Command line code on line 3\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn duplicate_class_has_stable_runtime_error() {
    let error = runtime_error("<?php\nclass Box {}\nclass box {}\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "class box is already defined");
}

#[test]
fn undefined_class_reports_php_fatal() {
    let execution = execution("<?php\n$box = new Missing();\n");

    assert!(execution
        .stdout
        .starts_with("Fatal error: Uncaught Error: Class \"Missing\" not found"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn object_to_string_conversion_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
echo $box;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid string conversion: object of class Box cannot be converted to string"
    );
}

#[test]
fn object_comparison_uses_current_php_truthy_result() {
    let execution = execution(
        r#"<?php
class Box {}
$left = new Box();
$right = new Box();
echo $left == $right;
"#,
    );

    assert_eq!(execution.stdout, "1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn undefined_object_property_read_emits_php_warning() {
    let execution = execution(
        r#"<?php
class Box {}
$box = new Box();
echo $box->missing;
"#,
    );

    assert_eq!(
        execution.stdout,
        "Warning: Undefined property: Box::$missing in Command line code on line 4\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn invalid_property_target_read_emits_php_warning() {
    let execution = execution(
        r#"<?php
$value = 1;
echo $value->name;
"#,
    );

    assert_eq!(
        execution.stdout,
        "Warning: Attempt to read property \"name\" on int in Command line code on line 3\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn non_public_property_access_reports_php_fatal() {
    let execution = execution(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
echo $box->secret;
"#,
    );

    assert!(execution
        .stdout
        .starts_with("Fatal error: Uncaught Error: Cannot access private property Box::$secret"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn array_offset_write_requires_array_compatible_target() {
    let error = runtime_error("<?php\n$value = 1;\n$value[] = 2;\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid array access: cannot write offset on int"
    );
}

#[test]
fn break_outside_loop_has_stable_runtime_error() {
    let error = runtime_error("<?php\nbreak;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid loop control: break cannot be used outside a loop"
    );
}

#[test]
fn continue_outside_loop_has_stable_runtime_error() {
    let error = runtime_error("<?php\ncontinue;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid loop control: continue cannot be used outside a loop"
    );
}

#[test]
fn continue_inside_switch_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
switch (1) {
    case 1:
        continue;
}
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 9);
    assert_eq!(
        error.message,
        "invalid loop control: continue inside switch is not implemented; use break for switch cases in the current subset"
    );
}

#[test]
fn foreach_non_array_iterable_has_stable_runtime_error() {
    let error = runtime_error("<?php\nforeach (42 as $value) echo $value;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "invalid foreach: can only iterate arrays, ordinary public-property objects, or bounded Iterator objects in the current subset, got int"
    );
}

#[test]
fn foreach_null_and_undefined_iterables_warn_and_continue() {
    let execution = run_source(
        "<?php\nfunction test() {\n    foreach (null as $value) { echo \"bad\"; }\n}\ntest();\nforeach ($missing as $value);\necho \"Done\\n\";\n",
    )
    .unwrap();

    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
    assert!(execution.stdout.contains(
        "Warning: foreach() argument must be of type array|object, null given in Command line code on line 3"
    ));
    assert!(execution
        .stdout
        .contains("Warning: Undefined variable $missing in Command line code on line 6"));
    assert!(execution.stdout.ends_with("Done\n"));
}

#[test]
fn division_by_zero_reports_php_fatal() {
    let execution = execution("<?php\necho 1 / 0;\n");

    assert!(execution
        .stdout
        .starts_with("Fatal error: Uncaught DivisionByZeroError: Division by zero"));
    assert_eq!(execution.exit_code, 255);
}

#[test]
fn non_numeric_string_arithmetic_is_catchable_type_error() {
    let execution = run_source(
        "<?php\ntry { echo \"abc\" + 1; } catch (TypeError $e) { echo $e->getMessage(); }\n",
    )
    .unwrap();
    assert_eq!(execution.stdout, "Unsupported operand types: string + int");

    let execution =
        run_source("<?php\nfunction add_bad($value) { echo $value + 1; }\nadd_bad(\"abc\");\n")
            .unwrap();
    assert_eq!(execution.exit_code, 255);
    assert!(
        execution
            .stdout
            .contains("Fatal error: Uncaught TypeError: Unsupported operand types: string + int"),
        "{}",
        execution.stdout
    );
    assert!(
        execution
            .stdout
            .contains("Stack trace:\n#0 Command line code(3): add_bad('abc')"),
        "{}",
        execution.stdout
    );
}

#[test]
fn numeric_string_operator_recovery_and_arithmetic_errors_are_catchable() {
    let execution = run_source(
        r#"<?php
error_reporting(E_ERROR);
var_dump("2abc" * "3");
var_dump("2abc" << "3.4a");
try { var_dump("abc" << "1"); } catch (TypeError $e) { echo "type:", $e->getMessage(), "\n"; }
try { var_dump(1 / 0); } catch (DivisionByZeroError $e) { echo "div:", $e->getMessage(), "\n"; }
try { var_dump(1 % 0); } catch (DivisionByZeroError $e) { echo "mod:", $e->getMessage(), "\n"; }
try { var_dump(1 << -1); } catch (ArithmeticError $e) { echo "shift:", $e->getMessage(), "\n"; }
try { var_dump(-"abc"); } catch (TypeError $e) { echo "unary:", $e->getMessage(), "\n"; }
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(6)\nint(16)\ntype:Unsupported operand types: string << string\ndiv:Division by zero\nmod:Modulo by zero\nshift:Bit shift by negative number\nunary:Unsupported operand types: string * int\n"
    );
}

#[test]
fn large_float_echo_and_operator_int_coercion_match_php_64bit_edges() {
    let execution = run_source(
        r#"<?php
$overflow = 9223372036854775807 + 1;
echo $overflow, "\n";
var_dump($overflow & -1);
var_dump($overflow | 7);
var_dump($overflow % 7);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "9.2233720368548E+18\nint(-9223372036854775808)\nint(-9223372036854775801)\nint(-1)\n"
    );
}

#[test]
fn isset_can_check_undefined_variables_without_reading_them() {
    let execution = run_source("<?php\necho isset($missing);\n$x = 1;\necho isset($x);\n")
        .expect("isset should not throw for missing direct variables");

    assert_eq!(execution.stdout, "1");
}

#[test]
fn complex_isset_operands_remain_explicitly_unsupported() {
    let error = runtime_error("<?php\nfunction make() { return [1]; }\necho isset(make()[0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, direct object-property array offset operands, and supported static property operands are supported"
    );
}

#[test]
fn isset_non_public_property_access_returns_false_without_read_error() {
    let execution = execution(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
echo isset($box->secret);
"#,
    );

    assert_eq!(execution.stdout, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn runaway_user_function_recursion_hits_stable_depth_guard() {
    let error = with_large_stack(|| {
        runtime_error(
            r#"<?php
function loop($n) {
    return loop($n + 1);
}
echo loop(0);
"#,
        )
    });

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        format!(
            "maximum user function call depth exceeded for loop(): limit {}",
            MAX_USER_FUNCTION_CALL_DEPTH
        )
    );
}

#[test]
fn execution_step_budget_reports_last_loop_location() {
    let error = run_source_with_execution_step_limit(
        r#"<?php
$i = 0;
while (true) {
    $i = $i + 1;
}
"#,
        8,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "maximum execution step budget exceeded after 8 step(s); last location <unknown>:3:1"
    );
}

#[test]
fn execution_step_budget_catches_empty_loop_bodies() {
    let error = run_source_with_execution_step_limit(
        r#"<?php
while (true) {
}
"#,
        3,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 1);
    assert_eq!(
        error.message,
        "maximum execution step budget exceeded after 3 step(s); last location <unknown>:2:1"
    );
}
