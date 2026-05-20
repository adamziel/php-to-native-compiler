<?php
error_reporting(0);

class Milestone2283_Source {
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
}

class Milestone2283_Sink implements ArrayAccess {
    public $source;
    public $stored;

    public function __construct($source) {
        $this->source = $source;
    }

    public function offsetGet($offset) {
        return null;
    }

    public function offsetSet($offset, $value) {
        $this->stored = $value;
        $this->source->store[$offset] = array(
            "ref" => array("value" => "replacement-ref"),
            "plain" => "replacement-plain",
        );
    }

    public function offsetExists($offset) {
        return false;
    }

    public function offsetUnset($offset) {
    }
}

$source = new Milestone2283_Source();
$sink = new Milestone2283_Sink($source);
$ref =& $source->store["slot"]["ref"]["value"];
$copy = $source->store["slot"];

$sink["slot"] = $copy;
$ref = "copy-ref";
$sink->stored["plain"] = "copy-plain";
$sink->stored["ref"]["value"] = "copy-leaf";

echo $ref, "|",
    $source->store["slot"]["ref"]["value"], "|",
    $sink->stored["ref"]["value"], "|",
    $source->store["slot"]["plain"], "|",
    $sink->stored["plain"];
