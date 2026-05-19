<?php
class Milestone1853_Base {
    public $store = array();
    public $trace = array();

    public function &slot($name) {
        $this->trace[] = "slot";
        return $this->store[$name];
    }

    public function &__get($name) {
        $this->trace[] = "get";
        return Milestone1853_Base::slot($name);
    }
}

class Milestone1853_Box extends Milestone1853_Base {
}

$source = "seed";
$box = new Milestone1853_Box();
$box->store["missing"] = array("value" => &$source, "plain" => array("value" => "copy"));

$alias =& $box->missing;
$alias["value"] = "changed";
$copy = $box->missing;
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
