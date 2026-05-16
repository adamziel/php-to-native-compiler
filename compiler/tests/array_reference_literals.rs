use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn array_reference_elements_evaluate_current_values_without_aliasing() {
    let execution = run_source(
        r#"<?php
$value = "Ada";
$items = array(&$value, "name" => &$value);
echo $items[0], "|", $items["name"];
$value = "Grace";
echo "|", $items[0], "|", $items["name"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "Ada|Ada|Ada|Ada");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_copies_preserve_direct_reference_element_identity() {
    let execution = run_source(
        r#"<?php
$value = "x";
$left = [];
$left["slot"] =& $value;
$right = $left;
$right["slot"] = "b";
echo $value, "|", $left["slot"], "|", $right["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b|b|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn copied_reference_elements_sync_through_source_alias_and_copied_slot() {
    let execution = run_source(
        r#"<?php
$left = ["slot" => "x"];
$alias =& $left["slot"];
$right = $left;
$alias = "y";
echo $right["slot"], "|", $left["slot"];
$right["slot"] = "z";
echo "|", $alias, "|", $left["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "y|y|z|z");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_copies_without_references_remain_copy_on_write_values() {
    let execution = run_source(
        r#"<?php
$left = ["slot" => "x"];
$right = $left;
$right["slot"] = "b";
echo $left["slot"], "|", $right["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "x|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_copies_preserve_direct_reference_element_identity() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = [];
}
$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$copy = $box->items;
$copy["slot"] = "b";
echo $value, "|", $box->items["slot"], "|", $copy["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "b|b|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_copied_reference_elements_sync_through_source_alias_and_copied_slot() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["slot" => "x"];
}
$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$copy = $box->items;
$value = "y";
echo $copy["slot"], "|", $box->items["slot"];
$copy["slot"] = "z";
echo "|", $value, "|", $box->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "y|y|z|z");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_copies_without_references_remain_copy_on_write_values() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = ["slot" => "x"];
}
$box = new Box();
$copy = $box->items;
$copy["slot"] = "b";
echo $box->items["slot"], "|", $copy["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "x|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_property_array_reassignment_detaches_old_reference_elements() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = [];
}
$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$box->items = ["slot" => "new"];
$copy = $box->items;
$copy["slot"] = "b";
echo $value, "|", $box->items["slot"], "|", $copy["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "x|new|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn object_variable_reassignment_detaches_old_property_reference_elements() {
    let execution = run_source(
        r#"<?php
class Box {
    public $items = [];
}
$value = "x";
$box = new Box();
$box->items["slot"] =& $value;
$box = new Box();
$box->items["slot"] = "b";
echo $value, "|", $box->items["slot"];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "x|b");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_keys_remain_explicitly_unsupported() {
    let error = parse_error(
        r#"<?php
$key = "name";
$value = "Ada";
$items = array(&$key => $value);
"#,
    );

    assert_eq!(error.line, 4);
    assert_eq!(error.column, 17);
    assert_eq!(
        error.message,
        "unsupported array reference key: reference keys are not implemented"
    );
}
