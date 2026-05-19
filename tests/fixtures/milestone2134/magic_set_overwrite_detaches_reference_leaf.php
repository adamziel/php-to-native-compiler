<?php
class Milestone2134_Box {
    private $items;
    public $log = array();

    public function seed(&$source) {
        $this->items = array("box" => array("leaf" => &$source));
    }

    public function &box() {
        return $this->items["box"];
    }

    public function __set($name, $value) {
        $this->log[] = "set:" . $name . ":" . $value;
        $this->items = array();
    }

    public function size() {
        return count($this->items);
    }
}

$source = "seed";
$box = new Milestone2134_Box();
$box->seed($source);
$alias =& $box->box();
$box->missing = 1;
$alias["leaf"] = "mutated";

echo implode(",", $box->log), "|", $source, "|", $alias["leaf"], "|", $box->size();
