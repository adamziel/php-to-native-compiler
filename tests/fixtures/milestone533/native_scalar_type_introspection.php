<?php
$null = null;
$bool = false;
$int = 7;
$float = 3.5;
$string = "x";

echo gettype($null), "\n";
echo gettype($bool), "\n";
echo gettype($int), "\n";
echo gettype($float), "\n";
echo gettype($string), "\n";
echo is_null($null) ? "1" : "0", is_null($int) ? "1" : "0", "\n";
echo is_bool($bool) ? "1" : "0", is_bool($int) ? "1" : "0", "\n";
echo is_int($int) ? "1" : "0", is_integer($int) ? "1" : "0", is_long($int) ? "1" : "0", is_int($string) ? "1" : "0", "\n";
echo is_float($float) ? "1" : "0", is_double($float) ? "1" : "0", is_float($int) ? "1" : "0", "\n";
echo is_string($string) ? "1" : "0", is_string($int) ? "1" : "0", "\n";
echo is_array($string) ? "1" : "0", "\n";
echo is_scalar($bool) ? "1" : "0", is_scalar($int) ? "1" : "0", is_scalar($float) ? "1" : "0", is_scalar($string) ? "1" : "0", is_scalar($null) ? "1" : "0";
