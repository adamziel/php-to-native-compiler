<?php
class Milestone1863_Bag implements ArrayAccess {
    public $trace = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($_REQUEST[$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get:" . $offset;
        return $_REQUEST[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $_REQUEST[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($_REQUEST[$offset]);
    }
}

$source = "seed";
$_REQUEST = array("slot" => array("ref" => &$source, "plain" => array("value" => "copy")));

$bag = new Milestone1863_Bag();
$alias =& $bag["slot"];
$alias["ref"] = "changed";

$copy = $bag["slot"];
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $_REQUEST["slot"]["plain"]["value"], "|", implode(",", $bag->trace);
