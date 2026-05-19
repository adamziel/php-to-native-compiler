<?php
error_reporting(0);

class Milestone2175_TargetBag {
    public $store = array();
    public $cb;

    public function __construct() {
        $this->cb = array($this, "save");
    }

    public function save($name, $value) {
        $this->store[$name] = $value;
    }

    public function __set($name, $value) {
        $cb = $this->cb;
        $cb($name, $value);
    }
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];

$target = new Milestone2175_TargetBag();
$target->slot = $source["slot"];
$target->store["slot"]["ref"] = "new";
$target->store["slot"]["plain"] = "copy";

echo $alias,
    "|",
    $source["slot"]["ref"],
    "|",
    $target->store["slot"]["ref"],
    "|",
    $source["slot"]["plain"],
    "|",
    $target->store["slot"]["plain"];
