<?php
function milestone2009_touch_bucket(&$value) {
    $value["ref"] = array("value" => "inside");
    return $value;
}

class Milestone2009_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        return milestone2009_touch_bucket($bucket);
    }
}

$box = new Milestone2009_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
