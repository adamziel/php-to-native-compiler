<?php
class Bag {
    public function &items(&$items) {
        return $items;
    }
}

class StaticBag {
    public static function &items(&$items) {
        return $items;
    }
}

$bag = new Bag();
$items = ["a" => "one", "b" => "two"];
foreach ($bag->items($items) as $key => &$item) {
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

$items = ["x" => "ex", "y" => "why"];
foreach (StaticBag::items($items) as $key => &$item) {
    $item = $key . "=" . $item;
    if ($key === "x") {
        $items["z"] = "zed";
    }
}
echo $items["x"], "|", $items["y"], "|", $items["z"], "|", $item, "\n";
$items["z"] = "static-direct";
echo $item, "|";
$item = "static-tail";
echo $items["z"], "|", $item;
