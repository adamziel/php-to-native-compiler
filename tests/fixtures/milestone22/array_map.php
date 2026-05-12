<?php
function label_value($value) {
    return "mapped:" . $value;
}

$items = [];
$items["first"] = "Ada";
$items[5] = "Bob";
$items["empty"] = "";
$items[] = "Linus";

$mapped = array_map("label_value", $items);
print_r(array_keys($mapped));
echo $mapped[0], "|", $mapped[1], "|", $mapped[2], "|", $mapped[3], "\n";
$mapped[] = "after";
echo $mapped[4], "\n";
print_r($items);

$call = "array_map";
$lengths = $call("strlen", ["empty" => "", "zero" => "0", "space" => " "]);
echo count($lengths), "|", $lengths[0], "|", $lengths[1], "|", $lengths[2];
