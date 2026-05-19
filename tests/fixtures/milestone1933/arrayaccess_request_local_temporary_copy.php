<?php
class Milestone1933_Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($_REQUEST["store"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $_REQUEST["store"][$offset];
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $_REQUEST["store"][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($_REQUEST["store"][$offset]);
    }
}

$_REQUEST["store"] = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $_REQUEST["store"]["slot"]["ref"]["value"];
$bag = new Milestone1933_Bag();

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $_REQUEST["store"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $_REQUEST["store"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
