<?php
error_reporting(0);

class Milestone2250_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $alias =& $bucket["ref"]["value"];
        $alias = "copy-ref";
        $bucket["plain"]["value"] = "copy-plain";
        return $bucket;
    }
}

$box = new Milestone2250_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $box->store["slot"]["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
