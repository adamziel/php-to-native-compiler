<?php
class Milestone1967_Inner implements ArrayAccess {
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

class Milestone1967_Outer implements ArrayAccess {
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

class Milestone1967_Holder {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }
}

function milestone1967_holder($bag) {
    return new Milestone1967_Holder($bag);
}

function milestone1967_mutate(&$value) {
    $value = "factory";
}

$outer = new Milestone1967_Outer(new Milestone1967_Inner());
milestone1967_mutate(milestone1967_holder($outer)->bag["box"]["leaf"]);

echo "leaf=", $outer->inner->items["leaf"],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
