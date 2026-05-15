<?php
$items = array(2 => "two", "name" => "Ada", 5 => "five");
echo array_pop($items), "|";
echo count($items), "|";
$items[] = "new";
echo $items[5], "|";
echo array_pop($items), "|";
echo array_pop($items), "|";
var_dump(array_pop($items));
$call = "array_pop";
$stack = array("first", "second");
echo $call($stack), "|", count($stack);
