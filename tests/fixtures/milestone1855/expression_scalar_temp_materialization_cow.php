<?php
error_reporting(0);

function milestone1855_make_false() {
    return false;
}

function milestone1855_make_null() {
    return null;
}

$falseKey =& milestone1855_make_false()["slot"];
$falseKey = "false-key";

$falseAppend =& milestone1855_make_false()["outer"][];
$falseAppend = "false-append";

$nullKey =& milestone1855_make_null()["slot"];
$nullKey = "null-key";

$nullAppend =& milestone1855_make_null()["outer"][];
$nullAppend = "null-append";

echo $falseKey, "|", $falseAppend, "|", $nullKey, "|", $nullAppend;
