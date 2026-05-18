<?php
function milestone1692_notice_handler($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}
set_error_handler("milestone1692_notice_handler", E_NOTICE);

class Milestone1692_ByValueMagicNestedAppendBag implements ArrayAccess {
    public $items = ["outer" => []];

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

class Milestone1692_ByValueMagicNestedAppendBox {
    private $store;

    public function __construct($store) {
        $this->store = $store;
    }

    public function __get($name) {
        return $this->store;
    }
}

$bag = new Milestone1692_ByValueMagicNestedAppendBag();
$box = new Milestone1692_ByValueMagicNestedAppendBox($bag);
$box->missing["outer"][] = ["id" => "new"];
echo count($bag->items["outer"]);
