<?php
error_reporting(0);

class Milestone2116_Box {
    public $store = array();
    public $log = array();

    public function &__get($name) {
        $this->log[] = "get:" . $name;
        $this->store[$name]["ref"]["value"] = "get";
        return $this->store[$name];
    }
}

$ref = array("value" => "original");
$box = new Milestone2116_Box();
$box->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);

unset($box->slot["plain"]);

echo (isset($box->store["slot"]) ? "still" : "gone"),
    "|",
    (isset($box->store["slot"]["plain"]) ? "plain" : "plain-gone"),
    "|",
    implode(",", $box->log),
    "|",
    $ref["value"];
