<?php
class Milestone1962_Inner implements ArrayAccess {
    public $items = array("" => "empty");
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

class Milestone1962_Outer implements ArrayAccess {
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

class Milestone1962_Holder {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function holder() {
        return $this;
    }
}

$outer = new Milestone1962_Outer(new Milestone1962_Inner());
$holder = new Milestone1962_Holder($outer);

$alias =& $holder->holder()->bag["box"][];
$alias = "appended";

echo "leaf=", $outer->inner->items[""],
    "|outer=", implode(",", $outer->hits),
    "|inner=", implode(",", $outer->inner->hits);
