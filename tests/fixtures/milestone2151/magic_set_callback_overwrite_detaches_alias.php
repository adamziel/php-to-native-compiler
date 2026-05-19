<?php
error_reporting(0);

class Milestone2151_Box {
    public $store = array();
    public $log = array();

    public function __set($name, $value) {
        $this->log[] = "set";
        call_user_func(array($this, "replace"), $name, $value);
    }

    public function replace($name, $value) {
        $this->store = array($name => array("ref" => $value));
    }
}

$box = new Milestone2151_Box();
$box->store["slot"] = array("ref" => "old");
$alias =& $box->store["slot"]["ref"];

$box->slot = "new";
$alias = "alias";

echo implode(",", $box->log), "|", $alias, "|", $box->store["slot"]["ref"];
