<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items[0] = "zero-key";
$items["2"] = "two-key";
$items["02"] = "zero-two-key";
$items[] = "appended";
$items["numeric"] = "10.0";
$items["text"] = "abc";

var_dump(array_search("", $items));
var_dump(array_search("0", $items));
var_dump(array_search("zero-key", $items));
var_dump(array_search("two-key", $items));
var_dump(array_search("zero-two-key", $items));
var_dump(array_search("appended", $items));
var_dump(array_search("10", $items));
var_dump(array_search("missing", $items));

$call = "array_search";
var_dump($call("abc", $items));
