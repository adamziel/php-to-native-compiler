<?php
error_reporting(0);

class Milestone2120_Bag implements ArrayAccess {
    public $store = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $this->log[] = "exists:" . $offset;
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        $slot =& $this->store[$offset];
        $slot["touch"] = "side";
        return $slot;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $this->log[] = "unset:" . $offset;
        unset($this->store[$offset]);
    }
}

$bag = new Milestone2120_Bag();
$bag->store["box"] = array("leaf" => "value");

$isset = isset($bag["box"]["leaf"]) ? "yes" : "no";
$empty = empty($bag["box"]["leaf"]) ? "empty" : "filled";
unset($bag["box"]["leaf"]);

echo $isset, "|", $empty, "|", (isset($bag->store["box"]["leaf"]) ? "still" : "gone"), "|",
    $bag->store["box"]["touch"], "|", implode(",", $bag->log);
