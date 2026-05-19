<?php
error_reporting(0);

function milestone2236_wrap($copy) {
    return array("wrap" => array($copy));
}

class Milestone2236_Box implements ArrayAccess {
    public $store = array();

    public function offsetGet($name) {
        $bucket = $this->store[$name];
        $reflection = new ReflectionFunction("milestone2236_wrap");
        $args = $reflection->invokeArgs(array($bucket));
        return $args["wrap"][0];
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

$box = new Milestone2236_Box();
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
    ";";

$copy["ref"]["value"] = "copy";
$copy["plain"]["value"] = "plain-copy";

echo $ref, "|", $copy["ref"]["value"], "|",
    $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
