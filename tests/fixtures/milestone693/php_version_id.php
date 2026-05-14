<?php
echo defined("PHP_VERSION_ID") ? "1" : "0";
echo PHP_VERSION_ID > 0 ? "1" : "0";
echo constant("PHP_VERSION_ID") === PHP_VERSION_ID ? "1" : "0";
echo "\n";

$name = "PHP_VERSION_ID";
echo constant($name) >= 80000 ? "1" : "0";
