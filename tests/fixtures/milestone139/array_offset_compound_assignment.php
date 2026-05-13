<?php
$items = ["count" => 1, 2 => 10, "text" => "php"];
$items["count"] += 4;
$items[2] *= 3;
$items["text"] .= "-native";
echo $items["count"], ":", $items[2], ":", $items["text"], "\n";
echo ($items["count"] -= 2), ":", $items["count"], "\n";
$key = "count";
echo ($items[$key] /= 3), ":", $items[$key], "\n";

function key_name() {
    echo "key\n";
    return "count";
}
function next_value() {
    echo "rhs\n";
    return 2;
}
$items[key_name()] += next_value();
echo $items["count"], "\n";

$items["loop"] = 0;
for ($items["i"] = 0; $items["i"] < 3; $items["i"] += 1) {
    $items["loop"] += $items["i"];
}
echo $items["loop"], ":", $items["i"];
