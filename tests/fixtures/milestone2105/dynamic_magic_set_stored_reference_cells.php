<?php
error_reporting(0);

class Milestone2105_Box {
    public $store = array();

    public function __set($name, $value) {
        $key = $name;
        $this->store[$key] = $value;
    }
}

$box = new Milestone2105_Box();
$name = "slot";
$ref = array("value" => "original");
$payload = array(
    "ref" => &$ref,
    "plain" => array("value" => "plain-original"),
);

$box->$name = $payload;

$copy = $box->store["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo $ref["value"], "|", $box->store["slot"]["plain"]["value"];
