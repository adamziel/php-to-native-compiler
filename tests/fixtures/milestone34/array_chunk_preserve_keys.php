<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$preserved = array_chunk($items, 2, true);
echo count($preserved), "|", count($preserved[0]), "|", count($preserved[1]), "|", count($preserved[2]), "\n";
echo $preserved[0]["name"], "|", $preserved[0][5], "|", $preserved[1][2], "|", $preserved[1]["02"], "|", $preserved[2][-1], "|", $preserved[2][6], "\n";
if (array_key_exists(0, $preserved[0])) {
    echo "first-reindexed\n";
} else {
    echo "first-preserved\n";
}
$first = $preserved[0];
$first[] = "after";
echo $first[6], "\n";

$default_false = array_chunk($items, 2, false);
echo $default_false[0][0], "|", $default_false[0][1], "\n";
if (array_key_exists("name", $default_false[0])) {
    echo "default-false-preserved\n";
} else {
    echo "default-false-reindexed\n";
}

$call = "array_chunk";
$dynamic = $call($items, 3, true);
echo count($dynamic), "|", $dynamic[0]["name"], "|", $dynamic[0][5], "|", $dynamic[0][2], "|", $dynamic[1]["02"], "|", $dynamic[1][-1], "|", $dynamic[1][6], "\n";
echo $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
