<?php
class Milestone1820Bag implements ArrayAccess {
    public static $slots = array(
        "slot" => "s0",
        "nested" => array("leaf" => "n0"),
    );

    public function offsetExists($offset): bool {
        return true;
    }

    public function &offsetGet($offset): mixed {
        echo "get:", $offset, "|";
        return self::$slots;
    }

    public function offsetSet($offset, $value): void {}
    public function offsetUnset($offset): void {}
}

$bag = new Milestone1820Bag();
$alias =& $bag["root"];
$alias["slot"] = "s1";
echo "slot=", Milestone1820Bag::$slots["slot"], "|";

$bag["root"]["nested"]["leaf"] = "n1";
echo "nested=", Milestone1820Bag::$slots["nested"]["leaf"];
