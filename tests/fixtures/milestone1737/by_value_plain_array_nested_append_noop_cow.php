<?php
function milestone1737_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1737_notice", E_NOTICE);

class Milestone1737_ByValueBag implements ArrayAccess {
    public $items = [
        "outer" => [
            "leaf" => [],
        ],
    ];

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

class Milestone1737_Holder {
    public $bag;
}

class Milestone1737_ByValueMagicArrayAccessBox {
    public $bag;

    public function __construct($bag) {
        $this->bag = $bag;
    }

    public function __get($name) {
        return $this->bag;
    }
}

class Milestone1737_ByValueMagicPlainArrayBox {
    public $store = [
        "missing" => [
            "outer" => [
                "leaf" => [],
            ],
        ],
    ];

    public function __get($name) {
        return $this->store[$name];
    }
}

$propertyFunction = "property-original";
$propertyNode = ["function" => &$propertyFunction];
$propertyBag = new Milestone1737_ByValueBag();
$holder = new Milestone1737_Holder();
$holder->bag = $propertyBag;
$holder->bag["outer"]["leaf"][] = [
    "id" => $propertyNode,
];

$magicFunction = "magic-original";
$magicNode = ["function" => &$magicFunction];
$magicBag = new Milestone1737_ByValueBag();
$magicBox = new Milestone1737_ByValueMagicArrayAccessBox($magicBag);
$magicBox->missing["outer"]["leaf"][] = [
    "id" => $magicNode,
];

$plainFunction = "plain-original";
$plainNode = ["function" => &$plainFunction];
$plainBox = new Milestone1737_ByValueMagicPlainArrayBox();
$plainBox->missing["outer"]["leaf"][] = [
    "id" => $plainNode,
];

echo count($propertyBag->items["outer"]["leaf"]),
    "|",
    $propertyFunction,
    "|",
    count($magicBag->items["outer"]["leaf"]),
    "|",
    $magicFunction,
    "|",
    count($plainBox->store["missing"]["outer"]["leaf"]),
    "|",
    $plainFunction;
