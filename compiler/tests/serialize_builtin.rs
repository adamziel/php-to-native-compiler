use php_compiler::run_source;

#[test]
fn unserialize_warns_on_extra_data_and_returns_valid_prefix() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('i:5;i:6;'));
var_dump(unserialize('N;i:6;'));
var_dump(unserialize('b:1;i:6;'));
var_dump(unserialize('a:1:{s:3:"foo";b:1;}i:6;'));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 4 of 8 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 2 of 6 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Extra data starting at offset 20 of 24 bytes"));
    assert!(execution.stdout.contains("int(5)\n"));
    assert!(execution.stdout.contains("NULL\n"));
    assert!(execution.stdout.contains("bool(true)\n"));
    assert!(execution
        .stdout
        .contains("array(1) {\n  [\"foo\"]=>\n  bool(true)\n}\n"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unserialize_rejects_signed_lengths_and_reports_parser_offset() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('s:+1:"x";'));
var_dump(unserialize('a:-0:{}'));
var_dump(unserialize('O:+8:"stdClass":0:{}'));
var_dump(unserialize('O:-8:"stdClass":0:{}'));
var_dump(unserialize('C:+11:"ArrayObject":0:{}'));
var_dump(unserialize('C:-11:"ArrayObject":0:{}'));
var_dump(unserialize('a:1:{i:0;r:+1;}'));
var_dump(unserialize('a:1:{i:0;R:-1;}'));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Error at offset 0 of 9 bytes"));
    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Error at offset 0 of 7 bytes"));
    assert_eq!(
        execution
            .stdout
            .matches("Warning: unserialize(): Error at offset 0 of 20 bytes")
            .count(),
        2
    );
    assert_eq!(
        execution
            .stdout
            .matches("Warning: unserialize(): Error at offset 0 of 24 bytes")
            .count(),
        2
    );
    assert!(
        execution
            .stdout
            .matches("Warning: unserialize(): Error at offset 9 of 15 bytes")
            .count()
            >= 2
    );
    assert_eq!(execution.stdout.matches("bool(false)\n").count(), 8);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unserialize_truncated_object_properties_report_reached_offset() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('O:3:"obj":1:{'));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains("Warning: unserialize(): Error at offset 13 of 13 bytes"));
    assert_eq!(execution.stdout.matches("bool(false)\n").count(), 1);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unserialize_malformed_custom_payloads_report_offsets_and_guard_sizes() {
    let execution = run_source(
        r#"<?php
var_dump(unserialize('C:1:"A":3x{foo}'));
var_dump(unserialize('C:1:"A":3:xfoo}'));
var_dump(unserialize('C:1:"A":3:{foox'));
var_dump(unserialize('C:1:"A":'));
var_dump(unserialize('a:1000000000:{}'));
var_dump(unserialize('O:1000000000:"":0:{}'));
var_dump(unserialize('O:1:"X":1000000000:{}'));
var_dump(unserialize('C:1:"X":1000000000:{}'));
unserialize('O:2:"yy": ');
unserialize('O:2:"yy":: ');
unserialize('a:1:{i:0;R:1;}');
unserialize('a:1:{i:0;r:1;}');
"#,
    )
    .unwrap();

    for offset in [8, 9, 10, 13, 14, 18, 20] {
        assert!(
            execution
                .stdout
                .contains(&format!("unserialize(): Error at offset {offset}")),
            "{}",
            execution.stdout
        );
    }
    assert!(execution
        .stdout
        .contains("Insufficient data for unserializing - 1000000000 required, 1 present"));
    assert_eq!(execution.stdout.matches("bool(false)\n").count(), 8);
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn serialize_unserialize_objects_and_allowed_classes_options() {
    let execution = run_source(
        r#"<?php
class foo {
    public $x = "bar";
}
class D extends foo {}
$z = array(new foo(), 2, "3");
$s = serialize($z);

echo $s, "\n";
var_dump(unserialize($s));
var_dump(unserialize($s, ["allowed_classes" => false]));
var_dump(unserialize($s, ["allowed_classes" => ["bar"]]));
var_dump(unserialize($s, ["allowed_classes" => ["FOO"]]));
var_dump(unserialize(serialize(new D), ["allowed_classes" => ["foo"]]));
"#,
    )
    .unwrap();

    assert!(execution
        .stdout
        .contains(r#"a:3:{i:0;O:3:"foo":1:{s:1:"x";s:3:"bar";}i:1;i:2;i:2;s:1:"3";}"#));
    assert!(execution.stdout.contains("object(foo)#"));
    assert!(execution.stdout.contains("object(__PHP_Incomplete_Class)#"));
    assert!(execution
        .stdout
        .contains("[\"__PHP_Incomplete_Class_Name\"]=>\n    string(3) \"foo\""));
    assert!(execution
        .stdout
        .contains("[\"__PHP_Incomplete_Class_Name\"]=>\n  string(1) \"D\""));
    assert_eq!(execution.stdout.matches("object(foo)#").count(), 2);
    assert_eq!(
        execution
            .stdout
            .matches("object(__PHP_Incomplete_Class)#")
            .count(),
        3
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn serialize_unserialize_preserves_base_private_shadowed_by_child_protected() {
    let execution = run_source(
        r#"<?php
class Base {
    private $id;
    public function __construct($id) {
        $this->id = $id;
    }
}
class Derived extends Base {
    protected $id;
    public function __construct($id) {
        parent::__construct($id + 20);
        $this->id = $id;
    }
}
$a = new Derived(44);
$u = unserialize(serialize($a));
print_r($u);
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains("[id:Base:private] => 64"));
    assert!(execution.stdout.contains("[id:protected] => 44"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn unserialize_allowed_classes_validation_and_stringable_values() {
    let execution = run_source(
        r#"<?php
class foo {
    public $x = "bar";
}
class Name {
    public function __toString(): string {
        return 'Foo';
    }
}
$z = array(new foo(), 2, "3");
$s = serialize($z);

try {
    unserialize($s, ["allowed_classes" => null]);
} catch (TypeError $e) {
    echo "TypeError: ", $e->getMessage(), "\n";
}
try {
    unserialize($s, ["allowed_classes" => [42]]);
} catch (TypeError $e) {
    echo "TypeError: ", $e->getMessage(), "\n";
}
try {
    unserialize($s, ["allowed_classes" => ["  whitespace  "]]);
} catch (ValueError $e) {
    echo "ValueError: ", $e->getMessage(), "\n";
}
try {
    unserialize($s, ["allowed_classes" => ["have\0nul_byte"]]);
} catch (ValueError $e) {
    echo "ValueError: ", $e->getMessage(), "\n";
}
try {
    unserialize($s, ["allowed_classes" => [new stdClass]]);
} catch (Error $e) {
    echo "Error: ", $e->getMessage(), "\n";
}

var_dump(unserialize($s, ["allowed_classes" => [new Name]]));
"#,
    )
    .unwrap();

    assert!(execution.stdout.contains(
        "TypeError: unserialize(): Option \"allowed_classes\" must be of type array|bool, null given"
    ));
    assert!(execution.stdout.contains(
        "TypeError: unserialize(): Option \"allowed_classes\" must be an array of class names, int given"
    ));
    assert!(execution.stdout.contains(
        "ValueError: unserialize(): Option \"allowed_classes\" must be an array of class names, \"  whitespace  \" given"
    ));
    assert!(execution.stdout.contains(
        "ValueError: unserialize(): Option \"allowed_classes\" must be an array of class names, \"have\" given"
    ));
    assert!(execution
        .stdout
        .contains("Error: Object of class stdClass could not be converted to string"));
    assert!(execution.stdout.contains("object(foo)#"));
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
