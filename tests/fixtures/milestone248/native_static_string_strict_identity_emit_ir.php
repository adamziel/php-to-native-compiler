<?php
$left = "php";
$same = "php";
$right = "native";
$joined = "p" . "hp";

echo $left === $same, "\n";
echo $left === $joined, "\n";
echo $left !== $right, "\n";
echo "x", $left === $right, "y";
