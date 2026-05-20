<?php
class Lane2288PlainMagicParentBox {
    public $store = array();
    public $last = "";

    public function &__get($name) {
        $this->last = $name;
        $copy = $this->store;
        return $copy[$name];
    }
}

class Lane2288PlainMagicChildBox {
    public $store = array();
    public $last = "";

    public function &__get($name) {
        $this->last = $name;
        $copy = $this->store[$name];
        return $copy;
    }
}

class Lane2288PlainArrayAccessParentBag implements ArrayAccess {
    public $store = array();
    public $last = "";

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function &offsetGet(mixed $offset): mixed {
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

class Lane2288PlainArrayAccessChildBag implements ArrayAccess {
    public $store = array();
    public $last = "";

    public function offsetExists(mixed $offset): bool {
        return isset($this->store[$offset]);
    }

    public function &offsetGet(mixed $offset): mixed {
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

$magicParent = new Lane2288PlainMagicParentBox();
$magicParent->store = array("slot" => array("leaf" => "magic-parent-source"));
$magicParent->slot["leaf"] = "magic-parent-copy";

$arrayParent = new Lane2288PlainArrayAccessParentBag();
$arrayParent->store = array("slot" => array("leaf" => "array-parent-source"));
$arrayParent["slot"]["leaf"] = "array-parent-copy";

$magicChild = new Lane2288PlainMagicChildBox();
$magicChild->store = array("slot" => array("leaf" => "magic-child-source"));
$magicChild->slot["leaf"] = "magic-child-copy";

$arrayChild = new Lane2288PlainArrayAccessChildBag();
$arrayChild->store = array("slot" => array("leaf" => "array-child-source"));
$arrayChild["slot"]["leaf"] = "array-child-copy";

echo $magicParent->store["slot"]["leaf"], "|", $arrayParent->store["slot"]["leaf"], "|";
echo $magicChild->store["slot"]["leaf"], "|", $arrayChild->store["slot"]["leaf"], "|";
echo $magicParent->last, "|", $arrayParent->last, "|", $magicChild->last, "|", $arrayChild->last;
