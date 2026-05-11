<?php
$items = [];
$items[] = "zero";
$items[] = "one";
$items[1] = "one updated";
$items["2"] = "two";
$items[] = "three";
$items["02"] = "zero two";
$key = "name";
$items[$key] = "Ada";

$created[] = "created";
$by_index[2] = "indexed";
$nullable = null;
$nullable[] = "from null";

echo $items[0], "\n";
echo $items["1"], "\n";
echo $items[2], "\n";
echo $items[3], "\n";
echo $items["02"], "\n";
echo $items["name"], "\n";
echo count($items), "\n";
echo $created[0], "\n";
echo $by_index[2], "\n";
echo $nullable[0], "\n";
print_r($items);
