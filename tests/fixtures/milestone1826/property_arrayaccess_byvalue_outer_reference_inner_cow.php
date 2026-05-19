<?php
class Milestone1826Inner implements ArrayAccess {
    public $items = array("leaf" => "v");
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

class Milestone1826Outer implements ArrayAccess {
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

class Milestone1826Holder {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }
}

$holder = new Milestone1826Holder(new Milestone1826Outer(new Milestone1826Inner()));
$alias =& $holder->bag["box"]["leaf"];
$alias = "z";

echo "leaf=", $holder->bag->inner->items["leaf"],
    "|outer=", count($holder->bag->hits),
    "|inner=", count($holder->bag->inner->hits);
