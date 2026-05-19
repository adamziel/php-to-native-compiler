<?php
error_reporting(0);

function milestone2051_touch_bucket(&...$values) {
    $values[0]["ref"]["value"] = "inside";
    $values[0]["plain"]["value"] = "inside-plain";
    return $values[0];
}

class Milestone2051_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        return call_user_func("milestone2051_touch_bucket", $bucket);
    }
}

$box = new Milestone2051_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
