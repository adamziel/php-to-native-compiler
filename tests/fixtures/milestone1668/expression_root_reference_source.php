<?php
class Milestone1668Box {
    public $items = array("slot" => "old");
    public $dynamic = array("slot" => "dyn-old");
}

function milestone1668_box() {
    static $box;
    if (!$box) {
        $box = new Milestone1668Box();
    }
    return $box;
}

$box = milestone1668_box();
$alias =& milestone1668_box()->items["slot"];
$alias = "via-alias";
echo $box->items["slot"], "|";
$box->items["slot"] = "via-box";
echo $alias, "\n";

$property = "dynamic";
$dynamicAlias =& milestone1668_box()->{$property}["slot"];
$dynamicAlias = "dynamic-alias";
echo $box->dynamic["slot"], "|";
$box->dynamic["slot"] = "dynamic-box";
echo $dynamicAlias;
