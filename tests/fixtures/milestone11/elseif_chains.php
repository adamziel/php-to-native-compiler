<?php
$value = 2;
if ($value == 1) {
    echo "one";
} elseif ($value == 2) {
    echo "two";
} elseif ($missing) {
    echo "missing";
} else {
    echo "else";
}
echo "\n";

$word = "fallback";
if ($word == "none") echo "none";
elseif ($word == "fallback") echo "single";
else echo "else";
echo "\n";

$flag = false;
if ($flag) {
    echo "flag";
} elseif (0) {
    echo "zero";
} else {
    echo "last";
}
