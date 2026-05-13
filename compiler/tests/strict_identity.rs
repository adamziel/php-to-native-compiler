use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

#[test]
fn strict_identity_executes_for_current_scalar_subset() {
    let source = r#"<?php
function bit($value) {
    if ($value) {
        return "1";
    }
    return "0";
}

function row($label, $left, $right) {
    echo $label, ":",
        bit($left === $right),
        bit($left !== $right),
        "\n";
}

row("null|null", null, null);
row("null|false", null, false);
row("false|false", false, false);
row("false|int0", false, 0);
row("true|int1", true, 1);
row("int1|int1", 1, 1);
row("int1|float1", 1, 1.0);
row("float1|float1", 1.0, 1.0);
row("str1|int1", "1", 1);
row("str1|str1", "1", "1");
"#;

    let execution = run_source(source).unwrap();

    assert_eq!(
        execution.stdout,
        "\
null|null:10
null|false:01
false|false:10
false|int0:01
true|int1:01
int1|int1:10
int1|float1:01
float1|float1:10
str1|int1:01
str1|str1:10
"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn strict_identity_rejects_arrays_until_array_identity_exists() {
    let error = run_source("<?php\necho [] === [];\n").unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported comparison: strict identity for arrays is not implemented"
    );
}

#[test]
fn strict_identity_rejects_objects_until_object_identity_exists() {
    let error = run_source(
        r#"<?php
class Box {}
$left = new Box();
$right = new Box();
echo $left === $right;
"#,
    )
    .unwrap_err();

    assert_eq!(error.phase, Phase::Runtime);
    assert_eq!(error.line, 5);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported comparison: strict identity for objects is not implemented"
    );
}

#[test]
fn emit_ir_rejects_strict_identity_until_native_comparison_lowering_exists() {
    let error = emit_ir_source("<?php\necho 1 === 1;\n").unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert_eq!(
        error.message,
        "LLVM comparison lowering rejects comparison operators until native PHP comparison coercions exist; phpc run handles current scalar comparison diagnostics"
    );
}
