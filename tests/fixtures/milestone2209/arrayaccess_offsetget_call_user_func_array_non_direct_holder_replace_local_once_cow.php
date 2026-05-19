<?php
error_reporting(0);

function &milestone2209_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2209_ArgsHolder {
    public $args = array();
}

class Milestone2209_Bag implements ArrayAccess {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function holder(&$copy) {
        $this->calls = $this->calls + 1;
        $copy = array(
            "ref" => array("value" => "local-replacement"),
            "plain" => array("value" => "local-replacement"),
        );
        return $this->holder;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        $this->holder->args = array($bucket);
        $alias =& call_user_func_array(
            "milestone2209_pick_ref",
            $this->holder($bucket)->args
        );
        $alias = "inside";
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

$bag = new Milestone2209_Bag();
$bag->holder = new Milestone2209_ArgsHolder();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $bag->store["slot"]["ref"]["value"];

$copy = $bag["slot"];
echo $ref, "|", $copy["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $bag->calls, ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $bag->calls;
