<?php
class Milestone1804Bag implements ArrayAccess {
    private $items = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    public function &slot($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $cb = array($this, "slot");
        $this->log[] = "get:" . $offset;
        return call_user_func($cb, $offset);
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

$bag = new Milestone1804Bag();
$alias =& $bag["slot"]["node"];
$alias["leaf"] = "call-user-func-array-callable";
$copy = $alias;
$copy["leaf"] = "call-user-func-array-callable-plain";

echo $bag["slot"]["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $bag->log);
