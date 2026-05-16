<?php
$value = "x";
$left = [];
$left["slot"] =& $value;
$right = $left;
$right["slot"] = "b";
echo $value, "|", $left["slot"], "|", $right["slot"], "\n";

$left = ["slot" => "x"];
$alias =& $left["slot"];
$right = $left;
$alias = "y";
echo $right["slot"], "|", $left["slot"];
$right["slot"] = "z";
echo "|", $alias, "|", $left["slot"], "\n";

$left = ["slot" => "x"];
$right = $left;
$right["slot"] = "b";
echo $left["slot"], "|", $right["slot"];
