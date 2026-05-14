<?php
$null = null;
$bool = false;
$int = 7;
$float = 3.5;
$string = "x";

echo is_object($null) ? "1" : "0";
echo is_object($bool) ? "1" : "0";
echo is_object($int) ? "1" : "0";
echo is_object($float) ? "1" : "0";
echo is_object($string) ? "1" : "0";
echo "\n";
echo get_debug_type($null), "\n";
echo get_debug_type($bool), "\n";
echo get_debug_type($int), "\n";
echo get_debug_type($float), "\n";
echo get_debug_type($string);
