<?php
$items = array("slot" => "seed", "outer" => array("leaf" => "nested"));
$alias =& $items["slot"];
$second =& $alias;

unset($items["slot"]);
$alias = "after";
echo array_key_exists("slot", $items) ? "array-slot:set:" . $items["slot"] : "array-slot:unset";
echo "|alias=", $alias, "|second=", $second, "\n";
$second = "again";
echo array_key_exists("slot", $items) ? "array-slot:set:" . $items["slot"] : "array-slot:unset";
echo "|alias=", $alias, "|second=", $second, "\n";

$leaf =& $items["outer"]["leaf"];
$other =& $leaf;
unset($items["outer"]);
$leaf = "changed";
echo array_key_exists("outer", $items) ? "array-parent:set" : "array-parent:unset";
echo "|leaf=", $leaf, "|other=", $other, "\n";

class Milestone1654_Bag {
    public $items = array("slot" => "box", "outer" => array("leaf" => "branch"));
}

$bag = new Milestone1654_Bag();
$propertyAlias =& $bag->items["slot"];
$propertySecond =& $propertyAlias;

unset($bag->items["slot"]);
$propertyAlias = "property-after";
echo array_key_exists("slot", $bag->items) ? "property-slot:set:" . $bag->items["slot"] : "property-slot:unset";
echo "|alias=", $propertyAlias, "|second=", $propertySecond, "\n";
$propertySecond = "property-again";
echo array_key_exists("slot", $bag->items) ? "property-slot:set:" . $bag->items["slot"] : "property-slot:unset";
echo "|alias=", $propertyAlias, "|second=", $propertySecond, "\n";

$propertyLeaf =& $bag->items["outer"]["leaf"];
$propertyOther =& $propertyLeaf;
unset($bag->items["outer"]);
$propertyLeaf = "property-changed";
echo array_key_exists("outer", $bag->items) ? "property-parent:set" : "property-parent:unset";
echo "|leaf=", $propertyLeaf, "|other=", $propertyOther;
