<?php
function milestone1958_id($value) {
    return $value;
}

function milestone1958_mutate($bucket) {
    $bucket["ref"]["value"] = "callee";
    $bucket["plain"]["value"] = "plain-callee";
}

class Milestone1958_Bag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $args = array($this->store[$offset]);
        return call_user_func_array("milestone1958_id", $args);
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

$bag = new Milestone1958_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $bag->store["slot"]["ref"]["value"];

milestone1958_mutate($bag["slot"]);

echo $alias, "|", $bag->store["slot"]["ref"]["value"], "|",
    $bag->store["slot"]["plain"]["value"];
