<?php
error_reporting(0);

function &milestone2257_pick_leaf(&$value) {
    return $value["ref"]["value"];
}

function milestone2257_make_holder($copy) {
    $holder = new stdClass();
    $holder->args = array($copy);
    return $holder;
}

class Milestone2257_Source {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2257_Sink implements ArrayAccess {
    public $seen = array();

    public function offsetSet($offset, $value) {
        $holder = milestone2257_make_holder($value);
        $alias =& call_user_func_array("milestone2257_pick_leaf", $holder->args);
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

$source = new Milestone2257_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$sink = new Milestone2257_Sink();
$sink["x"] = $source->slot;

echo $ref, "|", $sink["x"]["ref"]["value"], "|",
    $source->store["slot"]["plain"]["value"], "|", $sink["x"]["plain"]["value"];
