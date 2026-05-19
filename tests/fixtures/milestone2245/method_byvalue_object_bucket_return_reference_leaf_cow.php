<?php
error_reporting(0);

class Milestone2245_Box {
    public $store;

    public function __construct(&$leaf) {
        $this->store = array(
            "slot" => array(
                "ref" => &$leaf,
                "plain" => "plain",
            ),
        );
    }

    public function get($name) {
        $bucket = $this->store[$name];
        $bucket["ref"] = "method";
        $bucket["plain"] = "local";
        return $bucket;
    }
}

$leaf = "orig";
$box = new Milestone2245_Box($leaf);
$copy = $box->get("slot");
$copy["ref"] = "copy-ref";
$copy["plain"] = "copy-plain";

echo $leaf, "|", $box->store["slot"]["ref"], "|", $box->store["slot"]["plain"], "|", $copy["plain"];
