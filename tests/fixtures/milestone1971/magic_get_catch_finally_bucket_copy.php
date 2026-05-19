<?php
class Milestone1971_Picked extends Exception {}

class Milestone1971_Box {
    public $items;
    public $hits = array();

    public function __construct() {
        $this->items = array("x" => array("leaf" => "seed"));
    }

    public function __get($name) {
        try {
            throw new Milestone1971_Picked();
        } catch (Milestone1971_Picked $e) {
            return $this->items[$name];
        } finally {
            $this->hits[] = $name;
        }
    }
}

$box = new Milestone1971_Box();
$alias =& $box->items["x"]["leaf"];
$copy = $box->x;
$copy["leaf"] = "finally";

echo "leaf=", $alias,
    "|backing=", $box->items["x"]["leaf"],
    "|hits=", implode(",", $box->hits);
