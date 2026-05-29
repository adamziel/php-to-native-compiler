use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn strspn_and_strcspn_execute_current_byte_mask_subset() {
    let execution = run_source(
        r#"<?php
$a = "22222222aaaa bbb1111 cccc";
$b = "1234";
echo strspn($a, $b), "|";
echo strspn($a, $b, 2), "|";
echo strspn($a, $b, 2, 3), "|";
echo strcspn($a, $b), "|";
echo strcspn($a, $b, 9), "|";
echo strcspn($a, $b, 9, 6), "|";
echo strcspn("a", "B", 1, 2147483647);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "8|6|3|0|7|6|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strspn_and_strcspn_apply_php_offset_and_length_windows() {
    let execution = run_source(
        r#"<?php
echo strspn("abc", "a", -4), "|";
echo strspn("abc", "a", 2147483647), "|";
echo strspn("abc", "abc", 1, -1), "|";
echo strcspn("abc", "z", -2147483648), "|";
echo strcspn("abc", "z", 1, -1), "|";
echo strcspn("abc", "z", 2147483647, 0);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "1|0|1|3|1|0");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strspn_and_strcspn_are_available_through_metadata_and_dynamic_calls() {
    let execution = run_source(
        r#"<?php
$spn = "strspn";
$cspn = "strcspn";
echo function_exists($spn) ? "fn" : "missing";
echo "|";
echo is_callable($cspn) ? "callable" : "missing";
echo "|";
echo $spn("abcdef", "abc");
echo "|";
echo $cspn("abcdef", "def");
echo "|";
$function = new ReflectionFunction("StrCspn");
echo $function->getName(), ":", $function->getNumberOfParameters(), ":", $function->getNumberOfRequiredParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|3|3|strcspn:4:2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strspn_and_strcspn_reject_arrays_and_bad_arity() {
    let array_subject = run_source("<?php\nstrspn(['abc'], 'a');\n").unwrap_err();
    assert_eq!(array_subject.phase, Phase::Runtime);
    assert_eq!(array_subject.line, 2);
    assert_eq!(array_subject.column, 1);
    assert_eq!(
        array_subject.message,
        "unsupported call strspn(): string argument arrays are not implemented in the current subset"
    );

    let array_characters = run_source("<?php\nstrcspn('abc', ['a']);\n").unwrap_err();
    assert_eq!(array_characters.phase, Phase::Runtime);
    assert_eq!(array_characters.line, 2);
    assert_eq!(array_characters.column, 1);
    assert_eq!(
        array_characters.message,
        "unsupported call strcspn(): characters argument arrays are not implemented in the current subset"
    );

    let bad_offset = run_source("<?php\nstrspn('abc', 'a', []);\n").unwrap();
    assert_eq!(bad_offset.exit_code, 255);
    assert!(
        bad_offset.stdout.contains(
            "TypeError: strspn(): Argument #3 ($offset) must be of type int, array given"
        ),
        "{}",
        bad_offset.stdout
    );

    let too_few = run_source("<?php\nstrcspn('abc');\n").unwrap_err();
    assert_eq!(too_few.phase, Phase::Runtime);
    assert_eq!(too_few.line, 2);
    assert_eq!(too_few.column, 1);
    assert_eq!(
        too_few.message,
        "arity mismatch for strcspn(): expected 2 to 4 argument(s), got 1"
    );
}

#[test]
fn emit_ir_folds_strspn_and_strcspn_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("strspn") ? "1" : "0";
echo is_callable("strcspn") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
