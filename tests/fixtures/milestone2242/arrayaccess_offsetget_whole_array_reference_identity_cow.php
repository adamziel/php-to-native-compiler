<?php
error_reporting(0);

function milestone2242_wrap(&$value) {
    return array(&$value);
}

class Milestone2242_Box implements ArrayAccess {
    public $store = array();

    public function offsetGet($name) {
        $args = milestone2242_wrap($this->store[$name]);
        $args[0]["plain"]["value"] = "inside";
        return $this->store[$name];
    }

    public function offsetSet($name, $value) {
        $this->store[$name] = $value;
    }

    public function offsetExists($name) {
        return isset($this->store[$name]);
    }

    public function offsetUnset($name) {
        unset($this->store[$name]);
    }
}

$box = new Milestone2242_Box();
$box->store = array(
    "slot" => array(
        "plain" => array("value" => "plain-original"),
    ),
);

$copy = $box["slot"];
echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"], ";";

$copy["plain"]["value"] = "copy";

echo $box->store["slot"]["plain"]["value"], "|", $copy["plain"]["value"];
