<?php
function milestone2000_passthrough(&$value) {
    return $value;
}

class Milestone2000_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = "milestone2000_passthrough";
        return $helper($bucket);
    }
}

$box = new Milestone2000_Box();
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
