<?php
function &wp_refcow_pick_child(&$items, $key) {
    return $items[$key];
}

$items = ["outer" => ["a" => "one", "b" => "two"]];
$mirror =& $items;

foreach (wp_refcow_pick_child($mirror, "outer") as $key => &$value) {
    $value = $value . ":" . $key;
    if ($key === "a") {
        $items["outer"]["c"] = "three";
    }
}

echo $items["outer"]["a"], "|", $mirror["outer"]["b"], "|", $value, "\n";
$items["outer"]["c"] = "direct";
echo $value, "|";
$value = "tail";
echo $mirror["outer"]["c"], "|", $value;
