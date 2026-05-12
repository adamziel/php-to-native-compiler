<?php
$items = [];
$items[] = false;
$items[] = 0;
$items[] = "0";
$items[] = 10;
$items[] = "10";
$items[] = null;
$items[] = "abc";

if (in_array("", $items, true)) {
    echo "unexpected-empty\n";
} else {
    echo "empty-missing\n";
}
if (in_array(false, $items, true)) {
    echo "false-match\n";
}
if (in_array(0, $items, true)) {
    echo "int-zero-match\n";
}
if (in_array("0", $items, true)) {
    echo "string-zero-match\n";
}
if (in_array(10.0, $items, true)) {
    echo "unexpected-float\n";
} else {
    echo "float-missing\n";
}
if (in_array(10, $items, true)) {
    echo "int-ten-match\n";
}
if (in_array("10", $items, true)) {
    echo "string-ten-match\n";
}
if (in_array(null, $items, true)) {
    echo "null-match\n";
}
if (in_array("missing", $items, true)) {
    echo "unexpected-missing\n";
} else {
    echo "string-missing\n";
}
if (in_array("10.0", $items, false)) {
    echo "false-flag-uses-loose\n";
}

$call = "in_array";
if ($call("abc", $items, true)) {
    echo "dynamic-strict-match";
}
