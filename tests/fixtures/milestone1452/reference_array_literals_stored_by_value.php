<?php
class WP_RefCow_Reference_Array_Literal_Box {
    public $items = ["slot" => "object"];
}

function wp_refcow_literal_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_literal_pick(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$value = "direct";
$args = array(&$value, "literal");
call_user_func_array("wp_refcow_literal_mark", $args);
echo $value, "|", $args[0], "\n";

$alias =& call_user_func_array("wp_refcow_literal_pick", $args);
$alias = $alias . ":alias";
echo $value, "|", $args[0], "|", $alias, "\n";

$items = ["slot" => "array"];
$arrayArgs = array(&$items["slot"], "offset");
$copy = $arrayArgs;
$copy[0] = "copy";
echo $items["slot"], "|", $arrayArgs[0], "|", $copy[0], "\n";

$box = new WP_RefCow_Reference_Array_Literal_Box();
$objectArgs = array("value" => &$box->items["slot"], "suffix" => "property");
call_user_func_array("wp_refcow_literal_mark", $objectArgs);
echo $box->items["slot"], "|", $objectArgs["value"];
