<?php
function milestone1925_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1925_notice", E_NOTICE);

class Milestone1925_InnerBag implements ArrayAccess {
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

class Milestone1925_OuterBag implements ArrayAccess {
    public $inner;

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return true;
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->inner;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
    }
}

$function = "original";
$inner = new Milestone1925_InnerBag();
$inner->items["slot"] = array("id" => array("function" => &$function));
$outer = new Milestone1925_OuterBag();
$outer->inner = $inner;

$alias =& $outer["ignored"]["slot"]["id"]["function"];
$alias = "alias";
$inner->items["slot"]["id"]["function"] = "bucket";

echo $function, "|", $alias, "|", $inner->items["slot"]["id"]["function"];
