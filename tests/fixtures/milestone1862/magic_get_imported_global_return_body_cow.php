<?php
class Milestone1862_Box {
    public $trace = array();

    public function &__get($name) {
        global $store;
        $this->trace[] = "get:" . $name;
        return $store[$name];
    }
}

$source = "seed";
$store = array("missing" => array("ref" => &$source, "plain" => array("value" => "copy")));

$box = new Milestone1862_Box();
$alias =& $box->missing;
$alias["ref"] = "changed";

$copy = $box->missing;
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
