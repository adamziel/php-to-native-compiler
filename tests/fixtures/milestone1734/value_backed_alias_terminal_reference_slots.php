<?php
class Milestone1734_Box {
    public $items = [];
}

$items = [];
$items["slot"] = "seed";
$alias =& $items["slot"];

$target = [];
$target["copy"] =& $alias;
$target["outer"]["copy"] =& $alias;

$box = new Milestone1734_Box();
$box->items["copy"] =& $alias;
$box->items["outer"][] =& $alias;

$target["copy"] = "direct-target-write";
$target["outer"]["copy"] = "nested-target-write";
$box->items["copy"] = "property-target-write";
$box->items["outer"][0] = "property-append-write";

echo $items["slot"],
    "|",
    $alias,
    "|",
    $target["copy"],
    "|",
    $target["outer"]["copy"],
    "|",
    $box->items["copy"],
    "|",
    $box->items["outer"][0],
    "\n";

$propertyBox = new Milestone1734_Box();
$propertyBox->items["slot"] = "property-seed";
$propertyAlias =& $propertyBox->items["slot"];
$propertyTarget = [];
$propertyTarget["copy"] =& $propertyAlias;
$propertyTarget["copy"] = "property-target-write";

echo $propertyBox->items["slot"],
    "|",
    $propertyAlias,
    "|",
    $propertyTarget["copy"],
    "\n";

$dynamicBox = new Milestone1734_Box();
$propertyName = "items";
$dynamicBox->{$propertyName}["slot"] = "dynamic-seed";
$dynamicAlias =& $dynamicBox->{$propertyName}["slot"];
$dynamicTarget = [];
$dynamicTarget["copy"] =& $dynamicAlias;
$dynamicTarget["copy"] = "dynamic-target-write";

echo $dynamicBox->items["slot"],
    "|",
    $dynamicAlias,
    "|",
    $dynamicTarget["copy"];
