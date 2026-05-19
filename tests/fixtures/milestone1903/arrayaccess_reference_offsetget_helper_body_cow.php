<?php
class Milestone1903_Bag implements ArrayAccess {
    public $items = array();
    public $trace = array();

    private function &pick($offset) {
        $this->trace[] = "pick:" . $offset;
        if ($offset === "slot") {
            return $this->items[$offset];
        }
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get:" . $offset;
        return $this->pick($offset);
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

$bag = new Milestone1903_Bag();
$bag->items["slot"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $bag->items["slot"]["leaf"];

$copy = $bag["slot"];
$copy["leaf"] = "changed";
$copy["plain"]["value"] = "copy-changed";

echo $alias, "|", $bag->items["slot"]["leaf"], "|", $bag->items["slot"]["plain"]["value"], "|", implode(",", $bag->trace);
