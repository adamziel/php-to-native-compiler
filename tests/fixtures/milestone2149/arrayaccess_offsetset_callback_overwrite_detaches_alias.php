<?php
error_reporting(0);

class Milestone2149_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->log[] = "set";
        call_user_func(array($this, "replace"), $offset, $value);
    }

    public function replace($offset, $value) {
        $this->store = array($offset => array("ref" => $value));
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$bag = new Milestone2149_Bag();
$bag->store["slot"] = array("ref" => "old");
$alias =& $bag->store["slot"]["ref"];

$bag["slot"] = "new";
$alias = "alias";

echo implode(",", $bag->log), "|", $alias, "|", $bag->store["slot"]["ref"];
