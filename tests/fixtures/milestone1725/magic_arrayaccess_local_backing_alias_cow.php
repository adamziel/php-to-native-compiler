<?php
class Milestone1725_InnerBag implements ArrayAccess {
    public $items = ["inner" => []];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $items =& $this->items;
        $slot = $offset;
        return $items[$slot];
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

class Milestone1725_OuterBag implements ArrayAccess {
    public $items = [];

    public function __construct($inner) {
        $this->items["outer"] = $inner;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $items =& $this->items;
        $slot = $offset;
        return $items[$slot];
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

class Milestone1725_ByRefMagicBox {
    public $store = [];

    public function &__get($name) {
        $store =& $this->store;
        $slot = $name;
        return $store[$slot];
    }
}

class Milestone1725_ByValueMagicBox {
    public $store = [];

    public function __get($name) {
        $store =& $this->store;
        $slot = $name;
        return $store[$slot];
    }
}

$magicFunction = "magic-seed";
$magicNode = ["function" => &$magicFunction];
$magicBox = new Milestone1725_ByRefMagicBox();
$magicBox->store["plain"] = [];
$magicBox->plain["outer"]["leaf"] = [
    "id" => $magicNode,
    "plain" => ["function" => "magic-plain-seed"],
];
$magicBox->store["plain"]["outer"]["leaf"]["id"]["function"] = "magic-local-alias-cow";
$magicBox->store["plain"]["outer"]["leaf"]["plain"]["function"] = "magic-plain-copy";

$function = "seed-function";
$label = "seed-label";
$node = ["function" => "placeholder", "label" => "placeholder"];
$node["function"] =& $function;
$node["label"] =& $label;

$inner = new Milestone1725_InnerBag();
$outer = new Milestone1725_OuterBag($inner);
$box = new Milestone1725_ByValueMagicBox();
$box->store["missing"] = $outer;

$box->missing["outer"]["inner"][] = [
    "id" => $node,
    "plain" => ["function" => "plain-seed", "label" => "plain-seed"],
];

$inner->items["inner"][0]["id"]["function"] = "local-alias-cow";
$inner->items["inner"][0]["id"]["label"] = "local-alias-label";
$inner->items["inner"][0]["plain"]["function"] = "plain-copy-mutated";
$inner->items["inner"][0]["plain"]["label"] = "plain-copy-label";

echo $function,
    "|",
    $label,
    "|",
    $inner->items["inner"][0]["plain"]["function"],
    "|",
    $inner->items["inner"][0]["plain"]["label"],
    "|",
    $magicFunction,
    "|",
    $magicBox->store["plain"]["outer"]["leaf"]["plain"]["function"];
