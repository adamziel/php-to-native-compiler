<?php
class Profile {
    public $name;
    public $isFalse;
    public $zero;
    public $emptyString;
    public $zeroString;
    public $items;
    public $filled;
}

$profile = new Profile();
$profile->isFalse = false;
$profile->zero = 0;
$profile->emptyString = "";
$profile->zeroString = "0";
$profile->items = [];
$profile->filled = "Ada";

if (empty($profile->name)) {
    echo "null-slot:empty\n";
}
if (empty($profile->isFalse)) {
    echo "false:empty\n";
}
if (empty($profile->zero)) {
    echo "zero:empty\n";
}
if (empty($profile->emptyString)) {
    echo "empty-string:empty\n";
}
if (empty($profile->zeroString)) {
    echo "zero-string:empty\n";
}
if (empty($profile->items)) {
    echo "empty-array:empty\n";
}
if (empty($profile->filled)) {
    echo "filled:empty\n";
} else {
    echo "filled:not-empty\n";
}
if (empty($profile->missing)) {
    echo "missing-property:empty\n";
}

$number = 42;
if (empty($number->name)) {
    echo "scalar-target:empty\n";
}
if (empty($missing->name)) {
    echo "missing-target:empty";
}
