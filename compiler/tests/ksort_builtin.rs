use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn ksort_numeric_sorts_direct_variable_array_keys_in_place() {
    let execution = run_source(
        r#"<?php
$items = [];
$items[10] = "ten";
$items[2] = "two";
$items["5"] = "five";
$result = ksort($items, SORT_NUMERIC);
echo $result ? "true" : "false";
echo "|";
echo array_keys($items)[0], ",", array_keys($items)[1], ",", array_keys($items)[2];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "true|2,5,10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ksort_numeric_sorts_direct_object_property_array_keys_in_place() {
    let execution = run_source(
        r#"<?php
class WP_Hook {
    public $callbacks = array();

    public function register() {
        $this->callbacks[10] = "ten";
        $this->callbacks[2] = "two";
        ksort($this->callbacks, SORT_NUMERIC);
        echo array_keys($this->callbacks)[0], "|", array_keys($this->callbacks)[1];
    }
}

$hook = new WP_Hook();
$hook->register();
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "2|10");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn ksort_rejects_unreached_sort_modes_and_non_array_targets() {
    let flag_error = runtime_error("<?php\n$items = [];\nksort($items, SORT_LOCALE_STRING);\n");
    assert_eq!(flag_error.line, 3);
    assert_eq!(flag_error.column, 1);
    assert!(
        flag_error
            .message
            .contains("sort flag parameter 5 is not supported"),
        "{}",
        flag_error.message
    );

    let target_error = runtime_error("<?php\n$value = 42;\nksort($value, SORT_NUMERIC);\n");
    assert_eq!(target_error.line, 3);
    assert_eq!(target_error.column, 1);
    assert_eq!(
        target_error.message,
        "unsupported call ksort(): first argument must be array, got int"
    );
}

#[test]
fn emit_ir_rejects_ksort_until_native_by_reference_array_lowering_exists() {
    let error = emit_ir_source("<?php\n$items = [];\nksort($items, SORT_NUMERIC);\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("function calls") || error.message.contains("arrays"),
        "{}",
        error.message
    );
}
