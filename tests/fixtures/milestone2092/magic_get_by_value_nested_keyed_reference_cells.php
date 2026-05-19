<?php
error_reporting(0);

class Milestone2092_Box {
    public $store = array();

    public function __get($name) {
        $bucket = array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => $this->store[$name]["plain"],
        );
        return $bucket;
    }
}

$box = new Milestone2092_Box();
$ref = "original";
$box->store["missing"] = array("ref" => &$ref, "plain" => "plain-original");

$box->missing["ref"] = "copy";
$box->missing["plain"] = "plain-copy";

echo $ref, "|", $box->store["missing"]["plain"];
