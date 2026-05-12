<?php
class Box {}

$declared = get_declared_interfaces();
print_r($declared);
echo count($declared), "\n";

$call = "get_declared_interfaces";
$dynamic = $call();
echo count($dynamic);
