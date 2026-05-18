<?php
function milestone1657_pair(&$left, &$right) {
    $left = $left . ":left";
    echo $left, "|", $right, "\n";
    $right = $right . ":right";
    echo $left, "|", $right, "\n";
}

$items = array("slot" => "seed");
$alias =& $items["slot"];
milestone1657_pair($alias, $items["slot"]);
echo "array=", $alias, "|", $items["slot"], "\n";

class Milestone1657_Bag {
    public $items = array("slot" => "box");
}

$bag = new Milestone1657_Bag();
$propertyAlias =& $bag->items["slot"];
milestone1657_pair($propertyAlias, $bag->items["slot"]);
echo "property=", $propertyAlias, "|", $bag->items["slot"];
