<?php
error_reporting(0);

class Milestone2279_Box implements ArrayAccess {
    public $store = array();

    public function keep($bucket) {
    }

    public function offsetGet($offset) {
        $bucket = $this->store;
        $this->keep($bucket);
        return $bucket[$offset];
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

$box = new Milestone2279_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box["slot"];
$copy["ref"]["value"] = "copy-ref";
$copy["plain"]["value"] = "copy-plain";

echo $ref, "|", $box->store["slot"]["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
