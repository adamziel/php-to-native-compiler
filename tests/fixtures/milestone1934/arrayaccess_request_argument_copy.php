<?php
function milestone1934_mutate($bucket) {
    $bucket["ref"]["value"] = "callee";
    $bucket["plain"]["value"] = "plain-callee";
}

class Milestone1934_Bag implements ArrayAccess {
    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($_REQUEST["store"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $_REQUEST["store"][$offset];
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
$bag = new Milestone1934_Bag();

milestone1934_mutate($bag["slot"]);

echo $alias, "|", $_REQUEST["store"]["slot"]["ref"]["value"], "|",
    $_REQUEST["store"]["slot"]["plain"]["value"];
