<?php
error_reporting(0);

class Milestone1828Box {
    public $items = array(
        "x" => array("leaf" => "v"),
    );
    public $hits = array();

    public function __get($name) {
        $this->hits[] = $name;
        return $this->items[$name];
    }
}

$holders = array("box" => new Milestone1828Box());
$alias =& $holders["box"]->x["leaf"];
$alias = "detached";

echo "leaf=", $holders["box"]->items["x"]["leaf"],
    "|alias=", $alias,
    "|hits=", count($holders["box"]->hits);
