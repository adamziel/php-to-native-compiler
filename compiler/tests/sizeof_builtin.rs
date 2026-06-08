use php_compiler::{emit_ir_source, run_source};

#[test]
fn sizeof_alias_counts_arrays_and_countable_objects() {
    let execution = run_source(
        r##"<?php
class Counter implements Countable {
    public function count() {
        return 4;
    }
}

$items = array("a" => 1, "b" => 2, "c" => 3);
echo sizeof($items), "|";
echo sizeof(new Counter()), "|";
echo sizeof(array_chunk(array(1, 2, 3, 4, 5), 2));
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3|4|3");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn count_and_sizeof_support_normal_and_recursive_modes() {
    let execution = run_source(
        r##"<?php
$items = [1, [2, [3, 4]], "empty" => []];
echo count($items), "|";
echo count($items, COUNT_NORMAL), "|";
echo count($items, COUNT_RECURSIVE), "|";
echo sizeof($items, 0), "|";
echo sizeof($items, 1), "|";
echo COUNT_NORMAL, ":", COUNT_RECURSIVE;
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "3|3|7|3|7|0:1");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn count_and_sizeof_mode_errors_are_php_shaped() {
    let execution = run_source(
        r##"<?php
try {
    count([1], 2);
} catch (ValueError $e) {
    echo "value:", $e->getMessage(), "\n";
}
try {
    sizeof([1], []);
} catch (TypeError $e) {
    echo "type:", $e->getMessage();
}
"##,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "value:count(): Argument #2 ($mode) must be either COUNT_NORMAL or COUNT_RECURSIVE\n\
type:sizeof(): Argument #2 ($mode) must be of type int, array given"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn sizeof_alias_metadata_is_available_for_capability_checks() {
    let execution = run_source(
        r##"<?php
$call = "sizeOf";
echo function_exists($call) ? "fn" : "missing";
echo "|";
echo is_callable($call) ? "callable" : "not";
echo "|";
$function = new ReflectionFunction($call);
echo $function->getName(), ":", $function->getNumberOfRequiredParameters(), "/", $function->getNumberOfParameters(), "|";
echo $function->invoke(array("x", "y"));
"##,
    )
    .unwrap();

    assert_eq!(execution.stdout, "fn|callable|sizeof:1/2|2");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn emit_ir_folds_sizeof_metadata() {
    let ir = emit_ir_source(
        r##"<?php
echo function_exists("sizeof") ? "1" : "0";
echo is_callable("sizeof") ? "1" : "0";
"##,
    )
    .unwrap();

    assert_eq!(ir.matches("c\"1\\00\"").count(), 2, "{ir}");
    assert!(!ir.contains("function_exists"), "{ir}");
    assert!(!ir.contains("is_callable"), "{ir}");
}
