<?php
$items = [];
$alias =& $items[];
$alias = "array-alias";
echo $items[0], "\n";
$items[0] = "array-slot";
echo $alias, "\n";

$nested = ["outer" => [2 => "seed"]];
$nested_alias =& $nested["outer"][];
$nested_alias = "nested-alias";
echo $nested["outer"][3], "\n";
$nested["outer"][3] = "nested-slot";
echo $nested_alias, "\n";

class Box {
    public $items;
    public $groups = ["outer" => [4 => "seed"]];
}
$box = new Box();
$prop_alias =& $box->items[];
$prop_alias = "property-alias";
echo $box->items[0], "\n";
$box->items[0] = "property-slot";
echo $prop_alias, "\n";

$group_alias =& $box->groups["outer"][];
$group_alias = "group-alias";
echo $box->groups["outer"][5], "\n";
$box->groups["outer"][5] = "group-slot";
echo $group_alias, "\n";
