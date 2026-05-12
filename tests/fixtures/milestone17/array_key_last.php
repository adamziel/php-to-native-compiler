<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

var_dump(array_key_last($items));

$string_last = [];
$string_last["name"] = "Ada";
$string_last[5] = "five";
$string_last["2"] = "two";
$string_last["02"] = "zero two";
$string_last["2"] = "two updated";
var_dump(array_key_last($string_last));

$int_last = [];
$int_last["name"] = "Ada";
$int_last["02"] = "zero two";
$int_last["2"] = "two";
var_dump(array_key_last($int_last));

$empty = [];
var_dump(array_key_last($empty));

$call = "array_key_last";
var_dump($call($items));
