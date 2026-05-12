<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$right = array_pad($items, 8, "pad");
print_r($right);
echo count($right), "|", $right["name"], "|", $right[0], "|", $right[1], "|", $right["02"], "|", $right[2], "|", $right[3], "|", $right[4], "|", $right[5], "\n";
$right[] = "after";
echo $right[6], "\n";

$left = array_pad($items, -8, "pad");
print_r($left);
echo count($left), "|", $left[0], "|", $left[1], "|", $left["name"], "|", $left[2], "|", $left[3], "|", $left["02"], "|", $left[4], "|", $left[5], "\n";
$left[] = "after-left";
echo $left[6], "\n";

$noop = array_pad($items, 3, "pad");
if (array_key_exists(0, $noop)) {
    echo "noop-reindexed\n";
} else {
    echo "noop-preserved\n";
}
echo $noop["name"], "|", $noop[5], "|", $noop[2], "|", $noop["02"], "|", $noop[-1], "|", $noop[6], "\n";
$noop[] = "after-noop";
echo $noop[7], "\n";

$empty_right = array_pad([], 3, "pad");
echo count($empty_right), "|", $empty_right[0], "|", $empty_right[1], "|", $empty_right[2], "\n";

$empty_left = array_pad([], -2, "left");
echo count($empty_left), "|", $empty_left[0], "|", $empty_left[1], "\n";

$call = "array_pad";
$dynamic = $call(["first" => "Ada"], 3, "pad");
print_r($dynamic);
echo "done";
