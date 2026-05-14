<?php
$int = 0;
$float = 3.5;
$numeric_string = " 42 ";
$numeric_fraction = "-.5";
$numeric_trailing_dot = "5.";
$numeric_exponent = "8e2";
$empty = "";
$space = " ";
$text = "8foo";
$hexish = "0x10";
$bool = true;
$null = null;

echo is_numeric($int) ? "1" : "0";
echo is_numeric($float) ? "1" : "0";
echo is_numeric($numeric_string) ? "1" : "0";
echo is_numeric($numeric_fraction) ? "1" : "0";
echo is_numeric($numeric_trailing_dot) ? "1" : "0";
echo is_numeric($numeric_exponent) ? "1" : "0";
echo is_numeric($empty) ? "1" : "0";
echo is_numeric($space) ? "1" : "0";
echo is_numeric($text) ? "1" : "0";
echo is_numeric($hexish) ? "1" : "0";
echo is_numeric($bool) ? "1" : "0";
echo is_numeric($null) ? "1" : "0";
