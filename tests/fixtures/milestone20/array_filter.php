<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["true"] = true;
$items["zero"] = 0;
$items["float-zero"] = 0.0;
$items["one"] = 1;
$items["empty-string"] = "";
$items["zero-string"] = "0";
$items["space"] = " ";
$items["text"] = "Ada";
$items["empty-array"] = [];
$items["nested-array"] = ["kept"];
$items[7] = "seven";
$items[] = "next";

$filtered = array_filter($items);
print_r(array_keys($filtered));
echo count($filtered), "\n";
echo $filtered["true"], "|", $filtered["one"], "|", $filtered["space"], "|", $filtered["text"], "|", count($filtered["nested-array"]), "|", $filtered[7], "|", $filtered[8], "\n";
if (array_key_exists("null", $filtered)) {
    echo "null kept\n";
} else {
    echo "null removed\n";
}
if (array_key_exists("empty-array", $filtered)) {
    echo "empty array kept\n";
} else {
    echo "empty array removed\n";
}
$filtered[] = "after";
echo $filtered[9], "\n";

$call = "array_filter";
$again = $call($items);
echo count($again), "|", count($again["nested-array"]);
