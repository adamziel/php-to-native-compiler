<?php
class Milestone1926_Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($GLOBALS["store"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $GLOBALS["store"][$offset];
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
$bag = new Milestone1926_Bag();

$copy = $bag["slot"];
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $GLOBALS["store"]["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $GLOBALS["store"]["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
