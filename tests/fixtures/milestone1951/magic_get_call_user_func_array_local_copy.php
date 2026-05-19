<?php
function milestone1951_id($value) {
    $tmp = $value;
    return $tmp;
}

class Milestone1951_Box {
    public $store = array();

    public function __get($name) {
        return call_user_func_array("milestone1951_id", array($this->store[$name]));
    }
}

$box = new Milestone1951_Box();
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
