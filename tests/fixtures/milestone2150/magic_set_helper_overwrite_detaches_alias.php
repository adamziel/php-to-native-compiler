<?php
error_reporting(0);

class Milestone2150_Box {
    public $store = array();
    public $log = array();

    public function __set($name, $value) {
        $this->log[] = "set";
        $this->replace($name, $value);
    }

    public function replace($name, $value) {
        $this->store = array($name => array("ref" => $value));
    }
}

$box = new Milestone2150_Box();
$box->store["slot"] = array("ref" => "old");
$alias =& $box->store["slot"]["ref"];

$box->slot = "new";
$alias = "alias";

echo implode(",", $box->log), "|", $alias, "|", $box->store["slot"]["ref"];
