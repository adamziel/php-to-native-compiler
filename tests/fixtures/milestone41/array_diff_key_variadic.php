<?php
$base = [];
$base["name"] = "Ada";
$base[1] = "one";
$base["2"] = "two";
$base["02"] = "zero two";
$base[-1] = "negative";
$base["drop"] = "drop";
$base[8] = "eight";
$base["keep"] = "keep";
$base[] = "next";

$first = [];
$first["name"] = true;
$first["1"] = true;
$first[2] = true;
$first[-1] = true;

$second = [];
$second["02"] = true;
$second["drop"] = true;

$third = [];
$third[9] = true;

$diffed = array_diff_key($base, $first, $second, $third);
print_r($diffed);
echo count($diffed), "\n";
echo $diffed[8], "|", $diffed["keep"], "\n";
$diffed[] = "after";
echo $diffed[9], "\n";
print_r($base);

$call = "array_diff_key";
$again = $call($base, $first, $second, $third);
echo $again[8], "|", $again["keep"], "\n";

$none = array_diff_key(["name" => "x"], $first, $second, $third);
print_r($none);
echo count($none), "\n";
echo "done";
