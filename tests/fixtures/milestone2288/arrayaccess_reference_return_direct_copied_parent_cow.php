<?php
class Lane2288ArrayAccessDirectCopiedParentBag implements ArrayAccess {
    public $store = array();
    public $side = "";
    public $last = "";

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function &offsetGet(mixed $offset): mixed {
        $this->side = "side";
        $this->last = $offset;
        $copy = $this->store;
        return $copy[$offset];
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Lane2288ArrayAccessDirectCopiedParentBag();
$bag->store = array("slot" => array("child" => "source-child"));
$alias =& $bag->store["slot"];

$bag["slot"]["child"] = "copy-child";
$bag["slot"]["extra"] = "alias-extra";

echo $bag->store["slot"]["child"], "|", $alias["child"], "|";
echo isset($alias["extra"]) ? $alias["extra"] : "missing";
echo "|", $bag->side, "|", $bag->last;
