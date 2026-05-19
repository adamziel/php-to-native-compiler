<?php
error_reporting(0);

class Milestone2118_Inner implements ArrayAccess {
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

class Milestone2118_Outer implements ArrayAccess {
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

class Milestone2118_Holder {
    public $bag;
}

$inner = new Milestone2118_Inner();
$holder = new Milestone2118_Holder();
$holder->bag = new Milestone2118_Outer($inner);

$isset = isset($holder->bag["box"]["leaf"]) ? "yes" : "no";
$empty = empty($holder->bag["box"]["leaf"]) ? "empty" : "filled";
unset($holder->bag["box"]["leaf"]);

echo $isset, "|", $empty, "|", (isset($inner->store["leaf"]) ? "still" : "gone"), "|",
    implode(",", $holder->bag->log), "|", implode(",", $inner->log);
