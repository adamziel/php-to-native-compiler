<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[] = "next";

$chunks = array_chunk($items, 2);
echo count($chunks), "|", count($chunks[0]), "|", count($chunks[1]), "|", count($chunks[2]), "\n";
echo $chunks[0][0], "|", $chunks[0][1], "|", $chunks[1][0], "|", $chunks[1][1], "|", $chunks[2][0], "\n";
if (array_key_exists("02", $chunks[1])) {
    echo "string-key-kept\n";
} else {
    echo "string-key-reindexed\n";
}
$second = $chunks[1];
$second[] = "after";
echo $second[2], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[6], "\n";

$one = array_chunk($items, 99);
echo count($one), "|", count($one[0]), "|", $one[0][4], "\n";

$empty = array_chunk([], 2);
echo count($empty), "\n";

$call = "array_chunk";
$again = $call($items, 3);
echo count($again), "|", $again[0][0], "|", $again[0][2], "|", $again[1][0], "|", $again[1][1];
