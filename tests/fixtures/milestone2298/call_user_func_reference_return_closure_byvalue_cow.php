<?php
class Milestone2298_CallUserFuncBox {
    public $store = array();
}

$box = new Milestone2298_CallUserFuncBox();
$box->store = array(
    "slot" => array(
        "ref" => array("leaf" => "source-ref"),
        "plain" => array("leaf" => "source-plain"),
    ),
);

$alias =& $box->store["slot"]["ref"];
$callback = function &() use ($box) {
    return $box->store;
};

$copy = call_user_func($callback);
$copy["slot"]["ref"]["leaf"] = "copy-ref";
$copy["slot"]["plain"]["leaf"] = "copy-plain";

echo $box->store["slot"]["ref"]["leaf"], "|",
    $alias["leaf"], "|",
    $box->store["slot"]["plain"]["leaf"], "|",
    $copy["slot"]["plain"]["leaf"];
