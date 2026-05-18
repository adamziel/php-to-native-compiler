<?php
class Milestone1710_PrefixLeafBag implements ArrayAccess {
    public $items = ["bucket" => ["leaf" => []]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items["bucket"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items["bucket"][$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items["bucket"][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items["bucket"][$offset]);
    }
}

class Milestone1710_SuffixLeafBag implements ArrayAccess {
    public $items = ["leaf" => ["bucket" => []]];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]["bucket"]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        return $this->items[$offset]["bucket"];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->items[$offset]["bucket"] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]["bucket"]);
    }
}

class Milestone1710_OuterBag implements ArrayAccess {
    public $items = [];

    public function __construct($leaf) {
        $this->items["outer"] = $leaf;
    }

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
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1710_ArrayAccessMagicBox {
    private $store;

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        return $this->store;
    }
}

$prefixFunction = "prefix-original";
$prefixLabel = "prefix-label-original";
$prefixNode = [
    "function" => &$prefixFunction,
    "label" => &$prefixLabel,
];
$prefixLeaf = new Milestone1710_PrefixLeafBag();
$prefixBox = new Milestone1710_ArrayAccessMagicBox(new Milestone1710_OuterBag($prefixLeaf));
$prefixAlias =& $prefixBox->missing["outer"]["leaf"];
$prefixAlias[] = [
    "id" => $prefixNode,
    "plain" => [
        "function" => "prefix-plain-original",
        "label" => "prefix-plain-original",
    ],
];
$prefixLeaf->items["bucket"]["leaf"][0]["id"]["function"] = "via-aa-prefix-source";
$prefixLeaf->items["bucket"]["leaf"][0]["id"]["label"] = "via-aa-prefix-label";
$prefixBucket = $prefixLeaf->items["bucket"]["leaf"][0];

$suffixFunction = "suffix-original";
$suffixLabel = "suffix-label-original";
$suffixNode = [
    "function" => &$suffixFunction,
    "label" => &$suffixLabel,
];
$suffixLeaf = new Milestone1710_SuffixLeafBag();
$suffixBox = new Milestone1710_ArrayAccessMagicBox(new Milestone1710_OuterBag($suffixLeaf));
$suffixAlias =& $suffixBox->missing["outer"]["leaf"];
$suffixAlias[] = [
    "id" => $suffixNode,
    "plain" => [
        "function" => "suffix-plain-original",
        "label" => "suffix-plain-original",
    ],
];
$suffixLeaf->items["leaf"]["bucket"][0]["id"]["function"] = "via-aa-suffix-source";
$suffixLeaf->items["leaf"]["bucket"][0]["id"]["label"] = "via-aa-suffix-label";
$suffixBucket = $suffixLeaf->items["leaf"]["bucket"][0];

echo $prefixFunction,
    "|",
    $prefixLabel,
    "|",
    $prefixBucket["id"]["function"],
    "|",
    $prefixBucket["id"]["label"],
    "|",
    $prefixBucket["plain"]["function"],
    "|",
    $prefixBucket["plain"]["label"],
    "\n",
    $suffixFunction,
    "|",
    $suffixLabel,
    "|",
    $suffixBucket["id"]["function"],
    "|",
    $suffixBucket["id"]["label"],
    "|",
    $suffixBucket["plain"]["function"],
    "|",
    $suffixBucket["plain"]["label"];
