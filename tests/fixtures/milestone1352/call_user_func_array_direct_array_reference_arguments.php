<?php
function wp_refcow_update(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

function &wp_refcow_alias(&$value, $suffix) {
    $value = $value . ":" . $suffix;
    return $value;
}

$_REQUEST["payload"] = ["slot" => "request"];
$GLOBALS["wp_refcow_bag"] = ["slot" => "global"];
$items = ["outer" => ["slot" => "array"]];

call_user_func_array("wp_refcow_update", array(&$_REQUEST["payload"]["slot"], "callback"));
call_user_func_array("wp_refcow_update", array(&$GLOBALS["wp_refcow_bag"]["slot"], "callback"));
call_user_func_array("wp_refcow_update", array(&$items["outer"]["slot"], "callback"));

echo $_REQUEST["payload"]["slot"], "|", $GLOBALS["wp_refcow_bag"]["slot"], "|", $items["outer"]["slot"], "\n";

$request_alias =& call_user_func_array("wp_refcow_alias", array(&$_REQUEST["payload"]["slot"], "alias"));
$request_alias = $request_alias . ":done";

$global_alias =& call_user_func_array("wp_refcow_alias", array(&$GLOBALS["wp_refcow_bag"]["slot"], "alias"));
$global_alias = $global_alias . ":done";

$array_alias =& call_user_func_array("wp_refcow_alias", array(&$items["outer"]["slot"], "alias"));
$array_alias = $array_alias . ":done";

echo $_REQUEST["payload"]["slot"], "|", $request_alias, "\n";
echo $GLOBALS["wp_refcow_bag"]["slot"], "|", $global_alias, "\n";
echo $items["outer"]["slot"], "|", $array_alias;
