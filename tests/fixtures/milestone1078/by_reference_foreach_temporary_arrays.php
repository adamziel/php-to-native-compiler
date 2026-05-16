<?php
foreach (["a", "b"] as &$value) {
    $value = $value . "!";
    echo $value, "|";
}
echo "after=", $value, "|";
$value = "changed";
echo $value, "\n";

function temporary_items() {
    return ["x" => 1, "y" => 2];
}

foreach (temporary_items() as $key => &$item) {
    $item = $item + 10;
    echo $key, ":", $item, "|";
}
echo "after=", $item, "|";
$item = 99;
echo $item;
