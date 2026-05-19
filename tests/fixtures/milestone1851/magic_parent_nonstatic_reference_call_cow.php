<?php
class Milestone1851_Base {
    public $store = array();
    public $trace = array();

    public function &slot($name) {
        $this->trace[] = "slot";
        return $this->store[$name];
    }
}

class Milestone1851_Box extends Milestone1851_Base {
    public function &__get($name) {
        $this->trace[] = "get";
        return parent::slot($name);
    }
}

$source = "seed";
$box = new Milestone1851_Box();
$box->store["missing"] = array("value" => &$source, "plain" => array("value" => "copy"));

$alias =& $box->missing;
$alias["value"] = "changed";
$copy = $box->missing;
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
