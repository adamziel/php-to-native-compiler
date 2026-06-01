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
fn array_column_coerces_weak_scalar_keys_and_reports_type_errors() {
    let source = r#"<?php
$rows = [["php7", "foo"], ["php8", "bar"]];
print_r(array_column($rows, false));
print_r(array_column($rows, true));

$indexed = [["php" => 7, "foo"], ["php" => 8, "bar"]];
print_r(array_column($indexed, "php", false));
print_r(array_column($indexed, "php", true));

try {
    array_column($rows, []);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
try {
    array_column($indexed, "php", []);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [0] => php7\n    [1] => php8\n)\nArray\n(\n    [0] => foo\n    [1] => bar\n)\nArray\n(\n    [foo] => 7\n    [bar] => 8\n)\nArray\n(\n    [0] => 7\n    [1] => 8\n)\narray_column(): Argument #2 ($column_key) must be of type string|int|null, array given\narray_column(): Argument #3 ($index_key) must be of type string|int|null, array given\n"
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
fn array_column_rejects_non_int_string_null_keys_in_strict_types() {
    let source = r#"<?php
declare(strict_types=1);

$rows = [["php7", "foo"], ["php8", "bar"]];
$indexed = [["php" => 7, "foo"], ["php" => 8, "bar"]];

foreach ([false, true, 1.0, []] as $key) {
    try {
        var_dump(array_column($rows, $key));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}

foreach ([false, true, 1.0, []] as $key) {
    try {
        var_dump(array_column($indexed, "php", $key));
    } catch (TypeError $e) {
        echo $e->getMessage(), "\n";
    }
}

print_r(array_column($rows, 0));
print_r(array_column($rows, "1"));
print_r(array_column($rows, null));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "array_column(): Argument #2 ($column_key) must be of type string|int|null, false given\narray_column(): Argument #2 ($column_key) must be of type string|int|null, true given\narray_column(): Argument #2 ($column_key) must be of type string|int|null, float given\narray_column(): Argument #2 ($column_key) must be of type string|int|null, array given\narray_column(): Argument #3 ($index_key) must be of type string|int|null, false given\narray_column(): Argument #3 ($index_key) must be of type string|int|null, true given\narray_column(): Argument #3 ($index_key) must be of type string|int|null, float given\narray_column(): Argument #3 ($index_key) must be of type string|int|null, array given\nArray\n(\n    [0] => php7\n    [1] => php8\n)\nArray\n(\n    [0] => foo\n    [1] => bar\n)\nArray\n(\n    [0] => Array\n        (\n            [0] => php7\n            [1] => foo\n        )\n\n    [1] => Array\n        (\n            [0] => php8\n            [1] => bar\n        )\n\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_column_uses_object_string_keys_numeric_object_properties_and_magic_get() {
    let source = r#"<?php
class ColumnKey {
    public function __toString() {
        return "last_name";
    }
}
class IndexKey {
    public function __toString() {
        return "first_name";
    }
}
class Secret {
    private $prop;
    public function __construct($value) {
        $this->prop = $value;
    }
    public function __isset($name) {
        return $name === "prop";
    }
    public function __get($name) {
        return "__get($this->prop)";
    }
}

$records = [
    ["first_name" => "Ada", "last_name" => "Lovelace"],
    ["first_name" => "Grace", "last_name" => "Hopper"],
];
print_r(array_column($records, new ColumnKey(), new IndexKey()));

$numeric = new stdClass();
$numeric->{1} = "numeric";
$rows = [$numeric, new Secret("hidden")];
print_r(array_column($rows, 1));
print_r(array_column($rows, "prop"));
"#;

    let execution = run_source(source).unwrap();
    assert_eq!(
        execution.stdout,
        "Array\n(\n    [Ada] => Lovelace\n    [Grace] => Hopper\n)\nArray\n(\n    [0] => numeric\n)\nArray\n(\n    [0] => __get(hidden)\n)\n"
    );
    assert_eq!(execution.exit_code, 0);
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
