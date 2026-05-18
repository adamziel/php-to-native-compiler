<?php
class Milestone1802Bag implements ArrayAccess {
    public $items = array();
    public $log = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $fn = function &(&$value) {
            return $value;
        };
        $this->log[] = "get:" . $offset;
        return call_user_func_array($fn, array(&$this->items[$offset]));
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

$bag = new Milestone1802Bag();
$alias =& $bag["slot"]["node"];
$alias["leaf"] = "call-user-func-array-closure";
$copy = $alias;
$copy["leaf"] = "call-user-func-array-closure-plain";

echo $bag["slot"]["node"]["leaf"], "|", $copy["leaf"], "|", implode("|", $bag->log);
