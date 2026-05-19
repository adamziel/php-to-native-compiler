<?php
class Milestone2040_Carrier {
    public $payload;
}

class Milestone2040_Box {
    public $store = array();

    public function __get($name) {
        $carrier = new Milestone2040_Carrier();
        $carrier->payload = $this->store[$name];
        $carrier->payload["plain"]["value"] = "plain-inside";
        return $carrier->payload;
    }
}

$box = new Milestone2040_Box();
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
