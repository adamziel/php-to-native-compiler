<?php
$args = array(2 => "two", "name" => "Ada", 5 => "five");
$count = array_unshift($args, "new", "first");

echo $count, "|";
echo $args[0], "|", $args[1], "|", $args[2], "|", $args["name"], "|", $args[3], "|";
$call = "array_unshift";
echo $call($args, "zero"), "|";
echo $args[0], "|", $args[1], "|", $args[4];
