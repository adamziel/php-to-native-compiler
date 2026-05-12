<?php
$left = [];
$left["first"] = "L1";
$left[5] = "L2";

$middle = [];
$middle[] = "M1";
$middle[] = "M2";
$middle[] = "M3";

$right = [];
$right["r"] = "R1";

$mapped = array_map(null, $left, $middle, $right);
print_r(array_keys($mapped));
echo count($mapped), "|", count($mapped[0]), "|", count($mapped[1]), "|", count($mapped[2]), "\n";
echo $mapped[0][0], "|", $mapped[0][1], "|", $mapped[0][2], "\n";
if ($mapped[1][2] === null) {
    echo "right-null\n";
}
if ($mapped[2][0] === null) {
    echo "left-null\n";
}
echo $mapped[2][1], "\n";
$mapped[] = ["after"];
echo count($mapped), "|", count($mapped[3]), "|", $mapped[3][0], "\n";
print_r($left);
print_r($middle);
print_r($right);

$call = "array_map";
$dynamic = $call(null, ["x" => "A", "y" => "B"], ["one" => "1"], ["p" => "P", "q" => "Q", "r" => "R"], ["last" => "Z"]);
echo count($dynamic), "|", count($dynamic[0]), "|", count($dynamic[1]), "|", count($dynamic[2]), "\n";
echo $dynamic[0][0], "|", $dynamic[0][1], "|", $dynamic[0][2], "|", $dynamic[0][3], "\n";
if ($dynamic[1][1] === null) {
    echo "dynamic-second-null\n";
}
echo $dynamic[1][0], "|", $dynamic[1][2], "\n";
if ($dynamic[2][0] === null) {
    echo "dynamic-left-null\n";
}
if ($dynamic[2][1] === null) {
    echo "dynamic-second-null-tail\n";
}
if ($dynamic[2][3] === null) {
    echo $dynamic[2][2], "|dynamic-fourth-null";
}
