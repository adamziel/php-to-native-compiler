<?php
$left = [];
$left["name"] = "Ada";
$left[5] = "five";
$left["2"] = "two";
$left["02"] = "zero two";
$left[] = "left next";

$right = [];
$right["name"] = "Bea";
$right[7] = "seven";
$right["02"] = "zero two right";
$right[] = "right next";
$right["extra"] = "extra";

$merged = array_merge($left, $right);
print_r($merged);
echo count($merged), "\n";
echo $merged["name"], "|", $merged[0], "|", $merged[1], "|", $merged["02"], "|", $merged[2], "|", $merged[3], "|", $merged[4], "|", $merged["extra"], "\n";
$merged[] = "after";
echo $merged[5], "\n";
print_r($left);
print_r($right);

$call = "array_merge";
$again = $call($left, $right);
echo $again["name"], "|", $again[0], "|", $again["02"], "|", $again["extra"], "\n";

$zero = array_merge();
print_r($zero);
echo count($zero), "\n";

$single = array_merge($left);
print_r($single);
echo count($single), "\n";
$single[] = "single after";
echo $single[3], "\n";

$third = [];
$third["name"] = "Cy";
$third[10] = "ten";
$third["extra"] = "third extra";
$third[] = "third next";

$variadic = array_merge($left, $right, $third);
print_r($variadic);
echo count($variadic), "\n";
echo $variadic["name"], "|", $variadic[0], "|", $variadic[1], "|", $variadic["02"], "|", $variadic[2], "|", $variadic[3], "|", $variadic[4], "|", $variadic["extra"], "|", $variadic[5], "|", $variadic[6], "\n";

$again_three = $call($left, $right, $third);
echo $again_three["name"], "|", $again_three[5], "|", $again_three[6], "|", $again_three["extra"];
