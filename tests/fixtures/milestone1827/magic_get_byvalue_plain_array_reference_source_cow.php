<?php
error_reporting(0);

class Milestone1827Box {
    public $items = array(
        "x" => array("leaf" => "v"),
        "y" => array(),
    );
    public $hits = array();

    public function __get($name) {
        $this->hits[] = $name;
        return $this->items[$name];
    }
}

$box = new Milestone1827Box();
$alias =& $box->x["leaf"];
$alias = "detached";
$append =& $box->y[];
$append = "append";

echo "leaf=", $box->items["x"]["leaf"],
    "|alias=", $alias,
    "|append=", $append,
    "|y_count=", count($box->items["y"]),
    "|hits=", count($box->hits);
