<?php
function milestone1995_passthrough($value) {
    $tmp = $value;
    return $tmp;
}

class Milestone1995_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $holder = array("copy" => $bucket);
        $args = array();
        $args[0] = $holder["copy"];
        return call_user_func_array("milestone1995_passthrough", $args);
    }
}

$box = new Milestone1995_Box();
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
