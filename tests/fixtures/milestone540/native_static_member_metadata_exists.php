<?php
$class = "Box";
$property = "name";
$method = "open";

echo property_exists("Box", "name") ? "1" : "0";
echo property_exists($class, $property) ? "1" : "0";
echo method_exists("Box", "open") ? "1" : "0";
echo method_exists($class, $method) ? "1" : "0";
