<?php
echo defined("PHP_VERSION") ? "1" : "0";
echo PHP_VERSION === constant("PHP_VERSION") ? "1" : "0";
echo PHP_VERSION !== "" ? "1" : "0";
echo "\n";

$name = "PHP_VERSION";
echo defined($name) ? "1" : "0";
