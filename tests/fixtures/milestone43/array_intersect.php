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
$left["drop"] = "drop";
$left[] = "next";

$right = [];
$right[] = "";
$right[] = "0";
$right[] = "1";
$right[] = "10";
$right[] = "abc";
$right[] = "eight";
$right[] = "missing";

$intersected = array_intersect($left, $right);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["true"], "|", $intersected["one"], "|", $intersected["zero"], "|", $intersected["string-zero"], "|", $intersected["int-ten"], "|", $intersected["float-ten"], "|", $intersected["text"], "|", $intersected[8], "\n";
$intersected[] = "after";
echo $intersected[10], "\n";
print_r($left);
print_r($right);

$call = "array_intersect";
$again = $call($left, $right);
echo $again["true"], "|", $again["one"], "|", $again["zero"], "|", $again["string-zero"], "|", $again["int-ten"], "|", $again["float-ten"], "|", $again["text"], "|", $again[8], "\n";

$empty = array_intersect([], $right);
print_r($empty);
echo count($empty), "\n";

$all = array_intersect(["x" => "x"], ["x"]);
print_r($all);
echo count($all), "\n";

$none = array_intersect(["name" => "x"], ["y"]);
print_r($none);
echo count($none), "\n";
echo "done";
