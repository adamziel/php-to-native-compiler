<?php
$store = array(
    "slot" => array(
        "ref" => array("leaf" => "source-ref"),
        "plain" => array("leaf" => "source-plain"),
    ),
);

function &get_store() {
    global $store;
    return $store;
}

$alias =& $store["slot"]["ref"];
$copy = call_user_func_array("get_store", array());
$copy["slot"]["ref"]["leaf"] = "copy-ref";
$copy["slot"]["plain"]["leaf"] = "copy-plain";

echo $store["slot"]["ref"]["leaf"], "|",
    $alias["leaf"], "|",
    $store["slot"]["plain"]["leaf"], "|",
    $copy["slot"]["plain"]["leaf"];
