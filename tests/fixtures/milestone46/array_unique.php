<?php
$items = [];
$items[5] = "five";
$items[9] = "five";
$items[2] = "two";
$items["null"] = null;
$items["false"] = false;
$items["empty"] = "";
$items["true"] = true;
$items["one"] = 1;
$items["string-one"] = "1";
$items["int-ten"] = 10;
$items["float-ten"] = 10.0;
$items["string-ten-float"] = "10.0";
$items["text"] = "abc";
$items["dup-text"] = "abc";
$items[] = "next";

$unique = array_unique($items);
print_r($unique);
echo count($unique), "\n";
echo $unique[5], "|", $unique[2], "|", $unique["true"], "|", $unique["int-ten"], "|", $unique["string-ten-float"], "|", $unique[10], "\n";
$unique[] = "after";
echo $unique[11], "\n";
print_r($items);

$call = "array_unique";
$again = $call($items);
echo $again[5], "|", $again[2], "|", $again["true"], "|", $again["int-ten"], "|", $again["string-ten-float"], "|", $again[10], "\n";

$empty = array_unique([]);
print_r($empty);
echo count($empty), "\n";
echo "done";
