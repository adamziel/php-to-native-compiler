<?php
class Milestone1850_Bag implements ArrayAccess {
    public $items = array();
    public $trace = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    public function &slot($offset) {
        $this->trace[] = "slot";
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get";
        return self::slot($offset);
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

$source = "seed";
$bag = new Milestone1850_Bag();
$bag->items["slot"] = array("value" => &$source, "plain" => array("value" => "copy"));

$alias =& $bag["slot"];
$alias["value"] = "changed";
$copy = $bag["slot"];
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $bag->items["slot"]["plain"]["value"], "|", implode(",", $bag->trace);
