<?php
function milestone1977_passthrough($value) {
    return $value;
}

class Milestone1977_Box {
    public $store = array();

    public function __get($name) {
        $helper = "milestone1977_passthrough";
        return $helper($this->store[$name]);
    }
}

$box = new Milestone1977_Box();
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
