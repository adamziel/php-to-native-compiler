<?php
$first = "one";
$second = "two";
$third = "three";
$items = [3 => &$first, "name" => "plain", 2 => &$second, 1 => &$third];

$chunks = array_chunk($items, 2);
var_dump($chunks);
$second = "changed";
var_dump($chunks);

$preserved = array_chunk($items, 2, true);
var_dump($preserved);
$third = "later";
var_dump($preserved);
