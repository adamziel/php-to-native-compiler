<?php
class Milestone1859_Bag implements ArrayAccess {
    public $trace = array();

    public function normalize($offset) {
        $this->trace[] = "normalize:" . $offset;
        return "g_" . $offset;
    }

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($GLOBALS["store"][$this->normalize($offset)]);
    }

    #[ReturnTypeWillChange]
    public function &offsetGet($offset) {
        $this->trace[] = "get";
        $key = $this->normalize($offset);
        return $GLOBALS["store"][$key];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $GLOBALS["store"][$this->normalize($offset)] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {
        unset($GLOBALS["store"][$this->normalize($offset)]);
    }
}

$source = "seed";
$GLOBALS["store"] = array(
    "g_slot" => array("ref" => &$source, "plain" => array("value" => "copy")),
);

$bag = new Milestone1859_Bag();
$alias =& $bag["slot"];
$alias["ref"] = "changed";

$copy = $bag["slot"];
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $GLOBALS["store"]["g_slot"]["plain"]["value"], "|",
    implode(",", $bag->trace);
