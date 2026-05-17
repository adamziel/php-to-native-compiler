<?php
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
