<?php
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
