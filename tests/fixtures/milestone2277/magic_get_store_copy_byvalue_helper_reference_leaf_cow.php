<?php
error_reporting(0);

class Milestone2277_Box {
    public $store = array();

    public function keep($bucket) {
    }

    public function __get($name) {
        $bucket = $this->store;
        $this->keep($bucket);
        return $bucket[$name];
    }
}

$box = new Milestone2277_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy-ref";
$copy["plain"]["value"] = "copy-plain";

echo $ref, "|", $box->store["slot"]["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
