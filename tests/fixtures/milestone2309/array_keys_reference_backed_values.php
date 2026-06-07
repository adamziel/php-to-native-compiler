<?php
$arr = array(1, "1", "", NULL, 0, false, true, array());

$s = &$arr[0];
var_dump(array_keys($arr, $s, true));

$s = &$arr[1];
var_dump(array_keys($arr, $s, true));

$s = &$arr[2];
var_dump(array_keys($arr, $s, true));

$s = &$arr[3];
var_dump(array_keys($arr, $s, true));

$s = &$arr[4];
var_dump(array_keys($arr, $s, true));

$s = &$arr[5];
var_dump(array_keys($arr, $s, true));

$s = &$arr[6];
var_dump(array_keys($arr, $s, true));

$s = &$arr[7];
$last = array_keys($arr, $s, true);
echo "array(1) {\n  [0]=>\n  int(", $last[0], ")\n}";
