<?php
class Milestone1994_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $holder = array("outer" => array());
        $holder["outer"]["copy"] = $bucket;
        return $holder["outer"]["copy"];
    }
}

$box = new Milestone1994_Box();
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
