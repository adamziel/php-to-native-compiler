<?php
error_reporting(0);

function &milestone2239_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2239_ArgsHolder {
    public $args = array();
}

function milestone2239_holder($copy) {
    $holder = new Milestone2239_ArgsHolder();
    $holder->args = array(&$copy);
    return $holder;
}

class Milestone2239_Source {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2239_Sink implements ArrayAccess {
    public $seen = array();

    public function offsetSet($name, $value) {
        $holder = milestone2239_holder($value);
        $alias =& call_user_func_array("milestone2239_pick_ref", $holder->args);
        $alias = "inside";
        $this->seen[$name] = $value;
    }

    public function offsetGet($name) {
        return $this->seen[$name];
    }

    public function offsetExists($name) {
        return isset($this->seen[$name]);
    }

    public function offsetUnset($name) {
        unset($this->seen[$name]);
    }
}

$source = new Milestone2239_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];
$sink = new Milestone2239_Sink();

$sink["slot"] = $source->slot;
$copy = $sink["slot"];
echo $ref, "|", $copy["ref"]["value"], "|",
    $source->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $source->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
