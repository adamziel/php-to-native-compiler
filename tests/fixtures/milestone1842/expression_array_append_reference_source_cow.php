<?php
function milestone1842_make_array() {
    return array("outer" => array("existing" => "seed"));
}

$alias =& milestone1842_make_array()[];
$alias = "appended";

$nested =& milestone1842_make_array()["outer"][];
$nested = "nested";

echo $alias, "|", $nested;
