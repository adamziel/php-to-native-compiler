<?php
error_reporting(0);

class Milestone2119_Inner implements ArrayAccess {
    public $store = array("leaf" => "value");
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $this->log[] = "iexists:" . $offset;
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "iget:" . $offset;
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $this->log[] = "iunset:" . $offset;
        unset($this->store[$offset]);
    }
}

class Milestone2119_Outer implements ArrayAccess {
    public $inner;
    public $log = array();

    public function __construct($inner) {
        $this->inner = $inner;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        $this->log[] = "exists:" . $offset;
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        return $this->inner;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {}

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        $this->log[] = "unset:" . $offset;
    }
}

class Milestone2119_Box {
    public $bag;
    public $log = array();

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function __isset($name) {
        $this->log[] = "isset:" . $name;
        return true;
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return $this->bag;
    }
}

$inner = new Milestone2119_Inner();
$box = new Milestone2119_Box(new Milestone2119_Outer($inner));

$isset = isset($box->missing["box"]["leaf"]) ? "yes" : "no";
$empty = empty($box->missing["box"]["leaf"]) ? "empty" : "filled";
unset($box->missing["box"]["leaf"]);

echo $isset, "|", $empty, "|", (isset($inner->store["leaf"]) ? "still" : "gone"), "|",
    implode(",", $box->log), "|", implode(",", $box->bag->log), "|", implode(",", $inner->log);
