<?php
error_reporting(0);

class Milestone1829Inner implements ArrayAccess {
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

class Milestone1829Outer implements ArrayAccess {
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

class Milestone1829Box {
    public $store;
    public $gets = array();

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        $this->gets[] = $name;
        return $this->store;
    }
}

$box = new Milestone1829Box(new Milestone1829Outer(new Milestone1829Inner()));
$alias =& $box->missing["box"]["leaf"];
$alias = "z";

echo "leaf=", $box->store->inner->items["leaf"],
    "|gets=", count($box->gets),
    "|outer=", count($box->store->hits),
    "|inner=", count($box->store->inner->hits);
