<?php
error_reporting(0);

class Milestone2247_Box implements ArrayAccess {
    public $store;

    public function __construct(&$leaf) {
        $this->store = array(
            "slot" => array(
                "ref" => &$leaf,
                "plain" => "plain",
            ),
        );
    }

    public function mutate($bucket) {
        $bucket["ref"] = "helper";
        $bucket["plain"] = "local";
        return $bucket;
    }

    public function offsetGet($offset) {
        $bucket = $this->store[$offset];
        return $this->mutate($bucket);
    }

    public function offsetSet($offset, $value) {
    }

    public function offsetExists($offset) {
        return true;
    }

    public function offsetUnset($offset) {
    }
}

$leaf = "orig";
$box = new Milestone2247_Box($leaf);
$copy = $box["slot"];
$copy["ref"] = "copy-ref";
$copy["plain"] = "copy-plain";

echo $leaf, "|", $box->store["slot"]["ref"], "|", $box->store["slot"]["plain"], "|", $copy["plain"];
