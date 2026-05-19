<?php
error_reporting(0);

class Milestone2266_Source {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2266_Sink implements ArrayAccess {
    public $seen = array();

    public function offsetSet($offset, $value) {
        $payload =& $value;
        $this->seen[$offset] = $payload;
    }

    public function offsetGet($offset) {
        return $this->seen[$offset];
    }

    public function offsetExists($offset) {
        return isset($this->seen[$offset]);
    }

    public function offsetUnset($offset) {
        unset($this->seen[$offset]);
    }
}

$source = new Milestone2266_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$sink = new Milestone2266_Sink();
$sink["x"] = $source->slot;
$sink["x"]["ref"]["value"] = "inside";

echo $ref, "|", $sink["x"]["ref"]["value"], "|", $sink["x"]["plain"]["value"];
