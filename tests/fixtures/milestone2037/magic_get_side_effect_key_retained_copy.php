<?php
class Milestone2037_Box {
    public $store = array();

    public function key($name) {
        return $name;
    }

    public function __get($name) {
        $holder = array();
        $holder["copy"] = $this->store[$this->key($name)];
        $holder["copy"]["plain"]["value"] = "plain-inside";
        return $holder["copy"];
    }
}

$box = new Milestone2037_Box();
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
