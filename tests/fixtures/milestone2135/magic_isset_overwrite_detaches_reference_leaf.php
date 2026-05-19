<?php
class Milestone2135_Box {
    private $items;
    public $log = array();

    public function seed(&$source) {
        $this->items = array("box" => array("leaf" => &$source));
    }

    public function &box() {
        return $this->items["box"];
    }

    public function __isset($name) {
        $this->log[] = "isset:" . $name;
        $this->items = array();
        return false;
    }

    public function size() {
        return count($this->items);
    }
}

$source = "seed";
$box = new Milestone2135_Box();
$box->seed($source);
$alias =& $box->box();
$seen = isset($box->missing);
$alias["leaf"] = "mutated";

echo implode(",", $box->log), "|", ($seen ? "yes" : "no"), "|", $source, "|", $alias["leaf"], "|", $box->size();
