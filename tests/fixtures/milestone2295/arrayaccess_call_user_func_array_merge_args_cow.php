<?php
function milestone2295_arrayaccess_id($arr) {
    return $arr;
}

class Bag implements ArrayAccess {
    public $store;

    public function offsetGet(mixed $offset): mixed {
        $copy = $this->store[$offset];
        return call_user_func_array(
            "milestone2295_arrayaccess_id",
            array_merge(array($copy))
        );
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Bag();
$bag->store = array("a" => array("plain" => "old", "ref" => array("v" => "leaf")));
$alias =& $bag->store["a"]["ref"]["v"];
$tmp = $bag["a"];
$tmp["ref"]["v"] = "updated";
echo $bag->store["a"]["plain"], "\n";
echo $bag->store["a"]["ref"]["v"];
