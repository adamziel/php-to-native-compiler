<?php
error_reporting(0);

class Milestone2143_Box {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $bucket =& $this->store[$name];
        return $bucket;
    }
}

$box = new Milestone2143_Box();
$box->store["slot"] = array("ref" => "old", "plain" => "plain");
$source =& $box->store["slot"]["ref"];

$alias =& $box->slot;
$alias["ref"] = "new";
$alias["plain"] = "copy";

echo implode(",", $box->log), "|", $source, "|", $box->store["slot"]["ref"], "|", $alias["ref"], "|", $box->store["slot"]["plain"], "|", $alias["plain"];
