<?php
class Milestone1908_Mutator {
    public function mutate($array) {
        $array["leaf"] = "changed";
        $array["plain"]["value"] = "copy-changed";
    }
}

class Milestone1908_Bag implements ArrayAccess {
    public $items = array();
    public $trace = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get:" . $offset;
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$bag = new Milestone1908_Bag();
$bag->items["slot"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $bag->items["slot"]["leaf"];
$mutator = new Milestone1908_Mutator();

$mutator->mutate($bag["slot"]);

echo $alias, "|", $bag->items["slot"]["leaf"], "|", $bag->items["slot"]["plain"]["value"], "|", implode(",", $bag->trace);
