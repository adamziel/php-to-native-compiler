<?php
function milestone2060_handler($errno, $message, $file, $line) {
    echo "diag:", $message, "\n";
    return true;
}
set_error_handler("milestone2060_handler", E_ALL);

class Milestone2060_Bag implements ArrayAccess {
    public $items;

    public function __construct() {
        $this->items = array("slot" => false);
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
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

$bag = new Milestone2060_Bag();
$bag["slot"]["leaf"] = "x";
echo "type=", gettype($bag->items["slot"]);

