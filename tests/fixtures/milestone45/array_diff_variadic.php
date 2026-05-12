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
$first[] = "extra";

$second = [];
$second[] = "drop";
$second[] = "eight";

$third = [];
$third[] = "two";

$diffed = array_diff($base, $first, $second, $third);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed["keep"], "|", $diffed[9], "\n";
$diffed[] = "after";
echo $diffed[10], "\n";
print_r($base);

$call = "array_diff";
$again = $call($base, $first, $second, $third);
echo $again["keep"], "|", $again[9], "\n";

$none = array_diff(["name" => "Ada"], $first, $second, $third);
print_r($none);
echo count($none), "\n";
echo "done";
