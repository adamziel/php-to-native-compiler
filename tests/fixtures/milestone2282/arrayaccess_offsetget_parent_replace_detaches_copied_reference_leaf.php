<?php
error_reporting(0);

class Milestone2282_Box implements ArrayAccess {
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

    public function offsetGet($offset) {
        $copy = $this->store[$offset];
        $this->store[$offset] = array(
            "ref" => array("value" => "replacement-ref"),
            "plain" => "replacement-plain",
        );
        return $copy;
    }

    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$box = new Milestone2282_Box();
$ref =& $box->store["slot"]["ref"]["value"];
$copy = $box["slot"];

$ref = "copy-ref";
$copy["plain"] = "copy-plain";
$copy["ref"]["value"] = "copy-leaf";

echo $ref, "|",
    $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"], "|",
    $copy["plain"];
