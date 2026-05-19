<?php
error_reporting(0);

function &milestone2260_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

class Milestone2260_Source {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2260_Sink implements ArrayAccess {
    public $seen = array();
    public $args = array();

    public function prepare($copy, &$args) {
        $args = array($copy);
    }

    public function offsetSet($offset, $value) {
        $this->args = array();
        $this->prepare($value, $this->args);
        $alias =& call_user_func_array("milestone2260_pick_leaf", $this->args);
        $alias = "inside";
        $this->seen[$offset] = $value;
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

$source = new Milestone2260_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$sink = new Milestone2260_Sink();
$sink["x"] = $source->slot;

echo $ref, "|", $sink["x"]["ref"]["value"], "|",
    $source->store["slot"]["plain"]["value"], "|", $sink["x"]["plain"]["value"];
