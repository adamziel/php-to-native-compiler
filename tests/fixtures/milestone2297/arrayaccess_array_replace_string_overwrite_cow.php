<?php
class Bag implements ArrayAccess {
    public $left;
    public $right;

    public function offsetGet(mixed $offset): mixed {
        $left = $this->left;
        $right = $this->right;
        $replaced = array_replace(array("slot" => $left), array("slot" => $right));
        return $replaced["slot"];
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->right[$offset] = $value;
    }

    public function offsetExists(mixed $offset): bool {
        return isset($this->right[$offset]);
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->right[$offset]);
    }
}

$bag = new Bag();
$bag->left = array("ref" => array("v" => "left"));
$leftAlias =& $bag->left["ref"]["v"];
$bag->right = array("ref" => array("v" => "right"));
$rightAlias =& $bag->right["ref"]["v"];
$tmp = $bag["missing"];
$tmp["ref"]["v"] = "updated";
echo $bag->left["ref"]["v"], "\n";
echo $bag->right["ref"]["v"];
