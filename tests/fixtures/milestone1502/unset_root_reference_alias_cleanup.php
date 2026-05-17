<?php
$items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
$alias =& $items["slot"];
$leaf =& $items["outer"]["leaf"];

unset($items);
echo isset($items) ? "array:set" : "array:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";
$alias = "after";
$leaf = "changed";
echo isset($items) ? "array:set" : "array:unset";
echo "|alias=", $alias, "|leaf=", $leaf, "\n";

class WP_RefCow_Unset_Root_Bag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
}

$bag = new WP_RefCow_Unset_Root_Bag();
$propertyAlias =& $bag->items["slot"];
$propertyLeaf =& $bag->items["outer"]["leaf"];

unset($bag);
echo isset($bag) ? "object:set" : "object:unset";
echo "|alias=", $propertyAlias, "|leaf=", $propertyLeaf, "\n";
$propertyAlias = "property-after";
$propertyLeaf = "property-changed";
echo isset($bag) ? "object:set" : "object:unset";
echo "|alias=", $propertyAlias, "|leaf=", $propertyLeaf;
