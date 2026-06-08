<?php
$value = "ref";
$items = ["first" => "plain"];
$items["ref"] =& $value;
$items[] = "tail";

$count = array_unshift($items, "head");
$value = "changed";
echo $count, "|", $items[0], "|", $items["first"], "|", $items["ref"], "|", $items[1], "\n";

$items["ref"] = "through-item";
echo $value, "\n";

$call = "array_unshift";
$nested = ["outer" => $items];
echo $call($nested["outer"], "nested-head"), "|";
$value = "nested-changed";
echo $nested["outer"]["ref"], "\n";
