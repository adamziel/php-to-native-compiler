<?php
$class = "Box";
$target = "Box";
$allow = true;

echo is_a("Box", "Box") ? "1" : "0";
echo is_a("Box", "Box", true) ? "1" : "0";
echo is_a($class, $target, $allow) ? "1" : "0";
echo is_subclass_of("Box", "Box") ? "1" : "0";
echo is_subclass_of($class, $target, $allow) ? "1" : "0";
