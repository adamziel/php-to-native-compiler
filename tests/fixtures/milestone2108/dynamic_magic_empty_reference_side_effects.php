<?php
error_reporting(0);

class Milestone2108_Box {
    public $store = array();
    public $log = array();

    public function __isset($name) {
        $this->log[] = "isset:" . $name;
        $this->store[$name]["ref"]["value"] = "isset";
        return isset($this->store[$name]);
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $this->store[$name]["ref"]["value"] = "get";
        return $this->store[$name];
    }
}

$ref = array("value" => "original");
$box = new Milestone2108_Box();
$box->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$name = "slot";

echo (empty($box->$name) ? "empty" : "filled"),
    "|",
    implode(",", $box->log),
    "|",
    $ref["value"],
    "|",
    $box->store["slot"]["plain"]["value"];
