<?php
function combine_three($a, $b, $c) {
    return $a;
}
$left = ["Ada"];
$middle = ["Grace"];
$right = ["Linus"];
echo array_map("combine_three", $left, $middle, $right);
