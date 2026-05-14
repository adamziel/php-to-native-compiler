use php_compiler::error::Phase;
use php_compiler::interpreter::MAX_USER_FUNCTION_CALL_DEPTH;
use php_compiler::run_source;
use std::thread;

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
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
fn undefined_variable_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho $missing;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined variable '$missing'");
}

#[test]
fn user_function_arity_mismatch_has_stable_runtime_error() {
    let error = runtime_error(
        "<?php\nfunction identity($value) {\n    return $value;\n}\necho identity();\n",
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "arity mismatch for identity(): expected 1 argument(s), got 0"
    );
}

#[test]
fn unsupported_builtin_call_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho count(1);\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call count(): only arrays are supported"
    );
}

#[test]
fn unsupported_array_key_has_stable_runtime_error() {
    let error = runtime_error("<?php\n$items = [true => \"yes\"];\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 11);
    assert_eq!(
        error.message,
        "invalid array key: bool keys are not supported; only int and string keys are implemented"
    );
}

#[test]
fn long_array_unsupported_key_uses_stable_runtime_error() {
    let error = runtime_error("<?php\n$items = array(true => \"yes\");\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 16);
    assert_eq!(
        error.message,
        "invalid array key: bool keys are not supported; only int and string keys are implemented"
    );
}

#[test]
fn undefined_array_key_has_stable_runtime_error() {
    let error = runtime_error("<?php\n$items = [];\necho $items[0];\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined array key 0");
}

#[test]
fn duplicate_class_has_stable_runtime_error() {
    let error = runtime_error("<?php\nclass Box {}\nclass box {}\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 1);
    assert_eq!(error.message, "class box is already defined");
}

#[test]
fn undefined_class_has_stable_runtime_error() {
    let error = runtime_error("<?php\n$box = new Missing();\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 8);
    assert_eq!(error.message, "undefined class Missing");
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
fn object_comparison_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
class Box {}
$left = new Box();
$right = new Box();
echo $left == $right;
"#,
    );

    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported comparison: object comparisons are not implemented"
    );
}

#[test]
fn undefined_object_property_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
class Box {}
$box = new Box();
echo $box->missing;
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "undefined property Box::$missing");
}

#[test]
fn invalid_property_target_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
$value = 1;
echo $value->name;
"#,
    );

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid property access: cannot read property $name from int"
    );
}

#[test]
fn non_public_property_access_has_stable_runtime_error() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
echo $box->secret;
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
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
        "invalid foreach: can only iterate arrays in the current subset, got int"
    );
}

#[test]
fn invalid_arithmetic_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho 1 / 0;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(error.message, "invalid arithmetic for /: division by zero");
}

#[test]
fn non_numeric_string_arithmetic_has_stable_runtime_error() {
    let error = runtime_error("<?php\necho \"abc\" + 1;\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "invalid arithmetic for +: string is not numeric"
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
    let error = runtime_error("<?php\n$items = [[1]];\necho isset($items[0][0]);\n");

    assert_eq!(error.line, 3);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported call isset(): only direct variables, direct array offset operands, direct object property operands, and supported static property operands are supported"
    );
}

#[test]
fn isset_non_public_property_access_remains_explicitly_unsupported() {
    let error = runtime_error(
        r#"<?php
class Box {
    private $secret;
}
$box = new Box();
echo isset($box->secret);
"#,
    );

    assert_eq!(error.line, 6);
    assert_eq!(error.column, 12);
    assert_eq!(
        error.message,
        "unsupported object property access: non-public property Box::$secret requires same-class method context in the current subset"
    );
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
