<?php
class Box {}

$box = new Box();
$values = [
    0,
    -7,
    3.5,
    "0",
    " 42 ",
    "+8",
    "-.5",
    "5.",
    "8e2",
    "",
    " ",
    "8foo",
    "0x10",
    true,
    null,
    ["1"],
    $box,
];
foreach ($values as $value) {
    echo is_numeric($value) ? "1" : "0";
}
echo "\n";

$call = "is_numeric";
echo $call("10.5") ? "1" : "0", $call("text") ? "1" : "0";
