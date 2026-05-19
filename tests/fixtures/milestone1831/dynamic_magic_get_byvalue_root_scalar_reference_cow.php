<?php
error_reporting(0);

class Milestone1831Box {
    public $hits = 0;

    public function __get($name) {
        $this->hits++;
        if ($name === "n") {
            return 10;
        }
        return "fallback";
    }
}

$box = new Milestone1831Box();
$name = "n";
$alias =& $box->{$name};
$alias = 42;
$read = $box->{$name};

echo "alias=", $alias, "|read=", $read, "|hits=", $box->hits;
