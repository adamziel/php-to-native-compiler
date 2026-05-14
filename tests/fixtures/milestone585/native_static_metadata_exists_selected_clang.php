<?php
$name = "Box";
$autoload = false;

echo class_exists("Box") ? "1" : "0";
echo class_exists($name, $autoload) ? "1" : "0";
echo interface_exists("I") ? "1" : "0";
echo trait_exists("T", true) ? "1" : "0";
echo enum_exists("E") ? "1" : "0";
