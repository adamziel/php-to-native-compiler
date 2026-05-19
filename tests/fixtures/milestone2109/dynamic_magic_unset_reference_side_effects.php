<?php
error_reporting(0);

class Milestone2109_Box {
    public $store = array();
    public $log = array();

    public function __unset($name) {
        $this->log[] = "unset:" . $name;
        $this->store[$name]["ref"]["value"] = "unset";
        unset($this->store[$name]["plain"]);
    }
}

$ref = array("value" => "original");
$box = new Milestone2109_Box();
$box->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$name = "slot";

unset($box->$name);

echo (isset($box->store["slot"]) ? "still" : "gone"),
    "|",
    (isset($box->store["slot"]["plain"]) ? "plain" : "plain-gone"),
    "|",
    implode(",", $box->log),
    "|",
    $ref["value"];
