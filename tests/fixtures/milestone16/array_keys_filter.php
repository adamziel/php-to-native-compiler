<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["empty"] = "";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["numeric-string"] = "10.0";
$items["text"] = "abc";

$empty = array_keys($items, "");
print_r($empty);

$zero = array_keys($items, "0");
print_r($zero);

$ten = array_keys($items, "10");
print_r($ten);

$text = array_keys($items, "abc");
print_r($text);

$missing = array_keys($items, "missing");
print_r($missing);

$call = "array_keys";
$dynamic = $call($items, "10.0");
print_r($dynamic);
