<?php
class Milestone1965_Inner implements ArrayAccess {
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

class Milestone1965_Outer implements ArrayAccess {
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

class Milestone1965_Holder {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }
}

function milestone1965_mutate(&$value) {
    $value = "property";
}

$outer = new Milestone1965_Outer(new Milestone1965_Inner());
$holder = new Milestone1965_Holder($outer);
milestone1965_mutate($holder->bag["box"]["leaf"]);

echo "leaf=", $outer->inner->items["leaf"],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
