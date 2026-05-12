<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["true"] = true;
$items["zero"] = 0;
$items["zero-string"] = "0";
$items["space"] = " ";
$items["text"] = "Ada";
$items[] = "tail";

$filtered = array_filter($items, null);
print_r(array_keys($filtered));
echo count($filtered), "\n";
echo $filtered["true"], "|", strlen($filtered["space"]), "|", $filtered["text"], "|", $filtered[0], "\n";

$call = "array_filter";
$again = $call($items, null);
echo count($again), "\n";
if (array_key_exists("null", $again)) {
    echo "null kept\n";
} else {
    echo "null removed\n";
}
if (array_key_exists("zero-string", $again)) {
    echo "zero string kept\n";
} else {
    echo "zero string removed\n";
}
