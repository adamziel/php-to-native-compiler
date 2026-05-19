<?php
function milestone1930_mutate($bucket) {
    $bucket["ref"]["value"] = "callee";
    $bucket["plain"]["value"] = "plain-callee";
}

class Milestone1930_Bag implements ArrayAccess {
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
$bag = new Milestone1930_Bag();

milestone1930_mutate($bag["slot"]);

echo $alias, "|", $GLOBALS["store"]["slot"]["ref"]["value"], "|",
    $GLOBALS["store"]["slot"]["plain"]["value"];
