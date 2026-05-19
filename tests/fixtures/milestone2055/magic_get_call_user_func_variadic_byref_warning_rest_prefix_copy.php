<?php
error_reporting(0);

function milestone2055_touch_bucket(&$ignored, &...$values) {
    $values[0]["ref"]["value"] = "inside";
    $values[0]["ref"] = "replaced";
    $values[0]["plain"]["value"] = "inside-plain";
    return $values[0];
}

class Milestone2055_Box {
    public $store = array();

    public function __get($name) {
        $ignored = "fixed";
        $bucket = $this->store[$name];
        return call_user_func("milestone2055_touch_bucket", $ignored, $bucket);
    }
}

$box = new Milestone2055_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
