<?php
class Milestone1807Bag implements ArrayAccess {
    private $items = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    public function &__call($method, $args) {
        $this->log[] = "call:" . $method;
        return $this->items[$args[0]];
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->log[] = "get:" . $offset;
        return $this->slot($offset);
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        if ($offset === null) {
            $this->items[] = $value;
            return;
        }
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$bag = new Milestone1807Bag();
$alias =& $bag["slot"]["node"];
$alias["leaf"] = "arrayaccess-magic-call";
$copy = $alias;
$copy["leaf"] = "arrayaccess-magic-call-plain";

echo $bag["slot"]["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $bag->log);
