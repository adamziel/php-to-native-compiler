<?php
class Milestone1980_Box {
    public $store = array();

    public function __get($name) {
        $helper = function($value) {
            $tmp = $value;
            return $tmp;
        };
        $args = array($this->store[$name]);
        return call_user_func_array($helper, $args);
    }
}

$box = new Milestone1980_Box();
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
