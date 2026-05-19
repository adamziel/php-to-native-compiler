<?php
error_reporting(0);

class Milestone2205_ArgsHolder {
    public $args = array();
    public $callback = array();
}

class Milestone2205_Box {
    public $store = array();
    public $holder;

    public function holder() {
        return $this->holder;
    }

    public function &pickRef(&$value) {
        return $value["ref"]["value"];
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder()->args = array($bucket);
        $this->holder()->callback = array($this, "pickRef");
        $alias =& call_user_func_array(
            $this->holder()->callback,
            $this->holder()->args
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2205_Box();
$box->holder = new Milestone2205_ArgsHolder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
