<?php
error_reporting(0);

function &milestone2267_pick(&$value) {
    return $value["ref"]["value"];
}

class Milestone2267_Source {
    public $store = array();

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone2267_Holder implements ArrayAccess {
    public $store = array();

    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$source = new Milestone2267_Source();
$source->store = array(
    "slot" => array(
        "ref" => array("value" => "orig"),
        "plain" => array("value" => "plain"),
    ),
);
$ref =& $source->store["slot"]["ref"]["value"];

$bucket = $source->slot;
$holder = new Milestone2267_Holder();
$holder["args"] = array($bucket);
$alias =& call_user_func_array("milestone2267_pick", $holder["args"]);
$alias = "inside";

echo $ref, "|", $bucket["ref"]["value"], "|", $holder->store["args"][0]["ref"]["value"], "|", $holder->store["args"][0]["plain"]["value"];
