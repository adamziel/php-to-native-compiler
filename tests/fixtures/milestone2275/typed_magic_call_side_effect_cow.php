<?php
error_reporting(0);

class Milestone2275_Box {
    public $store = array();
    public $hits = 0;

    public function __call(string $name, array $args): mixed {
        $this->hits = $this->hits + 1;
        $key = $args[0];
        $bucket = $this->store[$key];

        if ($name === "load") {
            return $bucket;
        }

        return array();
    }
}

$box = new Milestone2275_Box();
$box->store["slot"] = array(
    "ref" => array("value" => "original"),
    "plain" => array("value" => "plain-original"),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->load("slot");
$copy["ref"]["value"] = "inside";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $box->store["slot"]["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], "|",
    $box->hits;
