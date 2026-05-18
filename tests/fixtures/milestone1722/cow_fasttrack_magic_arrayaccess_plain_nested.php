<?php
function milestone1722_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1722_notice", E_NOTICE);

class Milestone1722_ByRefMagicPlainArrayBox {
    public $store = [];

    public function &__get($name) {
        $slot = $name;
        return $this->store[$slot];
    }
}

class Milestone1722_ByValueMagicPlainArrayBox {
    public $store = [
        "missing" => [
            "outer" => [
                "leaf" => "seed",
            ],
        ],
    ];

    public function __get($name) {
        return $this->store[$name];
    }
}

class Milestone1722_InnerBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $slot = $offset;
        return $this->items[$slot];
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

class Milestone1722_OuterBag implements ArrayAccess {
    public $items = [];

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->items[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $slot = $offset;
        return $this->items[$slot];
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

class Milestone1722_Holder {
    public $bag;
}

$magicFunction = "magic-original";
$magicNode = [
    "function" => &$magicFunction,
    "plain" => "magic-plain-original",
];
$magicBox = new Milestone1722_ByRefMagicPlainArrayBox();
$magicBox->store["missing"] = [];
$magicBox->missing["outer"]["leaf"] = [
    "id" => $magicNode,
    "plain" => [
        "function" => "magic-copy-original",
    ],
];
$magicBox->store["missing"]["outer"]["leaf"]["id"]["function"] = "magic-plain-cow";
$magicBox->store["missing"]["outer"]["leaf"]["plain"]["function"] = "magic-copy-mutated";

$falseFunction = "false-original";
$falseNode = [
    "function" => &$falseFunction,
];
$falseBox = new Milestone1722_ByRefMagicPlainArrayBox();
$falseBox->store["missing"]["parent"] = false;
error_reporting(0);
$falseBox->missing["parent"]["leaf"] = [
    "id" => $falseNode,
];
error_reporting(E_ALL);
$falseBox->store["missing"]["parent"]["leaf"]["id"]["function"] = "false-parent-cow";

$innerFunction = "inner-original";
$innerNode = [
    "function" => &$innerFunction,
    "plain" => "inner-plain-original",
];
$inner = new Milestone1722_InnerBag();
$outer = new Milestone1722_OuterBag();
$outer->items["first"] = $inner;
$holder = new Milestone1722_Holder();
$holder->bag = $outer;
$holder->bag["first"]["second"]["leaf"] = [
    "id" => $innerNode,
    "plain" => [
        "function" => "inner-copy-original",
    ],
];
$inner->items["second"]["leaf"]["id"]["function"] = "mixed-chain-cow";
$inner->items["second"]["leaf"]["plain"]["function"] = "mixed-chain-copy-mutated";

$byValueMagic = new Milestone1722_ByValueMagicPlainArrayBox();
$byValueMagic->missing["outer"]["leaf"] = [
    "changed" => true,
];

$byValueBag = new Milestone1722_OuterBag();
$byValueBag->items["outer"] = [
    "leaf" => "seed",
];
$byValueHolder = new Milestone1722_Holder();
$byValueHolder->bag = $byValueBag;
$byValueHolder->bag["outer"]["leaf"] = [
    "changed" => true,
];

echo $magicFunction,
    "|",
    $magicBox->store["missing"]["outer"]["leaf"]["plain"]["function"],
    "|",
    $falseFunction,
    "|",
    $innerFunction,
    "|",
    $inner->items["second"]["leaf"]["plain"]["function"],
    "|",
    $byValueMagic->store["missing"]["outer"]["leaf"],
    "|",
    $byValueBag->items["outer"]["leaf"];
