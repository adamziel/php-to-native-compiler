<?php
class Milestone1905_Box {
    public $store = array();
    public $trace = array();

    private function &pick($name) {
        $this->trace[] = "pick:" . $name;
        while (true) {
            return $this->store[$name];
        }
        return $this->store[$name];
    }

    public function &__get($name) {
        $this->trace[] = "get:" . $name;
        return $this->pick($name);
    }
}

$box = new Milestone1905_Box();
$box->store["missing"] = array("leaf" => "seed", "plain" => array("value" => "copy"));
$alias =& $box->store["missing"]["leaf"];
$holders = array("box" => $box);
$property = "missing";

$direct = $box->{$property};
$direct["leaf"] = "direct";
$copy = $holders["box"]->missing;
$copy["leaf"] = "holder";
$copy["plain"]["value"] = "copy-changed";

echo $alias, "|", $box->store["missing"]["leaf"], "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
