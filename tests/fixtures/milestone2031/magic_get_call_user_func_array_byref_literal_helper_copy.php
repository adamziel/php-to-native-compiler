<?php
function milestone2031_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
    return $value;
}

class Milestone2031_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        return call_user_func_array("milestone2031_touch_bucket", array(&$bucket));
    }
}

$box = new Milestone2031_Box();
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
    $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
