<?php
class Milestone1884_SourceIterator implements Iterator, ArrayAccess {
    public $store = [];
    public $keys = ["outer"];
    public $pos = 0;

    public function seed($group, $key, $value) {
        $this->store[$group][$key] = $value;
    }

    #[ReturnTypeWillChange]
    public function rewind() {
        $this->pos = 0;
    }

    #[ReturnTypeWillChange]
    public function valid() {
        return isset($this->keys[$this->pos]);
    }

    #[ReturnTypeWillChange]
    public function key() {
        return $this->keys[$this->pos];
    }

    #[ReturnTypeWillChange]
    public function current() {
        return $this->store[$this->keys[$this->pos]];
    }

    #[ReturnTypeWillChange]
    public function next() {
        $this->pos = $this->pos + 1;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->store[$offset]);
    }
}

class Milestone1884_Holder {
    public $iter;
    public $log = [];

    public function snapshotLoopValue() {
        foreach ($this->iter as $key => $bucket) {
            $this->log[] = "iter:" . $key;
            return $bucket;
        }
        return ["leaf" => "fallback"];
    }
}

$iter = new Milestone1884_SourceIterator();
$iter->seed("outer", "leaf", "old");
$alias =& $iter["outer"]["leaf"];
$holder = new Milestone1884_Holder();
$holder->iter = $iter;
$copy = $holder->snapshotLoopValue();
$alias = "new";
$copy["leaf"] = "copy";
echo $iter->store["outer"]["leaf"], "|", $alias, "|", $copy["leaf"], "|", implode(",", $holder->log);
