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
