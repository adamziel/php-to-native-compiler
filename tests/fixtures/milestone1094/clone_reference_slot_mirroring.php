<?php
class Box {
    public $items = [];
}

$box = new Box();
$box->items["slot"] = "original";
$slot =& $box->items["slot"];

$copy = clone $box;
$copy->items["slot"] = "copy-slot";
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$slot = "alias-slot";
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$copy->items = ["slot" => "detached"];
echo $slot, "|", $box->items["slot"], "|", $copy->items["slot"], "\n";

$items =& $box->items;
$copy2 = clone $box;
$copy2->items["slot"] = "whole-property";
echo $items["slot"], "|", $box->items["slot"], "|", $copy2->items["slot"];
