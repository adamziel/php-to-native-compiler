<?php
class Box {
    public $items = ["slot" => "x"];
    public $empty;
}

$box = new Box();
$alias =& $box->items["slot"];
$alias = "from-alias";
echo $box->items["slot"], "|";
$box->items["slot"] = "from-slot";
echo $alias, "\n";

$key = "missing";
$missing =& $box->empty[$key];
$missing = "created";
echo $box->empty["missing"], "|";
$box->empty["missing"] = "updated";
echo $missing;
