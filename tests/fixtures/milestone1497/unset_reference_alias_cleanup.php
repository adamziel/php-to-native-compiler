<?php
class WP_RefCow_Unset_Alias_Bag {
    public $items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
}

$items = ["slot" => "seed", "outer" => ["leaf" => "nested"]];
$alias =& $items["slot"];
$leaf =& $items["outer"]["leaf"];

unset($items["slot"]);
echo array_key_exists("slot", $items) ? "array-slot:set" : "array-slot:unset";
echo "|alias=", $alias, "\n";
$alias = "after";
echo array_key_exists("slot", $items) ? "array-slot:set:" . $items["slot"] : "array-slot:unset";
echo "|alias=", $alias, "\n";

unset($items["outer"]);
echo array_key_exists("outer", $items) ? "array-outer:set" : "array-outer:unset";
echo "|leaf=", $leaf, "\n";
$leaf = "changed";
echo array_key_exists("outer", $items) ? "array-outer:set" : "array-outer:unset";
echo "|leaf=", $leaf, "\n";

$bag = new WP_RefCow_Unset_Alias_Bag();
$propertyAlias =& $bag->items["slot"];
$propertyLeaf =& $bag->items["outer"]["leaf"];

unset($bag->items["slot"]);
echo array_key_exists("slot", $bag->items) ? "property-slot:set" : "property-slot:unset";
echo "|alias=", $propertyAlias, "\n";
$propertyAlias = "property-after";
echo array_key_exists("slot", $bag->items) ? "property-slot:set:" . $bag->items["slot"] : "property-slot:unset";
echo "|alias=", $propertyAlias, "\n";

unset($bag->items["outer"]);
echo array_key_exists("outer", $bag->items) ? "property-outer:set" : "property-outer:unset";
echo "|leaf=", $propertyLeaf, "\n";
$propertyLeaf = "property-changed";
echo array_key_exists("outer", $bag->items) ? "property-outer:set" : "property-outer:unset";
echo "|leaf=", $propertyLeaf;
