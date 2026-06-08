<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$cases = [
    "int-one" => 1,
    "int-zero" => 0,
    "string-one" => "1",
    "string-zero" => "0",
    "empty-string" => "",
    "null" => null,
    "float" => 2.5,
];

foreach ($cases as $label => $flag) {
    $reversed = array_reverse($items, $flag);
    echo $label, ":";
    if (array_key_exists(6, $reversed)) {
        echo "preserved:", $reversed[6], "|", $reversed[-1], "|", $reversed["name"], "\n";
    } else {
        echo "reindexed:", $reversed[0], "|", $reversed[1], "|", $reversed["name"], "\n";
    }
}

$call = "array_reverse";
$dynamic = $call($items, "yes");
echo "dynamic:", $dynamic[6], "|", $dynamic[-1], "|", $dynamic["name"], "\n";

try {
    array_reverse($items, []);
} catch (TypeError $e) {
    echo "type-error:", $e->getMessage(), "\n";
}

echo "source:", $items["name"], "|", $items[5], "|", $items[2], "|", $items["02"], "|", $items[-1], "|", $items[6];
