<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $item) {
    echo $item, "|";
}
echo "\nlast:", $item, "\n";

$numbers = [1, 2, 3, 4, 5];
foreach ($numbers as $number) {
    if ($number == 2) {
        continue;
    }
    if ($number == 4) {
        break;
    }
    echo $number, ",";
}
echo "after:", $number, "\n";

$empty = [];
$item = "kept";
foreach ($empty as $item) {
    echo "unreachable";
}
echo "empty:", $item, "\n";
