<?php
function combine_three($left, $middle, $right) {
    if ($left === null) {
        $left = "NULL";
    }
    if ($middle === null) {
        $middle = "NULL";
    }
    if ($right === null) {
        $right = "NULL";
    }
    return $left . ":" . $middle . ":" . $right;
}

$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$middle = [];
$middle[] = "M1";
$middle[] = "M2";
$middle[] = "M3";

$right = [];
$right["r"] = "R1";

$mapped = array_map("combine_three", $left, $middle, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", $mapped[0], "|", $mapped[1], "|", $mapped[2], "\n";
$mapped[] = "after";
echo count($mapped), "|", $mapped[3], "\n";
print_r($left);
print_r($middle);
print_r($right);

$call = "array_map";
$dynamic = $call("combine_three", ["x" => "A", "y" => "B"], ["one" => "1"], ["p" => "P", "q" => "Q", "r" => "R"]);
echo count($dynamic), "|", $dynamic[0], "|", $dynamic[1], "|", $dynamic[2], "\n";

$builtin = array_map("var_dump", [1], [2], [3]);
echo count($builtin), "|";
if ($builtin[0] === null) {
    echo "builtin-return-null";
}
