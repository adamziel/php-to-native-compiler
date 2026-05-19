<?php
class Milestone1968_Inner implements ArrayAccess {
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

class Milestone1968_Outer implements ArrayAccess {
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

class Milestone1968_Magic {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function __get($name) {
        return $this->bag;
    }
}

function milestone1968_mutate(&$value) {
    $value = "magic";
}

$outer = new Milestone1968_Outer(new Milestone1968_Inner());
$magic = new Milestone1968_Magic($outer);
milestone1968_mutate($magic->missing["box"]["leaf"]);

echo "leaf=", $outer->inner->items["leaf"],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
