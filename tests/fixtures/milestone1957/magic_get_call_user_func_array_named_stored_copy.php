<?php
function milestone1957_id($value) {
    return $value;
}

class Milestone1957_Box {
    public $store = array();

    public function __get($name) {
        $args = array("value" => $this->store[$name]);
        return call_user_func_array("milestone1957_id", $args);
    }
}

$box = new Milestone1957_Box();
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
