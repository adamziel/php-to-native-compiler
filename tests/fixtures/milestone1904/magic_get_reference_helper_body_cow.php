<?php
class Milestone1904_Box {
    public $store = array();
    public $trace = array();

    private function &pick($name) {
        $this->trace[] = "pick:" . $name;
        if ($name === "missing") {
            return $this->store[$name];
        }
        return $this->store[$name];
    }

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return $this->pick($name);
    }
}

$box = new Milestone1904_Box();
$box->store["missing"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $box->store["missing"]["leaf"];

$copy = $box->missing;
$copy["leaf"] = "changed";
$copy["plain"]["value"] = "copy-changed";

echo $alias, "|", $box->store["missing"]["leaf"], "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
