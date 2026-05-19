<?php
class Milestone2039_Box {
    public $store = array();

    public function __get($name) {
        $holder = array("copy" => $this->store[$name]);
        for ($i = 0; $i < 1; $i = $i + 1) {
            if ($i === 0) {
                $holder["copy"]["plain"]["value"] = "plain-inside";
            }
        }
        return $holder["copy"];
    }
}

$box = new Milestone2039_Box();
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
