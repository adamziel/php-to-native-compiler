<?php
function wp_refcow_alias_literal_mark(&$value, $suffix) {
    $value = $value . ":" . $suffix;
}

function &wp_refcow_alias_literal_pick(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$value = "direct";
$registry = [];
$args =& $registry["args"];
$args = array(&$value, "literal");
call_user_func_array("wp_refcow_alias_literal_mark", $args);
echo $value, "|", $args[0], "|", $registry["args"][0], "\n";

$alias =& call_user_func_array("wp_refcow_alias_literal_pick", $args);
$alias = $alias . ":alias";
echo $value, "|", $args[0], "|", $registry["args"][0], "|", $alias, "\n";

$items = ["slot" => "array"];
$arrayArgs =& $registry["arrayArgs"];
$arrayArgs = array(&$items["slot"], "offset");
$copy = $registry["arrayArgs"];
$copy[0] = "copy";
echo $items["slot"], "|", $arrayArgs[0], "|", $registry["arrayArgs"][0], "|", $copy[0];
