<?php
$items = array("name" => "Ada", 5 => "five", "2" => "two");
echo current($items), "|";
$items["name"] = "Grace";
echo current($items), "|";
var_dump(current(array()));
$call = "current";
echo $call(array("head", "tail"));
