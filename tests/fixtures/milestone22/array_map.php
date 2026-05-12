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
echo $mapped["first"], "|", $mapped[5], "|", $mapped["empty"], "|", $mapped[6], "\n";
$mapped[] = "after";
echo $mapped[7], "\n";
print_r($items);

$call = "array_map";
$lengths = $call("strlen", ["empty" => "", "zero" => "0", "space" => " "]);
echo count($lengths), "|", $lengths["empty"], "|", $lengths["zero"], "|", $lengths["space"];
