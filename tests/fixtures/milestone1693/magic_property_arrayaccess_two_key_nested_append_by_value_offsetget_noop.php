<?php
function milestone1693_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1693_notice_handler", E_NOTICE);

class Milestone1693_ByValueMagicTwoKeyNestedAppendBag implements ArrayAccess {
    public $items = ["outer" => ["inner" => []]];

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

class Milestone1693_ByValueMagicTwoKeyNestedAppendBox {
    private $store;

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        return $this->store;
    }
}

$bag = new Milestone1693_ByValueMagicTwoKeyNestedAppendBag();
$box = new Milestone1693_ByValueMagicTwoKeyNestedAppendBox($bag);
$box->missing["outer"]["inner"][] = ["id" => "new"];
echo count($bag->items["outer"]["inner"]);
