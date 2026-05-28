use php_compiler::run_source;

#[test]
fn typed_property_reference_assignment_coerces_shared_reference_writes() {
    let execution = run_source(
        r#"<?php
class Test {
    public string $x;
    public string $y;
}

$test = new Test;
$ref = "";
$test->x =& $ref;
$test->y =& $ref;
$val = 42;
$ref = $val;
var_dump($ref, $val);
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "string(2) \"42\"\nint(42)\n");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn typed_property_reference_assignment_rejects_incompatible_writes() {
    let error = run_source(
        r#"<?php
class Packet {
    public int $id;
}

$packet = new Packet;
$ref = 1;
$packet->id =& $ref;
$ref = [];
"#,
    )
    .expect_err("array write through int typed property reference should fail");

    assert!(
        error
            .to_string()
            .contains("typed property Packet::$id expects int, got array"),
        "unexpected error: {error}"
    );
}
