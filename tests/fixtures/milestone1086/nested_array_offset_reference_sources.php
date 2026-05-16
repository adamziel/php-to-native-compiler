<?php
class Box {
    public $items;
}

$items = ["outer" => ["inner" => "x"]];
$alias =& $items["outer"]["inner"];
$alias = "direct-alias";
echo $items["outer"]["inner"], "|";
$items["outer"]["inner"] = "direct-slot";
echo $alias, "\n";

$items = [];
$missing =& $items["created"]["slot"];
$missing = "materialized";
echo $items["created"]["slot"], "\n";

$box = new Box();
$box->items = ["outer" => ["inner" => "x"]];
$property_alias =& $box->items["outer"]["inner"];
$property_alias = "property-alias";
echo $box->items["outer"]["inner"], "|";
$box->items["outer"]["inner"] = "property-slot";
echo $property_alias, "\n";

$box = new Box();
$property_missing =& $box->items["created"]["slot"];
$property_missing = "property-materialized";
echo $box->items["created"]["slot"];
