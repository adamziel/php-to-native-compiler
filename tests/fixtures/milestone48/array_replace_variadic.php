<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left[] = "left next";
$left["keep"] = "keep";

$first = [];
$first["name"] = "Bea";
$first[7] = "seven";
$first["keep"] = "first keep";
$first[] = "first next";

$second = [];
$second["name"] = "Cy";
$second["7"] = "seven second";
$second[9] = "nine";
$second["extra"] = "extra";
$second[5] = "five second";

$third = [];
$third["name"] = "Di";
$third["extra"] = "extra third";
$third[] = "third zero";
$third[10] = "ten";

$replaced = array_replace($left, $first, $second, $third);
print_r($replaced);
echo count($replaced), "\n";
echo $replaced["name"], "|", $replaced[5], "|", $replaced[2], "|", $replaced[6], "|", $replaced["keep"], "|", $replaced[7], "|", $replaced[8], "|", $replaced[9], "|", $replaced["extra"], "|", $replaced[0], "|", $replaced[10], "\n";
$replaced[] = "after";
echo $replaced[11], "\n";
print_r($left);
print_r($first);
print_r($second);
print_r($third);

$call = "array_replace";
$again = $call($left, $first, $second, $third);
echo $again["name"], "|", $again["extra"], "|", $again[10], "\n";

$single = array_replace($left);
print_r($single);
$single[] = "single after";
echo $single[7];
