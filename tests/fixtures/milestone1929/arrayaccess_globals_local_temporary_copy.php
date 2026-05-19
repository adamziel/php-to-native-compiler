<?php
class Milestone1929_Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($GLOBALS["store"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        $bucket = $GLOBALS["store"][$offset];
        return $bucket;
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $GLOBALS["store"][$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($GLOBALS["store"][$offset]);
    }
}

$GLOBALS["store"] = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $GLOBALS["store"]["slot"]["ref"]["value"];
$bag = new Milestone1929_Bag();

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $GLOBALS["store"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $GLOBALS["store"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
