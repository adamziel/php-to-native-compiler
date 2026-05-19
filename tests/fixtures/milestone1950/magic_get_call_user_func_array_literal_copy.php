<?php
function milestone1950_id($value) {
    return $value;
}

class Milestone1950_Box {
    public $store = array();

    public function __get($name) {
        return call_user_func_array("milestone1950_id", array($this->store[$name]));
    }
}

$box = new Milestone1950_Box();
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
