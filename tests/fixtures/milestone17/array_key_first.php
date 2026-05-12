<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items["2"] = "two updated";
$items[-1] = "negative";
$items[] = "next";

var_dump(array_key_first($items));

$int_first = [];
$int_first["2"] = "two";
$int_first["02"] = "zero two";
$int_first["name"] = "Ada";
var_dump(array_key_first($int_first));

$empty = [];
var_dump(array_key_first($empty));

$call = "array_key_first";
var_dump($call($items));
