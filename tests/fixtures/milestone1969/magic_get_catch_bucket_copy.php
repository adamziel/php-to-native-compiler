<?php
class Milestone1969_Picked extends Exception {}

class Milestone1969_Box {
    public $items;

    public function __construct() {
        $this->items = array("x" => array("leaf" => "seed"));
    }

    public function __get($name) {
        try {
            throw new Milestone1969_Picked();
        } catch (Milestone1969_Picked $e) {
            return $this->items[$name];
        }
    }
}

$box = new Milestone1969_Box();
$alias =& $box->items["x"]["leaf"];
$copy = $box->x;
$copy["leaf"] = "magic";

echo "leaf=", $alias, "|backing=", $box->items["x"]["leaf"];
