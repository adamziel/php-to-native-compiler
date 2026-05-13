<?php
function trace_xor($name, $value) {
    echo "trace:", $name, "\n";
    return $value;
}

var_dump(false xor false);
var_dump(false xor true);
var_dump(true xor false);
var_dump(true xor true);
var_dump("0" xor false);
var_dump("php" xor []);
var_dump([1] xor "php");

$word_xor_false = false xor true;
$word_xor_true = true xor false;
var_dump($word_xor_false);
var_dump($word_xor_true);

var_dump(true xor false and false);
var_dump(true or true xor true);

$left = false;
$right = false;
var_dump(($left = true) xor ($right = false));
var_dump($left);
var_dump($right);

var_dump(false xor trace_xor("rhs-true", true));
var_dump(true xor trace_xor("rhs-true-again", true));
