<?php
class Lane2288ArrayAccessCopiedLocalBag implements ArrayAccess {
    public $store = array();
    public $side = "";
    public $last = "";

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function &offsetGet(mixed $offset): mixed {
        $this->side = "side";
        $this->last = $offset;
        $copy = $this->store[$offset];
        return $copy;
    }

    public function offsetSet(mixed $offset, mixed $value): void {
        $this->store[$offset] = $value;
    }

    public function offsetUnset(mixed $offset): void {
        unset($this->store[$offset]);
    }
}

$bag = new Lane2288ArrayAccessCopiedLocalBag();
$bag->store = array(
    "slot" => array(
        "ref" => array("leaf" => "source-ref"),
        "plain" => array("leaf" => "source-plain"),
    ),
);
$alias =& $bag->store["slot"]["ref"];

$bag["slot"]["ref"]["leaf"] = "copy-ref";
$bag["slot"]["plain"]["leaf"] = "copy-plain";

echo $bag->store["slot"]["ref"]["leaf"], "|", $alias["leaf"], "|";
echo $bag->store["slot"]["plain"]["leaf"];
echo "|", $bag->side, "|", $bag->last;
