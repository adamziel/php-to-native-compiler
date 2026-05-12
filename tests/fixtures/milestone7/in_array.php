<?php
$items = [];
$items[] = null;
$items[] = false;
$items[] = 10;
$items[] = "10.0";
$items[] = "abc";

if (in_array("", $items)) {
    echo "empty-matches-null\n";
}
if (in_array("0", $items)) {
    echo "zero-matches-false\n";
}
if (in_array("10", $items)) {
    echo "numeric-string-matches-int\n";
}
if (in_array(10.0, $items)) {
    echo "float-matches-int\n";
}
if (in_array("abc", $items)) {
    echo "string-match\n";
}
if (in_array(11, $items)) {
    echo "unexpected-int\n";
} else {
    echo "missing-int\n";
}
if (in_array("missing", $items)) {
    echo "unexpected-string\n";
} else {
    echo "missing-string\n";
}

$call = "in_array";
if ($call("abc", $items)) {
    echo "dynamic-match";
}
