use php_compiler::{emit_ir_source, run_source};

#[test]
fn setlocale_exposes_truthful_c_locale_capability() {
    let execution = run_source(
        r#"<?php
echo function_exists("setlocale") ? "fn" : "missing";
echo "|";
echo defined("LC_ALL") && defined("LC_CTYPE") && defined("LC_MESSAGES") ? "constants" : "no-constants";
echo "|";
echo setlocale(LC_ALL, 0);
echo "|";
echo setlocale(LC_CTYPE, "POSIX");
echo "|";
echo setlocale(LC_ALL, ["definitely_missing.UTF-8", "C"]);
echo "|";
echo setlocale(LC_TIME, "definitely_missing.UTF-8") === false ? "unsupported" : "claimed";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|constants|C|C|C|unsupported");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn setlocale_rejects_unsupported_argument_shapes_without_claiming_locale_support() {
    let bad_category = run_source("<?php\nsetlocale(LC_ALL, \"C\");\n").unwrap();
    assert_eq!(bad_category.stdout, "");
    assert_eq!(bad_category.exit_code, 0);

    let non_int_category = run_source("<?php\nsetlocale(\"LC_ALL\", \"C\");\n").unwrap_err();
    assert_eq!(
        non_int_category.message,
        "unsupported call setlocale(): category argument must be int in the current subset, got string"
    );

    let bad_locale = run_source("<?php\nsetlocale(LC_ALL, false);\n").unwrap_err();
    assert_eq!(
        bad_locale.message,
        "unsupported call setlocale(): locale argument must be string, array, int 0, or null in the current subset, got bool"
    );
}

#[test]
fn setlocale_metadata_folds_for_skipif_capability_checks() {
    let ir = emit_ir_source(
        r#"<?php
echo function_exists("setlocale") ? "1" : "0";
echo defined("LC_ALL") ? "1" : "0";
echo defined("LC_MESSAGES") ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.matches("c\"1\\00\"").count() >= 3, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("LC_ALL"), "{ir}");
    assert!(!ir.contains("LC_MESSAGES"), "{ir}");
}

#[test]
fn strcoll_uses_c_locale_byte_collation_subset() {
    let execution = run_source(
        r#"<?php
setlocale(LC_COLLATE, "C");
echo function_exists("strcoll") ? "fn" : "missing";
echo "|";
echo strcoll("a", "A") > 0 ? "gt" : "not";
echo "|";
echo strcoll("abc", "abc") === 0 ? "eq" : "not";
echo "|";
echo strcoll("A", "a") < 0 ? "lt" : "not";
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|gt|eq|lt");
    assert_eq!(execution.exit_code, 0);
}
