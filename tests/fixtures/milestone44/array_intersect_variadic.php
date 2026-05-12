<?php
$base = [];
$base["name"] = "Ada";
$base[1] = "1";
$base["two"] = "two";
$base["ten"] = 10;
$base["float-ten"] = 10.0;
$base["drop"] = "drop";
$base[8] = "eight";
$base["keep"] = "keep";
$base[] = "next";

$first = [];
$first[] = "Ada";
$first[] = "1";
$first[] = "10";
$first[] = "eight";
$first[] = "next";
$first[] = "extra";

$second = [];
$second[] = "Ada";
$second[] = "10";
$second[] = "eight";
$second[] = "drop";
$second[] = "next";

$third = [];
$third[] = "Ada";
$third[] = "10";
$third[] = "eight";
$third[] = "next";

$intersected = array_intersect($base, $first, $second, $third);
print_r($intersected);
echo count($intersected), "\n";
echo $intersected["name"], "|", $intersected["ten"], "|", $intersected["float-ten"], "|", $intersected[8], "|", $intersected[9], "\n";
$intersected[] = "after";
echo $intersected[10], "\n";
print_r($base);

$call = "array_intersect";
$again = $call($base, $first, $second, $third);
echo $again["name"], "|", $again["ten"], "|", $again["float-ten"], "|", $again[8], "|", $again[9], "\n";

$none = array_intersect(["name" => "x"], $first, $second, $third);
print_r($none);
echo count($none), "\n";
echo "done";
