<?php
$items = [];
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["null"] = null;
$items[2] = "int-key";
$items["text"] = "abc";

var_dump(array_search("", $items, true));
var_dump(array_search(false, $items, true));
var_dump(array_search(0, $items, true));
var_dump(array_search("0", $items, true));
var_dump(array_search(10.0, $items, true));
var_dump(array_search(10, $items, true));
var_dump(array_search("10", $items, true));
var_dump(array_search(null, $items, true));
var_dump(array_search("int-key", $items, true));
var_dump(array_search("missing", $items, true));
var_dump(array_search("10.0", $items, false));

$call = "array_search";
var_dump($call("abc", $items, true));
