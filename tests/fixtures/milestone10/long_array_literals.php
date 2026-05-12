<?php
$items = array(
    "first",
    2 => "two",
    "2" => "two updated",
    "02" => "zero two",
    "name" => "Ada",
    1 + 2 => "three",
);
$nested = ARRAY(
    "inner" => array("left", "right"),
);

echo count($items), "\n";
print_r($items);
echo $items[0], "|", $items[2], "|", $items["02"], "|", $items["name"], "|", $items[3], "\n";
echo $nested["inner"][1], "\n";
