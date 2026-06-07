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
    let execution = run_source(
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
    .expect("array write through int typed property reference should produce a PHP fatal");

    assert!(
        execution
            .stdout
            .contains("Cannot assign array to reference held by property Packet::$id of type int"),
        "unexpected stdout: {}",
        execution.stdout
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 255);
}
