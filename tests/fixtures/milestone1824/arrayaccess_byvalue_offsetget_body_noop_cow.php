<?php
error_reporting(0);

class Milestone1824Bag implements ArrayAccess {
    public $items = array(
        "x" => array("leaf" => "v"),
        "" => "append",
    );
    public $hits = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->hits[] = gettype($offset) . ":" . (string) $offset;
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

$bag = new Milestone1824Bag();
$alias =& $bag["x"]["leaf"];
$alias = "detached";
$bag["x"]["leaf"] = "write-noop";
$append =& $bag[];

echo "leaf=", $bag->items["x"]["leaf"],
    "|alias=", $alias,
    "|append=", $append,
    "|hits=", count($bag->hits),
    "|last=", $bag->hits[count($bag->hits) - 1];
