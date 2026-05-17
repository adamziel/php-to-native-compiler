<?php
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
