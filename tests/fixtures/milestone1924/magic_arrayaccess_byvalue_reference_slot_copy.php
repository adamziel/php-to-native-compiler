<?php
function milestone1924_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1924_notice", E_NOTICE);

class Milestone1924_Bag implements ArrayAccess {
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

class Milestone1924_Box {
    public $bag;

    public function __get($name) {
        return $this->bag;
    }
}

$function = "original";
$box = new Milestone1924_Box();
$bag = new Milestone1924_Bag();
$bag->items["slot"] = array("id" => array("function" => &$function));
$box->bag = $bag;

$alias =& $box->missing["slot"]["id"]["function"];
$alias = "alias";
$bag->items["slot"]["id"]["function"] = "bucket";

echo $function, "|", $alias, "|", $bag->items["slot"]["id"]["function"];
