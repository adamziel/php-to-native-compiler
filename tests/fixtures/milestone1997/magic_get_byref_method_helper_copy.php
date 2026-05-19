<?php
class Milestone1997_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        return $this->helper($bucket);
    }

    public function helper(&$value) {
        $tmp = $value;
        return $tmp;
    }
}

$box = new Milestone1997_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
