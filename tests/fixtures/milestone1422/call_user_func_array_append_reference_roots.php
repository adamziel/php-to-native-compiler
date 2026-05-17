<?php
function wp_refcow_append_mark(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

function &wp_refcow_append_pick(&$value, $suffix) {
    $value = ($value === null ? "null" : $value) . ":" . $suffix;
    return $value;
}

class WP_RefCow_Append_Store {
    public $items = [];
}

$items = [];
$args = [];
$args[] =& $items[];
$args[] = "direct";
echo call_user_func_array("wp_refcow_append_mark", $args), "|", $items[0], "|", $args[0], "\n";

$alias =& call_user_func_array("wp_refcow_append_pick", $args);
$alias = $alias . ":alias";
echo $items[0], "|", $args[0], "|", $alias, "\n";

$store = new WP_RefCow_Append_Store();
$named = [];
$named["value"] =& $store->items[];
$named["suffix"] = "property";
echo call_user_func_array("wp_refcow_append_mark", $named), "|", $store->items[0], "|", $named["value"];
