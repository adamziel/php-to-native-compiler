<?php
error_reporting(0);

class Milestone2195_Box {
    public $store = array();

    public function __call($name, $args) {
        return $this->store[$args[0]];
    }

    public function __get($name) {
        return $this->load($name);
    }
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];
$box = new Milestone2195_Box();
$box->store["slot"] = $source["slot"];

$value = $box->slot;
$value["ref"] = "new";
$value["plain"] = "copy";

echo $alias, "|", $source["slot"]["ref"], "|", $value["ref"], "|", $source["slot"]["plain"], "|", $value["plain"];
