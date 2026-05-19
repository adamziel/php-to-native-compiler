<?php
error_reporting(0);

class Milestone2133_Holder {
    public $store = array();
    public $log = array();

    public function __set($name, $value) {
        $this->log[] = "set:" . $name;
        $this->put($name, $value);
    }

    public function put($name, $value) {
        $this->log[] = "put:" . $name;
        $this->store[$name] = $value;
    }
}

$ref = "old";
$holder = new Milestone2133_Holder();
$holder->slot = array("ref" => &$ref, "plain" => "plain-original");
$holder->store["slot"]["ref"] = "new";

echo implode(",", $holder->log), "|", $ref, "|", $holder->store["slot"]["plain"];
