<?php
$items = array("slot" => "old");
$old =& $items["slot"];
$same =& $old;
$new = "new";
$items["slot"] =& $new;
$new = "changed";
echo "array-rebind=", $items["slot"], "|", $old, "|", $same, "|", $new, "\n";
$same = "old-write";
echo "array-old=", $items["slot"], "|", $old, "|", $same, "|", $new, "\n";

class Milestone1659_Bag {
    public $items = array("slot" => "box-old");
}

$bag = new Milestone1659_Bag();
$propertyOld =& $bag->items["slot"];
$propertySame =& $propertyOld;
$propertyNew = "box-new";
$bag->items["slot"] =& $propertyNew;
$propertyNew = "box-changed";
echo "property-rebind=", $bag->items["slot"], "|", $propertyOld, "|", $propertySame, "|", $propertyNew, "\n";
$propertySame = "box-old-write";
echo "property-old=", $bag->items["slot"], "|", $propertyOld, "|", $propertySame, "|", $propertyNew;
