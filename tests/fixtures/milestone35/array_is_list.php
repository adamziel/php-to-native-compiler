<?php
$empty = [];
var_dump(array_is_list($empty));

$list = ["zero", "one"];
$list[] = "two";
var_dump(array_is_list($list));

$normalized = [];
$normalized["0"] = "zero";
$normalized["1"] = "one";
var_dump(array_is_list($normalized));

$out_of_order = [];
$out_of_order[1] = "one";
$out_of_order[0] = "zero";
var_dump(array_is_list($out_of_order));

$gap = [];
$gap[0] = "zero";
$gap[2] = "two";
var_dump(array_is_list($gap));

$string_key = [];
$string_key[0] = "zero";
$string_key["01"] = "one";
var_dump(array_is_list($string_key));

$negative = [];
$negative[-1] = "negative";
$negative[0] = "zero";
var_dump(array_is_list($negative));

$after_unset = [0 => "zero", 1 => "one", 2 => "two"];
unset($after_unset[1]);
var_dump(array_is_list($after_unset));

$reindexed = array_values($after_unset);
var_dump(array_is_list($reindexed));

$call = "array_is_list";
var_dump($call([0 => "a", 1 => "b"]));
var_dump($call([1 => "b", 0 => "a"]));
