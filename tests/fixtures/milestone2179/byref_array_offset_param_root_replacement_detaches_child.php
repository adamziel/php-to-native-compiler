<?php
function milestone2179_two_refs(&$x, &$y) {
    $x = array("v" => 2);
    $y = 3;
}

$a = array("v" => 1);
$slot =& $a["v"];

milestone2179_two_refs($a, $slot);

echo $a["v"], "|", $slot;
