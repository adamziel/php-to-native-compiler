<?php
function milestone1921_notice($errno, $message, $file, $line) {
    echo "notice:", $message, "\n";
    return true;
}

set_error_handler("milestone1921_notice", E_NOTICE);

class Milestone1921_Bag implements ArrayAccess {
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

$function = "original";
$bag = new Milestone1921_Bag();
$bag->items["slot"] = array(
    "id" => array("function" => &$function),
    "plain" => array("function" => "plain-original"),
);

$alias =& $bag["slot"]["id"]["function"];
$alias = "alias";
$plain =& $bag["slot"]["plain"]["function"];
$plain = "plain-alias";

$bag->items["slot"]["id"]["function"] = "bucket";
$bag->items["slot"]["plain"]["function"] = "plain-bucket";

echo $function, "|", $alias, "|", $bag->items["slot"]["id"]["function"], "|",
    $plain, "|", $bag->items["slot"]["plain"]["function"];
