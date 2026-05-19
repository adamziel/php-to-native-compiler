<?php
error_reporting(0);

class Milestone2246_Box {
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

    public function __get($name) {
        $bucket = $this->store[$name];
        return $this->mutate($bucket);
    }
}

$leaf = "orig";
$box = new Milestone2246_Box($leaf);
$copy = $box->slot;
$copy["ref"] = "copy-ref";
$copy["plain"] = "copy-plain";

echo $leaf, "|", $box->store["slot"]["ref"], "|", $box->store["slot"]["plain"], "|", $copy["plain"];
