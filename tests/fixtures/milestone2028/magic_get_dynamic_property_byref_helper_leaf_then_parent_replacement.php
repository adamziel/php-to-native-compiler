<?php
function milestone2028_touch_bucket(&$value) {
    $value["ref"]["value"] = "inside";
    $value["plain"]["value"] = "plain-inside";
    $value["ref"] = array("value" => "replaced");
}

class Milestone2028_Holder {
    public $bucket = array();
}

class Milestone2028_Box {
    public $store = array();

    public function __get($name) {
        $property = "bucket";
        $holder = new Milestone2028_Holder();
        $holder->{$property} = $this->store[$name];
        milestone2028_touch_bucket($holder->{$property});
        return $holder->{$property};
    }
}

$box = new Milestone2028_Box();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$alias =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $alias, "|", $box->store["slot"]["ref"]["value"], "|",
    $copy["ref"]["value"], "|", $box->store["slot"]["plain"]["value"], "|",
    $copy["plain"]["value"];
