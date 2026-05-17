<?php
function &items(&$items) {
    return $items;
}

$items = ["a" => "one", "b" => "two"];
foreach (items($items) as $key => &$item) {
    $item = $item . ":" . $key;
    if ($key === "a") {
        $items["c"] = "three";
    }
}

echo $items["a"], "|", $items["b"], "|", $items["c"], "|", $item, "\n";
$items["c"] = "direct";
echo $item, "|";
$item = "tail";
echo $items["c"], "|", $item, "\n";
unset($item);
$items["c"] = "detached";
echo $items["c"];
