<?php
error_reporting(0);

class Milestone2177_Box {
    public $store = array();

    public function load($name) {
        return $this->store[$name];
    }

    public function __get($name) {
        $method = new ReflectionMethod($this, "load");
        return $method->invokeArgs($this, array($name));
    }
}

$source = array("slot" => array("ref" => "old", "plain" => "plain"));
$alias =& $source["slot"]["ref"];

$box = new Milestone2177_Box();
$box->store["slot"] = $source["slot"];

$value = $box->slot;
$value["ref"] = "new";
$value["plain"] = "copy";

echo $alias,
    "|",
    $source["slot"]["ref"],
    "|",
    $value["ref"],
    "|",
    $source["slot"]["plain"],
    "|",
    $value["plain"];
