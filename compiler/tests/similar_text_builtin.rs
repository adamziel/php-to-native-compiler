use php_compiler::emit_ir_source;
use php_compiler::error::Phase;
use php_compiler::run_source;

#[test]
fn similar_text_matches_basic_phpt_cases() {
    let execution = run_source(
        r#"<?php
ini_set("precision", "14");
var_dump(similar_text("abcdefgh", "efg"));
var_dump(similar_text("abcdefgh", "mno"));
var_dump(similar_text("abcdefghcc", "c"));
var_dump(similar_text("abcdefghabcdef", "zzzzabcdefggg"));

$percent = 0;
similar_text("abcdefgh", "efg", $percent);
var_dump($percent);
similar_text("abcdefgh", "mno", $percent);
var_dump($percent);
similar_text("abcdefghcc", "c", $percent);
var_dump($percent);
similar_text("abcdefghabcdef", "zzzzabcdefggg", $percent);
var_dump($percent);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "int(3)\nint(0)\nint(1)\nint(7)\nfloat(54.54545454545455)\nfloat(0)\nfloat(18.181818181818183)\nfloat(51.851851851851855)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn similar_text_metadata_dynamic_calls_and_percent_restrictions() {
    let execution = run_source(
        r#"<?php
$name = "similar_text";
echo function_exists($name) ? "fn" : "missing";
echo "|";
echo is_callable($name) ? "callable" : "missing";
echo "|";
echo $name("abcdef", "abzdef");
echo "|";
$function = new ReflectionFunction("Similar_Text");
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), ":", $function->getNumberOfParameters();
echo "|";
$percent = null;
echo similar_text("abcd", "abxy", $percent), ":", $percent;
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|5|similar_text:2:3|2:50");
    assert_eq!(execution.exit_code, 0);

    let non_variable_percent =
        run_source("<?php\nsimilar_text('abc', 'abc', $bucket['percent']);\n").unwrap_err();
    assert_eq!(non_variable_percent.phase, Phase::Runtime);
    assert_eq!(non_variable_percent.line, 2);
    assert_eq!(non_variable_percent.column, 1);
    assert_eq!(
        non_variable_percent.message,
        "unsupported call similar_text(): percent output must be a direct variable in the current subset"
    );
}

#[test]
fn emit_ir_folds_similar_text_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("similar_text") ? "1" : "0";
echo is_callable("similar_text") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
