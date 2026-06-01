use php_compiler::{emit_ir_source, run_source};

#[test]
fn hebrev_matches_basic_php_phpt_rows() {
    let execution = run_source(
        r#"<?php
echo "*** Testing hebrev() : basic functionality ***\n";
$hebrew_text = "The hebrev function converts logical Hebrew text to visual text.\nThe function tries to avoid breaking words.\n";
var_dump(hebrev($hebrew_text));
var_dump(hebrev($hebrew_text, 15));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "*** Testing hebrev() : basic functionality ***\n\
string(109) \".The hebrev function converts logical Hebrew text to visual text\n\
.The function tries to avoid breaking words\n\
\"\n\
string(109) \"to visual text\n\
Hebrew text\n\
logical\n\
converts\n\
hebrev function\n\
.The\n\
breaking words\n\
tries to avoid\n\
.The function\n\
\"\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hebrev_preserves_php_line_breaking_and_punctuation_edges() {
    let execution = run_source(
        r#"<?php
echo bin2hex(hebrev("abc")), "|";
echo bin2hex(hebrev("abc", 1)), "|";
echo bin2hex(hebrev("abc", -1)), "|";
echo hebrev(".abc\n.def\n", -1), "|";
echo hebrev("abc\ndef", 1);
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "616263|626361|636261|cba.\n.fed\n|bca\nefd"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hebrev_metadata_dynamic_call_and_stringable_inputs_are_available() {
    let execution = run_source(
        r#"<?php
class LineBox {
    public function __toString() {
        return "abc";
    }
}
$call = "hebrev";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
echo bin2hex($call("abc", -1)), "|";
echo bin2hex(hebrev(new LineBox(), 1)), "|";
$reflection = new ReflectionFunction("Hebrev");
echo $reflection->getName(), ":", $reflection->getNumberOfRequiredParameters(), "/", $reflection->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|636261|626361|hebrev:1/2");
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_hebrev_function_metadata() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("hebrev") ? "1" : "0";
echo is_callable("hebrev") ? "1" : "0";
"#,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("hebrev"), "{ir}");
}
