<?php
class Milestone1984_Box {
    public $store = array();

    public function __get($name) {
        $bucket = $this->store[$name];
        $helper = function() use ($bucket) {
            return $bucket;
        };
        return call_user_func($helper);
    }
}

$box = new Milestone1984_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
