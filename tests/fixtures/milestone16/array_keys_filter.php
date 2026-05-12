<?php
$items = [];
$items["null"] = null;
$items["false"] = false;
$items["int-zero"] = 0;
$items["string-zero"] = "0";
$items["empty"] = "";
$items["int-ten"] = 10;
$items["string-ten"] = "10";
$items["numeric-string"] = "10.0";
$items["text"] = "abc";

$empty = array_keys($items, "");
print_r($empty);

$zero = array_keys($items, "0");
print_r($zero);

$ten = array_keys($items, "10");
print_r($ten);

$text = array_keys($items, "abc");
print_r($text);

$missing = array_keys($items, "missing");
print_r($missing);

$call = "array_keys";
$dynamic = $call($items, "10.0");
print_r($dynamic);

$strict_empty = array_keys($items, "", true);
print_r($strict_empty);

$strict_false = array_keys($items, false, true);
print_r($strict_false);

$strict_int_zero = array_keys($items, 0, true);
print_r($strict_int_zero);

$strict_string_zero = array_keys($items, "0", true);
print_r($strict_string_zero);

$strict_float_ten = array_keys($items, 10.0, true);
print_r($strict_float_ten);

$strict_int_ten = array_keys($items, 10, true);
print_r($strict_int_ten);

$strict_string_ten = array_keys($items, "10", true);
print_r($strict_string_ten);

$strict_null = array_keys($items, null, true);
print_r($strict_null);

$strict_missing = array_keys($items, "missing", true);
print_r($strict_missing);

$loose_false_flag = array_keys($items, "10.0", false);
print_r($loose_false_flag);

$dynamic_strict = $call($items, "abc", true);
print_r($dynamic_strict);
