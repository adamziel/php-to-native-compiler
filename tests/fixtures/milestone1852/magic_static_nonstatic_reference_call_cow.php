<?php
class Milestone1852_Base {
    public $store = array();
    public $trace = array();

    public function &slot($name) {
        $this->trace[] = "slot";
        return $this->store[$name];
    }

    public function &__get($name) {
        $this->trace[] = "get";
        return static::slot($name);
    }
}

class Milestone1852_Box extends Milestone1852_Base {
}

$source = "seed";
$box = new Milestone1852_Box();
$box->store["missing"] = array("value" => &$source, "plain" => array("value" => "copy"));

$alias =& $box->missing;
$alias["value"] = "changed";
$copy = $box->missing;
$copy["plain"]["value"] = "copy-changed";

echo $source, "|", $box->store["missing"]["plain"]["value"], "|", implode(",", $box->trace);
