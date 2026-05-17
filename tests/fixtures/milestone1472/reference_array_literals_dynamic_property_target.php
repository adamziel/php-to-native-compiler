<?php
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
