<?php
error_reporting(0);

class Milestone2065_Box {
    public $store = array();

    public function __get($name) {
        $property = "store";
        $bucket =& $this->{$property}[$name];
        return $bucket;
    }
}

$box = new Milestone2065_Box();
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

