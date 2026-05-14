<?php
class Box {}
class Profile {}

$declared = get_declared_classes();
print_r($declared);
echo count($declared), "|", $declared[0], "|", $declared[1], "|", $declared[2], "\n";

$call = "get_declared_classes";
$dynamic = $call();
echo $dynamic[0], "|", $dynamic[1], "|", $dynamic[2];
