<?php
$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$right = [];
$right["a"] = "R1";
$right["b"] = null;
$right["c"] = "R3";

$mapped = array_map(null, $left, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", count($mapped[0]), "|", count($mapped[1]), "|", count($mapped[2]), "\n";
echo $mapped[0][0], "|", $mapped[0][1], "\n";
if ($mapped[1][1] === null) {
    echo "right-null\n";
}
if ($mapped[2][0] === null) {
    echo "left-null\n";
}
echo $mapped[2][1], "\n";
$mapped[] = ["after"];
echo count($mapped), "|", count($mapped[3]), "|", $mapped[3][0], "\n";
print_r($left);
print_r($right);

$call = "array_map";
$dynamic = $call(null, ["x" => "A", "y" => "B", "z" => "C"], ["one" => "1"]);
echo count($dynamic), "|", count($dynamic[0]), "|", count($dynamic[1]), "|", count($dynamic[2]), "\n";
echo $dynamic[0][0], "|", $dynamic[0][1], "\n";
if ($dynamic[1][1] === null) {
    echo "dynamic-right-null\n";
}
if ($dynamic[2][1] === null) {
    echo $dynamic[2][0], "|dynamic-right-null";
}
