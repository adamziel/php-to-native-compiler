<?php
error_reporting(0);

class Milestone2090_Box {
    public $store = array();

    public function __set($name, $value) {
        $key = $name;
        $this->store[$key] = $value;
    }
}

$box = new Milestone2090_Box();
$ref = array("value" => "original");

$box->slot = array(
    "ref" => &$ref,
    "plain" => array("value" => "plain-original"),
);

$copy = $box->store["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $ref["value"], "|", $box->store["slot"]["plain"]["value"];
