<?php
error_reporting(0);

class Milestone2104_Box {
    public $store = array();
    public $log = array();

    public function __set($name, $value) {
        $this->log[] = "set:" . $name;
        $key = $name;
        $this->store[$key] = $value;
    }
}

$box = new Milestone2104_Box();
$name = "slot";
$ref = array("value" => "original");

$box->$name = array(
    "ref" => &$ref,
    "plain" => array("value" => "plain-original"),
);

$copy = $box->store["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "local";

echo implode(",", $box->log), "|", $ref["value"], "|", $box->store["slot"]["plain"]["value"];
