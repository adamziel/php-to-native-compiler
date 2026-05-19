<?php
error_reporting(0);

function milestone2050_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "inside-plain";
    return $value;
}

class Milestone2050_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $fn = "milestone2050_touch_bucket";
        return call_user_func($fn, $bucket);
    }
}

$box = new Milestone2050_Box();
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
