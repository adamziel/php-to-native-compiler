<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[] = "next";

foreach ($items as $key => $item) {
    echo $key, ":", $item, "|";
}
echo "\nlast:", $key, "=", $item, "\n";

$numbers = [10 => "ten", 20 => "twenty", 30 => "thirty", 40 => "forty"];
foreach ($numbers as $numberKey => $number) {
    if ($numberKey == 20) {
        continue;
    }
    if ($numberKey == 40) {
        break;
    }
    echo $numberKey, "=", $number, ",";
}
echo "after:", $numberKey, "=", $number, "\n";

$empty = [];
$key = "kept-key";
$item = "kept-value";
foreach ($empty as $key => $item) {
    echo "unreachable";
}
echo "empty:", $key, "=", $item, "\n";
