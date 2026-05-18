<?php
class Milestone1757_Box {
    public int $id = 1;
}

class Milestone1757_KeyedBag implements ArrayAccess {
    public $items = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $items =& $this->items;
        $items[$offset]["leaf"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1757_PrefixedBag implements ArrayAccess {
    public $items = array("bucket" => array());

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["bucket"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $bucket =& $this->items["bucket"];
        $bucket[$offset]["leaf"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["bucket"][$offset]);
    }
}

class Milestone1757_BranchBag implements ArrayAccess {
    public $items = array("bucket" => array());

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["bucket"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->items["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $items =& $this->items;
        if ($offset === null) {
            $items["bucket"][]["leaf"] = $value;
            return;
        }
        $items["bucket"][$offset]["leaf"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["bucket"][$offset]);
    }
}

$box = new Milestone1757_Box();
$alias =& $box->id;

$keyed = new Milestone1757_KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$prefixed = new Milestone1757_PrefixedBag();
$prefixed["outer"] = array("copy" => &$alias);
$prefixed->items["bucket"]["outer"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($prefixed->items["bucket"]["outer"]["leaf"]["copy"]), ":", $prefixed->items["bucket"]["outer"]["leaf"]["copy"], "\n";

$branch = new Milestone1757_BranchBag();
$branch[] = array("copy" => &$alias);
$branch->items["bucket"][0]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"][0]["leaf"]["copy"]), ":", $branch->items["bucket"][0]["leaf"]["copy"], "\n";

$branch["named"] = array("copy" => &$alias);
$branch->items["bucket"]["named"]["leaf"]["copy"] = "5";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"]["named"]["leaf"]["copy"]), ":", $branch->items["bucket"]["named"]["leaf"]["copy"];
