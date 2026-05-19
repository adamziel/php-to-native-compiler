<?php
error_reporting(0);

function &milestone2226_pick_ref(&$value) {
    return $value["ref"]["value"];
}

class Milestone2226_ArgsHolder {
    public $args = array();
}

class Milestone2226_Box implements ArrayAccess {
    public $store = array();
    public $holder;
    public $calls = 0;

    public function offsetGet($name) {
        $bucket = $this->store[$name];
        $this->holder->args = array();
        $helper = function ($copy) {
            $this->calls = $this->calls + 1;
            unset($this->holder->args);
            $this->holder->args = array($copy);
            return $this->holder;
        };
        $reflection = new ReflectionFunction($helper);
        $alias =& call_user_func_array(
            "milestone2226_pick_ref",
            $reflection->invokeArgs(array($bucket))->args
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

$box = new Milestone2226_Box();
$box->holder = new Milestone2226_ArgsHolder();
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
