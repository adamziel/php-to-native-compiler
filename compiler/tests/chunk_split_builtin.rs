use php_compiler::{emit_ir_source, run_source};

#[test]
fn chunk_split_splits_strings_with_default_and_custom_separators() {
    let execution = run_source(
        r#"<?php
echo chunk_split("abcdef", 2, "."), "|";
echo chunk_split("abc", 3, "+"), "|";
echo chunk_split("", 2, ".") === "." ? "empty-ending" : "not-empty";
echo "|", bin2hex(chunk_split("abcd", 2));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "ab.cd.ef.|abc+|empty-ending|61620d0a63640d0a"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chunk_split_preserves_php_string_literal_inputs_reached_by_phpt_variations() {
    let execution = run_source(
        r#"<?php
$chunklen = 6E+0;
$values = array(
    "Testing invalid \k and \m escape char",
    <<<EOT
This checks\t and \nwhite space chars plus \k
EOT,
);
foreach ($values as $value) {
    echo chunk_split($value, $chunklen, "+"), "|";
};
echo "done"
?>"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Testin+g inva+lid \\k+ and \\+m esca+pe cha+r+|This c+hecks\t+ and \n+white +space +chars +plus \\+k+|done"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chunk_split_uses_byte_lengths_and_accepts_empty_separator() {
    let execution = run_source(
        r#"<?php
echo bin2hex(chunk_split("abcdef", 2, "")), "|";
echo bin2hex(chunk_split("éé", 2, "-"));
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "616263646566|c3a92dc3a92d");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chunk_split_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r#"<?php
$call = "chunk_split";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$function = new ReflectionFunction("Chunk_Split");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|chunk_split:1/3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn chunk_split_rejects_missing_required_argument() {
    let execution = run_source(
        r#"<?php
try {
    chunk_split();
} catch (ArgumentCountError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert!(
        execution.stdout.contains("chunk_split()"),
        "{}",
        execution.stdout
    );
    assert!(execution.stdout.contains("expects"), "{}", execution.stdout);
}

#[test]
fn chunk_split_rejects_non_positive_lengths() {
    let zero = run_source(
        r#"<?php
try {
    chunk_split("abc", 0, ".");
} catch (ValueError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();
    assert_eq!(
        zero.stdout,
        "chunk_split(): Argument #2 ($length) must be greater than 0"
    );
}

#[test]
fn chunk_split_rejects_overflowed_float_lengths_as_type_errors() {
    let execution = run_source(
        r#"<?php
try {
    chunk_split("abc", PHP_INT_MAX * 3, ".");
} catch (TypeError $e) {
    echo $e->getMessage();
}
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "chunk_split(): Argument #2 ($length) must be of type int, float given"
    );
}

#[test]
fn emit_ir_folds_chunk_split_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("chunk_split") ? "1" : "0";
echo is_callable("chunk_split") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
