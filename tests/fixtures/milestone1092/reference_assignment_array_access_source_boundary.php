<?php
function notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("notice_handler", E_NOTICE);

class Bag implements ArrayAccess {
    public $items = ["name" => "seed"];
    #[ReturnTypeWillChange]
    public function offsetExists($offset) { return false; }
    #[ReturnTypeWillChange]
    public function offsetGet($offset) { return $this->items[$offset]; }
    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) { $this->items[$offset] = $value; }
    #[ReturnTypeWillChange]
    public function offsetUnset($offset) { }
}

$bag = new Bag();
$key = "name";
$alias =& $bag[$key];
$alias = "changed";
echo $alias, "|", $bag->items[$key];
