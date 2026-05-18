<?php
class Milestone1756_Box {
    public int $id = 1;
}

class Milestone1756_KeyedBag implements ArrayAccess {
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
        $slot = $offset;
        $leaf = "leaf";
        $this->items[$slot][$leaf] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1756_BranchBag implements ArrayAccess {
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
        $slot = $offset;
        $bucket = "bucket";
        $leaf = "leaf";
        if ($offset === null) {
            $this->items[$bucket][][$leaf] = $value;
            return;
        }
        $this->items[$bucket][$slot][$leaf] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

$box = new Milestone1756_Box();
$alias =& $box->id;

$keyed = new Milestone1756_KeyedBag();
$keyed["outer"] = array("copy" => &$alias);
$keyed->items["outer"]["leaf"]["copy"] = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($keyed->items["outer"]["leaf"]["copy"]), ":", $keyed->items["outer"]["leaf"]["copy"], "\n";

$branch = new Milestone1756_BranchBag();
$branch["named"] = array("copy" => &$alias);
$branch->items["bucket"]["named"]["leaf"]["copy"] = "3";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"]["named"]["leaf"]["copy"]), ":", $branch->items["bucket"]["named"]["leaf"]["copy"], "\n";

$branch[] = array("copy" => &$alias);
$branch->items["bucket"][0]["leaf"]["copy"] = "4";
echo gettype($box->id), ":", $box->id, "|", gettype($branch->items["bucket"][0]["leaf"]["copy"]), ":", $branch->items["bucket"][0]["leaf"]["copy"];
