<?php
function milestone1946_id($value) {
    $tmp = $value;
    return $tmp;
}

class Milestone1946_Box {
    public $store = array();

    public function __get($name) {
        return milestone1946_id($this->store[$name]);
    }
}

$box = new Milestone1946_Box();
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
