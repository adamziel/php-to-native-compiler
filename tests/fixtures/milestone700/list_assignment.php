<?php
$pair = ["first", "second", "ignored"];
list($a, $b) = $pair;
echo $a, "|", $b, "\n";

function rhs_pair() {
    echo "rhs\n";
    return ["left", "right"];
}

list($left, $right) = rhs_pair();
echo $left, "|", $right;
