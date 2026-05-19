<?php
function milestone2014_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
    return $value;
}

class Milestone2014_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $fn = "milestone2014_touch_bucket";
        return $fn($bucket);
    }
}

$box = new Milestone2014_Box();
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
