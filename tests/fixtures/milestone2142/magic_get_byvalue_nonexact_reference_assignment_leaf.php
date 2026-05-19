<?php
error_reporting(0);

class Milestone2142_Box {
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

$box = new Milestone2142_Box();
$box->store["slot"] = array("ref" => "old", "plain" => "plain");
$source =& $box->store["slot"]["ref"];

$alias =& $box->slot["ref"];
$alias = "new";

echo implode(",", $box->log), "|", $source, "|", $box->store["slot"]["ref"], "|", $alias, "|", $box->store["slot"]["plain"];
