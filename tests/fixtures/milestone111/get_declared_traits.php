<?php
class Box {}

$declared = get_declared_traits();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_traits";
$dynamic = $call();
echo count($dynamic);
