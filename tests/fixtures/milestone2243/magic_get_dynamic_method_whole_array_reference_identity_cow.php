<?php
error_reporting(0);

function milestone2243_wrap(&$value) {
    return array(&$value);
}

class Milestone2243_Box {
    public $store = array();

    public function touch(&$bucket) {
        $args = milestone2243_wrap($bucket);
        $args[0]["plain"]["value"] = "inside";
        return $bucket;
    }

    public function __get($name) {
        $method = "touch";
        return $this->{$method}($this->store[$name]);
    }
}

$box = new Milestone2243_Box();
$box->store = array(
    "slot" => array(
        "plain" => array("value" => "plain-original"),
    ),
);

$copy = $box->slot;
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["plain"]["value"] = "copy";

echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
