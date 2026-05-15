<?php
$list = array(1, "2", null);
$same = array_values($list);
$reordered = array(1 => "2", 0 => 1, 2 => null);
$different_type = array(1, 2, null);
$nested = array("items" => $list);
$nested_same = array("items" => $same);

echo array() === array() ? "empty-same\n" : "empty-different\n";
echo $list === $same ? "list-same\n" : "list-different\n";
echo $list === $reordered ? "order-same\n" : "order-different\n";
echo $list !== $reordered ? "not-order-same\n" : "not-order-different\n";
echo $list === $different_type ? "type-same\n" : "type-different\n";
echo $nested === $nested_same ? "nested-same\n" : "nested-different\n";
echo $list === null ? "array-null-same" : "array-null-different";
