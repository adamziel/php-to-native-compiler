use php_compiler::error::Phase;
use php_compiler::{emit_ir_source, run_source};

fn runtime_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Runtime);
    error
}

#[test]
fn array_column_extracts_array_rows_and_reindexes_values() {
    let source = r#"<?php
$rows = [];
$rows["first"] = ["name" => "Ada", "age" => 36];
$rows[7] = ["name" => null, "age" => 37];
$rows[] = ["age" => 38];
$rows[] = 42;

$names = array_column($rows, "name");
print_r($names);
$ages = array_column($rows, "age");
echo count($ages), "|", $ages[0], "|", $ages[1], "|", $ages[2], "\n";
$whole = array_column($rows, null);
echo count($whole), "|", $whole[0]["name"], "|", $whole[1]["age"], "|", $whole[2]["age"], "|", $whole[3], "\n";

$call = "array_column";
$dynamic = $call($rows, "name");
echo count($dynamic), "|", $dynamic[0], "|";
if ($dynamic[1] === null) {
    echo "null";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Ada\n    [1] => \n)\n3|36|37|38\n4|Ada|37|38|42\n2|Ada|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_column_extracts_public_object_rows_and_skips_missing_or_non_public_properties() {
    let source = r#"<?php
class Person {
    public $name;
    public $age;
    private $secret;
}

$ada = new Person();
$ada->name = "Ada";
$ada->age = 36;
$grace = new Person();
$grace->name = "Grace";
$grace->age = null;

$rows = [$ada, $grace, ["name" => "ArrayRow"], ["age" => 99], 42];
$names = array_column($rows, "name");
print_r($names);
$ages = array_column($rows, "age");
print_r($ages);
$secrets = array_column($rows, "secret");
echo count($secrets);
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => Ada\n    [1] => Grace\n    [2] => ArrayRow\n)\nArray\n(\n    [0] => 36\n    [1] => \n    [2] => 99\n)\n0"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_column_can_index_results_by_array_and_public_object_row_values() {
    let source = r#"<?php
class Person {
    public $id;
    public $name;
}

$person = new Person();
$person->id = "p1";
$person->name = "Linus";

$rows = [];
$rows[] = ["id" => 10, "name" => "Ada"];
$rows[] = ["id" => "10", "name" => "Grace"];
$rows[] = ["name" => "NoId"];
$rows[] = ["id" => "code", "name" => null];
$rows[] = $person;
$rows[] = 42;

$indexed = array_column($rows, "name", "id");
print_r($indexed);

$call = "array_column";
$whole = $call($rows, null, "id");
echo count($whole), "|", $whole[10]["name"], "|", $whole[11]["name"], "|", $whole["p1"]->name, "|";
if ($whole["code"]["name"] === null) {
    echo "null";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [10] => Grace\n    [11] => NoId\n    [code] => \n    [p1] => Linus\n)\n5|Grace|NoId|Linus|null"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_column_can_index_results_by_scalar_coerced_row_values() {
    let source = r#"<?php
$rows = [];
$rows[] = ["id" => true, "name" => "true"];
$rows[] = ["id" => false, "name" => "false"];
$rows[] = ["id" => null, "name" => "null"];
$rows[] = ["id" => 1.0, "name" => "float"];
$rows[] = ["name" => "missing"];

$indexed = array_column($rows, "name", "id");
print_r($indexed);

$call = "array_column";
$whole = $call($rows, null, "id");
echo count($whole), "|", $whole[1]["name"], "|", $whole[0]["name"], "|", $whole[""]["name"], "|", $whole[2]["name"];
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [1] => float\n    [0] => false\n    [] => null\n    [2] => missing\n)\n4|float|false|null|missing"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_column_requires_array_rows_argument() {
    let error = runtime_error("<?php\necho array_column(42, \"name\");\n");

    assert_eq!(error.line, 2);
    assert_eq!(error.column, 6);
    assert_eq!(
        error.message,
        "unsupported call array_column(): first argument must be array, got int"
    );
}

#[test]
fn array_column_rejects_unsupported_column_keys_and_index_key_argument() {
    let key_error = runtime_error("<?php\n$rows = [];\necho array_column($rows, true);\n");

    assert_eq!(key_error.line, 3);
    assert_eq!(key_error.column, 6);
    assert_eq!(
        key_error.message,
        "unsupported call array_column(): column key must be int, string, or null in the current subset, got bool"
    );

    let index_key_error = runtime_error(
        "<?php\n$rows = [[\"name\" => \"Ada\"]];\necho array_column($rows, \"name\", true);\n",
    );

    assert_eq!(index_key_error.line, 3);
    assert_eq!(index_key_error.column, 6);
    assert_eq!(
        index_key_error.message,
        "unsupported call array_column(): index key must be int, string, or null in the current subset, got bool"
    );

    let index_value_error = runtime_error("<?php\n$rows = [[\"id\" => 1.5, \"name\" => \"Ada\"]];\necho array_column($rows, \"name\", \"id\");\n");

    assert_eq!(index_value_error.line, 3);
    assert_eq!(index_value_error.column, 6);
    assert_eq!(
        index_value_error.message,
        "unsupported call array_column(): lossy or non-finite float index values are not supported; only null, bool, int, string, and integral finite float index values are implemented"
    );
}

#[test]
fn emit_ir_rejects_array_column_until_native_call_lowering_exists() {
    let error = emit_ir_source("<?php\necho array_column([[\"name\" => \"Ada\"]], \"name\");\n")
        .unwrap_err();

    assert_eq!(error.phase, Phase::Codegen);
    assert!(
        error.message.contains("array lowering rejects arrays"),
        "{}",
        error.message
    );

    let indexed_error = emit_ir_source(
        "<?php\necho array_column([[\"id\" => 1, \"name\" => \"Ada\"]], \"name\", \"id\");\n",
    )
    .unwrap_err();

    assert_eq!(indexed_error.phase, Phase::Codegen);
    assert!(
        indexed_error
            .message
            .contains("array lowering rejects arrays"),
        "{}",
        indexed_error.message
    );
}

#[test]
fn emit_ir_includes_array_column_in_native_callable_lookup_table() {
    let ir = emit_ir_source(
        r#"<?php
$name = "array_column";
echo function_exists("array_column") ? "1" : "0";
echo function_exists("ARRAY_COLUMN") ? "1" : "0";
echo function_exists($name) ? "1" : "0";
echo is_callable("array_column") ? "1" : "0";
echo is_callable($name, false) ? "1" : "0";
"#,
    )
    .unwrap();

    assert!(ir.contains("@.str.0"), "{ir}");
    assert!(!ir.contains("array_column"), "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
