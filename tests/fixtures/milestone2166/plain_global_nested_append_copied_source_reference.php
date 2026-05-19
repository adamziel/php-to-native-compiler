<?php
error_reporting(0);

class Milestone2166_SourceBag implements ArrayAccess {
    public $store = array();

    #[ReturnTypeWillChange]
    public function offsetExists($offset) {
        return isset($this->store[$offset]);
    }

    #[ReturnTypeWillChange]
    public function offsetGet($offset) {
        return $this->store[$offset];
    }

    #[ReturnTypeWillChange]
    public function offsetSet($offset, $value) {
        $this->store[$offset] = $value;
    }

    #[ReturnTypeWillChange]
    public function offsetUnset($offset) {}
}

$source = new Milestone2166_SourceBag();
$source->store["plain"] = array("ref" => "plain-old", "plain" => "plain-copy");
$source->store["global"] = array("ref" => "global-old", "plain" => "global-copy");
$plainAlias =& $source->store["plain"]["ref"];
$globalAlias =& $source->store["global"]["ref"];

$target = array("outer" => array());
$target["outer"][] = $source["plain"];
$target["outer"][0]["ref"] = "plain-new";
$target["outer"][0]["plain"] = "plain-target";

$GLOBALS["milestone2166_bucket"] = array("outer" => array());
$GLOBALS["milestone2166_bucket"]["outer"][] = $source["global"];
$GLOBALS["milestone2166_bucket"]["outer"][0]["ref"] = "global-new";
$GLOBALS["milestone2166_bucket"]["outer"][0]["plain"] = "global-target";

echo $plainAlias,
    "|",
    $source->store["plain"]["ref"],
    "|",
    $target["outer"][0]["ref"],
    "|",
    $source->store["plain"]["plain"],
    "|",
    $target["outer"][0]["plain"],
    "|",
    $globalAlias,
    "|",
    $source->store["global"]["ref"],
    "|",
    $GLOBALS["milestone2166_bucket"]["outer"][0]["ref"],
    "|",
    $source->store["global"]["plain"],
    "|",
    $GLOBALS["milestone2166_bucket"]["outer"][0]["plain"];
