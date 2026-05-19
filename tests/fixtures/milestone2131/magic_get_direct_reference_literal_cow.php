<?php
error_reporting(0);

class Milestone2131_Holder {
    private $store = array();
    public $log = array();

    public function seed(&$ref) {
        $this->store["slot"] = array("ref" => &$ref, "plain" => "plain-original");
    }

    public function plain() {
        return $this->store["slot"]["plain"];
    }

    public function __get($name) {
        $this->log[] = "get:" . $name;
        return array(
            "ref" => &$this->store[$name]["ref"],
            "plain" => $this->store[$name]["plain"],
        );
    }
}

$ref = "old";
$holder = new Milestone2131_Holder();
$holder->seed($ref);
$holder->slot["ref"] = "new";

echo implode(",", $holder->log), "|", $ref, "|", $holder->plain();
