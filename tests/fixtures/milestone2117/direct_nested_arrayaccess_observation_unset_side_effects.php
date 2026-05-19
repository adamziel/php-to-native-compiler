<?php
error_reporting(0);

class Milestone2117_Inner implements ArrayAccess {
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

class Milestone2117_Outer implements ArrayAccess {
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

$inner = new Milestone2117_Inner();
$outer = new Milestone2117_Outer($inner);

$isset = isset($outer["box"]["leaf"]) ? "yes" : "no";
$empty = empty($outer["box"]["leaf"]) ? "empty" : "filled";
unset($outer["box"]["leaf"]);

echo $isset, "|", $empty, "|", (isset($inner->store["leaf"]) ? "still" : "gone"), "|",
    implode(",", $outer->log), "|", implode(",", $inner->log);
