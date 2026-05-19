<?php
class Milestone1964_Inner implements ArrayAccess {
    public $items = array("leaf" => "seed");
    public $hits = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

class Milestone1964_Outer implements ArrayAccess {
    public $inner;
    public $hits = array();

    public function __construct($inner) {
        $this->inner = $inner;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->hits[] = $offset;
        return $this->inner;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

function milestone1964_mutate(&$value) {
    $value = "direct";
}

$outer = new Milestone1964_Outer(new Milestone1964_Inner());
milestone1964_mutate($outer["box"]["leaf"]);

echo "leaf=", $outer->inner->items["leaf"],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
