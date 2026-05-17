use php_compiler::error::Phase;
use php_compiler::run_source;

fn parse_error(source: &str) -> php_compiler::error::Diagnostic {
    let error = run_source(source).unwrap_err();
    assert_eq!(error.phase, Phase::Parse);
    error
}

#[test]
fn array_reference_elements_stored_by_value_preserve_direct_variable_aliases() {
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

    assert_eq!(execution.stdout, "Ada|Ada|Grace|Grace");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_elements_stored_by_value_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
function mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}
$value = "seed";
$args = array(&$value, "stored");
call_user_func_array("mark", $args);
echo $value, "|", $args[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:stored|seed:stored");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_alias_backed_variable_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
function mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}
$value = "seed";
$registry = [];
$args =& $registry["args"];
$args = array(&$value, "stored");
call_user_func_array("mark", $args);
echo $value, "|", $args[0], "|", $registry["args"][0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "seed:stored|seed:stored|seed:stored");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_elements_stored_by_value_preserve_array_offset_aliases() {
    let execution = run_source(
        r#"<?php
$items = ["slot" => "seed"];
$args = array(&$items["slot"]);
$copy = $args;
$copy[0] = "copy";
echo $items["slot"], "|", $args[0], "|", $copy[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "copy|copy|copy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_alias_backed_variable_preserve_array_offset_aliases() {
    let execution = run_source(
        r#"<?php
$items = ["slot" => "seed"];
$registry = [];
$args =& $registry["args"];
$args = array(&$items["slot"]);
$copy = $registry["args"];
$copy[0] = "copy";
echo $items["slot"], "|", $args[0], "|", $registry["args"][0], "|", $copy[0];
"#,
    )
    .unwrap();

    assert_eq!(execution.stdout, "copy|copy|copy|copy");
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_object_property_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
class RefcowLiteralStore {
    public $args = [];
}

function mark_refcow_literal_property(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &pick_refcow_literal_property(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$value = "seed";
$store = new RefcowLiteralStore();
$store->args = array(&$value, "property");
call_user_func_array("mark_refcow_literal_property", $store->args);
echo $value, "|", $store->args[0], "\n";

$alias =& call_user_func_array("pick_refcow_literal_property", $store->args);
$alias = $alias . ":alias";
echo $value, "|", $store->args[0], "|", $alias, "\n";

$items = ["slot" => "array"];
$store->args = array(&$items["slot"], "offset");
$copy = $store->args;
$copy[0] = "copy";
echo $items["slot"], "|", $store->args[0], "|", $copy[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:property|seed:property\nseed:property:property:alias|seed:property:property:alias|seed:property:property:alias\ncopy|copy|copy"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_array_offset_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
function mark_refcow_literal_offset(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &pick_refcow_literal_offset(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$value = "seed";
$registry = [];
$registry["args"] = array(&$value, "offset");
call_user_func_array("mark_refcow_literal_offset", $registry["args"]);
echo $value, "|", $registry["args"][0], "\n";

$alias =& call_user_func_array("pick_refcow_literal_offset", $registry["args"]);
$alias = $alias . ":alias";
echo $value, "|", $registry["args"][0], "|", $alias, "\n";

$items = ["slot" => "array"];
$registry["args"] = array(&$items["slot"], "copy");
$copy = $registry["args"];
$copy[0] = "copied";
echo $items["slot"], "|", $registry["args"][0], "|", $copy[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:offset|seed:offset\nseed:offset:offset:alias|seed:offset:offset:alias|seed:offset:offset:alias\ncopied|copied|copied"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_dynamic_property_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
function mark_refcow_literal_dynamic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &pick_refcow_literal_dynamic(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$property = "args";
$value = "seed";
$store = new stdClass();
$store->{$property} = array(&$value, "dynamic");
call_user_func_array("mark_refcow_literal_dynamic", $store->args);
echo $value, "|", $store->args[0], "\n";

$alias =& call_user_func_array("pick_refcow_literal_dynamic", $store->args);
$alias = $alias . ":alias";
echo $value, "|", $store->args[0], "|", $alias, "\n";

$items = ["slot" => "array"];
$store->{$property} = array(&$items["slot"], "copy");
$copy = $store->args;
$copy[0] = "copied";
echo $items["slot"], "|", $store->args[0], "|", $copy[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:dynamic|seed:dynamic\nseed:dynamic:dynamic:alias|seed:dynamic:dynamic:alias|seed:dynamic:dynamic:alias\ncopied|copied|copied"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn array_reference_literals_assigned_to_append_offsets_feed_call_user_func_array() {
    let execution = run_source(
        r#"<?php
class RefcowLiteralAppendStore {
    public $groups = [];
}

function mark_refcow_literal_append(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &pick_refcow_literal_append(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$value = "seed";
$args = [];
$args[] = array(&$value, "direct");
call_user_func_array("mark_refcow_literal_append", $args[0]);
echo $value, "|", $args[0][0], "\n";

$alias =& call_user_func_array("pick_refcow_literal_append", $args[0]);
$alias = $alias . ":alias";
echo $value, "|", $args[0][0], "|", $alias, "\n";

$items = ["slot" => "array"];
$registry = ["groups" => []];
$registry["groups"][] = array(&$items["slot"], "nested");
$copy = $registry["groups"][0];
$copy[0] = "copied";
echo $items["slot"], "|", $registry["groups"][0][0], "|", $copy[0], "\n";

$property_items = ["slot" => "property"];
$store = new RefcowLiteralAppendStore();
$store->groups[] = array(&$property_items["slot"], "property");
$stored = $store->groups[0];
call_user_func_array("mark_refcow_literal_append", $stored);
echo $property_items["slot"], "|", $store->groups[0][0], "|", $stored[0];
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "seed:direct|seed:direct\nseed:direct:direct:alias|seed:direct:direct:alias|seed:direct:direct:alias\ncopied|copied|copied\nproperty:property|property:property|property:property"
    );
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
