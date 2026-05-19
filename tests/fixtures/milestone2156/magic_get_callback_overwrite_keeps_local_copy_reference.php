<?php
error_reporting(0);

class Milestone2156_Box {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get";
        $value = $this->store[$name];
        call_user_func(array($this, "replace"), $name);
        return $value;
    }

    public function replace($name) {
        $this->store = array($name => array("ref" => "new"));
    }
}

$box = new Milestone2156_Box();
$box->store["slot"] = array("ref" => "old");
$alias =& $box->store["slot"]["ref"];

$value = $box->slot;
$value["ref"] = "value";
$alias = "alias";

echo implode(",", $box->log), "|", $alias, "|", $value["ref"], "|", $box->store["slot"]["ref"];
