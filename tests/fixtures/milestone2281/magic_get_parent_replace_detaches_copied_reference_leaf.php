<?php
error_reporting(0);

class Milestone2281_Box {
    public $store;

    public function __construct() {
        $cell = "orig";
        $this->store = array(
            "slot" => array(
                "ref" => array("value" => &$cell),
                "plain" => "plain",
            ),
        );
    }

    public function __get($name) {
        $copy = $this->store[$name];
        $this->store[$name] = array(
            "ref" => array("value" => "replacement-ref"),
            "plain" => "replacement-plain",
        );
        return $copy;
    }
}

$box = new Milestone2281_Box();
$ref =& $box->store["slot"]["ref"]["value"];
$copy = $box->slot;

$ref = "copy-ref";
$copy["plain"] = "copy-plain";
$copy["ref"]["value"] = "copy-leaf";

echo $ref, "|",
    $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"], "|",
    $copy["plain"];
