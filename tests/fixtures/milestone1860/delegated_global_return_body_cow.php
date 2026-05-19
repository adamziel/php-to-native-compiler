<?php
function &milestone1860_pick_global($name) {
    $GLOBALS["trace"][] = "pick:" . $name;
    return $GLOBALS["store"][$name];
}

class Milestone1860_Box {
    public $trace = array();

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return milestone1860_pick_global($name);
    }
}

class Milestone1860_Bag implements ArrayAccess {
    public $trace = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($GLOBALS["store"][$offset]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "offset:" . $offset;
        $picker = "milestone1860_pick_global";
        return $picker($offset);
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

$magicSource = "magic-seed";
$arraySource = "array-seed";
$GLOBALS["store"] = array(
    "magic" => array("ref" => &$magicSource),
    "array" => array("ref" => &$arraySource),
);
$GLOBALS["trace"] = array();

$box = new Milestone1860_Box();
$magicAlias =& $box->magic;
$magicAlias["ref"] = "magic-changed";

$bag = new Milestone1860_Bag();
$arrayAlias =& $bag["array"];
$arrayAlias["ref"] = "array-changed";

echo $magicSource, "|", $arraySource, "|",
    implode(",", $box->trace), "|", implode(",", $bag->trace), "|",
    implode(",", $GLOBALS["trace"]);
