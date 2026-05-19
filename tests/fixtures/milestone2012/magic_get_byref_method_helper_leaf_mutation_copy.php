<?php
class Milestone2012_Box {
    public $store = array();

    public function touch(&$value) {
        $value["ref"]["value"] = "inside";
        $value["plain"]["value"] = "plain-inside";
        return $value;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        return $this->touch($bucket);
    }
}

$box = new Milestone2012_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
