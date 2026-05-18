<?php
class Milestone1755_Box {
    public int $id = 1;
}

class Milestone1755_KeyedBag implements ArrayAccess {
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
        $leaf = "leaf";
        $this->items[$offset][$leaf] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1755_AppendBag implements ArrayAccess {
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
        $bucket = "bucket";
        $leaf = "leaf";
        if ($offset === null) {
            $this->items[$bucket][][$leaf] = $value;
            return;
        }
        $this->items[$bucket][$offset][$leaf] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$box = new Milestone1755_Box();
$alias =& $box->id;

$keyed = new Milestone1755_KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$append = new Milestone1755_AppendBag();
$append[] = array("copy" => &$alias);
$append->items["bucket"][0]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($append->items["bucket"][0]["leaf"]["copy"]), ":", $append->items["bucket"][0]["leaf"]["copy"], "\n";

$append["named"] = array("copy" => &$alias);
$append->items["bucket"]["named"]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($append->items["bucket"]["named"]["leaf"]["copy"]), ":", $append->items["bucket"]["named"]["leaf"]["copy"];
