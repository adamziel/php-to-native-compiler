<?php
error_reporting(0);

class Milestone2103_Holder {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        $this->store[$name]["touch"] = "side";
        $bucket = array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => $this->store[$name]["plain"],
        );
        return $bucket;
    }
}

$ref = "old";
$holder = new Milestone2103_Holder();
$holder->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");
$holder->slot["ref"] = "new";

echo implode(",", $holder->log), "|", $ref, "|", $holder->store["slot"]["plain"], "|", $holder->store["slot"]["touch"];
