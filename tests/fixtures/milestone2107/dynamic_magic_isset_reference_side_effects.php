<?php
error_reporting(0);

class Milestone2107_Box {
    public $store = array();
    public $log = array();

    public function __isset($name) {
        $this->log[] = "isset:" . $name;
        $this->store[$name]["ref"]["value"] = "seen";
        return isset($this->store[$name]);
    }
}

$ref = array("value" => "original");
$box = new Milestone2107_Box();
$box->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$name = "slot";

echo (isset($box->$name) ? "yes" : "no"),
    "|",
    implode(",", $box->log),
    "|",
    $ref["value"],
    "|",
    $box->store["slot"]["plain"]["value"];
