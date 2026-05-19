<?php
class Milestone1966_Inner implements ArrayAccess {
    public $items = array("first" => "seed", "second" => "seed");
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

class Milestone1966_Outer implements ArrayAccess {
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

class Milestone1966_Holder {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function holder() {
        return $this;
    }
}

function milestone1966_mutate(&$value, $next) {
    $value = $next;
}

$outer = new Milestone1966_Outer(new Milestone1966_Inner());
$holder = new Milestone1966_Holder($outer);
$property = "bag";

milestone1966_mutate($holder->holder()->bag["box"]["first"], "method");
milestone1966_mutate($holder->holder()->{$property}["box"]["second"], "dynamic");

echo "first=", $outer->inner->items["first"],
    "|second=", $outer->inner->items["second"],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
