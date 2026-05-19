<?php
error_reporting(0);

class Milestone2230_ArgsHolder {
    public $args = array();
    public $callback = array();
}

class Milestone2230_Box implements ArrayAccess {
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

    public function offsetGet($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array();
        $alias =& call_user_func_array(
            $this->prepare($bucket)->callback,
            $this->holder->args
        );
        $alias = "inside";
        return $bucket;
    }

    public function offsetExists($name) {
        return isset($this->store[$name]);
    }

    public function offsetSet($name, $value) {
        $this->store[$name] = $value;
    }

    public function offsetUnset($name) {
        unset($this->store[$name]);
    }
}

$box = new Milestone2230_Box();
$box->holder = new Milestone2230_ArgsHolder();
$box->store = array(
    "slot" => array(
        "ref" => array("value" => "original"),
        "plain" => array("value" => "plain-original"),
    ),
);
$ref =& $box->store["slot"]["ref"]["value"];

$copy = $box["slot"];
echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls, ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"],
    "|", $box->calls;
