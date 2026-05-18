<?php
class Milestone1664_Bag {
    public $items = array("slot" => "old");
}

$holders = array(
    "bag" => new Milestone1664_Bag(),
    "dynamic" => new Milestone1664_Bag(),
);

$bag = $holders["bag"];
$old =& $bag->items["slot"];
$same =& $old;
$new = "new";
$holders["bag"]->items["slot"] =& $new;
$new = "changed";
echo "holder-rebind=", $bag->items["slot"], "|", $old, "|", $same, "|", $new, "\n";
$same = "old-write";
echo "holder-old=", $bag->items["slot"], "|", $old, "|", $same, "|", $new, "\n";

$property = "items";
$dynamicBag = $holders["dynamic"];
$dynamicOld =& $dynamicBag->items["slot"];
$dynamicSame =& $dynamicOld;
$dynamicNew = "dynamic-new";
$holders["dynamic"]->{$property}["slot"] =& $dynamicNew;
$dynamicNew = "dynamic-changed";
echo "dynamic-rebind=", $dynamicBag->items["slot"], "|", $dynamicOld, "|", $dynamicSame, "|", $dynamicNew, "\n";
$dynamicSame = "dynamic-old-write";
echo "dynamic-old=", $dynamicBag->items["slot"], "|", $dynamicOld, "|", $dynamicSame, "|", $dynamicNew;
