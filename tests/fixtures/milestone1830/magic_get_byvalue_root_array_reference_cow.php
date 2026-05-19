<?php
error_reporting(0);

class Milestone1830Box {
    public $items = array("x" => array("leaf" => "v"));
    public $log = array();

    public function __get($name) {
        $this->log[] = "get:" . $name;
        foreach (array("loop") as $step) {
            $this->log[] = $step;
        }
        try {
            $this->log[] = "try";
            return $this->items[$name];
        } finally {
            $this->log[] = "finally";
        }
    }
}

$box = new Milestone1830Box();
$alias =& $box->x;
$alias["leaf"] = "detached";

echo $box->items["x"]["leaf"], "|", $alias["leaf"], "|", implode(",", $box->log);
