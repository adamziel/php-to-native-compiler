<?php
$left = [];
$left["null"] = null;
$left["false"] = false;
$left["empty"] = "";
$left["true"] = true;
$left["one"] = 1;
$left["zero"] = 0;
$left["string-zero"] = "0";
$left["int-ten"] = 10;
$left["float-ten"] = 10.0;
$left["string-ten-float"] = "10.0";
$left["text"] = "abc";
$left[8] = "eight";
$left["keep"] = "keep";
$left[] = "next";

$right = [];
$right[] = "";
$right[] = "0";
$right[] = "1";
$right[] = "10";
$right[] = "abc";
$right[] = "missing";

$diffed = array_diff($left, $right);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["string-ten-float"], "|", $diffed[8], "|", $diffed["keep"], "|", $diffed[9], "\n";
$diffed[] = "after";
echo $diffed[10], "\n";
print_r($left);
print_r($right);

$call = "array_diff";
$again = $call($left, $right);
echo $again["string-ten-float"], "|", $again[8], "|", $again["keep"], "|", $again[9], "\n";

$empty = array_diff([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_diff(["x" => "x"], []);
print_r($all);
echo count($all), "\n";

$none = array_diff(["name" => "x"], ["x"]);
print_r($none);
echo count($none), "\n";
echo "done";
