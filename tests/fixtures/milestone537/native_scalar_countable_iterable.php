<?php
$null = null;
$bool = false;
$int = 0;
$float = 3.5;
$string = "x";

echo is_countable($null) ? "1" : "0";
echo is_countable($bool) ? "1" : "0";
echo is_countable($int) ? "1" : "0";
echo is_countable($float) ? "1" : "0";
echo is_countable($string) ? "1" : "0";
echo "\n";
echo is_iterable($null) ? "1" : "0";
echo is_iterable($bool) ? "1" : "0";
echo is_iterable($int) ? "1" : "0";
echo is_iterable($float) ? "1" : "0";
echo is_iterable($string) ? "1" : "0";
