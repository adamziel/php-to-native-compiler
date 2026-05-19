<?php
error_reporting(0);

class Milestone2140_Box {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        if ($name === "slot") {
            $bucket = $this->store[$name];
        } else {
            $bucket = array();
        }
        return $bucket;
    }
}

$box = new Milestone2140_Box();
$box->store["slot"] = array("ref" => "old", "plain" => "plain");
$alias =& $box->store["slot"]["ref"];

$box->slot["ref"] = "new";
$box->slot["plain"] = "copy";

echo implode(",", $box->log), "|", $alias, "|", $box->store["slot"]["ref"], "|", $box->store["slot"]["plain"];
