<?php
error_reporting(0);

class Milestone2132_Holder {
    public $store = array();
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return $this->make($name);
    }

    public function make($name) {
        $this->log[] = "make:" . $name;
        return array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => $this->store[$name]["plain"],
        );
    }
}

$ref = "old";
$holder = new Milestone2132_Holder();
$holder->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");
$holder->slot["ref"] = "new";

echo implode(",", $holder->log), "|", $ref, "|", $holder->store["slot"]["plain"];
