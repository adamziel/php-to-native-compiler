<?php
$base = [];
$base["name"] = "Ada";
$base[1] = "one";
$base["2"] = "two";
$base["02"] = "zero two";
$base[-1] = "negative";
$base[] = "next";

$first = [];
$first["name"] = true;
$first["1"] = true;
$first[2] = true;
$first["02"] = true;
$first[3] = true;

$second = [];
$second["name"] = true;
$second["2"] = true;
$second["02"] = true;
$second[-1] = true;

$intersected = array_intersect_key($base, $first, $second);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["name"], "|", $intersected[2], "|", $intersected["02"], "\n";
$intersected[] = "after";
echo $intersected[3], "\n";
print_r($base);

$call = "array_intersect_key";
$again = $call($base, $first, $second);
echo $again["name"], "|", $again[2], "|", $again["02"], "\n";

$none = array_intersect_key(["only" => "value"], $first, $second);
print_r($none);
echo count($none), "\n";
echo "done";
