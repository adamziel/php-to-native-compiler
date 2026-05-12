<?php
$null = null;
$false = false;
$true = true;
$zero = 0;
$one = 1;
$empty_string = "";
$zero_string = "0";
$text = "text";
$empty_array = [];
$filled_array = [0];

$items = [];
$items["present"] = "value";
$items["null"] = null;
$items["false"] = false;
$items["zero"] = 0;
$items["empty"] = "";
$items["zero_string"] = "0";
$items["empty_array"] = [];
$items["filled_array"] = [0];
$items["2"] = "two";
$key = "present";

if (empty($missing)) {
    echo "missing:empty\n";
}
if (empty($null)) {
    echo "null:empty\n";
}
if (empty($false)) {
    echo "false:empty\n";
}
if (empty($zero)) {
    echo "zero:empty\n";
}
if (empty($empty_string)) {
    echo "empty-string:empty\n";
}
if (empty($zero_string)) {
    echo "zero-string:empty\n";
}
if (empty($true)) {
    echo "true:empty\n";
} else {
    echo "true:not-empty\n";
}
if (empty($one)) {
    echo "one:empty\n";
} else {
    echo "one:not-empty\n";
}
if (empty($text)) {
    echo "text:empty\n";
} else {
    echo "text:not-empty\n";
}
if (empty($empty_array)) {
    echo "empty-array:empty\n";
}
if (empty($filled_array)) {
    echo "filled-array:empty\n";
} else {
    echo "filled-array:not-empty\n";
}
if (empty($items[$key])) {
    echo "offset-present:empty\n";
} else {
    echo "offset-present:not-empty\n";
}
if (empty($items["null"])) {
    echo "offset-null:empty\n";
}
if (empty($items["false"])) {
    echo "offset-false:empty\n";
}
if (empty($items["zero"])) {
    echo "offset-zero:empty\n";
}
if (empty($items["empty"])) {
    echo "offset-empty-string:empty\n";
}
if (empty($items["zero_string"])) {
    echo "offset-zero-string:empty\n";
}
if (empty($items["missing"])) {
    echo "offset-missing:empty\n";
}
if (empty($missing_array[0])) {
    echo "offset-undefined-array:empty\n";
}
$number = 42;
if (empty($number[0])) {
    echo "offset-scalar-target:empty\n";
}
if (empty($items[2])) {
    echo "offset-int-normalized:empty";
} else {
    echo "offset-int-normalized:not-empty";
}
