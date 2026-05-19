<?php
error_reporting(0);

class Milestone2094_Box {
    public $store = array();

    public function __get($name) {
        $bucket = array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => $this->store[$name]["plain"],
        );
        return $bucket;
    }
}

$box = new Milestone2094_Box();
$ref = array("seed");
$box->store["missing"] = array("ref" => &$ref, "plain" => array("plain-original"));

$box->missing["ref"][] = "copy";
$box->missing["plain"][] = "plain-copy";

echo $ref[1], "|", count($box->store["missing"]["plain"]);
