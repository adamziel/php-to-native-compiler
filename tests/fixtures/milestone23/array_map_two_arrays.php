<?php
function pair_label($left, $right) {
    if ($left === null) {
        $left = "NULL";
    }
    if ($right === null) {
        $right = "NULL";
    }
    return $left . ":" . $right;
}

$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$right = [];
$right["a"] = "R1";
$right["b"] = "R2";
$right["c"] = "R3";

$mapped = array_map("pair_label", $left, $right);
print_r(array_keys($mapped));
echo $mapped[0], "|", $mapped[1], "|", $mapped[2], "\n";
print_r($left);
print_r($right);

$call = "array_map";
$dynamic = $call("pair_label", ["x" => "A", "y" => "B", "z" => "C"], ["one" => "1"]);
print_r($dynamic);
