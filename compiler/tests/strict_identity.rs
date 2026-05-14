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
fn emit_ir_lowers_same_type_dynamic_int_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
echo $sum === 3;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("select i1 %tmp1"), "{ir}");
    assert!(
        ir.contains("call i32 (ptr, ...) @printf(ptr @.fmt_str"),
        "{ir}"
    );
}

#[test]
fn emit_ir_lowers_same_type_dynamic_bool_strict_identity() {
    let ir = emit_ir_source(
        r#"<?php
$sum = 1 + 2;
$is_three = $sum === 3;
$choice = $is_three ? 3 : 4;
$maybe = $sum === $choice;
echo $maybe === true;
"#,
    )
    .unwrap();

    assert!(ir.contains("%tmp0 = add i64 1, 2"), "{ir}");
    assert!(ir.contains("%tmp1 = icmp eq i64 %tmp0, 3"), "{ir}");
    assert!(ir.contains("%tmp2 = select i1 %tmp1, i64 3, i64 4"), "{ir}");
    assert!(ir.contains("%tmp3 = icmp eq i64 %tmp0, %tmp2"), "{ir}");
    assert!(!ir.contains("icmp eq i1 %tmp3, true"), "{ir}");
    assert!(ir.contains("select i1 %tmp3"), "{ir}");
}
