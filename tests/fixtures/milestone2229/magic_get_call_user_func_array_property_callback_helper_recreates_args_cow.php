<?php
error_reporting(0);

class Milestone2229_ArgsHolder {
    public $args = array();
    public $callback = array();
}

class Milestone2229_Box {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function &pickRef(&$value) {
        return $value["ref"]["value"];
    }

    public function &prepare($copy) {
        $this->calls = $this->calls + 1;
        unset($this->holder->args);
        $this->holder->args = array($copy);
        $this->holder->callback = array($this, "pickRef");
        return $this->holder;
    }

    public function __get($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array();
        $alias =& call_user_func_array(
            $this->prepare($bucket)->callback,
            $this->holder->args
        );
        $alias = "inside";
        return $bucket;
    }
}

$box = new Milestone2229_Box();
$box->holder = new Milestone2229_ArgsHolder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box->slot;
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls, ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls;
