<?php
error_reporting(0);

class Milestone2160_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get";
        $value = $this->store[$offset];
        $replace = function($key) {
            $this->store = array($key => array("ref" => "new"));
        };
        call_user_func_array($replace, array($offset));
        return $value;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2160_Bag();
$bag->store["slot"] = array("ref" => "old");
$alias =& $bag->store["slot"]["ref"];

$value = $bag["slot"];
$value["ref"] = "value";
$alias = "alias";

echo implode(",", $bag->log), "|", $alias, "|", $value["ref"], "|", $bag->store["slot"]["ref"];
