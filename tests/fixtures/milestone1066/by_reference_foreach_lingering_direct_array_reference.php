<?php
$items = ["a", "b", "c"];

foreach ($items as &$item) {
    $item = $item . "!";
}

$items[2] = "direct";
echo $item;
echo "|";
$item = "tail";
echo $items[0], "|", $items[1], "|", $items[2], "|", $item;
unset($item);
$item = "detached";
echo "|", $items[2], "|", $item;
