<?php
$items = [];
$items["name"] = "Ada";
$items[5] = "five";
$items["2"] = "two";
$items["02"] = "zero two";
$items[-1] = "negative";
$items[] = "next";

$truthy_int = array_chunk($items, 2, 1);
echo $truthy_int[0]["name"], "|", $truthy_int[0][5], "\n";

$falsey_int = array_chunk($items, 2, 0);
echo $falsey_int[0][0], "|", $falsey_int[0][1], "\n";

$truthy_string = array_chunk($items, 2, "yes");
echo $truthy_string[2][-1], "|", $truthy_string[2][6], "\n";

$falsey_string = array_chunk($items, 2, "0");
echo $falsey_string[2][0], "|", $falsey_string[2][1], "\n";

$falsey_null = array_chunk($items, 2, null);
echo $falsey_null[0][0], "|", $falsey_null[0][1], "\n";

$truthy_float = array_chunk($items, 2, 0.25);
echo $truthy_float[1][2], "|", $truthy_float[1]["02"], "\n";

$call = "array_chunk";
$dynamic = $call($items, 2, "1");
echo $dynamic[0]["name"], "|", $dynamic[0][5], "\n";

try {
    array_chunk($items, 0, []);
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}

try {
    array_chunk($items, 2, []);
} catch (TypeError $e) {
    echo $e->getMessage();
}
