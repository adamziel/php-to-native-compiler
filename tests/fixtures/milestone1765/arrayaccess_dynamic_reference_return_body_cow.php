<?php
class Milestone1765_Bag implements ArrayAccess {
    public $items = [];
    public $hits = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->hits[] = $offset;
        if (!isset($this->items[$offset])) {
            $this->items[$offset] = [];
        }
        $bucket =& $this->items[$offset];
        $leaf = "leaf";
        return $bucket[$leaf];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1765_MagicBox {
    public $store = [];
    public $hits = [];

    public function &__get($name) {
        $this->hits[] = $name;
        if (!isset($this->store[$name])) {
            $this->store[$name] = [];
        }
        return $this->store[$name];
    }
}

$source = "seed";
$node = ["value" => &$source, "plain" => ["value" => "copy"]];

$bag = new Milestone1765_Bag();
$bag["outer"]["child"] = $node;
$bag->items["outer"]["leaf"]["child"]["value"] = "changed";
$bag->items["outer"]["leaf"]["child"]["plain"]["value"] = "plain-changed";

$alias =& $bag["outer"]["child"];
$alias["value"] = "alias-changed";
$alias["plain"]["value"] = "alias-plain";

$magicSource = "magic-seed";
$magicNode = ["value" => &$magicSource, "plain" => ["value" => "magic-copy"]];

$box = new Milestone1765_MagicBox();
$box->missing["child"] = $magicNode;
$box->store["missing"]["child"]["value"] = "magic-changed";
$magicAlias =& $box->missing["child"];
$magicAlias["value"] = "magic-alias";
$magicAlias["plain"]["value"] = "magic-plain";

echo $source,
    "|",
    $bag->items["outer"]["leaf"]["child"]["plain"]["value"],
    "|",
    $bag->hits[0],
    "|",
    $bag->hits[1],
    "|",
    $bag["outer"]["child"]["value"],
    "|",
    $magicSource,
    "|",
    $box->store["missing"]["child"]["plain"]["value"],
    "|",
    $box->hits[0],
    "|",
    $box->hits[1];
