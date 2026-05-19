<?php
function milestone1922_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1922_notice", E_NOTICE);

class Milestone1922_Bag implements ArrayAccess {
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
        $this->items[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($this->items[$offset]);
    }
}

class Milestone1922_Holder {
    public $bag;
}

$function = "original";
$holder = new Milestone1922_Holder();
$bag = new Milestone1922_Bag();
$bag->items["slot"] = array("id" => array("function" => &$function));
$holder->bag = $bag;
$property = "bag";

$alias =& $holder->{$property}["slot"]["id"]["function"];
$alias = "alias";
$bag->items["slot"]["id"]["function"] = "bucket";

echo $function, "|", $alias, "|", $bag->items["slot"]["id"]["function"];
