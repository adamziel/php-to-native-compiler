<?php
error_reporting(0);

class Milestone2155_Box {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get";
        $value = $this->store[$name];
        $this->store = array($name => array("ref" => "new"));
        return $value;
    }
}

$box = new Milestone2155_Box();
$box->store["slot"] = array("ref" => "old");
$alias =& $box->store["slot"]["ref"];

$value = $box->slot;
$value["ref"] = "value";
$alias = "alias";

echo implode(",", $box->log), "|", $alias, "|", $value["ref"], "|", $box->store["slot"]["ref"];
