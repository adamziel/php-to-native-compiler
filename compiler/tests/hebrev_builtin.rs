use php_compiler::{emit_ir_source, run_source};

#[test]
fn hebrev_matches_basic_php_phpt_rows() {
    let execution = run_source(
        r#"<?php
$hebrew_text = "The hebrev function converts logical Hebrew text to visual text.\nThe function tries to avoid breaking words.\n";
var_dump(hebrev($hebrew_text));
var_dump(hebrev($hebrew_text, 15));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "string(109) \".The hebrev function converts logical Hebrew text to visual text\n",
            ".The function tries to avoid breaking words\n",
            "\"\n",
            "string(109) \"to visual text\n",
            "Hebrew text\n",
            "logical\n",
            "converts\n",
            "hebrev function\n",
            ".The\n",
            "breaking words\n",
            "tries to avoid\n",
            ".The function\n",
            "\"\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hebrev_preserves_bounded_byte_and_wrapping_edges() {
    let execution = run_source(
        r#"<?php
$heb = chr(224) . chr(225) . "(x)" . chr(226);
$mix = "abc " . chr(224) . chr(225) . "(12)" . chr(226) . " xyz\n";
foreach ([0, 2, 5, -1] as $max) {
    echo $max, ":", bin2hex(hebrev($heb, $max)), "\n";
}
echo "mix:", bin2hex(hebrev($mix, 5)), "\n";
echo "empty:", bin2hex(hebrev("")), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        concat!(
            "0:e2287829e1e0\n",
            "2:29e1e0e22878\n",
            "5:e2287829e1e0\n",
            "-1:e0e1297828e2\n",
            "mix:6162630a28313229e1e078797a20e20a\n",
            "empty:\n",
        )
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn hebrev_metadata_dynamic_call_and_native_membership_are_available() {
    let execution = run_source(
        r#"<?php
$call = "hebrev";
echo function_exists("hebrev") ? "fn" : "missing";
echo "|", is_callable("hebrev") ? "callable" : "not";
echo "|", str_replace("\n", "<n>", $call("abc def\n", 5));
$fn = new ReflectionFunction("hebrev");
echo "|", $fn->getName(), ":", $fn->getNumberOfRequiredParameters(), "/", $fn->getNumberOfParameters();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|def<n>abc<n>|hebrev:1/2");
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
