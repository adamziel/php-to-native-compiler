<?php
error_reporting(0);

class Milestone2114_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $this->log[] = "exists:" . $offset;
        $this->store[$offset]["ref"]["value"] = "exists";
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        $this->store[$offset]["ref"]["value"] = "get";
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $this->log[] = "unset:" . $offset;
        $this->store[$offset]["ref"]["value"] = "unset";
        unset($this->store[$offset]["plain"]);
    }
}

class Milestone2114_Box {
    public $bag;
    public $log = array();

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function __isset($name) {
        $this->log[] = "isset:" . $name;
        return true;
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return $this->bag;
    }
}

$ref = array("value" => "original");
$bag = new Milestone2114_Bag();
$bag->store = array(
    "slot" => array(
        "ref" => &$ref,
        "plain" => array("value" => "plain-original"),
    ),
);
$box = new Milestone2114_Box($bag);

$empty = empty($box->missing["slot"]) ? "empty" : "filled";
unset($box->missing["slot"]);

echo $empty,
    "|",
    (isset($bag->store["slot"]) ? "still" : "gone"),
    "|",
    (isset($bag->store["slot"]["plain"]) ? "plain" : "plain-gone"),
    "|",
    implode(",", $box->log),
    "|",
    implode(",", $bag->log),
    "|",
    $ref["value"];
